use actix_web::{web, HttpRequest, HttpResponse, Responder, error::BlockingError};
use actix_web::body::Body;
use futures::{future::{err as fut_err, ok as fut_ok, Either}, Future};
use crate::{templates, database::GetImage};
use serde::{Serialize, Deserialize};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

//use futures::stream::Stream;

const IMAGE_SIZE: u32 = 128;

#[derive(Debug, Serialize)]
struct Charsheet{}

pub fn edit(_req: HttpRequest) -> impl Responder {
    //let to = req.match_info().get("name").unwrap_or("World");
    //format!("Hello there and go to hell!")

    match templates::render("upload.html", &Charsheet{}) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(err) => {
            eprintln!("Charsheet upload error: {:#?}", err);
            HttpResponse::InternalServerError().into()
        }
    }
}
/*
fn avatar_upload(multipart: actix_multipart::Multipart) -> impl Future<Item = HttpResponse, Error = Error> {
    use futures::{Future, Stream};
    println!("Multipart");
    multipart
        .map_err(actix_web::error::ErrorInternalServerError)
        .map(|field| println!("Field: {:?}", &field))
        //.flatten()
        .collect()
        .map(|_| HttpResponse::Ok().finish())
}
*/

#[derive(Deserialize)]
pub struct VersionKey {
    ver: Option<u32>,
    key: Option<u32>,
}

pub fn show(path: web::Path<u32>, query: web::Query<VersionKey>, data: web::Data<super::AppState>) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let VersionKey{ver, key} = *query;

    if key.is_none() {
        return Either::A(fut_ok(HttpResponse::Forbidden().finish()));
    }

    Either::B(
        data.sled_db.send(GetImage{id: *path, ver, key})
            .then(|res| {
                match res {
                    Ok(Ok(Some(image))) => {
                        HttpResponse::Ok().content_type("image/png").body(image)
                    },
                    Ok(Ok(None)) => {
                        HttpResponse::NotFound().finish()
                    },
                    Ok(Err(err) )=> {
                        HttpResponse::InternalServerError().body(format!("Error: {:?}", err))
                    },
                    Err(err) => {
                        HttpResponse::InternalServerError().body(format!("Error: {:?}", err))
                    }
                }
            })
    )
}

pub fn upload(req: HttpRequest, payload: web::Bytes, data: web::Data<super::AppState>) -> impl Future<Item = HttpResponse, Error = AvatarUploadError> {
    //println!("{:?}", req);
    //println!("{:?}", payload);
    //let payload = std::str::from_utf8(payload.as_ref()).unwrap();
    //let url = url::Url::parse(payload);
    //println!("{:?}", url);

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

    let tree = Arc::clone(&data.get_ref().root.fo4rp);

    Either::B(
        web::block(move || {
            let data = &payload[PREFIX_LEN..];
            let decoded = base64::decode_config(&data, base64::STANDARD).map_err(AvatarUploadError::Base64)?;
            //std::fs::write("test.png", &decoded).map_err(|_| ())
            let image = image::load_from_memory_with_format(&decoded, image::PNG).map_err(AvatarUploadError::ImageLoad)?;

            use image::GenericImageView;
            //println!("Width: {}, Height: {}", image.width(), image.height());
            if image.width() != IMAGE_SIZE || image.height() != IMAGE_SIZE {
                return Err(AvatarUploadError::ImageSize(image.width(), image.height()));
            }
            //image.save(&path).map_err(AvatarUploadError::ImageSave)
            let mut buffer = decoded;
            buffer.clear();
            image.write_to(&mut buffer, image::PNG).map_err(AvatarUploadError::ImageWrite)?;

            tree.set(format!("avatar/{:08X}/secret/{:08X}", 7, 8), &9u32.to_be_bytes()).map_err(AvatarUploadError::SledSet)?;
            tree.set(format!("avatar/{:08X}/image/{:08X}", 7, 8), buffer).map_err(AvatarUploadError::SledSet)?;
            Ok(())
        })
            .map_err(|err: BlockingError<AvatarUploadError>| {
                match err {
                    BlockingError::Error(err) => err,
                    BlockingError::Canceled => AvatarUploadError::Blocking,
                }
            })
            .map(|_| HttpResponse::Ok().finish())
    )
}

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
    SledSet(sled::Error),
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

/*
fn avatar_upload(multipart: actix_multipart::Multipart) -> impl Future<Item = HttpResponse, Error = Error> {
    use futures::{Future, Stream};
    use actix_form_data::Form;
    let form = Form::new()

    println!("Multipart");
    multipart
        .map_err(actix_web::error::ErrorInternalServerError)
        .map(|field| println!("Field: {:?}", &field))
        //.flatten()
        .collect()
        .map(|_| HttpResponse::Ok().finish())
}
*/