//use crate::game_window::GameWindow;
use sdl2::video::Window;

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

/*
fn create_window(
    video_subsystem: &VideoSubsystem,
    rect: &Rect,
    shaped: bool,
) -> Result<Window, WindowBuildError> {
    use sdl2::sys::SDL_WindowFlags;

    video_subsystem
        .window("FOnlineOverlay", rect.width(), rect.height())
        .set_window_flags(
            SDL_WindowFlags::SDL_WINDOW_BORDERLESS as u32
                | SDL_WindowFlags::SDL_WINDOW_ALWAYS_ON_TOP as u32
                | SDL_WindowFlags::SDL_WINDOW_RESIZABLE as u32
                | SDL_WindowFlags::SDL_WINDOW_SKIP_TASKBAR as u32,
        )
        .position(rect.x(), rect.y())
        .build()
}

fn make_transparent(window: &Window) -> Result<(), String> {
    use winapi::{
        shared::windef,
        um::{errhandlingapi as err, wingdi, winuser},
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

        if winuser::SetLayeredWindowAttributes(
            handle,
            color_key,
            255,
            winuser::LWA_COLORKEY,
            //winuser::LWA_ALPHA,
        ) == 0
        {
            return Err(format!("winuser::SetLayeredWindowAttributes"));
        }
    };
    Ok(())
}

pub fn update_visibility(game_window: &GameWindow, window: &mut Window, hide: bool) {
    use winapi::um::winuser;

    if hide {
        window.hide();
        return;
    }

    let game_window = game_window.raw();
    let handle = get_winapi_handle(window).0;
    let focus = unsafe { winuser::GetForegroundWindow() };
    //dbg!(game_window);
    //dbg!(handle);
    //dbg!(focus);
    let is_focused = focus == game_window || focus == handle;
    let is_visible = unsafe { winuser::IsWindowVisible(handle) != 0 };
    if is_focused != is_visible {
        if is_visible {
            window.hide();
        } else {
            window.show();
        }
    }
}
*/
