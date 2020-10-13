use lazy_static::lazy_static;

pub use protocol::message::server_dll_web::{ServerDllToWeb as MsgOut, ServerWebToDll as MsgIn};
use std::{
    io::{Read, Write},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

//pub type MsgIn = message::ServerWebToDll;
//pub type MsgOut = message::ServerDllToWeb;

lazy_static! {
    static ref BRIDGE: Bridge = { Bridge::launch() };
}

struct Bridge {
    sender: Mutex<Sender<MsgOut>>,
    receiver: Mutex<Receiver<MsgIn>>,
    thread: JoinHandle<()>,
}

pub fn init() {
    &*BRIDGE;
}

pub fn send_one(message: MsgOut) -> bool {
    let sender = BRIDGE.sender.lock().expect("poisoned sender");
    if let Err(_) = sender.send(message) {
        return false;
    }
    true
}

pub fn send(messages: &[MsgOut]) -> bool {
    let sender = BRIDGE.sender.lock().expect("poisoned sender");
    for message in messages {
        if let Err(_) = sender.send(*message) {
            return false;
        }
    }
    true
}

pub fn receive() -> Vec<MsgIn> {
    let receiver = BRIDGE.receiver.lock().expect("poisoned receiver");
    receiver.try_iter().collect()
}

impl Bridge {
    fn launch() -> Bridge {
        let (send_out, mut recv_out) = channel();
        let (mut send_in, recv_in) = channel();

        let thread = thread::spawn(move || loop {
            let res = Bridge::run(send_in.clone(), &mut recv_out);
            eprintln!("Bridge loop: {:?}", res);
            thread::sleep(Duration::from_millis(500));
        });
        //panic!("panic test");

        Bridge {
            sender: Mutex::new(send_out),
            receiver: Mutex::new(recv_in),
            thread,
        }
    }
    fn run(sender: Sender<MsgIn>, receiver: &mut Receiver<MsgOut>) -> bincode::Result<()> {
        let stream = std::net::TcpStream::connect_timeout(
            &"127.0.0.1:33852".parse().unwrap(),
            Duration::from_millis(500),
        )?;
        //stream.set_read_timeout(Some(Duration::from_millis(500)));
        //stream.set_write_timeout(Some(Duration::from_millis(500)));
        let mut reader = stream;
        let mut writer = reader.try_clone()?;

        let read_thread = thread::spawn(move || -> bincode::Result<_> {
            loop {
                let msg: MsgIn = bincode::deserialize_from(&mut reader)?;
                /*let mut buf = [0u8; std::mem::size_of::<MsgIn>()];
                //assert_eq!(std::mem::align_of_val(&buf), std::mem::align_of::<MsgIn>());
                reader.read_exact(&mut buf)?;
                let msg: MsgIn = unsafe { std::mem::transmute(buf) };*/
                if let Err(err) = sender.send(msg) {
                    return Ok(err);
                }
            }
        });
        let write_res = (|| -> std::io::Result<_> {
            loop {
                match receiver.recv() {
                    Ok(msg) => {
                        //let mut buf = [0u8; std::(mem::size_of::<MsgOut>()];
                        println!("writing... {:?}", &msg);
                        let buf: [u8; std::mem::size_of::<MsgOut>()] =
                            unsafe { std::mem::transmute(msg) };
                        writer.write(&buf)?;
                        //println!("writed: {:?}", &buf[..]);
                    }
                    Err(err) => {
                        return Ok(err);
                    }
                };
            }
        })();
        eprintln!("Writer exited: {:?}, joining reader", write_res);
        let read_res = read_thread.join().expect("error joining reader thread");
        eprintln!("Reader exited: {:?}", read_res);
        write_res?;
        read_res?;

        Ok(())
    }
}
