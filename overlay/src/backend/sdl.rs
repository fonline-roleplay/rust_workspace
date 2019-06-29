use crate::avatar_window::AvatarWindow;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    rect::Rect as SdlRect,
    render::{Canvas, Texture, TextureCreator},
    surface::Surface,
    video::{Window, WindowContext, WindowPos},
    //event::{Event, WindowEvent},
    //keyboard::Keycode,
    EventPump,
    VideoSubsystem,
};

use super::*;
use imgui::Image;

pub struct SdlBackend {
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: VideoSubsystem,
    pub event_pump: EventPump,
}
impl Backend for SdlBackend {
    type Window = SdlWindow;
    fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();

        let drivers: Vec<_> = sdl2::video::drivers().collect();
        println!("Available drivers: {:?}", drivers);

        let video_subsystem = sdl_context.video().unwrap();

        println!(
            "Current driver: {:?}",
            video_subsystem.current_video_driver()
        );

        let event_pump = sdl_context.event_pump().expect("event pump");

        SdlBackend {
            sdl_context,
            video_subsystem,
            event_pump,
        }
    }
    fn new_window(&self) -> Result<Self::Window, BackendError<Self>> {
        SdlWindow::new(&self.video_subsystem)
    }
    fn poll_events(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return true;
                }
                //Event::Window {win_event: WindowEvent::Enter, ..} => {
                //    sdl_context.mouse().show_cursor(false);
                //},
                //Event::Window {win_event: WindowEvent::Leave, ..} => {
                //    //sdl_context.mouse().show_cursor(true);
                //},
                _ => {}
            }
        }
        false
    }
}

pub struct SdlWindow {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
}

impl BackendWindow for SdlWindow {
    type Texture = sdl2::render::Texture;
    type Error = SdlError;

    fn show(&mut self) {
        self.window_mut().show()
    }
    fn hide(&mut self) {
        self.window_mut().hide()
    }
    fn set_position(&mut self, x: i32, y: i32) {
        let x = WindowPos::Positioned(x);
        let y = WindowPos::Positioned(y);
        let window = self.window_mut().set_position(x, y);
    }
    fn create_texture(&mut self, image: &mut ImageData) -> Result<Self::Texture, Self::Error> {
        let surface = image_data_to_surface(image)?;
        self.texture_creator
            .create_texture_from_surface(&surface)
            .map_err(SdlError::TextureFromSurface)
    }
    fn draw_texture(
        &mut self,
        texture: &Self::Texture,
        src: &Rect,
        dst: &Rect,
    ) -> Result<(), Self::Error> {
        let src = sdl2rect(src);
        let dst = sdl2rect(dst);

        self.canvas
            .copy(texture, src, dst)
            .map_err(SdlError::TextureCopy)?;
        self.canvas.present();
        Ok(())
    }
    fn drop_texture(&mut self, texture: Self::Texture) {
        unsafe {
            texture.destroy();
        }
    }
    fn handle(&self) -> *mut () {
        get_winapi_handle(self.canvas.window()).0 as _
    }
}

fn sdl2rect(rect: &Rect) -> SdlRect {
    SdlRect::new(rect.x, rect.y, rect.width, rect.height)
}

fn image_data_to_surface(image: &mut ImageData) -> Result<Surface, SdlError> {
    Surface::from_data(
        &mut image.bytes,
        image.width,
        image.height,
        image.pitch,
        PixelFormatEnum::RGB24,
    )
    .map_err(SdlError::SurfaceFromData)
}

impl SdlWindow {
    fn new(video_subsystem: &VideoSubsystem) -> Result<Self, SdlError> {
        use sdl2::sys::SDL_WindowFlags;
        let window = video_subsystem
            .window("FOnlineOverlay", 64, 64)
            .opengl()
            .set_window_flags(
                SDL_WindowFlags::SDL_WINDOW_BORDERLESS as u32
                    | SDL_WindowFlags::SDL_WINDOW_ALWAYS_ON_TOP as u32
                    | SDL_WindowFlags::SDL_WINDOW_SKIP_TASKBAR as u32,
            )
            .hidden()
            .build()
            .map_err(SdlError::WindowBuild)?;
        let canvas = window
            .into_canvas()
            .build()
            .map_err(SdlError::CanvasBuild)?;
        println!("Window canvas driver: {:?}", canvas.info().name);
        let texture_creator = canvas.texture_creator();
        Ok(SdlWindow {
            canvas,
            texture_creator,
        })
    }
    fn window(&self) -> &Window {
        self.canvas.window()
    }
    fn window_mut(&mut self) -> &mut Window {
        self.canvas.window_mut()
    }
}

#[derive(Debug)]
pub enum SdlError {
    WindowBuild(sdl2::video::WindowBuildError),
    TextureFromSurface(sdl2::render::TextureValueError),
    TextureCopy(String),
    SurfaceFromData(String),
    CanvasBuild(sdl2::IntegerOrSdlError),
}

pub fn get_winapi_handle(
    window: &Window,
) -> (winapi::shared::windef::HWND, winapi::shared::windef::HDC) {
    use sdl2::sys::{SDL_GetVersion, SDL_GetWindowWMInfo, SDL_SysWMinfo};
    unsafe {
        let mut wmInfo: SDL_SysWMinfo = std::mem::zeroed();
        SDL_GetVersion(&mut wmInfo.version);
        SDL_GetWindowWMInfo(window.raw(), &mut wmInfo);
        (wmInfo.info.win.window as _, wmInfo.info.win.hdc as _)
    }
}
