//use sdl2::{rect::Rect, video::WindowPos};
use crate::Rect;
use winapi::{shared::windef, um::winuser};

macro_rules! not_null (
    ($ret:expr) => {
        if $ret as usize == 0 {
            None
        } else {
            Some($ret)
        }
    }
);

pub struct GameWindow(windef::HWND);
impl GameWindow {
    fn from_handle(handle: windef::HWND) -> Self {
        GameWindow(handle)
    }
    pub fn find() -> Option<Self> {
        let ret = unsafe { winuser::FindWindowA(0 as _, "FOnline\0".as_ptr() as _) };
        //let ret = unsafe { winuser::FindWindowA( "Notepad\0".as_ptr() as _, 0 as _) };
        not_null!(ret).map(GameWindow::from_handle)
    }
    fn client_rect(&self) -> Option<windef::RECT> {
        let mut rect: windef::RECT = unsafe { std::mem::zeroed() };
        let ret = unsafe { winuser::GetClientRect(self.0, &mut rect) };
        not_null!(ret).map(|_| rect)
    }

    pub fn winapi_rect(&self) -> Option<windef::RECT> {
        let mut rect: windef::RECT = self.client_rect()?;
        let ret = unsafe {
            winuser::MapWindowPoints(
                self.0,
                0 as _,
                &mut rect as *mut windef::RECT as usize as _,
                2,
            )
        };
        not_null!(ret).map(|_| rect)
    }

    pub fn rect(&self) -> Option<Rect> {
        self.winapi_rect().map(|rect| {
            Rect::new(
                rect.left,
                rect.top,
                (rect.right - rect.left) as u32,
                (rect.bottom - rect.top) as u32,
            )
        })
    }

    pub fn window_pos(&self) -> Option<(i32, i32)> {
        self.winapi_rect().map(|rect| (rect.left, rect.top))
    }
    pub fn raw(&self) -> windef::HWND {
        self.0
    }
    pub fn to_foreground(&self) {
        unsafe { winuser::SetForegroundWindow(self.0) };
    }
}
