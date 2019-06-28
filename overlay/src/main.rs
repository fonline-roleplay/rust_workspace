mod overlay;
use overlay::Overlay;

mod avatar_window;
mod image_data;
mod windowing;

mod bridge;
mod game_window;
use game_window::GameWindow;

mod downloader;
mod reqres;
mod win_tools;

fn start(game_window: GameWindow, url: String) {
    let mut bridge = bridge::start();
    let requester = downloader::start(url);
    game_window.to_foreground();

    let mut game = Overlay::new(game_window, bridge, requester);
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

#[derive(Debug)]
pub enum SdlError {
    WindowBuild(sdl2::video::WindowBuildError),
    TextureFromSurface(sdl2::render::TextureValueError),
    TextureCopy(String),
    SurfaceFromData(String),
    CanvasBuild(sdl2::IntegerOrSdlError),
}
