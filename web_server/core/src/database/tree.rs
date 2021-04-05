use super::{
    tools::ivec_to_u32,
    versioned::{get_value, new_leaf, VersionedError},
    ArcSlice,
};
use std::{fmt::Write, ops::Bound};

#[derive(Clone)]
pub struct Root {
    tree: sled::Tree,
}
impl Root {
    pub fn new(tree: sled::Tree) -> Self {
        Root { tree }
    }
    pub fn tree(&self) -> &sled::Tree {
        &self.tree
    }
    pub fn trunk<B: Bark>(&self, id: u32, max_ver: Option<u32>, bark: B) -> Trunk<B> {
        Trunk {
            id,
            versions: versions(max_ver),
            bark,
            root: &self,
        }
    }
}

fn versions(ver: Option<u32>) -> (Bound<u32>, Bound<u32>) {
    ver.map(|v| (Bound::Unbounded, Bound::Included(v)))
        .unwrap_or((Bound::Unbounded, Bound::Unbounded))
}

pub trait Bark {
    fn secret(&self) -> &str;
    fn counter(&self) -> &str;
    fn trunk(&self) -> &str;
}

pub struct Trunk<'a, T: Bark> {
    id: u32,
    versions: (Bound<u32>, Bound<u32>),
    bark: T,
    root: &'a Root,
}

impl<'a, T: Bark> Trunk<'a, T> {
    pub fn bark(&self) -> &T {
        &self.bark
    }
    fn branch_key(&self, branch: &str) -> Result<String, VersionedError> {
        let mut key = String::with_capacity(32);
        write!(key, "{}/{:08X}/{}", self.bark.trunk(), self.id, branch)
            .map_err(VersionedError::WriteFmt)?;
        Ok(key)
    }
    fn _leaf_key(&self, branch: &str, leaf: u32) -> Result<String, VersionedError> {
        let mut key = String::with_capacity(32);
        write!(
            key,
            "{}/{:08X}/{}/{:08X}",
            self.bark.trunk(),
            self.id,
            branch,
            leaf
        )
        .map_err(VersionedError::WriteFmt)?;
        Ok(key)
    }
    fn check_secret(&self, input_key: Option<u32>) -> Result<bool, VersionedError> {
        let ver_secret = get_value(
            &self.root,
            self.bark.trunk(),
            self.id,
            self.bark.secret(),
            self.versions,
            ivec_to_u32,
        )?;
        Ok(match (ver_secret, input_key) {
            (_, None) => true,
            (None, _) => true,
            (Some((_ver, secret)), Some(input_key)) if secret == input_key => true,
            _ => false,
        })
    }

    pub fn get_versioned(
        &self,
        branch: &str,
        input_key: Option<u32>,
    ) -> Result<Leaf<ArcSlice>, VersionedError> {
        if !self.check_secret(input_key)? {
            return Err(VersionedError::AccessDenied);
        }
        let (ver, data) = get_value(
            &self.root,
            self.bark.trunk(),
            self.id,
            branch,
            self.versions,
            |buf| Ok(buf),
        )?
        .ok_or(VersionedError::NotFound)?;
        Ok(Leaf {
            data,
            ver,
            secret: None,
        })
    }

    pub fn set_versioned(&self, branch: &str, data: Vec<u8>) -> Result<Leaf<()>, VersionedError> {
        let mut secret = 0u32;
        while secret == 0 {
            secret = rand::random();
        }
        let secret_data = secret.to_be_bytes().to_vec();

        let ver = new_leaf(
            &self.root,
            self.bark.trunk(),
            self.id,
            self.bark.counter(),
            [(branch, data), (self.bark.secret(), secret_data)],
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

    // TODO: Use or remove
    /*pub fn update_branch<V, F>(&self, tree: &TreeRoot, branch: &str, f: F) -> Result<Option<sled::IVec>, VersionedError>
    where
        F: Fn(Option<&[u8]>) -> Option<V>>,
        sled::IVec: From<V>,
    {
        update_branch(
            tree,
            self.trunk,
            self.id,
            branch,
            f,
        )
    }*/

    pub fn get_bare_branch(&self, branch: &str) -> Result<sled::IVec, VersionedError> {
        let key = self.branch_key(branch)?;
        self.root
            .tree()
            .get(&key)
            .map_err(VersionedError::Sled)?
            .ok_or(VersionedError::NotFound)
    }

    pub fn get_bare_branch_or_default<F>(
        &self,
        branch: &str,
        default: &[u8],
        check: F,
    ) -> Result<Option<sled::IVec>, VersionedError>
    where
        F: for<'l> Fn(&'l [u8]) -> bool,
    {
        let key = self.branch_key(branch)?;
        let mut value = self.root.tree().get(&key).map_err(VersionedError::Sled)?;
        match &value {
            Some(value_ref) if check(value_ref.as_ref()) => {
                return Ok(value);
            }
            _ => {}
        }
        for _ in 0..10 {
            match self
                .root
                .tree()
                .compare_and_swap(&key, value, Some(default))
                .map_err(VersionedError::Sled)?
            {
                Ok(()) => {
                    return Ok(None);
                }
                Err(error) => {
                    eprintln!("Concurrent cas!");
                    match &error.current {
                        Some(other_ref) if check(other_ref.as_ref()) => {
                            return Ok(error.current);
                        }
                        _ => {
                            eprintln!("Concurrent cas with none!");
                            value = None;
                        }
                    }
                }
            }
        }
        Err(VersionedError::ConcurrentWrites)
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
