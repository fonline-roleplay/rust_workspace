use crate::{avatar_window::AvatarWindow, image_data::ImageData, Rect};
use std::fmt::Debug;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub type BackendEvent<B> = <B as Backend>::Event;
pub type BackendTexture<B> = <B as Backend>::Texture;
pub type BackendContext<B> = <B as Backend>::Context;
pub type BackendError<B> = <B as Backend>::Error;
pub type BackendResult<T, B> = Result<T, BackendError<B>>;
pub type ImGuiTextures<B> = imgui::Textures<Rc<BackendTexture<B>>>;
pub type BackendRef<B> = Rc<RefCell<B>>;
pub type WindowRef<B> = Rc<RefCell<<B as Backend>::Window>>;
pub type WindowWeak<B> = Weak<RefCell<<B as Backend>::Window>>;

pub trait Backend
where
    Self: Sized,
{
    type Window: BackendWindow<Back = Self>;
    type Event: GuiEvent<Self>;
    type Texture: Debug;
    type Error: Debug;
    type Context;
    fn new() -> Self;
    fn new_window(
        &mut self,
        title: String,
        width: u32,
        height: u32,
    ) -> BackendResult<WindowRef<Self>, Self>;
    fn new_popup(
        &mut self,
        title: String,
        width: u32,
        height: u32,
    ) -> BackendResult<WindowRef<Self>, Self>;
    fn poll_events<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Self::Event);
    fn drop_texture(context: &Rc<BackendContext<Self>>, texture: &BackendTexture<Self>) {}
}

pub trait BackendWindow {
    type Back: Backend;
    //fn change(pos: Some<(i32, i32)>, size: Option<(u32, u32)>, show: Option<bool>) -> bool;
    fn show(&mut self);
    fn hide(&mut self);
    fn to_foreground(&mut self);
    fn set_position(&mut self, x: i32, y: i32);
    fn set_size(&mut self, x: u32, y: u32);
    fn move_by_f32(&mut self, x: f32, y: f32);
    fn create_texture(
        &mut self,
        image: &mut ImageData,
    ) -> BackendResult<BackendTexture<Self::Back>, Self::Back>;
    fn draw_texture(
        &mut self,
        texture: &BackendTexture<Self::Back>,
        src: &Rect,
        dst: &Rect,
    ) -> BackendResult<(), Self::Back>;
    fn init_gui<F>(&mut self, init_context: F) -> BackendResult<(), Self::Back>
    where
        F: FnMut(&mut imgui::Context, GuiInfo) -> Result<(), ()>;
    fn draw_gui<F>(&mut self, run_ui: F) -> BackendResult<(), Self::Back>
    where
        F: FnMut(
            &imgui::Ui,
            &Rc<BackendContext<Self::Back>>,
            &mut ImGuiTextures<Self::Back>,
        ) -> Vec<imgui::TextureId>;
    fn handle_event(&mut self, event: &BackendEvent<Self::Back>) -> bool;
    fn drop_texture(&mut self, texture: BackendTexture<Self::Back>) {}
    fn window_handle(&self) -> *mut ();
}

pub trait GuiEvent<B: Backend> {
    fn is_close_request(&self) -> bool;
}

pub struct GuiInfo {
    pub hidpi_factor: f64,
}

#[cfg(feature = "backend-sdl")]
pub mod sdl;

#[cfg(feature = "backend-winit-gl")]
pub mod winit_gl;
