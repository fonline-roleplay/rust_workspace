use super::{tools::slice_to_u64, CharTrunk, Root, VersionedError};

pub fn get_ownership(root: &Root, char_id: u32) -> Result<Option<u64>, VersionedError> {
    let result = root
        .trunk(char_id, None, CharTrunk::default())
        .get_bare_branch("owner_id");
    match result {
        Ok(owner) => {
            let user_id = slice_to_u64(&*owner);
            if user_id.is_none() {
                eprintln!("Invalid owner, char_id: {}, bytes: {:?}", char_id, &*owner);
            }
            Ok(user_id)
        }
        Err(VersionedError::NotFound) => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn set_ownership(root: &Root, char_id: u32, user_id: u64) -> Result<(), VersionedError> {
    let new_owner = user_id.to_be_bytes();
    let result = root
        .trunk(char_id, None, CharTrunk::default())
        .get_bare_branch_or_default("owner_id", &new_owner, |_| true);
    match result {
        // aleady same owner
        Ok(Some(owner)) if &*owner == &new_owner => Ok(()),
        // successfully setted
        Ok(None) => Ok(()),
        Err(err) => Err(err),
        _ => Err(VersionedError::AccessDenied),
    }
}

pub fn get_auth(root: &Root, char_id: u32) -> Result<Option<sled::IVec>, VersionedError> {
    let result = root
        .trunk(char_id, None, CharTrunk::default())
        .get_bare_branch("authkey");
    match result {
        Ok(authkey) => Ok(Some(authkey)),
        Err(VersionedError::NotFound) => Ok(None),
        Err(err) => Err(err),
    }
}
