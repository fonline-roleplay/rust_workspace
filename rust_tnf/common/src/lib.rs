pub mod defines;
pub mod engine_types;
mod param_getters;

mod dll;
pub use dll::{console_init};

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn TestFuncRust() {
    println!("TestFuncRust!");
}
