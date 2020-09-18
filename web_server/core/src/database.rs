pub type ArcSlice = sled::IVec;

mod versioned;
pub use self::versioned::VersionedError;

mod tree;
pub use tree::{Leaf, Root};

mod character;
pub use character::CharTrunk;

mod tools;

#[derive(Clone)]
pub struct SledDb {
    db: sled::Db,
    pub root: Root,
}

impl SledDb {
    pub fn new(db: sled::Db) -> Self {
        let tree = db.open_tree("fo4rp").expect("Can't open 'fo4rp' Tree");
        let root = Root::new(tree);
        SledDb { db, root }
    }
}
