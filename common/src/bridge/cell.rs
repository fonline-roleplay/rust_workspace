use super::*;
use once_cell::sync::OnceCell;
use parking_lot::Mutex;

pub struct BridgeCell<H> {
    //TODO: get rid of OnceCell
    inner: OnceCell<Mutex<Option<H>>>,
}

impl<H> BridgeCell<H> {
    pub const fn new() -> Self {
        BridgeCell {
            inner: OnceCell::new(),
        }
    }
}

impl<T: BridgeTask> BridgeCell<BridgeHandle<T>> {
    pub fn connect(&self, addr: SocketAddr, handshake: u16, version: u16) {
        let inner = self.inner.get_or_init(|| Mutex::new(None));
        let mut guard = inner.lock();
        if let Some(mut old) = guard.take() {
            old.finish(true);
        }
        *guard = Some(BridgeHandle::<T>::start(addr, handshake, version));
    }

    pub fn with_online<O, F>(&self, mut f: F) -> Result<O, BridgeError>
    where
        F: FnMut(&mut BridgeHandle<T>) -> Result<O, BridgeError>,
    {
        self.with_some(|bridge| {
            if bridge.is_online() {
                f(bridge)
            } else {
                Err(BridgeError::NotOnline)
            }
        })
    }
    pub fn with_some<O, F>(&self, mut f: F) -> Result<O, BridgeError>
    where
        F: FnMut(&mut BridgeHandle<T>) -> Result<O, BridgeError>,
    {
        self.with(|bridge| match bridge {
            Some(bridge) => f(bridge),
            None => Err(BridgeError::EmptyBridgeCell),
        })
    }
    fn with<O, F>(&self, mut f: F) -> Result<O, BridgeError>
    where
        F: FnMut(&mut Option<BridgeHandle<T>>) -> Result<O, BridgeError>,
    {
        let inner = self.inner.get().ok_or(BridgeError::EmptyBridgeCell)?;
        let mut guard = inner.lock();

        f(&mut *guard)
    }
    pub fn finish(&self, join: bool) -> Result<(), BridgeError> {
        println!("BridgeCell: starting finish");
        self.with(|bridge| {
            if let Some(handle) = bridge.take() {
                println!("BridgeCell: about to finish");
                let res = handle.finish(join);
                println!("BridgeCell: finished {:?}", res);
                res
            } else {
                Ok(())
            }
        })
    }
}
