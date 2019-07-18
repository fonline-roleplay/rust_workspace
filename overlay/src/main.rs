mod overlay;
use overlay::Overlay;

mod avatar_window;
mod bridge;
mod game_window;
mod image_data;
mod ui_window;
mod windowing;
use game_window::GameWindow;

mod downloader;
mod reqres;
mod win_tools;

mod backend;
#[cfg(feature = "backend-sdl")]
type DefaultBackend = backend::sdl::SdlBackend;

#[cfg(feature = "backend-winit-gl")]
type DefaultBackend = backend::winit_gl::WinitGlBackend;

fn start(game_window: GameWindow, url: String) {
    let mut bridge = bridge::start();
    let requester = downloader::start(url);
    game_window.to_foreground();

    let mut game = Overlay::<DefaultBackend>::new(game_window, bridge, requester);
    game.run();
}

fn main() {
    let url = std::env::args()
        .nth(1)
        .expect("Pass web server address as argument."); //.unwrap_or("localhost:8000".into());
    let gui_thread = std::thread::spawn(|| {
        if let Some(game_window) = GameWindow::find() {
            start(game_window, url);
        }
    });
    gui_thread.join().expect("graceful exit");
}

#[derive(Debug, PartialEq)]
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
