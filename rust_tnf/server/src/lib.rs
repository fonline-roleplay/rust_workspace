#![cfg(windows)]

use tnf_common;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn SERVER() {
    // FOnline needs this to check if this is correct dll for server
}
