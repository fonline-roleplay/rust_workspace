use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use actix_codec::{AsyncRead, AsyncWrite};
use actix_server::{Io, Server};
use actix_service::{service_fn, IntoService, NewService};
use bytes::BytesMut;
use futures::stream::Stream;
use futures::{future, Future};
use tokio_codec::{Decoder, Encoder, Framed};
use tokio_tcp::TcpStream;

/// Simple logger service, it just prints fact of the new connections
fn logger<T: AsyncRead + AsyncWrite + std::fmt::Debug>(
    stream: T,
) -> impl Future<Item = T, Error = ()> {
    println!("New connection: {:?}", stream);
    future::ok(stream)
}

pub fn start() {
    let num = Arc::new(AtomicUsize::new(0));
    Server::build()
        .bind(
            // configure service pipeline
            "bridge",
            "127.0.0.1:33852",
            move || {
                let num = num.clone();
                // service for converting incoming TcpStream to a SslStream<TcpStream>
                service_fn(move |stream: Io<tokio_tcp::TcpStream>| {
                    use futures::future::Either;
                    use futures::sink::Sink;

                    let num = num.fetch_add(1, Ordering::Relaxed);
                    println!("got connection {:?}", num);
                    let framed = Framed::new(stream.into_parts().0, WebSide);
                    let (sink, stream) = framed.split();
                    stream.map(handle_message).fold(sink, |sink, msg| {
                        println!("{:?}", msg);
                        match msg {
                            Some(msg) => Either::A(sink.send(msg)),
                            None => Either::B(future::ok(sink)),
                        }
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
}

fn handle_message(msg_in: MsgIn) -> Option<MsgOut> {
    Some(match msg_in {
        MsgIn::PlayerConnected(cr_id) => MsgOut::UpdateClientAvatar(cr_id, 777),
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

use tnf_common::message::ServerDllToWeb as MsgIn;
use tnf_common::message::ServerWebToDll as MsgOut;

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
