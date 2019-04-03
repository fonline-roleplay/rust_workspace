#![cfg(windows)]

use tnf_common;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn CLIENT() {
    // FOnline needs this to check if this is correct dll for client
}
