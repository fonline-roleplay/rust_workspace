/// Entry point which will be called by the system once the DLL has been loaded
/// in the target process. Declaring this function is optional.
///
/// # Safety
///
/// What you can safely do inside here is very limited, see the Microsoft documentation
/// about "DllMain". Rust also doesn't officially support a "life before main()",
/// though it is unclear what that that means exactly for DllMain.
use winapi::um::consoleapi;

#[macro_export(local_inner_macros)]
macro_rules! dll_main(($callback_attach:expr, $callback_detach:expr) => { mod dll {
    use super::*;
    use winapi::{
        shared::{
            minwindef,
            minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
        },
    };

    #[no_mangle]
    #[allow(non_snake_case, unused_variables)]
    pub extern "system" fn DllMain(
    dll_module: HINSTANCE,
        call_reason: DWORD,
        reserved: LPVOID,
    ) -> BOOL {
        const DLL_PROCESS_ATTACH: DWORD = 1;
        const DLL_PROCESS_DETACH: DWORD = 0;

        match call_reason {
            DLL_PROCESS_ATTACH => {
                //tnf_common::console_init();
                let _ = {$callback_attach};
            },
            DLL_PROCESS_DETACH => {
                let _ = {$callback_detach};
            },
            _ => (),
        }
        minwindef::TRUE
    }
}});

pub fn console_init() {
    unsafe { consoleapi::AllocConsole() };
}
