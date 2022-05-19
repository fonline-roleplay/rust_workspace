use crate::{
    database::{ownership, CharTrunk, Root, VersionedError},
    utils::blocking,
    web::AppState,
};
use actix_codec::{Decoder, Encoder, Framed};
use actix_rt::net::TcpStream;
pub use actix_server::Server;
use actix_service::fn_service;
use actix_web::error::BlockingError;
use bytes::BytesMut;
use futures::{
    channel::mpsc::{channel, Sender, TrySendError},
    future, Future, StreamExt, TryFutureExt, TryStreamExt,
};
use mrhandy::{Condition, ConditionColor};
use parking_lot::RwLock;
use serde::Serialize;
use std::{convert::TryInto, ffi::CStr, sync::Arc};

pub use protocol::message::server_dll_web::{
    DayTime, ServerDllToWeb as MsgIn, ServerStatus, ServerWebToDll as MsgOut,
};
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
enum StatusKind {
    Online,
    Unwell,
    Offline,
    Unknown,
}

impl From<StatusKind> for StatusDisplay {
    fn from(kind: StatusKind) -> StatusDisplay {
        StatusDisplay { kind, status: None }
    }
}

#[derive(Debug, Serialize)]
struct StatusDisplay {
    kind: StatusKind,
    status: Option<ServerStatus>,
}

fn _number_emoji(num: u32) -> String {
    let normal = format!("{}", num);
    let mut buf = String::with_capacity(normal.len() * 2);
    for char in normal.chars() {
        buf.push(char);
        buf.push('\u{fe0f}');
        buf.push('\u{20e3}');
    }
    buf
}

fn _players_rus(num: u32) -> &'static str {
    match num {
        1 => "игрок",
        2..=4 => "игрока",
        _ => "игроков",
    }
}

impl StatusDisplay {
    fn condition(&self) -> Condition {
        Condition {
            name: self.condition_name(),
            color: self.condition_color(),
            //emoji: self.condition_emoji(),
        }
    }
    fn condition_name(&self) -> String {
        crate::templates::render("status.html", self, Default::default()).unwrap_or_else(|err| {
            eprintln!("StatusDisplay render error:\n{}\n{:?}", err, self);
            "<STATUS ERROR>".into()
        })
    }
    fn condition_color(&self) -> ConditionColor {
        match self.kind {
            StatusKind::Online => ConditionColor::Green,
            StatusKind::Unwell => ConditionColor::Yellow,
            StatusKind::Offline => ConditionColor::Red,
            StatusKind::Unknown => unreachable!(),
        }
    }
    /*fn condition_emoji(&self) -> String {
        match self {
            StatusDisplay::Online(ServerStatus { connections }) => match connections {
                0 => "desert",
                1 => "one",
                2 => "two",
                3 => "three",
                4 => "four",
                5 => "five",
                6 => "six",
                7 => "seven",
                8 => "eight",
                9 => "nine",
                _ => "fire",
            },
            StatusDisplay::Unwell => "dizzy_face",
            StatusDisplay::Offline => "stop_sign",
            StatusDisplay::Unknown => unreachable!(),
        }
        .to_owned()
    }*/
}

pub struct Status {
    current: StatusDisplay,
    //new: Option<(ServerStatus, Instant)>,
    new: Option<ServerStatus>,
}
impl Status {
    pub fn new() -> Self {
        Self {
            current: StatusDisplay {
                kind: StatusKind::Unknown,
                status: None,
            },
            new: None,
        }
    }
    pub fn update(&mut self, server: ServerStatus) {
        //self.new = Some((server, Instant::now()));
        self.new = Some(server);
    }
    pub async fn new_status(&mut self, mrhandy: &mrhandy::MrHandy) {
        use StatusKind::*;
        let new = match (&self.current.kind, self.new.take()) {
            (Online, None) => Unwell.into(),
            (Unknown, None) => Offline.into(),
            (_, None) => return,
            (Online, Some(ref new)) if self.current.status.as_ref() == Some(new) => return,
            (_, Some(new)) => StatusDisplay {
                status: Some(new),
                kind: Online,
            },
        };
        let condition = new.condition();

        if mrhandy.set_activity(condition).await {
            self.current = new;
            self.new = None;
        }
    }
}

#[derive(Clone)]
pub struct Bridge {
    sender: Arc<RwLock<Option<MsgOutSender>>>,
    //server: Option<Server>,
}
impl Bridge {
    pub fn new() -> Self {
        Bridge {
            sender: Arc::new(RwLock::new(None)), //Arc::new(AtomicCell::new(None))
                                                 //server: None,
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
    pub fn start(state: Arc<AppState>) -> impl Future<Output = Server> {
        /*if self.server.is_some() {
            panic!("Bridge server is already running");
        }*/
        let data = BridgeData { state };
        start_impl(data)
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
    state: Arc<AppState>,
}
impl BridgeData {
    fn root(&self) -> &Root {
        &self.state.sled_db.root
    }
    fn bridge(&self) -> &Bridge {
        &self.state.bridge
    }
}

async fn start_impl(data: BridgeData) -> Server {
    Server::build()
        .workers(1)
        .bind(
            // configure service pipeline
            "bridge",
            data.state.config.bridge.addr,
            move || {
                let data = data.clone();
                // service for converting incoming TcpStream to a SslStream<TcpStream>
                fn_service(move |tcp_stream: TcpStream| {
                    let data = data.clone();

                    let (sender, receiver) = channel(128);
                    data.bridge().set_sender(sender);

                    let framed = Framed::new(tcp_stream, WebSide);
                    let (sink, stream) = framed.split();

                    futures::stream::select(
                        stream
                            .map_err(BridgeError::Bincode)
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
        .run()
}

fn handle_message_async(
    msg_in: MsgIn,
    data: BridgeData,
) -> impl Future<Output = BridgeResult<MsgOut>> {
    async move {
        match msg_in {
            MsgIn::PlayerConnected(player_id) => Ok(MsgOut::SendConfig {
                player_id,
                url: CStr::from_bytes_with_nul(data.state.config.host.overlay_urls().as_bytes())
                    .expect("Can't fail, null byte supplied")
                    .to_owned(),
            }),
            MsgIn::PlayerAuth(cr_id) => {
                let root = data.root().clone();
                let fut = blocking(move || {
                    let owner =
                        ownership::get_ownership(&root, cr_id).map_err(BridgeError::Versioned)?;
                    if owner.is_some() {
                        return Ok(None);
                    }
                    let default: [u8; 12] = loop {
                        let random = rand::random();
                        if random != [0u8; 12] {
                            break random;
                        }
                    };
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
                    Ok(Some(authkey))
                })
                .map_ok(move |authkey| {
                    MsgOut::SendKeyToPlayer(cr_id, authkey.unwrap_or([0u32; 3]))
                });
                fut.await
            }
            MsgIn::DiscordSendMessage { channel, text } => {
                if let Some(mrhandy) = data.state.mrhandy.as_ref() {
                    let _ = mrhandy.send_message(channel, text).await;
                }
                Ok(MsgOut::Nop)
            }
            MsgIn::Status(server) => {
                let mut status = data.state.server_status.lock().await;
                status.update(server);
                Ok(MsgOut::Nop)
            }
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
    type Error = bincode::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        use bincode::ErrorKind as BinKind;
        use std::io::ErrorKind;
        if src.is_empty() {
            return Ok(None);
        }
        match partial_read(src, |buf| bincode::deserialize_from(buf)) {
            Err(err) => {
                if let BinKind::Io(err) = &*err {
                    if let ErrorKind::UnexpectedEof = err.kind() {
                        return Ok(None);
                    }
                }
                Err(err)
            }
            Ok(ok) => Ok(Some(ok)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_partial_read() {
        use bytes::BufMut;
        use std::io::Read;
        let mut bytes = BytesMut::new();
        bytes.put_slice(b"Hello world!");
        let res: std::io::Result<_> = partial_read(&mut bytes, |buf| {
            let mut hello = [0u8; 5];
            buf.read_exact(&mut hello)?;
            Ok(hello)
        });
        assert_eq!(&res.unwrap(), &b"Hello"[..]);
        assert_eq!(&bytes, &b" world!"[..]);
    }

    fn render_status(status: StatusDisplay) -> String {
        crate::templates::render("status.html", &status, Default::default()).unwrap()
    }

    #[test]
    fn test_render_status() {
        use StatusKind::*;
        let _config = crate::config::setup().expect("config.toml file");

        assert_eq!(
            "Нет связи",
            render_status(StatusDisplay {
                kind: Offline,
                status: None
            })
        );
        assert_eq!(
            "Серверу плохо",
            render_status(StatusDisplay {
                kind: Unwell,
                status: None
            })
        );
        assert_eq!(
            "Пустошь, День",
            render_status(StatusDisplay {
                kind: Online,
                status: Some(ServerStatus {
                    connections: 0,
                    day_time: DayTime::Day
                })
            })
        );
        assert_eq!(
            "Игроков: 1, Ночь",
            render_status(StatusDisplay {
                kind: Online,
                status: Some(ServerStatus {
                    connections: 1,
                    day_time: DayTime::Night
                })
            })
        );
    }

    /*#[test]
    fn test_bytes_reader() {
        use bytes::{Buf, BufMut};
        use std::io::Read;
        let mut bytes = BytesMut::new();
        bytes.put_slice(b"Hello world!");
        let res = {
            let mut reader = bytes.reader();
            let mut hello = [0u8; 5];
            reader.read_exact(&mut hello).unwrap();
            hello
        };
        assert_eq!(&res, &b"Hello"[..]);
        assert_eq!(&bytes, &b" world!"[..]);
    }*/
}

fn partial_read<O, E>(src: &mut BytesMut, fun: fn(&mut &[u8]) -> Result<O, E>) -> Result<O, E> {
    use bytes::Buf;
    let mut buf: &[u8] = &*src;
    let mut len = buf.len();
    let ret = fun(&mut buf)?;
    if buf.len() > len {
        panic!("buffer bigger than it was");
    }
    len -= buf.len();
    if len > 0 {
        src.advance(len);
    }
    Ok(ret)
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
    Bincode(bincode::Error),
    Blocking,
    TryInto,
    SenderDropped,
}

impl From<BlockingError> for BridgeError {
    fn from(_err: BlockingError) -> Self {
        BridgeError::Blocking
    }
}
