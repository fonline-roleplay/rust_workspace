pub mod defines;
#[cfg(windows)]
pub mod engine_types;
#[allow(non_camel_case_types)]
pub mod primitives;

#[cfg(windows)]
mod dll {
    pub mod init;
    mod param_getters;
}
#[cfg(windows)]
pub use dll::init::console_init;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn TestFuncRust() {
    println!("TestFuncRust!");
}
