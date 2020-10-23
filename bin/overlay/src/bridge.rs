use crate::overlay::{OverlayEvent, OverlayEventProxy};
use bridge::{BridgeMessage, BridgeServerHandle};
pub use protocol::message::client_dll_overlay::{
    Avatar, Char, ClientDllToOverlay as MsgIn, OverlayToClientDll as MsgOut, Position,
};
use protocol::message::client_dll_overlay::{HANDSHAKE, VERSION};
use std::net::SocketAddr;

pub type BridgeOverlayToClient = BridgeServerHandle<MsgIn, MsgOut, WinitChannel>;

pub fn start(proxy: OverlayEventProxy) -> BridgeOverlayToClient {
    let addr: SocketAddr = "127.0.0.1:33741".parse().expect("malformed socket address");
    BridgeOverlayToClient::start_ext(addr, HANDSHAKE, VERSION, WinitChannel(proxy), ())
}

#[derive(Debug, Clone)]
pub struct WinitChannel(OverlayEventProxy);

impl bridge::Channel<MsgIn> for WinitChannel {
    type Receiver = ();

    fn send(&self, msg: BridgeMessage<MsgIn>) -> Result<(), ()> {
        if let BridgeMessage::Data(msg) = msg {
            self.0
                .send_event(OverlayEvent::BridgeMessage(msg))
                .ok()
                .ok_or(())?;
        }
        Ok(())
    }
}
