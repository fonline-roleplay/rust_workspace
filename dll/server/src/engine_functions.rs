use crate::Server;
use fo_engine_functions::*;
use std::{
    ffi::CStr,
    os::raw::{c_char, c_int},
    ptr::{null, null_mut},
};
use tnf_common::{
    engine_types::{critter::Critter, item::Item, stl::IntVec, ScriptArray, ScriptString},
    primitives::*,
    state::State,
};

ffi_module!(ServerApi, "../../../ffi/API_Server.rs");
pub(crate) use ffi::ServerApi;

#[derive(Debug)]
pub enum ServerAPIError {
    CritterIsNotValid,
    CritterIsNotPlayer,
    RunClientScriptFailed,
    RunCritterScriptFailed,
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
    Server::with(|server| {
        if cr.IsNotValid {
            Err(ServerAPIError::CritterIsNotValid)
        } else if !cr.is_player() {
            Err(ServerAPIError::CritterIsNotPlayer)
        } else {
            if unsafe {
                server.api.Cl_RunClientScript(
                    cr,
                    func_name.as_ptr(),
                    p0,
                    p1,
                    p2,
                    p3.map(CStr::as_ptr).unwrap_or_else(null),
                    p4.map(<[_]>::as_ptr).unwrap_or_else(null),
                    p4.map(<[_]>::len).unwrap_or(0),
                )
            } {
                Ok(())
            } else {
                Err(ServerAPIError::RunClientScriptFailed)
            }
        }
    })
}

pub fn run_critter_script(
    cr: Option<&mut Critter>,
    func_name: &CStr,
    p0: i32,
    p1: i32,
    p2: i32,
    p3: Option<&CStr>,
    p4: Option<&[u32]>,
) -> Result<(), ServerAPIError> {
    Server::with(|server| {
        if cr.as_ref().map(|cr| cr.IsNotValid).unwrap_or(false) {
            Err(ServerAPIError::CritterIsNotValid)
        } else {
            if unsafe {
                server.api.Global_RunCritterScript(
                    cr.map(|cr| cr as *mut _).unwrap_or(null_mut()),
                    func_name.as_ptr(),
                    p0,
                    p1,
                    p2,
                    p3.map(CStr::as_ptr).unwrap_or_else(null),
                    p4.map(<[_]>::as_ptr).unwrap_or_else(null),
                    p4.map(<[_]>::len).unwrap_or(0),
                )
            } {
                Ok(())
            } else {
                Err(ServerAPIError::RunCritterScriptFailed)
            }
        }
    })
}

pub fn get_critter<'a>(id: u32) -> Option<&'a mut Critter> {
    Server::with(|server| unsafe { std::mem::transmute(server.api.Global_GetCritter(id)) })
}

#[no_mangle]
pub extern "C" fn Global_GetMsgStr(lang: usize, textMsg: usize, strNum: u32) -> *mut ScriptString {
    Server::with(|server| unsafe { server.api.Global_GetMsgStr(lang, textMsg, strNum) })
}

#[no_mangle]
pub extern "C" fn item_get_lexems(item: *mut Item) -> *mut ScriptString {
    Server::with(|server| unsafe { server.api.Item_GetLexems(item) })
}
/*
#[no_mangle]
pub extern "C" fn ConstantsManager_GetValue(collection: usize, string: *mut ScriptString) -> i32 {
    unsafe { SERVER_API.ConstantsManager_GetValue(collection, string) }
}
*/

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn DisplaySI(
    value: f32,
    units: Option<&ScriptString>,
    zeros: u8,
) -> *mut ScriptString {
    Server::with(|server| {
        use std::ffi::CStr;
        let mut units = units
            .map(|units| units.string())
            .unwrap_or_else(String::new);
        units.push('\0');
        let si = tnf_common::si::SI::new(value, &units, zeros);
        let output = si.to_string();
        ScriptString::from_string(&server.angelscript, &output)
    })
}

#[no_mangle]
pub extern "C" fn prev_hex(cr: &Critter, hex_x: &mut u16, hex_y: &mut u16) -> u32 {
    let game_tick = Server::with(|server| {
        unsafe {
            server.api.Timer_GameTick()
        }
    });
    *hex_x = cr.PrevHexX;
    *hex_y = cr.PrevHexY;
    game_tick - cr.PrevHexTick
}
