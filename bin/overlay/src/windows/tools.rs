use winapi::um::{
    processthreadsapi::GetCurrentProcessId, wincon::GetConsoleWindow,
    winuser::GetWindowThreadProcessId,
};
//use std::mem::zeroed;

#[derive(Debug, Copy, Clone)]
pub(crate) enum HasConsole {
    None,
    MyOwn,
    NotMine,
}

pub(crate) fn has_console() -> HasConsole {
    unsafe {
        let console = GetConsoleWindow();
        if console.is_null() {
            return HasConsole::None;
        }
        let mut console_process = 0;
        GetWindowThreadProcessId(console, &mut console_process);
        let current_process = GetCurrentProcessId();

        if current_process == console_process {
            HasConsole::MyOwn
        } else {
            HasConsole::NotMine
        }
    }
}
