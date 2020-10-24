use crate::game_window::GameWindow;
use viewports::{Viewport, ViewportFlags, WindowSpawner};
use winit::{
    event_loop::EventLoopWindowTarget,
    platform::windows::WindowExtWindows,
    window::{Window, WindowBuilder},
};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use winapi::{
    shared::windef,
    um::winuser,
};

unsafe impl HasRawWindowHandle for OverlaySpawner {
    fn raw_window_handle(&self) -> RawWindowHandle {
        //self.main_view
        self._game_window.raw_window_handle()
    }
}

pub(crate) struct OverlaySpawner{
    _game_window: GameWindow,
    _main_view: RawWindowHandle,
}
impl OverlaySpawner {
    pub(crate) fn new(_game_window: GameWindow, first_window: Option<&Window>) -> Self {
        //Self(game_window)
        let _main_view = first_window.expect("Expect main view window").raw_window_handle();
        Self{
            _game_window, _main_view
        }
    }
    fn is_alive(&self) -> bool {
        let parent = winapi_hwnd(self);
        unsafe { winuser::IsWindow(parent) != 0 }
    }   
}

impl<V: Viewport> WindowSpawner<V> for OverlaySpawner {
    fn build_window<T: 'static>(
        &mut self,
        event_loop: &EventLoopWindowTarget<T>,
        flags: ViewportFlags,
    ) -> Window {
        if !self.is_alive() {
            panic!("Parent window is dead.");
        }

        let decorations = !flags.contains(ViewportFlags::NO_DECORATIONS);
        let window = WindowBuilder::new()
            .with_decorations(decorations)
            .with_visible(false)
            //.with_always_on_top(true)
            .build(event_loop)
            .unwrap();
        //dbg!(flags);
        make_window_popup(&window).unwrap();
        reparent(&window, self);
        //set_parent(&window, self);
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

pub(crate) fn reparent<A: HasRawWindowHandle, B: HasRawWindowHandle>(window: &A, owner: &B) {
    let window = winapi_hwnd(window);
    let owner = winapi_hwnd(owner);
    unsafe {
        winuser::SetWindowLongPtrA(window, winuser::GWL_HWNDPARENT, owner as _);
    }
}

pub(crate) fn _set_parent<A: HasRawWindowHandle, B: HasRawWindowHandle>(window: &A, owner: &B) {
    let window = winapi_hwnd(window);
    let owner = winapi_hwnd(owner);
    unsafe {
        assert_eq!(winuser::SetParent(window, owner).is_null(), false);
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

pub(crate) fn _place_window_on_top<A: HasRawWindowHandle>(window: &A) {
    let window = winapi_hwnd(window);
    let flags = winuser::SWP_NOACTIVATE | winuser::SWP_NOMOVE | winuser::SWP_NOREDRAW | winuser::SWP_NOSIZE;
    assert!(unsafe  {
        winuser::SetWindowPos(window, winuser::HWND_TOP, 0, 0, 0, 0, flags)
    } != 0);
}
/*
pub(crate) fn place_window_on_bottom<A: HasRawWindowHandle>(window: &A) {
    let window = winapi_hwnd(window);
    let flags = winuser::SWP_NOACTIVATE | winuser::SWP_NOMOVE | winuser::SWP_NOREDRAW | winuser::SWP_NOSIZE;
    assert!(unsafe  {
        winuser::SetWindowPos(window, winuser::HWND_BOTTOMMOST, 0, 0, 0, 0, flags)
    } != 0);
}*/

pub(crate) fn _place_window_after<A: HasRawWindowHandle, B: HasRawWindowHandle>(window: &A, after: &B) {
    let window = winapi_hwnd(window);
    let after = winapi_hwnd(after);
    //dbg!(window);
    //dbg!(after);

    let flags = winuser::SWP_NOACTIVATE | winuser::SWP_NOMOVE | winuser::SWP_NOREDRAW | winuser::SWP_NOSIZE;
    // | winuser::SWP_NOSENDCHANGING

    assert!(unsafe  {
        winuser::SetWindowPos(after, window, 0, 0, 0, 0, flags)
    } != 0);

    /*assert!(unsafe  {
        winuser::SetWindowPos(window, after, 0, 0, 0, 0, flags)
    } != 0);
    assert!(unsafe  {
        winuser::SetWindowPos(after, window, 0, 0, 0, 0, flags)
    } != 0);*/
    /*unsafe {
        let defer = winuser::BeginDeferWindowPos(2);
        assert_eq!(defer.is_null(), false);
        let defer = winuser::DeferWindowPos(
            defer,
            window,
            after,
            0,
            0,
            0,
            0,
            flags
        );
        //assert_eq!(defer.is_null(), false);
        if defer.is_null() {
            println!("fail first");
            return;
        }
        let defer = winuser::DeferWindowPos(
            defer,
            after,
            window,
            0,
            0,
            0,
            0,
            flags
        );
        //assert_eq!(defer.is_null(), false);
        if defer.is_null() {
            println!("fail second");
            return;
        }
        let defer_result = winuser::EndDeferWindowPos(
            defer
        );
        assert!(defer_result != 0);
    }*/
}

#[allow(deprecated)]
pub(crate) fn winapi_hwnd<A: HasRawWindowHandle>(window: &A) -> windef::HWND {
    match window.raw_window_handle() {
        RawWindowHandle::Windows(handle) => handle.hwnd as _,
        RawWindowHandle::__NonExhaustiveDoNotUse(..) => unreachable!(),
    }
}

pub(crate) fn defer_order(windows: &[windef::HWND]) {
    let flags = winuser::SWP_NOACTIVATE | winuser::SWP_NOMOVE | winuser::SWP_NOREDRAW | winuser::SWP_NOSIZE;
    if windows.is_empty() {
        return;
    }

    unsafe {
        let mut defer = winuser::BeginDeferWindowPos(windows.len() as i32);
        assert_eq!(defer.is_null(), false);
        let mut iter = windows.windows(2).rev();
        while let Some(&[before, after]) = iter.next() {
            //dbg!(after, before);
            defer = winuser::DeferWindowPos(
                defer,
                before,
                after,
                0,
                0,
                0,
                0,
                flags
            );

            if defer.is_null() {
                println!("defer failed");
                return;
            }
        }

        let defer_result = winuser::EndDeferWindowPos(
            defer
        );
        assert!(defer_result != 0);
    }
}
