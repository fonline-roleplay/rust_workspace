use crate::{
    avatar_window::AvatarWindow,
    backend::{
        Backend, BackendError, BackendRef, BackendWindow, GuiEvent, ImGuiTextures, WindowRef,
    },
};
use std::collections::btree_map::{BTreeMap, Entry};

pub struct Windowing<B: Backend> {
    pub windows: BTreeMap<u32, AvatarWindow<B>>,
    pub char_textures: BTreeMap<u32, imgui::TextureId>,
    pub textures: ImGuiTextures<B>,
    pub backend: BackendRef<B>,
}

pub trait TextureForChar {
    fn texture_for_char(&mut self, char: u32) -> Option<imgui::TextureId>;
    fn debug(&self);
}

impl<B: Backend> TextureForChar for Windowing<B> {
    fn texture_for_char(&mut self, char: u32) -> Option<imgui::TextureId> {
        if let Some(id) = self.char_textures.get(&char) {
            return Some(*id);
        }
        let texture = self
            .windows
            .get(&char)
            .and_then(|window| window.texture())?;
        let id = self.textures.insert(texture);
        self.char_textures.insert(char, id);
        Some(id)
    }
    fn debug(&self) {
        println!("windows: {}", self.windows.len());
        for (char, window) in &self.windows {
            println!("{}: {:?}", char, window.char_texture);
        }
        println!("textures: {:?}", self.textures);
        println!("char_textures: {:?}", self.char_textures);
    }
}

impl<B: Backend> Windowing<B> {
    pub fn new(backend: BackendRef<B>) -> Self {
        Windowing {
            windows: BTreeMap::new(),
            char_textures: BTreeMap::new(),
            textures: imgui::Textures::new(),
            backend,
        }
    }
    pub fn window_for_char(&mut self, char: u32) -> Result<&mut AvatarWindow<B>, BackendError<B>> {
        if let Entry::Vacant(vacant) = self.windows.entry(char) {
            let inner = self
                .backend
                .borrow_mut()
                .new_popup("FOnlineOverlay".into(), 64, 64)?;
            let window = AvatarWindow::new(inner);
            vacant.insert(window);
        }
        Ok(self.windows.get_mut(&char).unwrap())
    }
    pub fn get_window_mut(&mut self, char: u32) -> Option<&mut AvatarWindow<B>> {
        self.windows.get_mut(&char)
    }
    pub fn maintain(&mut self, frame: u64, updated: bool) {
        let mut windows_to_drop = Vec::new();
        for (char, window) in &mut self.windows {
            if !window.maintain(frame, updated) {
                windows_to_drop.push(*char);
            }
        }
        for char in &windows_to_drop {
            self.windows.remove(char);
        }
    }
    pub fn poll_events(&mut self) -> bool {
        let mut exit = false;
        self.backend.borrow_mut().poll_events(|event| {
            if event.is_close_request() {
                exit = true;
            }
            //platform.handle_event(imgui.io_mut(), &window, &event);
        });
        exit
    }
}

pub trait OverlayWindow<B: Backend> {
    fn backend_window(&self) -> &WindowRef<B>;
}
