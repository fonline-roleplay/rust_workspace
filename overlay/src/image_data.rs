use crate::{downloader::Image, SdlError};
use sdl2::{pixels::PixelFormatEnum, surface::Surface};

pub struct ImageData {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    pitch: u32,
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
    pub fn surface(&mut self) -> Result<Surface, SdlError> {
        Surface::from_data(
            &mut self.bytes,
            self.width,
            self.height,
            self.pitch,
            PixelFormatEnum::RGB24,
        )
        .map_err(SdlError::SurfaceFromData)
    }
}
