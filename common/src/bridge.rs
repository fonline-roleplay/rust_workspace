use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fmt::Debug,
    iter::FilterMap,
    marker::PhantomData,
    net::{SocketAddr, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender, TryIter},
        Arc,
    },
    thread::{sleep, JoinHandle},
    time::Duration,
};

mod cell;
pub use cell::BridgeCell;

mod handle;
pub use handle::BridgeHandle;

mod worker;
use worker::BridgeWorker;

mod client;
pub use client::BridgeClient;
pub type BridgeClientCell<MsgIn, MsgOut> = BridgeCell<BridgeHandle<BridgeClient<MsgIn, MsgOut>>>;

mod server;
pub use server::BridgeServer;
pub type BridgeServerHandle<MsgIn, MsgOut> = BridgeHandle<BridgeServer<MsgIn, MsgOut>>;

mod with_bincode;

#[derive(Debug, Deserialize, Serialize)]
pub enum BridgeMessage<T> {
    Data(T),
    Handshake(u16, u16),
    Ping,
    Hang,
    Shutdown,
}

#[derive(Debug)]
pub enum BridgeError {
    Io(std::io::Error),
    BinCode(bincode::Error),
    Handshake(u16, u16),
    NoHandshake,
    ChannelDropped,
    EmptyBridgeCell,
    NotOnline,
    PingTimeout,
    Hang,
}

pub trait BridgeTask: 'static + Sized + Send {
    type MsgIn: 'static + Send + DeserializeOwned + Debug;
    type MsgOut: 'static + Send + Serialize + Debug;

    fn new() -> Self;
    fn process(worker: &mut BridgeWorker<Self>) -> Result<(), BridgeError>;
    fn shutdown(&mut self);
}
