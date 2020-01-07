#![windows_subsystem = "windows"]

use clap::{App, Arg, SubCommand};

mod overlay;
use overlay::Overlay;

mod avatar_window;
mod bridge;
mod game_window;
mod image_data;
mod ui_window;
mod windowing;
use game_window::GameWindow;
use std::time::Duration;

mod downloader;
mod reqres;
mod win_tools;

mod profile;

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
    let matches = App::new("FOnline Overlay")
        .author("qthree <qthree3@gmail.com>")
        .arg(
            Arg::with_name("pid")
                .help("Sets the client process id")
                .long("pid")
                .value_name("PID")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("wait")
                .help("Wait for client window")
                .short("w"),
        )
        .arg(
            Arg::with_name("URL")
                .help("Sets the web server address")
                .required(true)
                .index(1),
        )
        .get_matches();
    let url = matches
        .value_of("URL")
        .expect("Pass web server address as argument.") //unreachable?
        .to_owned();
    let pid = matches
        .value_of("pid")
        .and_then(|pid| pid.parse::<u32>().ok());
    let wait = matches.is_present("wait");

    /*let url = std::env::args()
    .nth(1)
    .expect("Pass web server address as argument."); //.unwrap_or("localhost:8000".into());
    */
    let gui_thread = std::thread::spawn(move || {
        let game_window = loop {
            let res = if let Some(pid) = pid {
                GameWindow::from_pid(pid)
            } else {
                GameWindow::find()
            };
            if res.is_none() && wait {
                std::thread::sleep(Duration::from_secs(1));
                continue;
            } else {
                break res;
            }
        };
        if let Some(game_window) = game_window {
            start(game_window, url);
        } else {
            println!("Can't find window.");
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
