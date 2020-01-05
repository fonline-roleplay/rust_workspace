use super::{
    tree::{Bark, Leaf, Root, Trunk},
    versioned::VersionedError,
    ArcSlice,
};
use bytes::Bytes;

//pub type CharTrunk<'a> = Trunk<'a, CharTrun>;
pub struct CharTrunk {
    image_branch: &'static str,
}
impl Default for CharTrunk {
    fn default() -> Self {
        CharTrunk {
            image_branch: "avatar",
        }
    }
}
impl Bark for CharTrunk {
    fn secret(&self) -> &str {
        "secret"
    }
    fn counter(&self) -> &str {
        "ver"
    }
    fn trunk(&self) -> &str {
        "char"
    }
}

impl<'a> Trunk<'a, CharTrunk> {
    pub fn get_image(&self, input_key: Option<u32>) -> Result<Leaf<ArcSlice>, VersionedError> {
        self.get_versioned(self.bark().image_branch, input_key)
    }
    pub fn set_image(&self, data: Vec<u8>) -> Result<Leaf<()>, VersionedError> {
        self.set_versioned(self.bark().image_branch, data)
    }
}
