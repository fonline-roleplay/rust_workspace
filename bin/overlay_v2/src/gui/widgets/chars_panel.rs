use super::{GuiState, TextureRequester, UiLogic};
use crate::imgui::{im_str, ChildWindow, ImString, ImageButton, Ui};
use std::time::Instant;

const AVATAR_SIZE: [f32; 2] = [64.0; 2];

pub(crate) struct CharsPanel;

impl CharsPanel {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl UiLogic for CharsPanel {
    const INITIAL_SIZE: (u32, u32) = (480, 640);
    const TITLE_BAR: bool = true;
    fn title(&self) -> ImString {
        im_str!("Characters").into()
    }
    fn draw(&mut self, ui: &Ui, state: &mut GuiState, texture_requester: &mut TextureRequester) {
        let now = Instant::now();

        let mut characters: Vec<_> = state
            .characters_iter()
            .filter_map(|(id, character)| Some((*id, character, now - character.last_seen()?)))
            .collect();
        characters
            .sort_by_key(|(char_id, _character, last_seen_elapsed)| (*last_seen_elapsed, *char_id));
        //let visible = characters.iter().take_while(|(_, _, last_seen_elapsed)| last_seen_elapsed.as_millis()<1000).count();

        //ui.text(im_str!("Видимых персонажей: {}", visible));

        let mut size = ui.content_region_avail();
        size[1] -= 5.0;
        ChildWindow::new("Сообщения")
            .size(size.into())
            .border(true)
            .build(ui, || {
                ui.columns(2, im_str!("columns"), false);
                ui.set_current_column_width(AVATAR_SIZE[0] + 16.0);
                for (_char_id, character, last_seen_elapsed) in characters {
                    match character
                        .avatar()
                        .and_then(|avatar| texture_requester.texture_for_char(avatar))
                    {
                        Some(texture_id) => {
                            let avatar = ImageButton::new(texture_id, AVATAR_SIZE);
                            avatar.build(ui);
                        }
                        None => {
                            ui.button(im_str!("?"), AVATAR_SIZE);
                        }
                    }
                    //ui.same_line(0.0);
                    ui.next_column();
                    ui.text(im_str!("Имя: {}", character.name()));
                    ui.text(im_str!(
                        "Видели: {} секунд назад",
                        last_seen_elapsed.as_secs()
                    ));

                    if let Some(last_heard) = character.last_heard() {
                        ui.text(im_str!(
                            "Слышали: {} секунд назад",
                            (now - last_heard).as_secs()
                        ));
                    }
                    ui.next_column();
                }
            });
    }
    fn padding(&self, _state: &GuiState) -> Option<(u8, u8)> {
        Some((5, 5))
    }
}
