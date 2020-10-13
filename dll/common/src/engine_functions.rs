use crate::{
    engine_types::{critter::Critter, item::Item, stl::IntVec, ScriptArray, ScriptString},
    primitives::*,
};
use fo_engine_functions::*;
use std::{
    ffi::CStr,
    os::raw::{c_char, c_int},
    ptr::{null, null_mut},
};
/*
pub type asEBehaviours = u32;
pub type asDWORD = ::std::os::raw::c_ulong;
pub type asBYTE = ::std::os::raw::c_uchar;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct asSFuncPtr {
    pub ptr: asSFuncPtr__bindgen_ty_1,
    pub flag: asBYTE,
}
*/
ffi_module!(AngelScriptApi, "../../../ffi/API_AngelScript.rs");
pub use ffi::AngelScriptApi;

#[derive(Debug)]
pub enum AngelScriptAPIError {}
