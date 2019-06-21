use super::{
    tools::slice_to_u32,
    versioned::{get_value, new_version, VersionedError},
    SledDb,
};
use actix::prelude::{Handler, Message};
use bytes::Bytes;
use std::ops::Bound;
use std::sync::Arc;

// Get image

fn get_image(
    tree: &Arc<sled::Tree>,
    id: u32,
    ver: Option<u32>,
    input_key: Option<u32>,
) -> Result<Option<Bytes>, VersionedError> {
    let ver = ver
        .map(|v| (Bound::Unbounded, Bound::Included(v)))
        .unwrap_or((Bound::Unbounded, Bound::Unbounded));
    let ver_secret = get_value(tree, "avatar", id, "secret", ver, slice_to_u32)?;
    match (ver_secret, input_key) {
        (_, None) => {}
        (None, _) => {}
        (Some((_ver, secret)), Some(input_key)) if secret == input_key => {}
        _ => return Err(VersionedError::AccessDenied),
    }

    let ver_value = get_value(tree, "avatar", id, "image", ver, |buf| {
        Some(Bytes::from(buf.as_ref()))
    })?;
    Ok(ver_value.map(|v| v.1))
}

pub struct GetImage {
    pub id: u32,
    pub ver: Option<u32>,
    pub key: Option<u32>,
}

impl Message for GetImage {
    type Result = Result<Option<Bytes>, VersionedError>;
}

impl Handler<GetImage> for SledDb {
    type Result = <GetImage as Message>::Result;

    fn handle(&mut self, msg: GetImage, _: &mut Self::Context) -> Self::Result {
        get_image(&self.fo4rp, msg.id, msg.ver, msg.key)
    }
}

// Set image

/*fn set_image(tree: &Arc<sled::Tree>, id: u32, data: Vec<u8>) -> Result<SetImageMeta, VersionedError> {
    unimplemented!()
}*/

pub struct SetImage {
    pub id: u32,
    pub data: Vec<u8>,
}

pub struct SetImageMeta {
    pub ver: u32,
    pub secret: u32,
}

impl Message for SetImage {
    type Result = Result<SetImageMeta, VersionedError>;
}

impl Handler<SetImage> for SledDb {
    type Result = <SetImage as Message>::Result;

    fn handle(&mut self, msg: SetImage, _: &mut Self::Context) -> Self::Result {
        let secret: u32 = rand::random();
        let secret_data = secret.to_be_bytes().to_vec();
        let ver = new_version(
            &self.fo4rp,
            "avatar",
            msg.id,
            "ver",
            [("image", msg.data), ("secret", secret_data)],
        )?;
        println!("new image, id: {}, ver: {}, key: {}", msg.id, ver, secret);
        Ok(SetImageMeta { ver, secret })
    }
}
