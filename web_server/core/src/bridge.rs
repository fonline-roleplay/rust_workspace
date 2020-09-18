use crate::config;
use crate::database::{CharTrunk, Root, VersionedError};
use actix_codec::{AsyncRead, AsyncWrite, Decoder, Encoder, Framed};
use actix_rt::net::TcpStream;
use actix_server::{FromStream, Server};
use actix_service::{fn_service, IntoService};
use actix_web::{error::BlockingError, web};
use bytes::BytesMut;
use futures::{
    channel::mpsc::{channel, Receiver, Sender, TrySendError},
    future,
    future::Either,
    Future, FutureExt, Sink, SinkExt, Stream, StreamExt, TryFutureExt, TryStreamExt,
};
use parking_lot::RwLock;
use std::{
    convert::TryInto,
    ffi::{CStr, CString},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

pub use protocol::message::server_dll_web::{ServerDllToWeb as MsgIn, ServerWebToDll as MsgOut};
pub type MsgOutSender = Sender<MsgOut>;
//pub type MsgOutSendError = SendError<MsgOut>;
pub type MsgOutSendError = TrySendError<MsgOut>;

type BridgeResult<T> = Result<T, BridgeError>;

/// Simple logger service, it just prints fact of the new connections
/*fn logger<T: AsyncRead + AsyncWrite + std::fmt::Debug>(
    stream: T,
) -> impl Future<Item = T, Error = ()> {
    println!("New connection: {:?}", stream);
    future::ok(stream)
}*/

#[derive(Clone)]
pub struct Bridge {
    sender: Arc<RwLock<Option<MsgOutSender>>>,
}
impl Bridge {
    pub fn new() -> Self {
        Bridge {
            sender: Arc::new(RwLock::new(None)), //Arc::new(AtomicCell::new(None))
        }
    }
    fn set_sender(&self, sender: MsgOutSender) {
        *self.sender.write() = Some(sender);
    }
    pub fn get_sender(&self) -> Option<MsgOutSender> {
        let lock = self.sender.read();
        match &*lock {
            Some(sender) if !sender.is_closed() => Some(sender.clone()),
            _ => None,
        }
    }
    pub fn start(&self, tree: Root, host: config::Host) {
        let data = BridgeData {
            bridge: self.clone(),
            tree,
            host,
        };
        start_impl(data);
    }
    /*pub send(sender: MsgOutSende, msg: MsgOut) -> Option<> {
        match sender.start_send(msg) {
            Ok()
            .map_err(AvatarUploadError::FuturesSyncSend)
        }
    }*/
}

#[derive(Clone)]
struct BridgeData {
    bridge: Bridge,
    tree: Root,
    host: config::Host,
}

fn start_impl(data: BridgeData) {
    let num = Arc::new(AtomicUsize::new(0));
    Server::build()
        .bind(
            // configure service pipeline
            "bridge",
            "127.0.0.1:33852",
            move || {
                let data = data.clone();
                // service for converting incoming TcpStream to a SslStream<TcpStream>
                fn_service(move |tcp_stream: TcpStream| {
                    use futures::future::Either;

                    let data = data.clone();

                    let (sender, receiver) = channel(128);
                    data.bridge.set_sender(sender);

                    let framed = Framed::new(tcp_stream, WebSide);
                    let (mut sink, mut stream) = framed.split();

                    futures::stream::select(
                        stream
                            .map_err(BridgeError::Io)
                            //.filter_map(handle_message)
                            .and_then(move |msg| handle_message_async(msg, data.clone()))
                            .boxed(),
                        receiver.map(Result::Ok), //.map_err(|_| BridgeError::SenderDropped),
                    )
                    .try_filter(drop_nop)
                    .map_err(|err| {
                        std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err)).into()
                    })
                    .inspect_ok(|msg| println!("Sending: {:?}", msg))
                    .forward(sink)
                })
            },
        )
        .unwrap()
        .start();
}

fn handle_message_async(
    msg_in: MsgIn,
    data: BridgeData,
) -> impl Future<Output = BridgeResult<MsgOut>> {
    async move {
        match msg_in {
            MsgIn::PlayerConnected(player_id) => Ok(MsgOut::SendConfig {
                player_id,
                url: CStr::from_bytes_with_nul(data.host.overlay_urls().as_bytes())
                    .expect("Can't fail, null byte supplied")
                    .to_owned(),
            }),
            MsgIn::PlayerAuth(cr_id) => {
                let root = data.tree;
                let fut = web::block(move || {
                    let default: [u8; 12] = rand::random();
                    let authkey = root
                        .trunk(cr_id, None, CharTrunk::default())
                        .get_bare_branch_or_default("authkey", &default[..], |val| val.len() == 12)
                        .map_err(BridgeError::Versioned)?;
                    let authkey = match authkey {
                        Some(authkey) => {
                            let slice = authkey.as_ref();
                            let bytes: [u8; 12] =
                                slice.try_into().map_err(|_| BridgeError::TryInto)?;
                            bytes
                        }
                        None => default,
                    };
                    let authkey: [u32; 3] = unsafe { std::mem::transmute(authkey) };
                    Ok(authkey)
                })
                //.from_err()
                .err_into()
                .map_ok(move |authkey| MsgOut::SendKeyToPlayer(cr_id, authkey));
                fut.await
            } //_ => MsgOut::Nop,
        }
    }
}

fn drop_nop(msg_out: &MsgOut) -> impl Future<Output = bool> {
    future::ready(match msg_out {
        MsgOut::Nop => false,
        _ => true,
    })
}

/*
fn handle(stream: TcpStream) {
    let (reader, writer) = stream.split();

}*/

struct WebSide;

impl Decoder for WebSide {
    type Item = MsgIn;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        const LEN: usize = std::mem::size_of::<MsgIn>();
        println!("decode len: {:?}", src.len());
        if LEN > src.len() {
            return Ok(None);
        }
        let bytes = src.split_to(LEN);
        println!("reading... {:?}", &bytes.as_ref()[..]);
        let mut buf = [0u8; LEN];
        buf.copy_from_slice(&bytes);
        Ok(Some(unsafe { std::mem::transmute(buf) }))
    }
}

impl Encoder<MsgOut> for WebSide {
    type Error = bincode::Error;

    fn encode(&mut self, item: MsgOut, dst: &mut BytesMut) -> Result<(), Self::Error> {
        //const LEN: usize = std::mem::size_of::<MsgOut>();
        //let buf: [u8; LEN] = unsafe { std::mem::transmute(item) };
        //dst.extend_from_slice(&buf);
        let buf = bincode::serialize(&item)?;
        dst.extend_from_slice(&buf);
        Ok(())
    }
}

#[derive(Debug)]
pub enum BridgeError {
    Versioned(VersionedError),
    Io(std::io::Error),
    Blocking,
    TryInto,
    SenderDropped,
}

impl From<BlockingError<BridgeError>> for BridgeError {
    fn from(err: BlockingError<BridgeError>) -> Self {
        match err {
            BlockingError::Error(err) => err,
            BlockingError::Canceled => BridgeError::Blocking,
        }
    }
}
