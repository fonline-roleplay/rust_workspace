#![cfg(windows)]

use tnf_common::{
    engine_functions::AngelScriptApi,
    engine_types::{ScriptArray, ScriptString},
    state::{State, StateSingleton},
};

//#[cfg(debug_assertions)]
/*tnf_common::dll_main!({}, {
    bridge::finish();
});*/

mod bridge;
#[allow(non_snake_case)]
pub(crate) mod engine_functions;

mod pui;

use fo_engine_functions::Container;
struct Client {
    api: Container<engine_functions::ClientApi>,
    angelscript: Container<AngelScriptApi>,
    pui: pui::Pui,
}
/*
impl Drop for Client {
    fn drop(&mut self) {
        tnf_common::message_info("Drop", "Client dropped.");
    }
}*/

impl State for Client {
    fn init() -> Self {
        Self {
            api: engine_functions::ClientApi::load().expect("Client engine's API"),
            angelscript: AngelScriptApi::load().expect("AngelScript API"),
            pui: pui::Pui::new(),
        }
    }
    fn singleton() -> &'static StateSingleton<Self> {
        &CLIENT_SINGLETON
    }
}
static CLIENT_SINGLETON: StateSingleton<Client> = StateSingleton::new();

tnf_common::dll_main!({}, {});

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn CLIENT() {
    // FOnline needs this to check if this is correct dll for client
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn DllMainEx(is_compiler: bool) {
    if !is_compiler {
        achtung::setup("reports", "rust_dll_client");

        tnf_common::check_dll_reload();
        Client::create();
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn DllFinish(is_compiler: bool) {
    if !is_compiler {
        bridge::disconnect_from_overlay(true);

        tnf_common::dll_finish();
        Client::destroy();
    }
}

#[no_mangle]
pub extern "C" fn open_link(link: &ScriptString) {
    let link = link.string();
    if link.starts_with("http://") || link.starts_with("https://") {
        std::thread::spawn(move || {
            #[cfg(debug_assertions)]
            println!("Opening link: {:?}", link);
            let _res = webbrowser::open(&link);
        });
    } else {
        println!("Invalid link: {:?}", link);
    }
}

#[no_mangle]
pub extern "C" fn open_link_auth(link: &ScriptString, p0: i32, p1: i32, p2: i32) {
    use std::fmt::Write;

    let mut link = link.string();
    if link.starts_with("http://") || link.starts_with("https://") {
        std::thread::spawn(move || {
            if p0 != 0 || p1 != 0 && p2 != 0 {
                let buf: [u32; 3] = [p0 as u32, p1 as u32, p2 as u32];
                let buf: [u8; 12] = unsafe { std::mem::transmute(buf) };
                link.push_str("?auth=");
                for &word in buf.iter() {
                    write!(&mut link, "{:02X}", word).expect("encoding auth key");
                }
            }
            #[cfg(debug_assertions)]
            println!("Opening link: {:?}", link);
            let _res = webbrowser::open(&link);
        });
    } else {
        println!("Invalid link: {:?}", link);
    }
}
