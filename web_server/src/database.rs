use std::sync::Arc;
use actix::prelude::{Actor, SyncContext, Message, Handler};

mod versioned;

mod image;
pub use self::image::GetImage;

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
            root: Arc::new(root), fo4rp
        }
    }
}

impl Actor for SledDb {
    type Context = SyncContext<Self>;
}
