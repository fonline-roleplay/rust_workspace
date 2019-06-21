use crate::{
    bridge,
    database::{CharTrunk, Leaf, TreeRoot, VersionedError},
    templates,
};
use actix_web::body::Body;
use actix_web::{error::BlockingError, web, HttpRequest, HttpResponse, Responder};
use futures::{
    future::{err as fut_err, ok as fut_ok, Either},
    Future,
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

// size of square image in pixels, 128 means 128x128
const IMAGE_SIZE: u32 = 128;

#[derive(Deserialize)]
pub struct VersionSecret {
    ver: Option<u32>,
    secret: Option<u32>,
}

// ===== Avatar editor =====

#[derive(Debug, Serialize)]
struct AvatarEditor {
    char_id: u32,
}

pub fn edit(path: web::Path<u32>) -> impl Responder {
    match templates::render("edit_avatar.html", &AvatarEditor { char_id: *path }) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(err) => {
            eprintln!("AvatarEditor error: {:#?}", err);
            HttpResponse::InternalServerError().into()
        }
    }
}

// ===== Show avatar =====

pub fn show(
    path: web::Path<u32>,
    query: web::Query<VersionSecret>,
    data: web::Data<super::AppState>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let VersionSecret { ver, secret } = *query;

    if secret.is_none() {
        return Either::A(fut_ok(HttpResponse::Forbidden().finish()));
    }

    let root = data.sled_db.root.clone();
    Either::B(
        web::block(move || {
            let instant = std::time::Instant::now();
            let leaf = CharTrunk::new(*path, ver).get_image(&root, secret)?;
            println!("Getting image, completed in {:?}", instant.elapsed());
            Ok(leaf)
        })
        .from_err()
        .then(|res| match res {
            Ok(image) => HttpResponse::Ok()
                .header("Q-ver", image.ver.to_string())
                .content_type("image/png")
                .body(image.data),
            Err(VersionedError::NotFound) => HttpResponse::NotFound().finish(),
            Err(err) => HttpResponse::InternalServerError().body(format!("Error: {:?}", err)),
        }),
    )
}

// ===== Upload avatar =====

pub fn upload(
    path: web::Path<u32>,
    _req: HttpRequest,
    payload: web::Bytes,
    data: web::Data<super::AppState>,
) -> impl Future<Item = HttpResponse, Error = AvatarUploadError> {
    const MIN_LEN: usize = 16;
    const MAX_LEN: usize = 128 * 1024;

    const PREFIX_LEN: usize = 22;
    const PREFIX: &[u8; PREFIX_LEN] = b"data:image/png;base64,";

    if payload.len() <= PREFIX_LEN || !payload.starts_with(PREFIX) {
        return Either::A(fut_err(AvatarUploadError::DataUrl));
    }

    let data_len = payload.len() - PREFIX_LEN;
    if data_len < MIN_LEN || data_len > MAX_LEN {
        return Either::A(fut_err(AvatarUploadError::DataLength(data_len)));
    }

    let char_id = *path;
    let root = data.sled_db.root.clone();
    let sender = data.bridge.get_sender();
    Either::B(
        web::block(move || {
            let data = &payload[PREFIX_LEN..];
            save_image(&root, char_id, data)
        })
        .from_err()
        .and_then(move |leaf| update_char_leaf(sender, char_id, leaf))
        .map(|_| HttpResponse::Ok().finish()),
    )
}

fn save_image(tree: &TreeRoot, char_id: u32, data: &[u8]) -> Result<Leaf<()>, AvatarUploadError> {
    let instant = std::time::Instant::now();
    let decoded =
        base64::decode_config(&data, base64::STANDARD).map_err(AvatarUploadError::Base64)?;
    println!("Decoded in {:?}", instant.elapsed());
    let instant2 = std::time::Instant::now();
    //std::fs::write("test.png", &decoded).map_err(|_| ())
    let image = image::load_from_memory_with_format(&decoded, image::PNG)
        .map_err(AvatarUploadError::ImageLoad)?;
    println!("Loaded in {:?}", instant2.elapsed());
    let instant2 = std::time::Instant::now();
    use image::GenericImageView;
    //println!("Width: {}, Height: {}", image.width(), image.height());
    if image.width() != IMAGE_SIZE || image.height() != IMAGE_SIZE {
        return Err(AvatarUploadError::ImageSize(image.width(), image.height()));
    }
    let mut buffer = decoded;
    buffer.clear();
    image
        .write_to(&mut buffer, image::PNG)
        .map_err(AvatarUploadError::ImageWrite)?;
    println!("Writed in {:?}", instant2.elapsed());
    let instant2 = std::time::Instant::now();

    let leaf = CharTrunk::new(char_id, None)
        .set_image(tree, buffer)
        .map_err(AvatarUploadError::SledVersioned)?;
    println!("Saved to db in {:?}", instant2.elapsed());

    println!("Fully saved in {:?}", instant.elapsed());

    Ok(leaf)
}

fn update_char_leaf(
    sender: Option<bridge::MsgOutSender>,
    id: u32,
    leaf: Leaf<()>,
) -> Result<(), AvatarUploadError> {
    match (sender, leaf) {
        (
            Some(mut sender),
            Leaf {
                ver,
                secret: Some(secret),
                ..
            },
        ) => sender
            .try_send(bridge::MsgOut::UpdateCharLeaf { id, ver, secret })
            .map_err(|_| AvatarUploadError::FuturesSyncSend),
        _ => Ok(()),
    }
}

// ===== AvatarUploadError =====

#[derive(Debug)]
pub enum AvatarUploadError {
    DataUrl,
    DataLength(usize),
    Blocking,
    Base64(base64::DecodeError),
    ImageLoad(image::ImageError),
    ImageSize(u32, u32),
    //ImageSave(std::io::Error),
    ImageWrite(image::ImageError),
    //SledSet(sled::Error),
    Mailbox(actix::MailboxError),
    SledVersioned(VersionedError),
    FuturesSyncSend,
}

impl From<BlockingError<AvatarUploadError>> for AvatarUploadError {
    fn from(err: BlockingError<AvatarUploadError>) -> Self {
        match err {
            BlockingError::Error(err) => err,
            BlockingError::Canceled => AvatarUploadError::Blocking,
        }
    }
}

impl std::fmt::Display for AvatarUploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl actix_web::error::ResponseError for AvatarUploadError {
    fn error_response(&self) -> HttpResponse {
        log::warn!("{:?}", self);

        use actix_web::http::StatusCode;
        HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
    }
    /*
    /// Constructs an error response
    fn render_response(&self) -> HttpResponse {


        use actix_web::{http::{header, StatusCode}, body::Body};

        let mut resp = self.error_response();
        let mut buf = web::BytesMut::new();
        let _ = write!(Writer(&mut buf), "{}", self);
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/plain"),
        );
        resp.set_body(Body::from(buf))
    }*/
}
/*
pub(crate) struct Writer<'a>(pub &'a mut web::BytesMut);

impl<'a> std::io::Write for Writer<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
*/
