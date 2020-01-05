pub mod defines;
#[cfg(all(windows, feature = "engine_types"))]
pub mod engine_types;
#[allow(non_camel_case_types)]
pub mod primitives;

pub mod utils;

pub mod message;

#[cfg(feature = "bridge")]
pub mod bridge;

#[cfg(all(windows, feature = "dll"))]
pub mod dll {
    pub mod init;
    pub mod param_getters;
}

#[cfg(all(windows, feature = "dll"))]
pub use dll::init::console_init;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn TestFuncRust() {
    println!("TestFuncRust!");
}
