use super::ImageData;
use crate::bridge::Char;

const MIN_LEN: usize = 16;
const MAX_LEN: usize = 128 * 1024;

pub struct Downloader {
    pub(super) url: String,
    pub(super) client: reqwest::blocking::Client,
}

impl Downloader {
    pub(super) fn serve(
        &mut self,
        responder: &super::Responder<Char, ImageData, DownloaderError>,
    ) -> Result<(), ()> {
        loop {
            let char = responder.wait_question()?;
            match self.process(char) {
                Ok(image) => {
                    responder.set_answer(image)?;
                }
                Err(err) => {
                    eprintln!("serve: {:?}", err);
                    responder.set_err(err)?;
                }
            }
        }
    }

    fn process(&self, char: Char) -> Result<ImageData, DownloaderError> {
        let bytes = self.download(char)?;
        let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
            .map_err(DownloaderError::ImageLoad)?;
        Ok(ImageData::from_image(image))
    }

    fn download(&self, char: Char) -> Result<Vec<u8>, DownloaderError> {
        let url = format!(
            "{}/char/{}/avatar?ver={}&secret={}",
            self.url, char.id, char.ver, char.secret
        );

        let mut res = self.client.get(&url).send().map_err(DownloaderError::Get)?;
        let len = res
            .headers()
            .get("q-length")
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
}

#[derive(Debug)]
pub enum DownloaderError {
    Get(reqwest::Error),
    ContentLength(usize),
    ContentLengthMissmatch(usize, usize),
    Body(reqwest::Error),
    ImageLoad(image::ImageError),
}
