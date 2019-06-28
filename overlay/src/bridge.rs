use std::net::SocketAddr;
pub use tnf_common::message::client_dll_overlay::{
    Avatar, Char, ClientDllToOverlay as MsgIn, OverlayToClientDll as MsgOut, Position,
};
use tnf_common::{
    bridge::BridgeServerHandle,
    message::client_dll_overlay::{HANDSHAKE, VERSION},
};

pub type BridgeOverlayToClient = BridgeServerHandle<MsgIn, MsgOut>;

pub fn start() -> BridgeOverlayToClient {
    let addr: SocketAddr = "127.0.0.1:33741".parse().expect("malformed socket address");
    BridgeOverlayToClient::start(addr, HANDSHAKE, VERSION)
}
