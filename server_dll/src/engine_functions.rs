use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use once_cell::sync::Lazy;
use std::{
    ffi::CStr,
    os::raw::{c_char, c_int},
    ptr::{null, null_mut},
};
use tnf_common::{
    engine_types::{critter::Critter, item::Item, stl::IntVec, ScriptArray, ScriptString},
    primitives::*,
};

macro_rules! dynamic_ffi {
    ($api:ident, $(pub fn $fun:ident($($arg:ident: $typ:ty$ (,)?)*) $(-> $ret:ty)? ;)*) => {
        #[derive(WrapperApi)]
        pub struct $api {
            $($fun: unsafe extern "C" fn($($arg: $typ,)*) $(-> $ret)? ,)*
        }
    }
}

#[allow(bad_style)]
mod ffi {
    use super::*;
    include!("../../ffi/API_Server.rs");
}
use ffi::ServerApi;

static SERVER_API: Lazy<Container<ServerApi>> =
    Lazy::new(|| unsafe { Container::load_self() }.expect("Can't load api"));

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
pub extern "C" fn item_get_lexems(item: *mut Item) -> *mut ScriptString {
    unsafe { std::mem::transmute(SERVER_API.Item_GetLexems(item)) }
}
