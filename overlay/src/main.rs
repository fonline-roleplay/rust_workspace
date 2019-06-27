use sdl2::{
    VideoSubsystem,
    pixels::{Color, PixelFormatEnum},
    event::{Event, WindowEvent},
    keyboard::Keycode,
    rect::Rect,
    render::{BlendMode, Texture, Canvas},
    surface::{Surface, SurfaceRef},
    video::{Window, DisplayMode, WindowBuildError},
    sys::{SDL_SetWindowShape, SDL_WindowShapeMode, WindowShapeMode, SDL_WindowShapeParams, SDL_Window},
};
use std::{
    time::Duration,
    collections::BTreeMap,
};

mod bridge;
use bridge::{MsgIn, MsgOut, Avatar, Char, Position, BridgeOverlayToClient};

mod game_window;
mod reqres;

mod downloader;
use downloader::ImageRequester;

fn create_window(video_subsystem: VideoSubsystem, game: &Game, shaped: bool) -> Result<Window, WindowBuildError> {
    use sdl2::sys::{
        SDL_WindowFlags,
    };

    video_subsystem.window("FOnlineOverlay", game.rect.width(), game.rect.height()).set_window_flags(
        SDL_WindowFlags::SDL_WINDOW_BORDERLESS as u32 |
        SDL_WindowFlags::SDL_WINDOW_ALWAYS_ON_TOP as u32 |
        SDL_WindowFlags::SDL_WINDOW_SKIP_TASKBAR as u32
    ).position(game.rect.x(), game.rect.y()).build()
}

fn make_transparent(window: &Window, game: &Game) -> Result<(), String>{
    use winapi::{
        um::{winuser, wingdi, errhandlingapi as err},
        shared::windef,
    };

    let handle = get_winapi_handle(window).0;
    unsafe {
        let hdc = winuser::GetDC(handle);
        if hdc as usize == 0 {
            return Err(format!("winuser::GetDC"));
        }

        let mut flags = winuser::GetWindowLongPtrA(handle, winuser::GWL_EXSTYLE);
        flags |= winuser::WS_EX_LAYERED as i32;
        //flags |= winuser::WS_EX_TRANSPARENT as i32;
        if winuser::SetWindowLongPtrA(handle, winuser::GWL_EXSTYLE, flags) == 0 {
            return Err(format!("winuser::SetWindowLongPtrA"));
        }
        let color_key = wingdi::RGB(0, 0, 0);

        if winuser::SetLayeredWindowAttributes (
            handle,
            color_key,
            255,
            winuser::LWA_COLORKEY,
            //winuser::LWA_ALPHA,
        ) == 0 {
            return Err(format!("winuser::SetLayeredWindowAttributes"));
        }
    };
    Ok(())
}

fn get_winapi_handle(window: &Window) -> (winapi::shared::windef::HWND, winapi::shared::windef::HDC) {
    use sdl2::sys::{
        SDL_SysWMinfo, SDL_GetVersion, SDL_GetWindowWMInfo,
    };
    unsafe {
        let mut wmInfo: SDL_SysWMinfo = std::mem::zeroed();
        SDL_GetVersion(&mut wmInfo.version);
        SDL_GetWindowWMInfo(window.raw(), &mut wmInfo);
        (wmInfo.info.win.window as _, wmInfo.info.win.hdc as _)
    }
}

pub fn update_visibility(game_window: &GameWindow, window: &mut Window, hide: bool) {
    use winapi::{
        um::winuser,
    };

    if hide {
        window.hide();
        return;
    }

    let game_window = game_window.raw();
    let handle = get_winapi_handle(window).0;
    let focus = unsafe {
        winuser::GetForegroundWindow()
    };
    //dbg!(game_window);
    //dbg!(handle);
    //dbg!(focus);
    let is_focused = focus == game_window || focus == handle;
    let is_visible = unsafe {
        winuser::IsWindowVisible(handle) != 0
    };
    if is_focused != is_visible {
        if is_visible {
            window.hide();
        } else {
            window.show();
        }
    }
}

pub use game_window::GameWindow;
use sdl2::pixels::PixelFormat;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;

fn main() {
    let url = std::env::args().nth(1).expect("Pass web server address as argument."); //.unwrap_or("localhost:8000".into());
    let gui_thread = std::thread::spawn(|| {
        if let Some(game_window) = GameWindow::find() {
            start(game_window, url);
        }
    });
    gui_thread.join().expect("graceful exit");
}

struct Game {
    rect: Rect,
    avatars: Vec<Avatar>,
    hide: bool,
    //images: BTreeMap<Char, AvatarImage>,
    frame: u64,
    dirty: bool,
    bridge: BridgeOverlayToClient,
    requester: ImageRequester,
}

impl Game {
    pub fn new(rect: Rect, bridge: BridgeOverlayToClient, requester: ImageRequester) -> Self {
        Game{
            rect,
            avatars: vec![],
            //images: BTreeMap::new(),
            frame: 0,
            hide: false,
            dirty: true,
            bridge,
            requester,
        }
    }
    fn update(&mut self) {
        //let loop_frame = self.frame % 360;
        /*let angle = (loop_frame as f32).to_radians();
        //dbg!(angle);
        for (i, avatar) in self.avatars.iter_mut().enumerate() {
            let x = i as i32 % 10;
            let y = i as i32 / 10;

            *avatar = (x*100 + (angle.cos() * 50.0) as i32, y*100 + (angle.sin() * 50.0) as i32)
        }*/
        self.frame += 1;
    }
}

enum AvatarImage<'a> {
    //Image(Image),
    Texture(Texture<'a>),
    Error(downloader::DownloaderError),
}

fn start(game_window: GameWindow, url: String) {
    let mut bridge = bridge::start();
    let requester = downloader::start(url);
    game_window.to_foreground();
    let mut game = Game::new(
        game_window.rect().expect("game window rect"),
        bridge,
        requester,
    );

    let sdl_context = sdl2::init().unwrap();

    let drivers: Vec<_> = sdl2::video::drivers().collect();
    println!("Available drivers: {:?}", drivers);

    let video_subsystem = sdl_context.video().unwrap();

    println!("Current driver: {:?}", video_subsystem.current_video_driver());

    let mut window = create_window(video_subsystem, &game, false).expect("created window");

    make_transparent(&window, &game).expect("window been transparent");

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut images: BTreeMap<Char, AvatarImage> = BTreeMap::new();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        if game.frame % 1 == 0 {
            if let Some(new_pos) = game_window.window_pos() {
                canvas.window_mut().set_position(new_pos.0, new_pos.1);
            } else {
                println!("Window closed");
                break 'running;
            }
            update_visibility(&game_window, canvas.window_mut(), game.hide);
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::Window {win_event: WindowEvent::Enter, ..} => {
                    sdl_context.mouse().show_cursor(false);
                },
                Event::Window {win_event: WindowEvent::Leave, ..} => {
                    //sdl_context.mouse().show_cursor(true);
                },
                _ => {}
            }
        }

        if game.bridge.is_online() {
            let _ = game.bridge.ping();

            let mut last_avatars = None;

            for msg in game.bridge.receive() {
                match msg {
                    MsgIn::UpdateAvatars(avatars) => {
                        last_avatars = Some(avatars);
                    },
                    MsgIn::OverlayHide(hide) => {
                        game.hide = hide;
                        game.dirty = true;
                    }
                }
            }

            if let Some(avatars) = last_avatars {
                if game.avatars != avatars {
                    game.avatars = avatars;
                    if !game.hide {
                        game.dirty = true;
                    }
                }
            }
        }

        if game.dirty {
            redraw(&mut game, &mut canvas, &texture_creator, &mut images);
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 240));
        game.update();
    }
    game.bridge.finish(true);
}


fn redraw<'a>(game: &mut Game, canvas: &mut Canvas<Window>,
          texture_creator: &'a TextureCreator<WindowContext>,
          images: &mut BTreeMap<Char, AvatarImage<'a>>) {
    let game_rect = Rect::new(0, 0, game.rect.width(), game.rect.height());

    canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
    canvas.clear();

    if !game.hide && !game.avatars.is_empty() {
        let mut visible_avatars = Vec::with_capacity(game.avatars.len());
        let mut requester_free = game.requester.is_free();
        if requester_free {
            if let Some((for_char, new_image)) = game.requester.receive() {
                match new_image {
                    Ok(image) => {
                        let width = image.width();
                        let height = image.height();
                        let pitch = width*3;
                        let mut bytes = image.into_raw();
                        let surface = Surface::from_data(&mut bytes, width, height, pitch, PixelFormatEnum::RGB24).expect("loaded surface");
                        let texture = texture_creator.create_texture_from_surface(&surface).expect("created texture");
                        images.insert(for_char, AvatarImage::Texture(texture));
                    },
                    Err(err) => {
                        images.insert(for_char, AvatarImage::Error(err));
                    }
                }
            }
        }
        for avatar in &game.avatars {
            match images.get(&avatar.char) {
                Some(image) => {
                    match &image {
                        AvatarImage::Texture(texture) => {
                            visible_avatars.push((texture, avatar.pos));
                        },
                        AvatarImage::Error(_) => {
                            // TODO: Implement error recovery
                        },
                    }
                },
                None => {
                    if requester_free {
                        game.requester.send(avatar.char);
                        requester_free = false;
                    }
                }
            }
        }

        let texture_rect = Rect::new(0, 0, 128, 128);
        for (texture, position) in visible_avatars {
            let avatar_rect = Rect::new(position.x-32, position.y-64-16, 64, 64);
            if let Some(_intersection) = game_rect.intersection(avatar_rect) {
                canvas.copy(texture, texture_rect, avatar_rect).expect("successfully copy texture");
            }
        }
    }
    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.draw_rect(game_rect);
    canvas.present();
}