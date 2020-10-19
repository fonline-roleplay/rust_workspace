use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fmt::Debug,
    io::{BufRead, BufReader},
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
pub type BridgeServerHandle<MsgIn, MsgOut, S = Sender<BridgeMessage<MsgIn>>> =
    BridgeHandle<BridgeServer<MsgIn, MsgOut>, S>;

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
    fn process<S: Channel<Self::MsgIn>>(
        worker: &mut BridgeWorker<Self, S>,
    ) -> Result<(), BridgeError>;
    fn shutdown(&mut self);
}

pub trait Channel<T>: Clone + Send + 'static {
    type Receiver;

    fn send(&self, msg: BridgeMessage<T>) -> Result<(), ()>;
}

pub type DefaultSender<T> = Sender<BridgeMessage<T>>;
pub type DefaultReceiver<T> = Receiver<BridgeMessage<T>>;

impl<T: Send + 'static> Channel<T> for DefaultSender<T> {
    type Receiver = DefaultReceiver<T>;

    fn send(&self, msg: BridgeMessage<T>) -> Result<(), ()> {
        self.send(msg).ok().ok_or(())
    }
}
