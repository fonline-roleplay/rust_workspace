use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use actix::Addr;
use actix_codec::{AsyncRead, AsyncWrite};
use actix_server::{Io, Server};
use actix_service::{service_fn, IntoService, NewService};
use bytes::BytesMut;
use futures::stream::Stream;
use futures::sync::mpsc::{channel, Receiver, Sender, TrySendError};
use futures::{future, Future};
use tokio_codec::{Decoder, Encoder, Framed};
use tokio_tcp::TcpStream;
//use crossbeam::atomic::AtomicCell;
use crate::database::SledDb;
use parking_lot::RwLock;

pub use tnf_common::message::ServerDllToWeb as MsgIn;
pub use tnf_common::message::ServerWebToDll as MsgOut;
pub type MsgOutSender = Sender<MsgOut>;
//pub type MsgOutSendError = SendError<MsgOut>;
pub type MsgOutSendError = TrySendError<MsgOut>;

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
    fn new() -> Self {
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
    /*pub send(sender: MsgOutSende, msg: MsgOut) -> Option<> {
        match sender.start_send(msg) {
            Ok()
            .map_err(AvatarUploadError::FuturesSyncSend)
        }
    }*/
}

pub fn start() -> Bridge {
    let num = Arc::new(AtomicUsize::new(0));
    let bridge = Bridge::new();
    let bridge_ret = bridge.clone();
    Server::build()
        .bind(
            // configure service pipeline
            "bridge",
            "127.0.0.1:33852",
            move || {
                let num = num.clone();
                let bridge = bridge.clone();
                // service for converting incoming TcpStream to a SslStream<TcpStream>
                service_fn(move |stream: Io<tokio_tcp::TcpStream>| {
                    use futures::future::Either;
                    use futures::sink::Sink;

                    let (sender, receiver) = channel(128);
                    bridge.set_sender(sender);

                    let num = num.fetch_add(1, Ordering::Relaxed);
                    println!("got connection {:?}", num);
                    let framed = Framed::new(stream.into_parts().0, WebSide);
                    let (sink, stream) = framed.split();
                    stream
                        .filter_map(handle_message)
                        .select(receiver.map_err(|_| std::io::ErrorKind::BrokenPipe.into()))
                        .fold(sink, |sink, msg| {
                            println!("Sending: {:?}", msg);
                            sink.send(msg)
                        })
                })

                /*// .and_then() combinator uses other service to convert incoming `Request` to a
                // `Response` and then uses that response as an input for next
                // service. in this case, on success we use `logger` service
                .and_then(logger)
                // Next service counts number of connections
                .and_then(move |_| {
                    let num = num.fetch_add(1, Ordering::Relaxed);
                    println!("got ssl connection {:?}", num);
                    future::ok(())
                })*/
            },
        )
        .unwrap()
        .start();
    bridge_ret
}

fn handle_message(msg_in: MsgIn) -> Option<MsgOut> {
    Some(match msg_in {
        //MsgIn::PlayerConnected(cr_id) => MsgOut::UpdateClientAvatar(cr_id, 777),
        MsgIn::PlayerAuth(cr_id) => MsgOut::SendKeyToPlayer(cr_id, [77, 77, 77]),
        _ => {
            return None;
        }
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

impl Encoder for WebSide {
    type Item = MsgOut;
    type Error = std::io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        const LEN: usize = std::mem::size_of::<MsgOut>();
        let buf: [u8; LEN] = unsafe { std::mem::transmute(item) };
        dst.extend_from_slice(&buf);
        Ok(())
    }
}
