use std::{
    net::{SocketAddr},
};
use tnf_common::{
    message::client_dll_overlay::{
        HANDSHAKE, VERSION,
    },
    bridge::{BridgeServerHandle},
};
pub use tnf_common::{
    message::client_dll_overlay::{
        ClientDllToOverlay as MsgIn,
        OverlayToClientDll as MsgOut,
        Avatar, Char, Position,
    }
};

pub type BridgeOverlayToClient = BridgeServerHandle<MsgIn, MsgOut>;

pub fn start() -> BridgeOverlayToClient {
    let addr: SocketAddr = "127.0.0.1:33741".parse().expect("malformed socket address");
    BridgeOverlayToClient::start(addr, HANDSHAKE, VERSION)
}
