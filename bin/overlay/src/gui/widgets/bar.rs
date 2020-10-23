use super::{FixedPosition, GuiState, TextureRequester, UiLogic, Layer};
use crate::gui::state::color::Color;
use crate::imgui::{self, im_str, ImStr, ImString, Ui};

pub struct Bar {
    pub show_chat: bool,
    pub show_chars_panel: bool,
    pub show_faces: bool,
    button: ToggleButton,
    size: (u32, u32),
}
impl Bar {
    pub fn new() -> Self {
        Bar {
            show_chat: true,
            show_chars_panel: true,
            show_faces: true,
            button: ToggleButton::new(),
            size: Self::INITIAL_SIZE,
        }
    }
}

struct ToggleButton {
    on: Color,
    off: Color,
}
impl ToggleButton {
    fn new() -> Self {
        ToggleButton {
            on: Color {
                normal: [0.3, 0.7, 0.1, 1.0],
                lighter: [0.2, 0.8, 0.3, 1.0],
                darker: [0.4, 0.6, 0.2, 1.0],
            },
            off: Color {
                normal: [0.7, 0.3, 0.1, 1.0],
                lighter: [0.8, 0.2, 0.3, 1.0],
                darker: [0.6, 0.4, 0.2, 1.0],
            },
        }
    }
    fn button_color(&self, active: bool) -> &Color {
        if active {
            &self.on
        } else {
            &self.off
        }
    }
    fn toggle(&self, ui: &Ui, label: &ImStr, size: [f32; 2], val: &mut bool) -> bool {
        if self.button_color(*val).button(ui, label, size) {
            *val = !*val;
            true
        } else {
            false
        }
    }
}

const PADDING: u8 = 5;

impl UiLogic for Bar {
    const INITIAL_SIZE: (u32, u32) = (320, 35);
    const TITLE_BAR: bool = false;
    fn title(&self) -> ImString {
        im_str!("FOnline Bar").into()
    }
    fn draw(
        &mut self,
        ui: &imgui::Ui,
        _state: &mut GuiState,
        _texture_requester: &mut TextureRequester,
    ) {
        let size = [0.0, 24.0];
        if self
            .button
            .toggle(ui, im_str!("Чат"), size, &mut self.show_chat)
        {}
        ui.same_line(0.0);
        if self
            .button
            .toggle(ui, im_str!("Люди"), size, &mut self.show_chars_panel)
        {}
        ui.same_line(0.0);
        if self
            .button
            .toggle(ui, im_str!("Лица"), size, &mut self.show_faces)
        {}
        ui.same_line(0.0);
        let cursor = ui.cursor_pos();
        self.size.0 = cursor[0].ceil() as u32;
        self.size.1 = cursor[1].ceil() as u32;
    }
    fn fixed_position(&self, _state: &GuiState) -> Option<FixedPosition> {
        Some(FixedPosition::TopRight)
    }
    fn fixed_size(&self, _state: &GuiState) -> Option<(u32, u32)> {
        Some(self.size)
    }
    fn padding(&self, _state: &GuiState) -> Option<(u8, u8)> {
        Some((PADDING, PADDING))
    }
    fn layer(&self) -> Layer {
        Layer::Bottom
    }
}
