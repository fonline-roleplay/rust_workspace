use crate::game_window::GameWindow;
use viewports::{Viewport, ViewportFlags, WindowSpawner};
use winit::{
    event_loop::EventLoopWindowTarget,
    platform::windows::WindowExtWindows,
    window::{Window, WindowBuilder},
};

use winapi::{
    shared::windef,
    um::winuser,
};
pub(crate) struct OverlaySpawner(GameWindow);
impl OverlaySpawner {
    pub(crate) fn new(game_window: GameWindow) -> Self {
        Self(game_window)
    }
}

impl<V: Viewport> WindowSpawner<V> for OverlaySpawner {
    fn build_window<T: 'static>(
        &mut self,
        event_loop: &EventLoopWindowTarget<T>,
        flags: ViewportFlags,
    ) -> Window {
        let decorations = !flags.contains(ViewportFlags::NO_DECORATIONS);
        let window = WindowBuilder::new()
            .with_decorations(decorations)
            .with_visible(false)
            //.with_always_on_top(true)
            .build(event_loop)
            .unwrap();
        dbg!(flags);
        make_window_popup(&window).unwrap();
        reparent(&window, self.0.raw());
        //window.set_visible(true);
        window
    }
    fn show_window(&mut self, viewport: &V) {
        show_no_activate(viewport.window());
        //viewport.window().set_visible(true);
    }
}

#[cfg(target_pointer_width = "32")]
type LongOrPtr = i32;
#[cfg(target_pointer_width = "64")]
type LongOrPtr = isize;

fn make_window_popup(window: &Window) -> Result<(), String> {
    let handle = window.hwnd() as _;

    //window.hide_cursor(true);

    unsafe {
        let mut flags = winuser::GetWindowLongPtrA(handle, winuser::GWL_STYLE);
        if flags == 0 {
            return Err(format!("winuser::GetWindowLongPtrA"));
        }
        flags |= winuser::WS_POPUP as LongOrPtr;
        if winuser::SetWindowLongPtrA(handle, winuser::GWL_STYLE, flags) == 0 {
            return Err(format!("winuser::SetWindowLongPtrA"));
        }

        let mut flags = winuser::GetWindowLongPtrA(handle, winuser::GWL_EXSTYLE);
        if flags == 0 {
            return Err(format!("winuser::GetWindowLongPtrA"));
        }
        flags |= winuser::WS_EX_NOACTIVATE as LongOrPtr;
        flags &= !winuser::WS_EX_APPWINDOW as LongOrPtr;
        if winuser::SetWindowLongPtrA(handle, winuser::GWL_EXSTYLE, flags) == 0 {
            return Err(format!("winuser::SetWindowLongPtrA"));
        }
    }
    Ok(())
}

fn reparent(window: &Window, owner: windef::HWND) {
    let handle = window.hwnd() as _;
    unsafe {
        winuser::SetWindowLongPtrA(handle, winuser::GWL_HWNDPARENT, owner as _);
    }
}

fn show_no_activate(window: &Window) {
    let handle = window.hwnd() as _;
    unsafe {
        winuser::ShowWindow(handle, winuser::SW_SHOWNA);
    }
}

fn _make_window_border(window: &Window) -> Result<(), String> {
    let handle = window.hwnd() as _;

    unsafe {
        let mut flags = winuser::GetWindowLongPtrA(handle, winuser::GWL_STYLE);
        if flags == 0 {
            return Err(format!("winuser::GetWindowLongPtrA"));
        }
        flags |= winuser::WS_BORDER as LongOrPtr;
        if winuser::SetWindowLongPtrA(handle, winuser::GWL_STYLE, flags) == 0 {
            return Err(format!("winuser::SetWindowLongPtrA"));
        }

        let mut flags = winuser::GetWindowLongPtrA(handle, winuser::GWL_EXSTYLE);
        if flags == 0 {
            return Err(format!("winuser::GetWindowLongPtrA"));
        }
        flags |= winuser::WS_EX_WINDOWEDGE as LongOrPtr;
        if winuser::SetWindowLongPtrA(handle, winuser::GWL_EXSTYLE, flags) == 0 {
            return Err(format!("winuser::SetWindowLongPtrA"));
        }
    }
    Ok(())
}

/*
pub(crate) struct Cursor(windef::HCURSOR);

impl Cursor {
    pub(crate) fn from_file(file_path: &std::ffi::CStr) -> Option<Cursor> {
        unsafe {
            let ptr = winuser::LoadCursorFromFileA(file_path.as_ptr());
            if ptr.is_null() {
                return None;
            }
            Some(Cursor(ptr))
        }
    }
    pub(crate) fn activate(&self) {
        unsafe {
            winuser::SetCursor(self.0);
        }
    }
}
*/