#![cfg(windows)]

mod bridge;
pub mod config;
mod engine_functions;
mod hooks;
mod param;
mod utils;

use tnf_common::{
    engine_functions::AngelScriptApi,
    state::{State, StateSingleton},
};

use fo_engine_functions::Container;
struct Server {
    api: Container<engine_functions::ServerApi>,
    angelscript: Container<AngelScriptApi>,
}

impl Server {
    fn statistics_connections(&self) -> u32 {
        unsafe { self.api.StatisticsCurOnline() }
    }
}

impl State for Server {
    fn init() -> Self {
        Self {
            api: engine_functions::ServerApi::load().expect("Client engine's API"),
            angelscript: AngelScriptApi::load().expect("AngelScript API"),
        }
    }
    fn singleton() -> &'static StateSingleton<Self> {
        &SERVER_SINGLETON
    }
}
static SERVER_SINGLETON: StateSingleton<Server> = StateSingleton::new();

tnf_common::dll_main!({}, {});

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn SERVER() {
    // FOnline needs this to check if this is correct dll for server
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn DllMainEx(is_compiler: bool) {
    if !is_compiler {
        achtung::setup("reports", "rust_dll_server");

        tnf_common::check_dll_reload();
        Server::create();
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn DllFinish(is_compiler: bool) {
    if !is_compiler {
        tnf_common::dll_finish();
        Server::destroy();
    }
}
