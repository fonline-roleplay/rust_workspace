use std::{
    net::{TcpListener, TcpStream},
    thread::{self, Thread, JoinHandle},
    time::Duration,
    sync::{
        Arc,
        atomic::AtomicBool
    },
};
use super::Char;
use crate::reqres::{Requester, Responder};

use image::ImageRgb8 as ImageVariant;
pub use image::RgbImage as Image;

const MIN_LEN: usize = 16;
const MAX_LEN: usize = 128 * 1024;

pub fn start() -> Requester<Char, Image, DownloaderError> {
    let responder = Arc::new(Responder::new());
    let requester= responder.clone();
    let thread = thread::spawn(move || {
        loop {
            if let Err(err) = serve(&responder) {
                eprint!("start: {:?}", err);
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
    Requester::new(requester, thread)
}

fn serve(responder: &Responder<Char, Image, DownloaderError>) -> std::io::Result<()> {
    loop {
        let char = responder.wait_question();
        match process(char) {
            Ok(image) => {
                responder.set_answer(image);
            },
            Err(err) => {
                eprintln!("serve: {:?}", err);
                responder.set_err(err);
            }
        }
    }
}

fn process(char: Char) -> Result<Image, DownloaderError> {
    let bytes = download(char)?;
    let image = image::load_from_memory_with_format(&bytes, image::PNG).map_err(DownloaderError::ImageLoad)?;
    match image {
        ImageVariant(image) => Ok(image),
        _ => Err(DownloaderError::WrongPixelFormat)
    }
}

fn download(char: Char) -> Result<Vec<u8>, DownloaderError> {
    let url = format!("http://localhost:8000/char/{}/avatar?ver={}&secret={}", char.id, char.ver, char.secret);
    let mut res = reqwest::get(&url).map_err(DownloaderError::Get)?;
    let len = res.headers().get("q-length")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.parse().ok())
        .unwrap_or(0u64) as usize;

    if MIN_LEN > len || len > MAX_LEN {
        return Err(DownloaderError::ContentLength(len));
    }
    let mut bytes = Vec::with_capacity(len);
    res.copy_to(&mut bytes).map_err(DownloaderError::Body)?;

    let actual_len = bytes.len();
    if actual_len != len {
        return Err(DownloaderError::ContentLengthMissmatch(len, actual_len));
    }

    Ok(bytes)
}

#[derive(Debug)]
pub enum DownloaderError {
    Get(reqwest::Error),
    ContentLength(usize),
    ContentLengthMissmatch(usize, usize),
    Body(reqwest::Error),
    ImageLoad(image::ImageError),
    WrongPixelFormat,
}
