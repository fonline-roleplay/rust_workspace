use std::{
    ffi::CStr,
    os::raw::{c_char, c_int},
    ptr::{null, null_mut},
};
use tnf_common::{
    engine_types::{critter::Critter, item::Item, stl::IntVec, ScriptArray, ScriptString},
    primitives::*,
};
use fo_engine_functions::*;

ffi_module!(SERVER_API, ServerApi, "../../../ffi/API_Server.rs");

#[derive(Debug)]
pub enum ServerAPIError {
    CritterIsNotValid,
    CritterIsNotPlayer,
}

pub fn run_client_script(
    cr: &mut Critter,
    func_name: &CStr,
    p0: i32,
    p1: i32,
    p2: i32,
    p3: Option<&CStr>,
    p4: Option<&[u32]>,
) -> Result<(), ServerAPIError> {
    if cr.IsNotValid {
        Err(ServerAPIError::CritterIsNotValid)
    } else if !cr.is_player() {
        Err(ServerAPIError::CritterIsNotPlayer)
    } else {
        unsafe {
            SERVER_API.Cl_RunClientScript(
                cr,
                func_name.as_ptr(),
                p0,
                p1,
                p2,
                p3.map(CStr::as_ptr).unwrap_or_else(null),
                p4.map(<[_]>::as_ptr).unwrap_or_else(null),
                p4.map(<[_]>::len).unwrap_or(0),
            );
        }
        Ok(())
    }
}

pub fn get_critter<'a>(id: u32) -> Option<&'a mut Critter> {
    unsafe { std::mem::transmute(SERVER_API.Global_GetCritter(id)) }
}

#[no_mangle]
pub extern "C" fn Global_GetMsgStr(lang: usize, textMsg: usize, strNum: u32) -> *mut ScriptString {
    unsafe { SERVER_API.Global_GetMsgStr(lang, textMsg, strNum) }
}

#[no_mangle]
pub extern "C" fn item_get_lexems(item: *mut Item) -> *mut ScriptString {
    unsafe { SERVER_API.Item_GetLexems(item) }
}
/*
#[no_mangle]
pub extern "C" fn ConstantsManager_GetValue(collection: usize, string: *mut ScriptString) -> i32 {
    unsafe { SERVER_API.ConstantsManager_GetValue(collection, string) }
}
*/
