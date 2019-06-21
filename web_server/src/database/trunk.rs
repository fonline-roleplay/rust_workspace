use super::{
    tools::slice_to_u32,
    versioned::{get_value, new_leaf, VersionedError},
    SledDb, TreeRoot,
};
use bytes::Bytes;
use std::marker::PhantomData;
use std::ops::Bound;

pub struct Trunk<T> {
    secret_branch: &'static str,
    counter_branch: &'static str,
    image_branch: &'static str,
    trunk: &'static str,
    id: u32,
    versions: (Bound<u32>, Bound<u32>),
    _marker: PhantomData<T>,
}

fn versions(ver: Option<u32>) -> (Bound<u32>, Bound<u32>) {
    ver.map(|v| (Bound::Unbounded, Bound::Included(v)))
        .unwrap_or((Bound::Unbounded, Bound::Unbounded))
}

pub struct Char;
pub type CharTrunk = Trunk<Char>;

impl Trunk<Char> {
    pub fn new(id: u32, max_ver: Option<u32>) -> Trunk<Char> {
        Trunk {
            secret_branch: "secret",
            counter_branch: "ver",
            image_branch: "avatar",
            trunk: "char",
            id,
            versions: versions(max_ver),
            _marker: PhantomData,
        }
    }
}

impl<T> Trunk<T> {
    fn check_secret(
        &self,
        tree: &TreeRoot,
        input_key: Option<u32>,
    ) -> Result<bool, VersionedError> {
        let ver_secret = get_value(
            tree,
            self.trunk,
            self.id,
            self.secret_branch,
            self.versions,
            slice_to_u32,
        )?;
        Ok(match (ver_secret, input_key) {
            (_, None) => true,
            (None, _) => true,
            (Some((_ver, secret)), Some(input_key)) if secret == input_key => true,
            _ => false,
        })
    }

    pub fn get_image(
        &self,
        tree: &TreeRoot,
        input_key: Option<u32>,
    ) -> Result<Leaf<Bytes>, VersionedError> {
        if !self.check_secret(tree, input_key)? {
            return Err(VersionedError::AccessDenied);
        }
        let (ver, data) = get_value(
            tree,
            self.trunk,
            self.id,
            self.image_branch,
            self.versions,
            |buf| Some(Bytes::from(buf.as_ref())),
        )?
        .ok_or(VersionedError::NotFound)?;
        Ok(Leaf {
            data,
            ver,
            secret: None,
        })
    }

    pub fn set_image(&self, tree: &TreeRoot, data: Vec<u8>) -> Result<Leaf<()>, VersionedError> {
        let secret: u32 = rand::random();
        let secret_data = secret.to_be_bytes().to_vec();

        let ver = new_leaf(
            tree,
            self.trunk,
            self.id,
            self.counter_branch,
            [(self.image_branch, data), (self.secret_branch, secret_data)],
        )?;
        println!(
            "new image, id: {}, ver: {}, secret: {}",
            self.id, ver, secret
        );
        Ok(Leaf {
            data: (),
            ver,
            secret: Some(secret),
        })
    }
}

// Set image

/*fn set_image(tree: &Arc<sled::Tree>, id: u32, data: Vec<u8>) -> Result<SetImageMeta, VersionedError> {
    unimplemented!()
}*/

pub struct Leaf<T> {
    pub data: T,
    pub ver: u32,
    pub secret: Option<u32>,
}
