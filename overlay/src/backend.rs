use crate::{avatar_window::AvatarWindow, image_data::ImageData, Rect};
use std::fmt::Debug;

pub type BackendError<B> = <<B as Backend>::Window as BackendWindow>::Error;

pub trait Backend {
    type Window: BackendWindow;
    fn new() -> Self;
    fn new_window(&self) -> Result<Self::Window, BackendError<Self>>;
    fn poll_events(&mut self) -> bool;
}

pub trait BackendWindow {
    type Texture;
    type Error: Debug;
    //fn change(pos: Some<(i32, i32)>, size: Option<(u32, u32)>, show: Option<bool>) -> bool;
    fn show(&mut self);
    fn hide(&mut self);
    fn set_position(&mut self, x: i32, y: i32);
    fn create_texture(&mut self, image: &mut ImageData) -> Result<Self::Texture, Self::Error>;
    fn draw_texture(
        &mut self,
        texture: &Self::Texture,
        src: &Rect,
        dst: &Rect,
    ) -> Result<(), Self::Error>;
    fn drop_texture(&mut self, texture: Self::Texture);
    fn handle(&self) -> *mut ();
}

mod sdl;
pub use sdl::SdlBackend;
