extern crate winapi;

use winapi::{
    shared::{
        minwindef,
        minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
    },
    um::consoleapi,
};

/// Entry point which will be called by the system once the DLL has been loaded
/// in the target process. Declaring this function is optional.
///
/// # Safety
///
/// What you can safely do inside here is very limited, see the Microsoft documentation
/// about "DllMain". Rust also doesn't officially support a "life before main()",
/// though it is unclear what that that means exactly for DllMain.
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
        DLL_PROCESS_ATTACH => demo_init(),
        DLL_PROCESS_DETACH => (),
        _ => (),
    }
    minwindef::TRUE
}

fn demo_init() {
    unsafe { consoleapi::AllocConsole() };
    println!("Hello from dll written in Rust!");
}

// ========= General stuff is above, FOnline-related stuff is below

mod defines;
mod engine_types;

use engine_types::{
    game_options::{game_state, GameOptions},
    mutual::CritterMutual,
    primitives::{int, uint},
};

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn getParam_Strength(cr: &CritterMutual, _: uint) -> int {
    use defines::fos as df;

    let mut val: int =
        cr.Params[df::ST_STRENGTH as usize] + cr.Params[df::ST_STRENGTH_EXT as usize];
    if cr.Params[ df::PE_ADRENALINE_RUSH as usize ] > 0 && getParam_Timeout( cr, df::TO_BATTLE ) > 0 // Adrenaline rush perk
        && cr.Params[ df::ST_CURRENT_HP as usize ] <= (
                cr.Params[ df::ST_MAX_LIFE as usize ] +
                cr.Params[ df::ST_STRENGTH as usize ] +
                cr.Params[ df::ST_ENDURANCE as usize ] * 2
            ) / 2
    {
        val += 1;
    }
    clamp(val, 1, 10)
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn getParam_Timeout(cr: &CritterMutual, index: uint) -> int {
    let full_second = game_state().map(|g| g.FullSecond).unwrap_or(0);
    let param = cr.Params[index as usize] as uint;
    if param > full_second {
        (param - full_second) as int
    } else {
        0
    }
}

fn clamp<T: std::cmp::Ord>(val: T, min: T, max: T) -> T
where
    T: Sized,
{
    assert!(min <= max);
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn TestFuncRust() {
    println!("TestFuncRust!");
}
