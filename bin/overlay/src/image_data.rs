use crate::downloader::Image;

pub struct ImageData {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
}

impl ImageData {
    pub fn new(image: Image) -> Self {
        ImageData {
            width: image.width(),
            height: image.height(),
            pitch: image.width() * 3,
            bytes: image.into_raw(),
        }
    }
}
