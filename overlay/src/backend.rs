use crate::{avatar_window::AvatarWindow, image_data::ImageData, Rect};
use std::fmt::Debug;
use std::rc::Rc;

pub type BackendError<B> = <<B as Backend>::Window as BackendWindow>::Error;
pub type ImGuiTextures<W> = imgui::Textures<Rc<<W as BackendWindow>::Texture>>;

pub trait Backend {
    type Window: BackendWindow;
    fn new() -> Self;
    fn new_window(&self, title: String, width: u32, height: u32) -> Result<Self::Window, BackendError<Self>>;
    fn new_popup(&self, title: String, width: u32, height: u32) -> Result<Self::Window, BackendError<Self>>;
    fn poll_events(&mut self) -> bool;
}

pub trait BackendWindow {
    type Texture;
    type Error: Debug;
    type Context;
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
    fn init_gui<F>(&mut self, init_context: F) -> Result<(), Self::Error>
        where F: FnMut(&mut imgui::Context, GuiInfo) -> Result<(),()>;
    fn draw_gui<F>(&mut self, run_ui: F) -> Result<(), Self::Error>
        where F: FnMut(&imgui::Ui, &Rc<Self::Context>, &mut ImGuiTextures<Self>) -> bool;
    fn drop_texture(&mut self, texture: Self::Texture);
    fn handle(&self) -> *mut ();
}

pub struct GuiInfo {
    pub hidpi_factor: f64,
}

#[cfg(feature = "backend-sdl")]
pub mod sdl;

#[cfg(feature = "backend-winit-gl")]
pub mod winit_gl;
