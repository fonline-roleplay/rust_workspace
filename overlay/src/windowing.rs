use crate::{
    avatar_window::AvatarWindow,
    backend::{Backend, BackendError, BackendWindow, GuiEvent},
};
use std::collections::btree_map::{BTreeMap, Entry};

pub struct Windowing<B: Backend> {
    pub windows: BTreeMap<u32, AvatarWindow<B>>,
    backend: B,
}

impl<B: Backend> Windowing<B> {
    pub fn new() -> Self {
        Windowing {
            windows: BTreeMap::new(),
            backend: B::new(),
        }
    }
    pub fn window_for_char(&mut self, char: u32) -> Result<&mut AvatarWindow<B>, BackendError<B>> {
        if let Entry::Vacant(vacant) = self.windows.entry(char) {
            let inner = self.backend.new_popup("FOnlineOverlay".into(), 64, 64)?;
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
        self.backend.poll_events(|event| {
            if event.is_close_request() {
                exit = true;
            }
            //platform.handle_event(imgui.io_mut(), &window, &event);
        });
        exit
    }
}

pub trait OverlayWindow<B: Backend> {
    fn backend_window(&self) -> &B::Window;
}