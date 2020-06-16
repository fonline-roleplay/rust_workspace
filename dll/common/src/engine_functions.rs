use std::{
    ffi::CStr,
    os::raw::{c_char, c_int},
    ptr::{null, null_mut},
};
use crate::{
    engine_types::{critter::Critter, item::Item, stl::IntVec, ScriptArray, ScriptString},
    primitives::*,
};
use fo_engine_functions::*;

ffi_module!(AS_API, AngelScriptApi, "../../../ffi/API_AngelScript.rs");

#[derive(Debug)]
pub enum AngelScriptAPIError {
}

pub fn Script_String(c_str: &CStr) -> *mut ScriptString {
    unsafe { AS_API.Script_String(c_str.as_ptr()) }
}
