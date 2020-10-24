#![windows_subsystem = "windows"]
mod config;
mod game_window;
mod gui;
mod overlay;

// TODO: replace
mod bridge;
mod requester;

#[cfg(windows)]
mod windows {
    pub(crate) mod ext;
    pub(crate) mod tools;
}
use windows::tools as win_tools;

use viewports::dependencies::imgui;

use futures::executor::block_on;
//use imgui::{im_str, FontSource, Condition};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use viewports::{
    wgpu::{Wgpu, WgpuManager},
    Manager, Platform, Viewport, WithLoop,
};

fn setup_error_handling() {
    use win_tools::{has_console, HasConsole};

    tracing_subscriber::fmt::init();

    let has_console = has_console();
    dbg!(has_console);

    match has_console {
        HasConsole::None | HasConsole::MyOwn => achtung::setup("reports", "overlay"),
        HasConsole::NotMine => {}
    }
}

fn setup_first_window<T: 'static>(event_loop: &EventLoop<T>, backends: wgpu::BackendBit, ) -> (WgpuManager, WindowId) {
    let instance = wgpu::Instance::new(backends);
    let mut manager = WgpuManager::new(instance);

    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(LogicalSize {
        width: 10.0,
        height: 10.0,
    });
    window.set_outer_position(winit::dpi::PhysicalPosition { x: 0, y: 0 });
    window.set_title("OverlayV2");
    //window.set_always_on_top(true);
    window.set_visible(false);

    let main_view = manager.add_window(window);

    (manager, main_view)
}

fn setup_adapter(manager: &WgpuManager, main_view: WindowId, power_preference: wgpu::PowerPreference) -> wgpu::Adapter {
    block_on(
        manager
            .instance()
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: manager.viewport(main_view).unwrap().surface(),
            }),
    )
    .expect("No suitable adapter found")
}

fn setup_imgui(hidpi_factor: f64) -> imgui::Context {
    use imgui::ConfigFlags;
    let mut imgui = imgui::Context::create();

    let io = imgui.io_mut();
    io.config_flags.insert(ConfigFlags::DOCKING_ENABLE);
    io.config_flags.insert(ConfigFlags::VIEWPORTS_ENABLE);

    //io.mouse_draw_cursor = true;

    let font_size = (16.0 * hidpi_factor) as f32;
    io.font_global_scale = (1.0 / hidpi_factor) as f32;
    add_fonts(&mut *imgui.fonts(), font_size);
    /*imgui.fonts().add_font(&[FontSource::DefaultFontData {
        config: Some(imgui::FontConfig {
            oversample_h: 1,
            pixel_snap_h: true,
            size_pixels: font_size,
            ..Default::default()
        }),
    }]);*/

    imgui
}

fn setup_renderer(adapter: &wgpu::Adapter, imgui: &mut imgui::Context) -> Wgpu {
    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            shader_validation: false,
        },
        None,
    ))
    .unwrap();
    Wgpu::new(imgui, device, queue)
}

fn main() {
    setup_error_handling();

    let config = config::Config::load();
    println!("{:?}", config);

    let event_loop = EventLoop::<overlay::OverlayEvent>::with_user_event();

    // Set up window and GPU
    let (mut manager, main_view) = setup_first_window(&event_loop, config.backend_bits());

    let mut overlay = overlay::Overlay::new(&config, event_loop.create_proxy(), main_view);

    let adapter = setup_adapter(&manager, main_view, config.power_preference());
    dbg!(adapter.get_info());

    let mut imgui = setup_imgui(1.0);

    let mut platform = Platform::init(&mut imgui, manager.viewport(main_view).unwrap());

    let mut renderer = setup_renderer(&adapter, &mut imgui);

    //let mut demo_open = true;

    //let mut path = std::path::PathBuf::new();
    //path.push("assets");
    //path.push("actarrow.ani");
    /*let cursor = windows::ext::Cursor::from_file(
        &std::ffi::CString::new("assets\\actarrow.ani").unwrap()
    ).expect("Cursor image");*/

    overlay.make_game_foreground();
    //overlay.reparent_game_window(&manager);

    event_loop.run(move |event, event_loop, control_flow| {
        //*control_flow = ControlFlow::Wait;

        //cursor.activate();

        platform.handle_event(imgui.io_mut(), &mut manager, &event);

        let mut manager_with_loop = overlay.spawning_manager(&mut manager, &event_loop);
        match event {
            Event::NewEvents(..) => {
                if overlay.should_exit() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::UserEvent(event) => {
                overlay.handle_event(event);
            }
            Event::WindowEvent { event, window_id } => {
                match event {
                    WindowEvent::CloseRequested if window_id == main_view => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::CursorEntered { .. } => {
                        /*if let Some(viewport) = manager_with_loop.viewport(window_id) {
                            use winit::window::CursorIcon;
                            dbg!("Cursor entered");
                            let window = viewport.window();
                            window.set_cursor_icon(CursorIcon::Crosshair);
                            //window.set_cursor_visible(true);
                        }*/
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                if overlay.should_render(&platform) {
                    overlay.reorder_windows2(&mut manager_with_loop, platform.focus_order(&mut imgui));

                    platform.frame(&mut imgui, &mut manager_with_loop, |ui, _delta| {
                        overlay.frame(ui, &mut renderer);
                        //ui.show_demo_window(&mut demo_open);
                    });
                    manager_with_loop.reqwest_redraws();
                }
            }
            Event::RedrawRequested(window_id) => {
                if let Some(draw_data) = platform.draw_data(&mut imgui, window_id) {
                    let viewport = manager_with_loop
                        .viewport_mut(window_id)
                        .expect("Expect viewport");
                    viewport.on_draw(&mut renderer, draw_data);
                }
            }
            Event::RedrawEventsCleared => {
                overlay.sleep_or_poll(&platform, control_flow);
            }
            _ => {}
        }
    });
}

fn _font_atlas(hidpi_factor: f64) -> imgui::SharedFontAtlas {
    let font_size = (16.0 * hidpi_factor) as f32;
    let mut atlas = imgui::SharedFontAtlas::create();
    add_fonts(&mut atlas, font_size);
    atlas
}

fn add_fonts(atlas: &mut imgui::FontAtlas, font_size: f32) {
    let config = imgui::FontConfig {
        oversample_h: 1,
        pixel_snap_h: true,
        //size_pixels: font_size,
        //rasterizer_multiply: 1.75,
        glyph_ranges: imgui::FontGlyphRanges::cyrillic(),
        ..Default::default()
    };
    let font = imgui::FontSource::TtfData {
        config: Some(config),
        data: include_bytes!("../assets/clacon.ttf"),
        //data: include_bytes!("../../resources/fallout_display.ttf"),
        size_pixels: font_size,
    };
    atlas.add_font(&[font]);
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
impl Rect {
    fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}
