use super::state::GuiState;
use crate::imgui::{ImString, Ui};
use crate::{requester::TextureRequester, Rect};

pub(super) mod avatar;
pub(super) mod bar;
pub(super) mod chars_panel;
pub(super) mod chat;

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
