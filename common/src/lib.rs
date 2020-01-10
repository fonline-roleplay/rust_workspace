pub use fo_defines as defines;
pub use fo_defines_fo4rp as defines_fo4rp;
pub use primitives;

#[cfg(all(windows, feature = "engine_types"))]
pub mod engine_types;

pub mod utils;

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
