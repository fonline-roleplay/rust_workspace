use super::{super::Avatar, FixedPosition, GuiState, TextureRequester, UiLogic, Layer};
use crate::imgui::{im_str, ImString, ImageButton, StyleVar, Ui};

impl UiLogic for Avatar {
    const INITIAL_SIZE: (u32, u32) = (64, 64);
    const TITLE_BAR: bool = false;
    fn title(&self) -> ImString {
        im_str!("Char_{}", self.char.id)
    }
    fn draw(&mut self, ui: &Ui, state: &mut GuiState, texture_requester: &mut TextureRequester) {
        let size = state.avatar_size as f32;
        match texture_requester.texture_for_char(self.char) {
            Some(texture_id) => {
                let style = ui.push_style_var(StyleVar::FramePadding([0.0; 2]));
                let avatar = ImageButton::new(texture_id, [size; 2]);
                avatar.build(ui);
                style.pop(ui);
            }
            None => {
                ui.button(im_str!("?"), [size, size]);
            }
        }
    }
    fn fixed_position(&self, state: &GuiState) -> Option<FixedPosition> {
        let size = state.avatar_size as i32;
        let (x, y) = position(self, size);
        Some(FixedPosition::TopLeft(x as f32, y as f32))
    }
    fn fixed_size(&self, state: &GuiState) -> Option<(u32, u32)> {
        Some((state.avatar_size as _, state.avatar_size as _))
    }
    fn visible(&self, state: &GuiState) -> bool {
        if let Some(rect) = state.game_rect {
            let size = state.avatar_size as i32;
            let (x, y) = position(self, size);
            (rect.width as i32 - size > x && x > 0) && (rect.height as i32 - size > y && y > 0)
        } else {
            false
        }
    }
    fn padding(&self, _state: &GuiState) -> Option<(u8, u8)> {
        Some((0, 0))
    }
    fn layer(&self) -> Layer {
        Layer::BottomMost
    }
}

fn position(avatar: &Avatar, size: i32) -> (i32, i32) {
    let x = avatar.pos.x - size / 2;
    let y = avatar.pos.y - size - 16;
    (x, y)
}
