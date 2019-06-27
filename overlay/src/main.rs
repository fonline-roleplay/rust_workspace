use sdl2::{
    VideoSubsystem,
    pixels::{Color, PixelFormatEnum, PixelFormat},
    event::{Event, WindowEvent},
    keyboard::Keycode,
    rect::Rect,
    render::{BlendMode, Texture, Canvas, TextureCreator},
    surface::{Surface, SurfaceRef, SurfaceContext},
    video::{Window, DisplayMode, WindowBuildError, WindowContext, WindowPos},
    sys::{SDL_SetWindowShape, SDL_WindowShapeMode, WindowShapeMode, SDL_WindowShapeParams, SDL_Window},
};
use std::{
    time::Duration,
    collections::BTreeMap,
};

mod bridge;
use bridge::{MsgIn, MsgOut, Avatar, Char, Position, BridgeOverlayToClient};

mod game_window;
pub use game_window::GameWindow;

mod reqres;

mod downloader;
use downloader::ImageRequester;

fn create_window(video_subsystem: &VideoSubsystem, game: &Game, shaped: bool) -> Result<Window, WindowBuildError> {
    use sdl2::sys::{
        SDL_WindowFlags,
    };

    video_subsystem.window("FOnlineOverlay", game.rect.width(), game.rect.height()).set_window_flags(
        SDL_WindowFlags::SDL_WINDOW_BORDERLESS as u32 |
        SDL_WindowFlags::SDL_WINDOW_ALWAYS_ON_TOP as u32 |
        SDL_WindowFlags::SDL_WINDOW_RESIZABLE as u32 |
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


        let mut flags = winuser::GetWindowLongPtrA(handle, winuser::GWL_STYLE);
        flags |= winuser::WS_POPUP as i32;
        if winuser::SetWindowLongPtrA(handle, winuser::GWL_STYLE, flags) == 0 {
            return Err(format!("winuser::SetWindowLongPtrA"));
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

fn is_foreground(game_window: &GameWindow, game: &Game) -> bool {
    use winapi::{
        um::winuser,
    };

    let game_window = game_window.raw();
/*
    let mut focus = unsafe {
        winuser::GetForegroundWindow()
    };

    loop {
        println!("focus: {}", focus as usize);
        if focus as usize == 0 {
            return false;
        }
        if focus == game_window {
            return true;
        }
        let flags = unsafe { winuser::GetWindowLongPtrA(focus, winuser::GWL_EXSTYLE) };
        if (flags as u32 & winuser::WS_EX_TOPMOST) == 0 {
            return false;
        }
        focus = unsafe { winuser::GetWindow(focus, winuser::GW_HWNDNEXT) };
    }*/

    //focus = unsafe { winuser::GetWindow(focus, winuser::GW_HWNDPREV) };

    let focus = unsafe {
        winuser::GetForegroundWindow()
    };
    if focus as usize == 0 {
        return false;
    }
    if focus == game_window {
        return true;
    }
    for window in game.windowing.windows.values() {
        let handle = get_winapi_handle(window.window()).0;
        if handle == focus {
            return true;
        }
    }
    false
}

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
    dirty: bool,
    is_foreground: bool,
    images: BTreeMap<Char, AvatarImage>,
    frame: u64,
    bridge: BridgeOverlayToClient,
    requester: ImageRequester,
    windowing: Windowing,
}

impl Game {
    pub fn new(rect: Rect, bridge: BridgeOverlayToClient, requester: ImageRequester) -> Self {
        Game{
            rect,
            avatars: vec![],
            images: BTreeMap::new(),
            frame: 0,
            hide: false,
            dirty: true,
            is_foreground: true,
            bridge,
            requester,
            windowing: Windowing::new(),
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

struct Windowing{
    windows: BTreeMap<u32, AvatarWindow>,
    sdl_context: sdl2::Sdl,
    video_subsystem: VideoSubsystem,
}
impl Windowing {
    fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();

        let drivers: Vec<_> = sdl2::video::drivers().collect();
        println!("Available drivers: {:?}", drivers);

        let video_subsystem = sdl_context.video().unwrap();

        println!("Current driver: {:?}", video_subsystem.current_video_driver());
        Windowing{
            windows: BTreeMap::new(),
            sdl_context,
            video_subsystem,
        }
    }
    fn window_for_char(&mut self, char: u32) -> Result<&mut AvatarWindow, SdlError> {
        use std::collections::btree_map::Entry;
        /*match self.windows.entry(char) {
            Entry::Occupied(mut window) => {
                return Ok(window.get_mut());
            },
            Entry::Vacant(vacant) => {
                let window = AvatarWindow::new(&self.video_subsystem, char)?;
                return Ok(vacant.insert(window));
            }
        }*/
        if let Entry::Vacant(vacant) = self.windows.entry(char) {
            let window = AvatarWindow::new(&self.video_subsystem)?;
            vacant.insert(window);
        }
        Ok(self.windows.get_mut(&char).unwrap())
    }
}

enum AvatarImage {
    Image(ImageData),
    //Texture(Texture<'a>),
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


/*
    let mut window = create_window(&video_subsystem, &game, false).expect("created window");

    make_transparent(&window, &game).expect("window been transparent");

    let mut canvas = window.into_canvas().build().unwrap();

    println!("Canvas driver: {:?}", &canvas.info().name);

    let texture_creator = canvas.texture_creator();
*/
    //let mut images: BTreeMap<Char, AvatarImage> = BTreeMap::new();

    let mut event_pump = game.windowing.sdl_context.event_pump().unwrap();

    'running: loop {
        if game.frame % 1 == 0 {
            if let Some(new_rect) = game_window.rect() { //.window_pos() {
                if new_rect != game.rect {
                    game.rect = new_rect;
                    if !game.hide {
                        game.dirty = true;
                    }
                }
                let is_foreground = is_foreground(&game_window, &game);
                if is_foreground != game.is_foreground {
                    game.is_foreground = is_foreground;
                    game.dirty = true;
                }
                //canvas.window_mut().set_position(new_pos.0, new_pos.1);
            } else {
                println!("Window closed");
                break 'running;
            }
            //update_visibility(&game_window, canvas.window_mut(), game.hide);
        }

        //if game.frame % 50 == 0 {
        //    canvas.window_mut().set_size(400+game.frame as u32 % 500, 400 ).expect("resize");
        //}

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                //Event::Window {win_event: WindowEvent::Enter, ..} => {
                //    sdl_context.mouse().show_cursor(false);
                //},
                //Event::Window {win_event: WindowEvent::Leave, ..} => {
                //    //sdl_context.mouse().show_cursor(true);
                //},
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
            redraw(&mut game);
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 240));
        game.update();
    }
    game.bridge.finish(true);
}


struct AvatarWindow {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    char_texture: Option<(Char, Texture)>,
    last_frame: u64,
    hidden: bool,
    pos_x: WindowPos,
    pos_y: WindowPos,
}

enum SdlError {
    WindowBuild(sdl2::video::WindowBuildError),
    TextureFromSurface(sdl2::render::TextureValueError),
    TextureCopy(String),
    SurfaceFromData(String),
    CanvasBuild(sdl2::IntegerOrSdlError),
}

impl AvatarWindow {
    fn new(video_subsystem: &VideoSubsystem) -> Result<Self, SdlError> {
        use sdl2::sys::{
            SDL_WindowFlags,
        };
        let window = video_subsystem.window("FOnlineOverlay", 64, 64).set_window_flags(
            SDL_WindowFlags::SDL_WINDOW_BORDERLESS as u32 |
                SDL_WindowFlags::SDL_WINDOW_ALWAYS_ON_TOP as u32 |
                SDL_WindowFlags::SDL_WINDOW_SKIP_TASKBAR as u32
        )
            .hidden()
            .build()
            .map_err(SdlError::WindowBuild)?;
        let canvas = window.into_canvas().build().map_err(SdlError::CanvasBuild)?;
        let texture_creator = canvas.texture_creator();
        Ok(AvatarWindow{canvas, texture_creator, char_texture: None, last_frame: 0,
            hidden: true,
            pos_x: WindowPos::Undefined,
            pos_y: WindowPos::Undefined,
        })
    }
    fn update(&mut self, char: Char, image: &mut ImageData) -> Result<(), SdlError> {
        if let Some((old_char, _)) = &self.char_texture {
            if *old_char == char {
                return Ok(());
            } else {
                let (_, old_texture) = self.char_texture.take().unwrap();
                unsafe { old_texture.destroy(); }
            }
        }
        let surface = image.surface()?;
        let texture = self.texture_creator.create_texture_from_surface(&surface).map_err(SdlError::TextureFromSurface)?;
        self.char_texture = Some((char, texture));
        Ok(())
    }
    fn draw(&mut self) -> Result<(), SdlError> {
        if let Some((_, texture)) = &self.char_texture {
            let src = Rect::new(0, 0, 128, 128);
            let dst = Rect::new(0, 0, 64, 64);
            self.canvas.copy(texture, src, dst).map_err(SdlError::TextureCopy)?;
            self.canvas.present();
        }
        Ok(())
    }
    fn show(&mut self) {
        if self.hidden {
            self.hidden = false;
            self.window_mut().show();
        }
    }
    fn hide(&mut self) {
        if !self.hidden {
            self.hidden = true;
            self.window_mut().hide();
        }
    }
    fn set_position(&mut self, x: i32, y: i32) {
        let x = WindowPos::Positioned(x);
        let y = WindowPos::Positioned(y);
        if self.pos_x != x {
            self.pos_x = x;
        } else if self.pos_y != y {
            self.pos_y = y;
        } else {
            return;
        }
        let window = self.window_mut();
        window.set_position(x, y);
    }
    fn window(&self) -> &Window {
        self.canvas.window()
    }
    fn window_mut(&mut self) -> &mut Window {
        self.canvas.window_mut()
    }
}

struct ImageData {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    pitch: u32,
}

impl ImageData {
    fn new(image: downloader::Image) -> Self {
        ImageData {
            width: image.width(),
            height: image.height(),
            pitch: image.width()*3,
            bytes: image.into_raw(),
        }
    }
    fn surface(&mut self) -> Result<Surface, SdlError> {
        Surface::from_data(&mut self.bytes, self.width, self.height, self.pitch, PixelFormatEnum::RGB24).map_err(SdlError::SurfaceFromData)
    }
}

fn redraw<'a>(game: &mut Game) {
    let game_rect = Rect::new(0, 0, game.rect.width(), game.rect.height());

    let mut requester_free = game.requester.is_free();
    if requester_free {
        if let Some((for_char, new_image)) = game.requester.receive() {
            match new_image {
                Ok(image) => {
                    game.images.insert(for_char, AvatarImage::Image(ImageData::new(image)));
                },
                Err(err) => {
                    game.images.insert(for_char, AvatarImage::Error(err));
                }
            }
        }
    }

    if game.is_foreground && !game.hide && !game.avatars.is_empty() {
        //let mut visible_avatars = Vec::with_capacity(game.avatars.len());

        for avatar in &game.avatars {
            match game.images.get_mut(&avatar.char) {
                Some(image) => {
                    match image {
                        AvatarImage::Image(image) => {
                            //visible_avatars.push((image, avatar.pos));
                            if let Ok(window) = game.windowing.window_for_char(avatar.char.id) {
                                window.update(avatar.char, image);
                                window.last_frame = game.frame;
                                let x = avatar.pos.x - 32;
                                let y = avatar.pos.y - 64 - 16;
                                if (game.rect.width() as i32 - 64 > x &&  x > 0)
                                    && (game.rect.height() as i32 - 64 > y && y > 0)
                                {
                                    window.set_position(game.rect.x() + x, game.rect.y() + y);
                                    window.show();
                                    window.draw();
                                } else {
                                    window.hide();
                                }
                            }
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
    }

    for (char, window) in &mut game.windowing.windows {
        if window.last_frame != game.frame {
            window.hide();
        }
    }
}
