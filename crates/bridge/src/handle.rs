use super::*;

use std::time::Instant;

impl<T: BridgeTask> BridgeHandle<T> {
    pub fn start(addr: SocketAddr, handshake: u16, version: u16) -> Self {
        let (sender_in, receiver_in) = channel();

        Self::start_ext(addr, handshake, version, sender_in, receiver_in)
    }

    pub fn receive<'a>(&'a mut self) -> impl Iterator<Item = T::MsgIn> + 'a {
        //self.sender.send(BridgeMessage::Data(msg)).map_err(|_| BridgeError::ChannelDropped)
        self.receiver.try_iter().filter_map(|msg| match msg {
            BridgeMessage::Data(data) => Some(data),
            _ => None,
        })
    }
}

impl<T: BridgeTask, S: Channel<T::MsgIn>> BridgeHandle<T, S> {
    pub fn is_online(&self) -> bool {
        self.online.load(Ordering::Relaxed)
    }
    /*pub(super) fn hang(&mut self) {
        let _ = self.sender.send(BridgeMessage::Hang);
    }*/
    pub fn start_ext(
        addr: SocketAddr,
        handshake: u16,
        version: u16,
        sender_in: S,
        receiver_in: S::Receiver,
    ) -> Self {
        let (sender_out, receiver_out) = channel();

        let inner = BridgeWorker::<T, S>::new(
            receiver_out,
            sender_in,
            sender_out.clone(),
            addr,
            handshake,
            version,
        );
        let online = inner.online_handle();

        let thread = std::thread::spawn(move || {
            inner.thread();
            println!("BridgeHandle: worker stopped")
        });

        BridgeHandle {
            sender: sender_out,
            receiver: receiver_in,
            thread,
            online,
            last_ping: Instant::now(),
        }
    }
    pub fn finish(self, join: bool) -> Result<(), BridgeError> {
        println!("BridgeHandle: finish");
        let res = self
            .sender
            .send(BridgeMessage::Shutdown)
            .map_err(|_| BridgeError::ChannelDropped);
        println!("BridgeHandle: shutdown sent: {:?}", res);
        if join {
            println!("Thread join...");
            let _res2 = self.thread.join();
        } else {
            println!("Skip thread join.");
            //sleep(Duration::from_millis(1000));
        }
        println!("BridgeHandle: finished");

        res
    }

    pub fn send(&mut self, msg: T::MsgOut) -> Result<(), BridgeError> {
        self.sender
            .send(BridgeMessage::Data(msg))
            .map_err(|_| BridgeError::ChannelDropped)
    }
    pub fn ping(&mut self) -> Result<(), BridgeError> {
        if self.last_ping.elapsed() > Duration::from_millis(1000) {
            self.sender
                .send(BridgeMessage::Ping)
                .map_err(|_| BridgeError::ChannelDropped)?;
            self.last_ping = Instant::now();
        }
        Ok(())
    }
}

pub struct BridgeHandle<
    T: BridgeTask,
    S: Channel<T::MsgIn> = DefaultSender<<T as BridgeTask>::MsgIn>,
> {
    sender: Sender<BridgeMessage<T::MsgOut>>,
    receiver: S::Receiver,
    thread: JoinHandle<()>,
    online: Arc<AtomicBool>,
    last_ping: Instant,
}
