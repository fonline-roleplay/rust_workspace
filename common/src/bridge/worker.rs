use super::*;

pub struct BridgeWorker<T: BridgeTask> {
    receiver: Receiver<BridgeMessage<T::MsgOut>>,
    sender: Sender<BridgeMessage<T::MsgIn>>,
    service: Sender<BridgeMessage<T::MsgOut>>,
    addr: SocketAddr,
    thread: Option<JoinHandle<Result<(), BridgeError>>>,
    online: Arc<AtomicBool>,
    handshake: u16,
    version: u16,
    task: T,
}

impl<T: BridgeTask> BridgeWorker<T> {
    pub(super) fn new(receiver: Receiver<BridgeMessage<T::MsgOut>>, sender: Sender<BridgeMessage<T::MsgIn>>, service: Sender<BridgeMessage<T::MsgOut>>, addr: SocketAddr, handshake: u16, version: u16) -> Self{
        BridgeWorker {receiver, sender, service, addr, thread: None, online: Arc::new(AtomicBool::new(false)), handshake, version, task: T::new()}
    }
    pub(super) fn thread(mut self) {
        loop {
            println!("Bridge worker thread: starting process");
            let res = T::process(&mut self);
            self.online.store(false, Ordering::Relaxed);
            println!("Bridge worker thread: offline");
            self.task.shutdown();
            println!("Bridge worker thread: task shutted down");
            if let Some(thread) = self.thread.take() {
                match thread.join() {
                    Ok(Ok(())) => {},
                    Ok(Err(err)) => {
                        eprintln!("MsgIn thread error: {:?}", err);
                    },
                    Err(err) => {
                        eprintln!("MsgIn thread join error: {:?}", err);
                    }
                }
            }
            println!("Bridge worker thread: reader thread joined");
            match res {
                Ok(_) => {
                    println!("Bridge worker thread: exit");
                    return;
                },
                Err(err) => {
                    eprintln!("Bridge worker thread: {:?}", err);
                    sleep(Duration::from_millis(500));
                    eprintln!("Bridge worker slept");
                    for msg in self.receiver.try_iter() {
                        eprintln!("msg: {:?}", &msg);
                        if let msg = BridgeMessage::<T>::Shutdown {
                            eprintln!("Bridge worker returning");
                            return;
                        }
                    }
                },
            }
        }
    }
    pub fn address(&self) -> &SocketAddr {
        &self.addr
    }
    pub fn spawn_reader<F: 'static+Send+FnOnce()->Result<(), BridgeError>>(&mut self, mut f: F) {
        let service = self.service.clone();
        self.thread = Some(std::thread::spawn(move || {
            let res = f();
            if res.is_err() {
                eprintln!("Reader thread: {:?}", res);
            }
            println!("spawn_reader");
            service.send(BridgeMessage::Hang);
            println!("reader return");
            res
        }));
    }
    pub fn set_online(&mut self) {
        self.online.store(true, Ordering::Relaxed);
    }
    pub fn online(&mut self) -> bool {
        self.online.load(Ordering::Relaxed)
    }
    pub fn online_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.online)
    }

    pub fn handshake(&self) -> BridgeMessage<T::MsgOut> {
        BridgeMessage::<T::MsgOut>::Handshake(self.handshake, self.version)
    }
    pub fn check_handshake(&self, handshake: BridgeMessage<T::MsgIn>) -> Result<(), BridgeError> {
        match handshake {
            BridgeMessage::<T::MsgIn>::Handshake(handshake, version) => {
                if handshake != self.handshake || version != self.version {
                    Err(BridgeError::Handshake(handshake, version))
                } else {
                    Ok(())
                }
            },
            msg => {
                Err(BridgeError::NoHandshake)
            }
        }
    }
    pub fn sender(&self) -> Sender<BridgeMessage<T::MsgIn>> {
        self.sender.clone()
    }
    pub fn receive(&mut self) -> Result<BridgeMessage<T::MsgOut>, BridgeError> {
        self.receiver.recv().map_err(|_| BridgeError::ChannelDropped)
    }
    pub fn task(&mut self) -> &mut T {
        &mut self.task
    }
}
