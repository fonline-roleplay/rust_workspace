use super::*;

use glium::{
    backend::{Context, Facade},
    glutin, Display, Surface, Texture2d,
};
use glutin::{dpi::LogicalPosition, Event, EventsLoop, NotCurrent, Window, WindowEvent, WindowId};
use imgui::Ui;
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::{Rc, Weak},
    time::Instant,
};

#[cfg(target_pointer_width = "32")]
type LONG_OR_PTR = i32;
#[cfg(target_pointer_width = "64")]
type LONG_OR_PTR = isize;

//pub trait GuiRun: FnMut(&Ui, &Rc<Context>, &mut Textures) -> bool {}
//impl<T: FnMut(&Ui, &Rc<Context>, &mut Textures) -> bool> GuiRun for T {}

pub struct WinitGlBackend {
    events_loop: EventsLoop,
    headless: glutin::Context<NotCurrent>,
    windows: HashMap<WindowId, WindowWeak<Self>>,
    font_atlas: Option<FontAtlasRef>, //redraw_windows: HashSet<WindowId>,
}

fn context_builder<'a>() -> glutin::ContextBuilder<'a, NotCurrent> {
    glutin::ContextBuilder::new()
        .with_vsync(false)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (2, 1)))
}

fn make_window_popup(window: &Window) -> Result<(), String> {
    use glutin::os::windows::WindowExt;
    use winapi::{
        shared::windef,
        um::{errhandlingapi as err, wingdi, winuser},
    };
    let handle = window.get_hwnd() as _;

    //window.hide_cursor(true);

    unsafe {
        let mut flags = winuser::GetWindowLongPtrA(handle, winuser::GWL_STYLE);
        if flags == 0 {
            return Err(format!("winuser::GetWindowLongPtrA"));
        }
        flags |= winuser::WS_POPUP as LONG_OR_PTR;
        if winuser::SetWindowLongPtrA(handle, winuser::GWL_STYLE, flags) == 0 {
            return Err(format!("winuser::SetWindowLongPtrA"));
        }

        let mut flags = winuser::GetWindowLongPtrA(handle, winuser::GWL_EXSTYLE);
        if flags == 0 {
            return Err(format!("winuser::GetWindowLongPtrA"));
        }
        flags |= winuser::WS_EX_NOACTIVATE as LONG_OR_PTR;
        flags &= !winuser::WS_EX_APPWINDOW as LONG_OR_PTR;
        if winuser::SetWindowLongPtrA(handle, winuser::GWL_EXSTYLE, flags) == 0 {
            return Err(format!("winuser::SetWindowLongPtrA"));
        }
    }
    Ok(())
}

fn make_window_border(window: &Window) -> Result<(), String> {
    use glutin::os::windows::WindowExt;
    use winapi::{
        shared::windef,
        um::{errhandlingapi as err, wingdi, winuser},
    };
    let handle = window.get_hwnd() as _;

    unsafe {
        let mut flags = winuser::GetWindowLongPtrA(handle, winuser::GWL_STYLE);
        if flags == 0 {
            return Err(format!("winuser::GetWindowLongPtrA"));
        }
        flags |= winuser::WS_BORDER as LONG_OR_PTR;
        if winuser::SetWindowLongPtrA(handle, winuser::GWL_STYLE, flags) == 0 {
            return Err(format!("winuser::SetWindowLongPtrA"));
        }

        let mut flags = winuser::GetWindowLongPtrA(handle, winuser::GWL_EXSTYLE);
        if flags == 0 {
            return Err(format!("winuser::GetWindowLongPtrA"));
        }
        flags |= winuser::WS_EX_WINDOWEDGE as LONG_OR_PTR;
        if winuser::SetWindowLongPtrA(handle, winuser::GWL_EXSTYLE, flags) == 0 {
            return Err(format!("winuser::SetWindowLongPtrA"));
        }
    }
    Ok(())
}

impl Backend for WinitGlBackend {
    type Window = WinitGlWindow;
    type Event = Event;
    type Texture = Texture2d;
    type Error = WinitGlError;
    type Context = glium::backend::Context;

    fn new() -> Self {
        let mut events_loop = glutin::EventsLoop::new();
        let headless = context_builder()
            .build_headless(&events_loop, glutin::dpi::PhysicalSize::new(1.0, 1.0))
            .expect("Headless GL context");
        dbg!(&headless.get_api());
        WinitGlBackend {
            events_loop,
            headless,
            windows: HashMap::new(),
            font_atlas: None,
            //redraw_windows: HashSet::new(),
        }
    }
    fn new_window(
        &mut self,
        title: String,
        width: u32,
        height: u32,
    ) -> BackendResult<WindowRef<Self>, Self> {
        let context = context_builder().with_shared_lists(&self.headless);
        let builder = glutin::WindowBuilder::new()
            .with_title(title)
            .with_decorations(false)
            .with_always_on_top(true)
            .with_resizable(false)
            //.with_visibility(false)
            .with_dimensions((width, height).into());
        let display = Display::new(builder, context, &self.events_loop)
            .map_err(WinitGlError::DisplayCreation)?;
        dbg!(display.get_opengl_version_string());
        let window_id = {
            let window = display.gl_window();
            let window = window.window();
            //make_window_border(window).map_err(WinitGlError::Platform)?;
            window.id()
        };

        let window = WinitGlWindow {
            display,
            window_id,
            gui: None,
            last_pos: None,
            dragging: None,
        };
        let window = Rc::new(RefCell::new(window));
        self.windows.insert(window_id, Rc::downgrade(&window));
        Ok(window)
    }
    fn new_popup(
        &mut self,
        title: String,
        width: u32,
        height: u32,
    ) -> BackendResult<WindowRef<Self>, Self> {
        let context = context_builder().with_shared_lists(&self.headless);
        let builder = glutin::WindowBuilder::new()
            .with_title(title)
            .with_decorations(false)
            .with_always_on_top(true)
            .with_resizable(false)
            .with_visibility(false)
            .with_dimensions((width, height).into());
        let display = Display::new(builder, context, &self.events_loop)
            .map_err(WinitGlError::DisplayCreation)?;
        dbg!(display.get_opengl_version_string());
        let window_id = {
            let window = display.gl_window();
            let window = window.window();
            make_window_popup(window).map_err(WinitGlError::Platform)?;
            window.id()
        };

        let window = WinitGlWindow {
            display,
            window_id,
            gui: None,
            last_pos: None,
            dragging: None,
        };
        let window = Rc::new(RefCell::new(window));
        self.windows.insert(window_id, Rc::downgrade(&window));
        Ok(window)
    }
    fn poll_events<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Self::Event),
    {
        let windows = &mut self.windows;
        let mut redraw = false;
        self.events_loop
            .poll_events(|global_event| match &global_event {
                Event::WindowEvent { window_id, event } => {
                    if let Some(window) = windows.get(&window_id) {
                        match window.upgrade() {
                            Some(window) => {
                                let mut window = window.borrow_mut();
                                redraw |= window.handle_event(&global_event);
                            }
                            None => {
                                windows.remove(&window_id);
                            }
                        }
                    }
                }
                _ => {
                    //println!("Another event: {:?}", global_event);
                }
            });
        redraw
    }

    fn drop_texture(context: &Rc<BackendContext<Self>>, texture: &BackendTexture<Self>) {
        texture.drop_for(context, false);
    }
    fn font_atlas(&mut self, hidpi_factor: f64) -> FontAtlasRef {
        dbg!(hidpi_factor);
        if let Some(atlas) = &self.font_atlas {
            return Rc::clone(atlas);
        }
        let font_size = (16.0 * hidpi_factor) as f32;
        /*
            imgui.fonts().add_default_font_with_config(
                ImFontConfig::new()
                    .oversample_h(1)
                    .pixel_snap_h(true)
                    .size_pixels(font_size),
            );
        */
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
            data: include_bytes!("../../resources/clacon.ttf"),
            //data: include_bytes!("../../resources/fallout_display.ttf"),
            size_pixels: font_size,
        };
        //imgui.fonts().add_font(&[font]);
        let mut atlas = imgui::SharedFontAtlas::create();
        atlas.add_font(&[font]);
        //imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        let atlas_ref = Rc::new(RefCell::new(atlas));
        self.font_atlas = Some(Rc::clone(&atlas_ref));
        atlas_ref
    }
}

impl GuiEvent<WinitGlBackend> for Event {
    fn is_close_request(&self) -> bool {
        if let Event::WindowEvent { event, window_id } = self {
            match event {
                WindowEvent::CloseRequested => true,
                _ => false,
            }
        } else {
            false
        }
    }
}

pub struct WinitGlWindow {
    window_id: WindowId,
    display: Display,
    gui: Option<Box<Gui>>,
    last_pos: Option<LogicalPosition>,
    dragging: Option<LogicalPosition>,
}

macro_rules! window {
    ($win:expr) => {
        $win.display.gl_window().window()
    };
}

impl BackendWindow for WinitGlWindow {
    type Back = WinitGlBackend;
    //fn change(pos: Some<(i32, i32)>, size: Option<(u32, u32)>, show: Option<bool>) -> bool;
    fn show(&mut self) {
        //window!(self).show();
        use glutin::os::windows::WindowExt;
        use winapi::um::winuser;

        let hwnd = window!(self).get_hwnd();
        unsafe {
            winuser::ShowWindow(hwnd as _, winuser::SW_SHOWNOACTIVATE);
        }
    }
    fn hide(&mut self) {
        window!(self).hide();
    }
    fn to_foreground(&mut self) {
        use glutin::os::windows::WindowExt;
        let hwnd = window!(self).get_hwnd();
        use winapi::um::winuser;
        unsafe { winuser::BringWindowToTop(hwnd as _) };
    }
    fn set_position(&mut self, x: i32, y: i32) {
        //window!(self).set_position((x, y).into());
        /*use winapi::{
            shared::windef,
            um::{errhandlingapi as err, wingdi, winuser},
        };
        let handle = self.window_handle() as _;
        unsafe {
            winuser::SetWindowPos(
                handle,
                0 as _,
                x,
                y,
                0,
                0,
                winuser::SWP_NOZORDER | winuser::SWP_NOZORDER,
            );
        }*/
        use glutin::os::windows::WindowExt;
        use winapi::um::winuser;

        let hwnd = window!(self).get_hwnd();
        unsafe {
            winuser::SetWindowPos(
                hwnd as _,
                std::ptr::null_mut(),
                x,
                y,
                0,
                0,
                winuser::SWP_ASYNCWINDOWPOS
                    | winuser::SWP_NOZORDER
                    | winuser::SWP_NOSIZE
                    | winuser::SWP_NOACTIVATE,
            );
            winuser::UpdateWindow(hwnd as _);
        }
    }
    fn set_size(&mut self, x: u32, y: u32) {
        window!(self).set_inner_size((x, y).into());
    }
    fn move_by_f32(&mut self, x: f32, y: f32) {
        let window_gl = self.display.gl_window();
        let window = window_gl.window();
        if let Some(win_pos) = window.get_position() {
            window.set_position((win_pos.x + x as f64, win_pos.y + y as f64).into());
        }
    }
    fn create_texture(
        &mut self,
        image: &mut ImageData,
    ) -> BackendResult<BackendTexture<Self::Back>, Self::Back> {
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
        texture: &BackendTexture<Self::Back>,
        src: &Rect,
        dst: &Rect,
    ) -> BackendResult<(), Self::Back> {
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

    fn init_gui<F>(
        &mut self,
        back: &mut Self::Back,
        init_context: F,
    ) -> BackendResult<(), Self::Back>
    where
        F: FnMut(&mut imgui::Context, GuiInfo) -> Result<(), ()>,
    {
        /*if self.gui.is_none() {
            let gui = Gui::init(&mut self.display)?;
            self.gui = Some(Box::new(gui));
        }*/
        //let gui = self.gui.as_mut().unwrap();
        let gui = Gui::init(&mut self.display, back, init_context)?;
        self.gui = Some(Box::new(gui));
        Ok(())
    }

    fn draw_gui<F>(&mut self, run_ui: F) -> BackendResult<(), Self::Back>
    where
        F: FnMut(
            &imgui::Ui,
            &Rc<BackendContext<Self::Back>>,
            &mut ImGuiTextures<Self::Back>,
        ) -> Vec<imgui::TextureId>,
    {
        if let Some(gui) = self.gui.as_mut() {
            gui.draw(&self.display, run_ui)
        } else {
            Err(WinitGlError::ImGuiInit)
        }
    }
    fn handle_event(&mut self, event: &BackendEvent<Self::Back>) -> bool {
        let window_gl = self.display.gl_window();
        let window = window_gl.window();

        /*println!(
            "window: {:?}, event: {:?}, gui: {}",
            window.id(),
            &event,
            self.gui.is_some()
        );*/

        if let Some(gui) = self.gui.as_mut() {
            /*match event {
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::Focused(false),
                } if *window_id == window.id() => {
                    let io = gui.imgui.io_mut();
                    io.key_shift = false;
                    io.key_ctrl = false;
                    io.key_alt = false;
                    io.key_super = false;
                    for key in io.keys_down.iter_mut() {
                        *key = false;
                    }
                }
                _ => {}
            }*/
            match event {
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::Focused(focused),
                } if *window_id == window.id() => {
                    gui.focused = *focused;
                }
                _ => {}
            }

            let imgui = gui.imgui.activate().expect("Can't acivate imgui context");
            gui.platform.handle_event(imgui.io_mut(), window, event);
            gui.imgui.suspend();

            false
        } else {
            match event {
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::Refresh,
                } if *window_id == window.id() => true,
                _ => false,
            }
        }
    }
    fn window_handle(&self) -> *mut () {
        use glutin::os::windows::WindowExt;
        window!(self).get_hwnd() as _
    }
}

#[derive(Debug)]
pub enum WinitGlError {
    DisplayCreation(glium::backend::glutin::DisplayCreationError),
    TextureCreation(glium::texture::TextureCreationError),
    GliumRenderer(imgui_glium_renderer::RendererError),
    SwapBuffers(glium::SwapBuffersError),
    ImGuiPrepareFrame(String),
    ImGuiInit,
    Platform(String),
}

enum GuiContext {
    Active(imgui::Context),
    Suspended(imgui::SuspendedContext),
    Invalid,
}

impl GuiContext {
    fn suspend(&mut self) {
        let context = std::mem::replace(self, GuiContext::Invalid);
        *self = match context {
            GuiContext::Active(context) => GuiContext::Suspended(context.suspend()),
            _ => context,
        };
    }
    fn activate(&mut self) -> Result<&mut imgui::Context, ()> {
        let context = std::mem::replace(self, GuiContext::Invalid);
        *self = match context {
            GuiContext::Suspended(context) => match context.activate() {
                Ok(context) => GuiContext::Active(context),
                Err(context) => GuiContext::Suspended(context),
            },
            _ => context,
        };
        match self {
            GuiContext::Active(context) => Ok(context),
            _ => Err(()),
        }
    }
}

struct Gui {
    imgui: GuiContext,
    platform: WinitPlatform,
    renderer: Renderer,
    last_frame: Instant,
    focused: bool,
}
impl Gui {
    fn init<F>(
        display: &mut Display,
        back: &mut WinitGlBackend,
        mut f: F,
    ) -> Result<Self, WinitGlError>
    where
        F: FnMut(&mut imgui::Context, GuiInfo) -> Result<(), ()>,
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        let hidpi_factor = window.get_hidpi_factor();

        let atlas = back.font_atlas(hidpi_factor);
        //let mut imgui = imgui::Context::create();
        let mut imgui = imgui::Context::create_with_shared_font_atlas(atlas);
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);

        //let hidpi_factor = platform.hidpi_factor();
        let info = GuiInfo { hidpi_factor };
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        f(&mut imgui, info).map_err(|_| WinitGlError::ImGuiInit)?;

        let mut renderer =
            Renderer::init(&mut imgui, &*display).map_err(WinitGlError::GliumRenderer)?;

        let imgui = GuiContext::Suspended(imgui.suspend());
        Ok(Gui {
            imgui,
            platform,
            renderer,
            last_frame: Instant::now(),
            focused: false,
        })
    }
    fn draw<F>(&mut self, display: &Display, mut run_ui: F) -> Result<(), WinitGlError>
    where
        F: FnMut(&Ui, &Rc<Context>, &mut ImGuiTextures<WinitGlBackend>) -> Vec<imgui::TextureId>,
    {
        let used_textures;
        let draw_data = {
            let imgui = self.imgui.activate().expect("Can't activate context");

            let gl_window = display.gl_window();
            let window = gl_window.window();

            let io = imgui.io_mut();
            self.platform
                .prepare_frame(io, &window)
                .map_err(WinitGlError::ImGuiPrepareFrame)?;
            self.last_frame = io.update_delta_time(self.last_frame);

            let ui = imgui.frame();

            used_textures = run_ui(&ui, display.get_context(), self.renderer.textures());
            //if self.focused {
            self.platform.prepare_render(&ui, &window);
            //}
            let draw_data = ui.render();

            draw_data
        };

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        self.renderer
            .render(&mut target, draw_data)
            .map_err(WinitGlError::GliumRenderer)?;
        {
            let textures = self.renderer.textures();
            let context = display.get_context();
            for texture in used_textures {
                if let Some(texture) = textures.remove(texture) {
                    WinitGlBackend::drop_texture(context, &*texture);
                }
            }
            *self.renderer.textures() = imgui::Textures::new();
        }
        self.imgui.suspend();
        target.finish().map_err(WinitGlError::SwapBuffers)
    }
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
