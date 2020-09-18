use super::{
    tools::{increment, slice_to_u32},
    ArcSlice, Root,
};
use actix_web::error::BlockingError;
use arrayvec::{Array, ArrayVec};
use bytes::Bytes;
use sled::IVec;
use std::{
    fmt::Write,
    ops::{Bound, RangeBounds},
};

#[derive(Debug)]
pub enum VersionedError {
    Sled(sled::Error),
    WriteFmt(std::fmt::Error),
    VersionEmpty,
    VersionUtf(std::str::Utf8Error),
    VersionParse(std::num::ParseIntError),
    ValueParse(ArcSlice),
    AccessDenied,
    CounterInvalid,
    UnexpectedOldValue,
    NotFound,
    Blocking,
    ConcurrentWrites,
}

impl From<BlockingError<VersionedError>> for VersionedError {
    fn from(err: BlockingError<VersionedError>) -> Self {
        match err {
            BlockingError::Error(err) => err,
            BlockingError::Canceled => VersionedError::Blocking,
        }
    }
}

//const MIN_U32: &str = "0000000000";
//const MAX_U32: &str = "4294967295";
const MIN_U32: &str = "00000000";
const MAX_U32: &str = "FFFFFFFF";

pub fn get_value<T, R: RangeBounds<u32>, F: Fn(IVec) -> Result<T, IVec>>(
    root: &Root,
    trunk: &str,
    id: u32,
    branch: &str,
    ver: R,
    parse: F,
) -> Result<Option<(u32, T)>, VersionedError> {
    let mut from = String::with_capacity(32);

    write!(from, "{}/{:08X}/{}/", trunk, id, branch).map_err(VersionedError::WriteFmt)?;
    let base_len = from.len();

    let mut to = from.clone();

    let lo = match ver.start_bound() {
        Bound::Included(inc) => {
            write!(to, "{:08X}", inc).map_err(VersionedError::WriteFmt)?;
            Bound::Included(to)
        }
        Bound::Excluded(exc) => {
            write!(to, "{:08X}", exc).map_err(VersionedError::WriteFmt)?;
            Bound::Excluded(to)
        }
        Bound::Unbounded => {
            to.push_str(MIN_U32);
            Bound::Included(to)
        }
    };

    let hi = match ver.end_bound() {
        Bound::Included(inc) => {
            write!(from, "{:08X}", inc).map_err(VersionedError::WriteFmt)?;
            Bound::Included(from)
        }
        Bound::Excluded(exc) => {
            write!(from, "{:08X}", exc).map_err(VersionedError::WriteFmt)?;
            Bound::Excluded(from)
        }
        Bound::Unbounded => {
            from.push_str(MAX_U32);
            Bound::Included(from)
        }
    };

    println!("from: {:?}, to: {:?}", &hi, &lo);

    for pair in root.tree().range((lo, hi)).rev() {
        let (full_key, value) = pair.map_err(VersionedError::Sled)?;
        println!("full_key: {:?}", std::str::from_utf8(&full_key));
        let key = full_key
            .get(base_len..)
            .ok_or(VersionedError::VersionEmpty)?;
        let key = std::str::from_utf8(key).map_err(VersionedError::VersionUtf)?;
        let key = u32::from_str_radix(key, 16).map_err(VersionedError::VersionParse)?;
        if !ver.contains(&key) {
            eprintln!("Strange version: {:?}", key);
            continue;
        }
        let value = parse(value).map_err(|ivec| VersionedError::ValueParse(ivec))?;
        return Ok(Some((key, value)));
    }
    Ok(None)
}

pub fn update_branch<V, F>(
    root: &Root,
    trunk: &str,
    id: u32,
    branch: &str,
    func: F,
) -> Result<Option<IVec>, VersionedError>
where
    F: Fn(Option<&[u8]>) -> Option<V>,
    IVec: From<V>,
{
    let mut key = String::with_capacity(32);
    write!(key, "{}/{:08X}/{}", trunk, id, branch).map_err(VersionedError::WriteFmt)?;
    root.tree()
        .update_and_fetch(key, func)
        .map_err(VersionedError::Sled)
}

pub fn inc_counter(root: &Root, trunk: &str, id: u32, branch: &str) -> Result<u32, VersionedError> {
    update_branch(root, trunk, id, branch, increment).and_then(|opt| {
        opt.and_then(|ivec| slice_to_u32(ivec.as_ref()))
            .ok_or(VersionedError::CounterInvalid)
    })
}

pub fn set_value<V>(
    root: &Root,
    trunk: &str,
    id: u32,
    branch: &str,
    ver: u32,
    value: V,
) -> Result<Option<IVec>, VersionedError>
where
    IVec: From<V>,
{
    let mut key = String::with_capacity(32);
    write!(key, "{}/{:08X}/{}/{:08X}", trunk, id, branch, ver).map_err(VersionedError::WriteFmt)?;

    root.tree().insert(key, value).map_err(VersionedError::Sled)
}

pub fn new_leaf<'a, V, A: Array<Item = (&'a str, V)>>(
    root: &Root,
    trunk: &str,
    id: u32,
    counter: &str,
    branch_values: A,
) -> Result<u32, VersionedError>
where
    IVec: From<V>,
{
    let ver = inc_counter(root, trunk, id, counter)?;
    let branch_values = ArrayVec::from(branch_values);
    for (branch, value) in branch_values {
        let old_value = set_value(root, trunk, id, branch, ver, value)?;
        if old_value.is_some() {
            eprintln!("Unexpected old value: {}/{}/{}/{}", trunk, id, branch, ver);
            //return Err(VersionedError::UnexpectedOldValue)
        }
    }
    Ok(ver)
}
