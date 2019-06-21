use actix::prelude::{Actor, Handler, Message, SyncContext};
use std::sync::Arc;

mod versioned;
pub use self::versioned::VersionedError;

mod image;
pub use self::image::{GetImage, SetImage};

mod tools;

#[derive(Clone)]
pub struct SledDb {
    root: Arc<sled::Db>,
    pub fo4rp: Arc<sled::Tree>,
}

impl SledDb {
    pub fn new(root: sled::Db) -> Self {
        let fo4rp = root.open_tree("fo4rp").expect("Can't open 'fo4rp' Tree");
        SledDb {
            root: Arc::new(root),
            fo4rp,
        }
    }
}

impl Actor for SledDb {
    type Context = SyncContext<Self>;
}
