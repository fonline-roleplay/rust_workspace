use super::{GuiState, TextureRequester, UiLogic};
use crate::imgui::{self, im_str, ImString, Ui};

pub(super) struct Dummy(pub(super) u32);

impl UiLogic for Dummy {
    const INITIAL_SIZE: (u32, u32) = (256, 256);
    const TITLE_BAR: bool = true;
    fn title(&self) -> ImString {
        im_str!("Dummy{}", self.0)
    }
    fn draw(
        &mut self,
        ui: &Ui,
        _state: &mut GuiState,
        _texture_requester: &mut TextureRequester,
    ) {
        
    }
}
