use std::sync::Arc;

mod versioned;
pub use self::versioned::VersionedError;

mod trunk;
pub use trunk::{CharTrunk, Leaf};

mod tools;

#[derive(Clone)]
pub struct SledDb {
    db: sled::Db,
    pub root: TreeRoot,
}

#[derive(Clone)]
pub struct TreeRoot {
    inner: Arc<sled::Tree>,
}
impl TreeRoot {
    fn root(&self) -> &Arc<sled::Tree> {
        &self.inner
    }
}

impl SledDb {
    pub fn new(db: sled::Db) -> Self {
        let root = db.open_tree("fo4rp").expect("Can't open 'fo4rp' Tree");
        let root = TreeRoot { inner: root };
        SledDb { db, root }
    }
}
