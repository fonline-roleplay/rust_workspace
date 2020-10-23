use crate::{Rect, Viewport, WgpuManager};
use std::time::Duration;
use winapi::{shared::windef, um::winuser};
use winit::platform::windows::WindowExtWindows;

#[derive(Clone)]
pub(crate) struct GameWindow(windef::HWND);
impl GameWindow {
    pub(crate) fn with_config(config: &crate::config::Config) -> Option<Self> {
        loop {
            let res = if let Some(pid) = config.pid() {
                GameWindow::from_pid(pid)
            } else {
                GameWindow::find()
            };
            if res.is_none() && config.wait() {
                std::thread::sleep(Duration::from_secs(1));
                continue;
            } else {
                break res;
            }
        }
    }
    fn from_handle(handle: windef::HWND) -> Self {
        GameWindow(handle)
    }
    pub(crate) fn from_pid(pid: u32) -> Option<Self> {
        if pid == 0 {
            return None;
        }
        struct EnumWindowsData {
            pid: u32,
            hwnd: Vec<(windef::HWND, String)>,
        };
        let mut data = EnumWindowsData { pid, hwnd: vec![] };
        unsafe extern "system" fn find_by_pid(hwnd: windef::HWND, data: isize) -> i32 {
            let data = &mut *(data as *mut EnumWindowsData);
            let mut process_id = 0;
            let _thread_id = winuser::GetWindowThreadProcessId(hwnd, &mut process_id);
            if process_id == data.pid {
                let mut buf = [0u8; 128];
                let len = winuser::GetClassNameA(hwnd, buf.as_mut_ptr() as _, buf.len() as i32);
                if len > 0 {
                    let name = std::str::from_utf8(&buf[0..len as usize]);
                    if let Ok(name) = name {
                        data.hwnd.push((hwnd, name.to_owned()));
                    }
                }
            }
            1
        }
        unsafe {
            winuser::EnumWindows(Some(find_by_pid), (&mut data) as *mut _ as _);
        }
        println!("Found windows: {}", data.hwnd.len());
        for window in &data.hwnd {
            println!("Window class: {}", &window.1);
        }
        data.hwnd
            .iter()
            .find(|(_hwnd, name)| &*name == "FLTK")
            .map(|(hwnd, name)| {
                println!("Selected: {}", name);
                Self::from_handle(*hwnd)
            })
    }
    pub(crate) fn find() -> Option<Self> {
        let ret = unsafe { winuser::FindWindowA(0 as _, "FOnline\0".as_ptr() as _) };
        if ret.is_null() {
            None
        } else {
            Some(GameWindow(ret))
        }
    }
    fn client_rect(&self) -> Option<windef::RECT> {
        let mut rect: windef::RECT = unsafe { std::mem::zeroed() };
        let ret = unsafe { winuser::GetClientRect(self.0, &mut rect) };
        if ret == 0 {
            None
        } else {
            Some(rect)
        }
    }

    pub(crate) fn winapi_rect(&self) -> Option<windef::RECT> {
        let mut rect: windef::RECT = self.client_rect()?;
        unsafe {
            winapi::um::errhandlingapi::SetLastError(0);
            let _ret = winuser::MapWindowPoints(
                self.0,
                0 as _,
                &mut rect as *mut windef::RECT as usize as _,
                2,
            );
            let err = winapi::um::errhandlingapi::GetLastError();
            if err == 0 {
                Some(rect)
            } else {
                None
            }
        }
    }

    pub(crate) fn rect(&self) -> Option<Rect> {
        self.winapi_rect().map(|rect| {
            Rect::new(
                rect.left,
                rect.top,
                (rect.right - rect.left) as u32,
                (rect.bottom - rect.top) as u32,
            )
        })
    }

    pub(crate) fn _window_pos(&self) -> Option<(i32, i32)> {
        self.winapi_rect().map(|rect| (rect.left, rect.top))
    }
    pub(crate) fn raw(&self) -> windef::HWND {
        self.0
    }
    pub(crate) fn to_foreground(&self) {
        unsafe { winuser::SetForegroundWindow(self.0) };
    }

    pub(crate) fn _is_game_foreground(&self, manager: &WgpuManager) -> bool {
        let game_window = self.raw();
        let focus = unsafe { winuser::GetForegroundWindow() };
        if focus as usize == 0 {
            return false;
        }
        if focus == game_window {
            return true;
        }
        for (_window_id, viewport) in manager.viewports_iter() {
            let window = viewport.window();
            let handle = window.hwnd() as usize;
            if handle == focus as usize {
                return true;
            }
        }
        false
    }
    pub(crate) fn is_alive(&self) -> bool {
        let game_window = self.raw();
        unsafe { winuser::IsWindow(game_window) != 0 }
    }
}
