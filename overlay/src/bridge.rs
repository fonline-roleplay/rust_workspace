use bridge::BridgeServerHandle;
pub use protocol::message::client_dll_overlay::{
    Avatar, Char, ClientDllToOverlay as MsgIn, OverlayToClientDll as MsgOut, Position,
};
use protocol::message::client_dll_overlay::{HANDSHAKE, VERSION};
use std::net::SocketAddr;

pub type BridgeOverlayToClient = BridgeServerHandle<MsgIn, MsgOut>;

pub fn start() -> BridgeOverlayToClient {
    let addr: SocketAddr = "127.0.0.1:33741".parse().expect("malformed socket address");
    BridgeOverlayToClient::start(addr, HANDSHAKE, VERSION)
}
