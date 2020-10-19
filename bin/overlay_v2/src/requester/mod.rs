mod downloader;
mod reqres;
use downloader::{Downloader, DownloaderError};
use reqres::{Requester, Responder};
use viewports::{
    dependencies::imgui::TextureId,
    wgpu::{ImageData, Wgpu},
};

use crate::bridge::Char;
use std::{
    collections::HashMap,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

pub struct ImageRequester {
    inner: Requester<Char, ImageData, DownloaderError>,
    images: HashMap<Char, AvatarImage>,
    textures: HashMap<u32, AvatarTexture>,
    free: bool,
}

#[derive(Debug, Copy, Clone)]
struct AvatarTexture {
    id: TextureId,
    ver: u32,
    last_used: Instant,
}

pub enum AvatarImage {
    Image {
        data: ImageData,
        last_access: Instant,
    },
    Error {
        err: DownloaderError,
        last_attempt: Instant,
    },
}

impl ImageRequester {
    pub fn start(url: String) -> ImageRequester {
        let responder = Arc::new(Responder::new());
        let requester = responder.clone();
        let thread = thread::spawn(move || {
            let client = reqwest::blocking::ClientBuilder::new()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("reqwest client");
            let mut downloader = Downloader { url, client };
            let _res = downloader.serve(&responder);
            println!("Downloader thread is exiting");
        });
        ImageRequester {
            inner: Requester::new(requester, thread),
            images: Default::default(),
            textures: Default::default(),
            free: true,
        }
    }
    fn update_inner(&mut self) -> Option<(Char, Result<ImageData, DownloaderError>)> {
        self.free = self.inner.is_free();
        if self.free {
            self.inner.receive()
        } else {
            None
        }
    }
    pub fn update(&mut self) {
        let now = Instant::now();
        let ok_treshold = Duration::from_secs(5 * 60);
        let err_treshold = Duration::from_secs(30);
        self.images.retain(|_, result| match result {
            AvatarImage::Image { last_access, .. } if now - *last_access > ok_treshold => false,
            AvatarImage::Error { last_attempt, .. } if now - *last_attempt > err_treshold => false,
            _ => true,
        });
        if let Some((for_char, new_image)) = self.update_inner() {
            match new_image {
                Ok(image) => {
                    self.images.insert(
                        for_char,
                        AvatarImage::Image {
                            data: image,
                            last_access: Instant::now(),
                        },
                    );
                }
                Err(err) => {
                    self.images.insert(
                        for_char,
                        AvatarImage::Error {
                            err,
                            last_attempt: Instant::now(),
                        },
                    );
                }
            }
        }
    }
    fn image_for_char(&mut self, for_char: Char) -> Option<&ImageData> {
        let make_attempt = match self.images.get_mut(&for_char) {
            Some(AvatarImage::Image { data, last_access }) => {
                *last_access = Instant::now();
                return Some(data);
            }
            None => true,
            _ => false,
        };
        if make_attempt && self.free {
            self.inner.send(for_char);
            self.free = false;
        }
        None
    }

    pub fn with_renderer<'a>(&'a mut self, renderer: &'a mut Wgpu) -> TextureRequester<'a> {
        TextureRequester {
            requester: self,
            renderer,
        }
    }
}

pub struct TextureRequester<'a> {
    requester: &'a mut ImageRequester,
    renderer: &'a mut Wgpu,
}

impl<'a> TextureRequester<'a> {
    pub fn texture_for_char(&mut self, for_char: Char) -> Option<TextureId> {
        let old_texture_id = match self.requester.textures.get_mut(&for_char.id) {
            Some(AvatarTexture { id, ver, last_used }) if for_char.ver == *ver => {
                *last_used = Instant::now();
                return Some(*id);
            }
            Some(AvatarTexture { id, .. }) => Some(*id),
            None => None,
        };
        if let Some(data) = self.requester.image_for_char(for_char) {
            let id = self.renderer.upload_image(data, old_texture_id);
            self.requester.textures.insert(
                for_char.id,
                AvatarTexture {
                    id,
                    ver: for_char.ver,
                    last_used: Instant::now(),
                },
            );
            Some(id)
        } else {
            None
        }
    }
    pub fn texture_for_cr_id(&mut self, cr_id: u32) -> Option<TextureId> {
        match self.requester.textures.get_mut(&cr_id) {
            Some(AvatarTexture {
                id,
                ver: _,
                last_used,
            }) => {
                *last_used = Instant::now();
                Some(*id)
            }
            _ => None,
        }
    }
    pub fn is_free(&self) -> bool {
        self.requester.free
    }
}
