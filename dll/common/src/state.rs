use parking_lot::RwLock;

pub struct StateSingleton<T> {
    lock: RwLock<Option<T>>,
}
impl<T> StateSingleton<T> {
    pub const fn new() -> Self {
        Self {
            lock: parking_lot::const_rwlock(None),
        }
    }
}

pub trait State: Sized + 'static {
    fn init() -> Self;
    fn singleton() -> &'static StateSingleton<Self>;
    fn with<R, F: FnOnce(&Self) -> R>(func: F) -> R {
        let guard = Self::singleton().lock.read();
        func(guard.as_ref().expect("Invalid dll state"))
    }
    fn with_mut<R, F: FnOnce(&mut Self) -> R>(func: F) -> R {
        let mut guard = Self::singleton().lock.write();
        func(guard.as_mut().expect("Invalid dll state"))
    }
    fn with_mut_option<R, F: FnOnce(&mut Option<Self>) -> R>(func: F) -> R {
        let mut guard = Self::singleton().lock.write();
        func(&mut *guard)
    }
    fn create() {
        Self::with_mut_option(|option| {
            if option.is_some() {
                panic!("Attempt to re-initialize dll state");
            }
            *option = Some(Self::init());
        })
    }
    fn destroy() {
        Self::with_mut_option(|option| {
            if option.is_none() {
                panic!("Attempt to destroy empty dll state");
            }
            *option = None;
        })
    }
}
