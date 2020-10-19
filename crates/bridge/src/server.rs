use super::*;
use bincode::{deserialize, deserialize_from, serialize, serialize_into};
use std::net::{Shutdown, TcpListener};

pub struct BridgeServer<MsgIn, MsgOut> {
    _in: PhantomData<MsgIn>,
    _out: PhantomData<MsgOut>,
    stream: Option<TcpStream>,
}

impl<MIn: 'static + Send + DeserializeOwned + Debug, MOut: 'static + Send + Serialize + Debug>
    BridgeTask for BridgeServer<MIn, MOut>
{
    type MsgIn = MIn;
    type MsgOut = MOut;

    fn new() -> Self {
        BridgeServer {
            _in: PhantomData,
            _out: PhantomData,
            stream: None,
        }
    }
    fn process<S: Channel<Self::MsgIn>>(
        worker: &mut BridgeWorker<Self, S>,
    ) -> Result<(), BridgeError> {
        /*let mut stream = TcpStream::connect_timeout(
            &worker.address(),
            std::time::Duration::from_micros(1000),
        ).map_err(BridgeError::Io)?;*/
        println!("Bridge server: binding");
        let mut listener = TcpListener::bind(worker.address()).map_err(BridgeError::Io)?;
        listener.set_nonblocking(true).map_err(BridgeError::Io)?;
        let timeout = Some(Duration::from_millis(2000));

        println!("Bridge server: listening...");
        let (mut stream, addr) = loop {
            sleep(Duration::from_millis(100));
            match listener.accept() {
                Ok(ok) => break ok,
                Err(err) => match err.kind() {
                    std::io::ErrorKind::WouldBlock => {
                        let msg_out = worker.try_receive()?;
                        if let Some(BridgeMessage::Shutdown) = msg_out {
                            println!("Bridge server writer: shutdown");
                            return Ok(());
                        }
                    }
                    _ => return Err(BridgeError::Io(err)),
                },
            }
        };

        println!("Bridge server: incoming");
        stream.set_nonblocking(false).map_err(BridgeError::Io)?;
        stream.set_nodelay(true).map_err(BridgeError::Io)?;
        stream.set_read_timeout(timeout).map_err(BridgeError::Io)?;
        stream.set_write_timeout(timeout).map_err(BridgeError::Io)?;

        println!("Bridge server: handshake");
        serialize_into(&mut stream, &worker.handshake()).map_err(BridgeError::BinCode)?;
        let handshake = deserialize_from(&mut stream).map_err(BridgeError::BinCode)?;
        worker.check_handshake(handshake)?;

        worker.task().stream = Some(stream.try_clone().map_err(BridgeError::Io)?);
        worker.set_online();
        println!("Bridge server: online");

        let sender = worker.sender();
        let mut reader = BufReader::new(stream.try_clone().map_err(BridgeError::Io)?);
        worker.spawn_reader(move || with_bincode::reader(reader, sender));

        loop {
            let msg_out = worker.receive()?;
            if let BridgeMessage::Shutdown = msg_out {
                println!("Bridge server writer: shutdown");
                return Ok(());
            }
            serialize_into(&mut stream, &msg_out).map_err(BridgeError::BinCode)?;
            if let BridgeMessage::Hang = msg_out {
                println!("Bridge server writer: hang");
                return Err(BridgeError::Hang);
            }
        }
    }
    fn shutdown(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
}

/*
fn serve(sender: &mut Sender<Vec<Avatar>>) -> std::io::Result<()> {
    use std::io::Read;

    let mut listener = TcpListener::bind("127.0.0.1:33741")?;
    let mut buf = Vec::with_capacity(128);

    for stream in listener.incoming() {
        //let _res = receive_avatars(stream?);
        stream?.read_to_end(&mut buf)?;

        let size = buf.len()/std::mem::size_of::<Avatar>();
        let slice = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const Avatar, size) };
        sender.send(slice.to_owned());
        buf.clear();
    }
    Ok(())
}

pub fn start() -> Receiver<Vec<Avatar>> {
    let (mut sender, receiver) = channel();
    thread::spawn(move || {
        loop {
            if let Err(err) = serve(&mut sender) {
                eprint!("{:?}", err);
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
    receiver
}
*/
