#![cfg(windows)]

use tnf_common::{
    engine_types::ScriptString,
    dll_main,
};
dll_main!({});

#[no_mangle]
pub extern "C" fn open_link(link: &ScriptString) {
    let link = link.string();
    if link.starts_with("http://") || link.starts_with("https://") {
        std::thread::spawn(move || {
            println!("Opening link: {:?}", link);
            let _res = webbrowser::open(&link);
        });
    } else {
        println!("Invalid link: {:?}", link);
    }
}
