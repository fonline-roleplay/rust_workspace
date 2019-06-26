use super::*;

use std::time::Instant;

impl<T: BridgeTask> BridgeHandle<T> {
    pub fn is_online(&self) -> bool {
        self.online.load(Ordering::Relaxed)
    }
    /*pub(super) fn hang(&mut self) {
        let _ = self.sender.send(BridgeMessage::Hang);
    }*/
    pub fn start(addr: SocketAddr, handshake: u16, version: u16) -> Self {
        let (sender_out, receiver_out) = channel();
        let (sender_in, receiver_in) = channel();

        let inner = BridgeWorker::<T>::new(receiver_out, sender_in, sender_out.clone(), addr, handshake, version);
        let online = inner.online_handle();

        let thread = std::thread::spawn(move || {
            inner.thread();
            println!("BridgeHandle: worker stopped")
        });

        BridgeHandle {sender: sender_out, receiver: receiver_in, thread, online, last_ping: Instant::now()}
    }
    pub fn finish(self, join: bool) -> Result<(), BridgeError> {
        println!("BridgeHandle: finish");
        let res = self.sender.send(BridgeMessage::Shutdown).map_err(|_| BridgeError::ChannelDropped);
        println!("BridgeHandle: shutdown sent: {:?}", res);
        if join {
            let _res2 = self.thread.join();
        } else {
            sleep(Duration::from_millis(1000));
        }
        println!("BridgeHandle: finished");

        res
    }

    pub fn send(&mut self, msg: T::MsgOut) -> Result<(), BridgeError> {
        self.sender.send(BridgeMessage::Data(msg)).map_err(|_| BridgeError::ChannelDropped)
    }
    pub fn receive<'a>(&'a mut self) -> impl Iterator<Item=T::MsgIn> +'a {
        //self.sender.send(BridgeMessage::Data(msg)).map_err(|_| BridgeError::ChannelDropped)
        self.receiver.try_iter().filter_map(|msg| {
            match msg {
                BridgeMessage::Data(data) => Some(data),
                _ => None,
            }
        })
    }
    pub fn ping(&mut self) -> Result<(), BridgeError> {
        if self.last_ping.elapsed() > Duration::from_millis(1000) {
            self.sender.send(BridgeMessage::Ping).map_err(|_| BridgeError::ChannelDropped)?;
            self.last_ping = Instant::now();
        }
        Ok(())
    }
}

pub struct BridgeHandle<T: BridgeTask> {
    sender: Sender<BridgeMessage<T::MsgOut>>,
    receiver: Receiver<BridgeMessage<T::MsgIn>>,
    thread: JoinHandle<()>,
    online: Arc<AtomicBool>,
    last_ping: Instant,
}
