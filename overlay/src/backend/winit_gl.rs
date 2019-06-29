use super::*;

use glium::{
    backend::{Context, Facade},
    glutin, Texture2d,
};
use glutin::{dpi::LogicalPosition, EventsLoop, Window, WindowId};
use imgui::{self, FontGlyphRange, ImFontConfig, Ui};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::{borrow::Cow, collections::HashSet, rc::Rc, time::Instant};

pub type Textures = imgui::Textures<Rc<Texture2d>>;

pub struct WinitGlBackend {
    events_loop: EventsLoop,
    redraw_windows: HashSet<WindowId>,
}

impl Backend for WinitGlBackend {
    type Window = WinitGlWindow;
    fn new() -> Self {
        let mut events_loop = glutin::EventsLoop::new();
        WinitGlBackend {
            events_loop,
            redraw_windows: HashSet::new(),
        }
    }
    fn new_window(&self) -> Result<Self::Window, BackendError<Self>> {
        use glium::{Display, Surface};

        let context = glutin::ContextBuilder::new().with_vsync(false);
        let builder = glutin::WindowBuilder::new()
            .with_title("FOnlineOverlay")
            .with_decorations(false)
            .with_always_on_top(true)
            .with_resizable(false)
            .with_visibility(false)
            .with_dimensions(glutin::dpi::LogicalSize::new(64f64, 64f64));
        let display = Display::new(builder, context, &self.events_loop)
            .map_err(WinitGlError::DisplayCreation)?;
        let window_id = display.gl_window().window().id();
        let window = WinitGlWindow { display, window_id };
        Ok(window)
    }
    fn poll_events(&mut self) -> bool {
        let mut exit = false;
        self.events_loop.poll_events(|event| {
            use glutin::{Event, WindowEvent};

            //platform.handle_event(imgui.io_mut(), &window, &event);

            if let Event::WindowEvent { event, window_id } = event {
                match event {
                    WindowEvent::CloseRequested => exit = true,
                    /*WindowEvent::Refresh => {
                        self.redraw_windows.insert(window_id);
                    }*/
                    _ => (),
                }
            }
        });
        exit
    }
}

pub struct WinitGlWindow {
    window_id: WindowId,
    display: glium::Display,
}

macro_rules! window {
    ($win:expr) => {
        $win.display.gl_window().window()
    };
}

impl BackendWindow for WinitGlWindow {
    type Texture = Texture2d;
    type Error = WinitGlError;
    //fn change(pos: Some<(i32, i32)>, size: Option<(u32, u32)>, show: Option<bool>) -> bool;
    fn show(&mut self) {
        window!(self).show();
    }
    fn hide(&mut self) {
        window!(self).hide();
    }
    fn set_position(&mut self, x: i32, y: i32) {
        window!(self).set_position((x, y).into());
    }
    fn create_texture(&mut self, image: &mut ImageData) -> Result<Self::Texture, Self::Error> {
        let raw = glium::texture::RawImage2d {
            data: Cow::Borrowed(&image.bytes),
            width: image.width,
            height: image.height,
            format: glium::texture::ClientFormat::U8U8U8,
        };
        Texture2d::new(&self.display, raw).map_err(WinitGlError::TextureCreation)
    }
    fn draw_texture(
        &mut self,
        texture: &Self::Texture,
        src: &Rect,
        dst: &Rect,
    ) -> Result<(), Self::Error> {
        use glium::Surface;

        let src = glium::Rect {
            left: src.x as u32,
            bottom: src.y as u32,
            width: src.width,
            height: src.height,
        };
        //let dst = glium::BlitTarget{left: dst.x as u32, bottom: dst.y as u32, width: dst.width as i32, height: dst.height as i32};
        let dst = glium::BlitTarget {
            left: dst.x as u32,
            bottom: dst.height,
            width: dst.width as i32,
            height: dst.y - dst.height as i32,
        };

        let mut target = self.display.draw();
        texture.as_surface().blit_color(
            &src,
            &target,
            &dst,
            glium::uniforms::MagnifySamplerFilter::Linear,
        );
        target.finish().map_err(WinitGlError::SwapBuffers)
    }

    fn drop_texture(&mut self, texture: Self::Texture) {}
    fn handle(&self) -> *mut () {
        use glutin::os::windows::WindowExt;
        window!(self).get_hwnd() as _
    }
}

#[derive(Debug)]
pub enum WinitGlError {
    DisplayCreation(glium::backend::glutin::DisplayCreationError),
    TextureCreation(glium::texture::TextureCreationError),
    SwapBuffers(glium::SwapBuffersError),
}
/*
pub fn run<F>(title: String, clear_color: [f32; 4], mut run_ui: F)
    where
        F: FnMut(&Ui, &Rc<Context>, &mut Textures) -> bool,
{
    use imgui_glium_renderer::GliumRenderer;

    let gl_window = display.gl_window();
    let window = gl_window.window();

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;

    imgui.fonts().add_default_font_with_config(
        ImFontConfig::new()
            .oversample_h(1)
            .pixel_snap_h(true)
            .size_pixels(font_size),
    );

    imgui.fonts().add_font_with_config(
        include_bytes!("../../../resources/mplus-1p-regular.ttf"),
        ImFontConfig::new()
            .merge_mode(true)
            .oversample_h(1)
            .pixel_snap_h(true)
            .size_pixels(font_size)
            .rasterizer_multiply(1.75),
        &FontGlyphRange::japanese(),
    );

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let mut renderer =
        GliumRenderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    let mut last_frame = Instant::now();
    let mut quit = false;

    loop {
        events_loop.poll_events(|event| {
            use glium::glutin::{Event, WindowEvent::CloseRequested};

            platform.handle_event(imgui.io_mut(), &window, &event);

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    CloseRequested => quit = true,
                    _ => (),
                }
            }
        });

        let io = imgui.io_mut();
        platform
            .prepare_frame(io, &window)
            .expect("Failed to start frame");
        last_frame = io.update_delta_time(last_frame);
        let ui = imgui.frame();
        if !run_ui(&ui, display.get_context(), renderer.textures()) {
            break;
        }

        let mut target = display.draw();
        target.clear_color(
            clear_color[0],
            clear_color[1],
            clear_color[2],
            clear_color[3],
        );
        platform.prepare_render(&ui, &window);
        renderer.render(&mut target, ui).expect("Rendering failed");
        target.finish().unwrap();

        if quit {
            break;
        }
    }
}
*/
