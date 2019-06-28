use super::*;
use bincode::{deserialize, deserialize_from, serialize, serialize_into};
use std::net::Shutdown;

pub struct BridgeClient<MsgIn, MsgOut> {
    _in: PhantomData<MsgIn>,
    _out: PhantomData<MsgOut>,
    stream: Option<TcpStream>,
}

impl<MIn: 'static + Send + DeserializeOwned + Debug, MOut: 'static + Send + Serialize + Debug>
    BridgeTask for BridgeClient<MIn, MOut>
{
    type MsgIn = MIn;
    type MsgOut = MOut;

    fn new() -> Self {
        BridgeClient {
            _in: PhantomData,
            _out: PhantomData,
            stream: None,
        }
    }
    fn process(worker: &mut BridgeWorker<Self>) -> Result<(), BridgeError> {
        println!("Bridge client: connecting");
        let timeout = Duration::from_millis(2000);
        let mut stream =
            TcpStream::connect_timeout(worker.address(), timeout).map_err(BridgeError::Io)?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(BridgeError::Io)?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(BridgeError::Io)?;

        println!("Bridge client: handshake");
        serialize_into(&mut stream, &worker.handshake()).map_err(BridgeError::BinCode)?;
        let handshake = deserialize_from(&mut stream).map_err(BridgeError::BinCode)?;
        worker.check_handshake(handshake)?;

        worker.task().stream = Some(stream.try_clone().map_err(BridgeError::Io)?);
        worker.set_online();
        println!("Bridge client: online");

        let sender = worker.sender();
        let mut reader = stream.try_clone().map_err(BridgeError::Io)?;
        worker.spawn_reader(move || with_bincode::reader(reader, sender));

        loop {
            let msg_out = worker.receive()?;
            if let BridgeMessage::Shutdown = msg_out {
                println!("Bridge client writer: shutdown");
                return Ok(());
            }
            serialize_into(&mut stream, &msg_out).map_err(BridgeError::BinCode)?;
            if let BridgeMessage::Hang = msg_out {
                println!("Bridge client writer: hang");
                worker.task().shutdown();
                break;
            }
        }
        Err(BridgeError::Hang)
    }
    fn shutdown(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
}
