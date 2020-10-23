use super::{state::GuiState, Layer};
use crate::imgui::{ImString, Ui};
use crate::{requester::TextureRequester, Rect};

mod avatar;
mod bar;
mod chars_panel;
mod chat;

use super::Avatar;
use bar::Bar;
use chars_panel::CharsPanel;
use chat::Chat;

pub(super) trait UiLogic {
    const INITIAL_SIZE: (u32, u32);
    const TITLE_BAR: bool;
    fn title(&self) -> ImString;
    fn draw(&mut self, ui: &Ui, state: &mut GuiState, texture_requester: &mut TextureRequester);
    fn fixed_position(&self, _state: &GuiState) -> Option<FixedPosition> {
        None
    }
    fn fixed_size(&self, _state: &GuiState) -> Option<(u32, u32)> {
        None
    }
    fn visible(&self, _state: &GuiState) -> bool {
        true
    }
    fn padding(&self, _state: &GuiState) -> Option<(u8, u8)> {
        None
    }
    fn layer(&self) -> Layer {
        Layer::Middle
    }
}

#[derive(Debug, Copy, Clone)]
pub(super) enum FixedPosition {
    TopLeft(f32, f32),
    TopRight,
}

impl FixedPosition {
    pub(super) fn apply(&self, rect: &Rect, size: [f32; 2]) -> [f32; 2] {
        match self {
            FixedPosition::TopLeft(x, y) => [rect.x as f32 + *x, rect.y as f32 + *y],
            FixedPosition::TopRight => [rect.x as f32 + rect.width as f32 - size[0], rect.y as f32],
        }
    }
}

pub(super) struct Widgets {
    bar: Bar,
    pub(super) chat: Chat,
    chars_panel: CharsPanel,
    pub(super) avatars: Vec<Avatar>,
}

impl Widgets {
    pub(super) fn new() -> Self {
        Self {
            bar: Bar::new(),
            chat: Chat::new(),
            chars_panel: CharsPanel::new(),
            avatars: vec![],
        }
    }
    pub(super) fn frame(&mut self, mut gui: super::GuiBundle) {
        if self.bar.show_faces {
            for avatar in &mut self.avatars {
                gui.render(avatar);
            }
        }

        gui.render(&mut self.bar);

        if self.bar.show_chars_panel {
            gui.render(&mut self.chars_panel);
        }

        if self.bar.show_chat {
            gui.render(&mut self.chat);
        }
    }
}
