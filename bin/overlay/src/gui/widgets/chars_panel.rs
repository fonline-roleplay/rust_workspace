use super::{GuiState, TextureRequester, UiLogic, super::state::Character};
use crate::imgui::{im_str, ChildWindow, ImString, ImageButton, Ui};
use std::time::{Instant, Duration};

const AVATAR_SIZE: [f32; 2] = [64.0; 2];

#[derive(Debug, Copy, Clone, PartialEq)]
enum Mode {
    Faces,
    All,
}

pub(crate) struct CharsPanel{
    mode: Mode,
}

impl CharsPanel {
    pub(crate) fn new() -> Self {
        Self{
            mode: Mode::Faces,
        }
    }
    fn draw_inner<'a>(&mut self, ui: &Ui, texture_requester: &mut TextureRequester, characters: impl Iterator<Item = (&'a Character, Option<Duration>)>, now: Instant) {
        let mut size = ui.content_region_avail();
        size[1] -= 5.0;
        ChildWindow::new("Персонажи")
            .size(size.into())
            .border(true)
            .build(ui, || {
                ui.columns(2, im_str!("columns"), false);
                ui.set_current_column_width(AVATAR_SIZE[0] + 16.0);
                for (character, last_seen_elapsed) in characters {
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
                    if let Some(last_seen_elapsed) = last_seen_elapsed {
                        ui.text(im_str!(
                            "Видели: {} секунд назад",
                            last_seen_elapsed.as_secs()
                        ));
                    }

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
}

impl UiLogic for CharsPanel {
    const INITIAL_SIZE: (u32, u32) = (480, 640);
    const TITLE_BAR: bool = true;
    fn title(&self) -> ImString {
        im_str!("Characters",)
    }
    fn draw(&mut self, ui: &Ui, state: &mut GuiState, texture_requester: &mut TextureRequester) {
        let now = Instant::now();

        ui.radio_button(im_str!("Лица"), &mut self.mode, Mode::Faces);
        ui.same_line(0.0);
        ui.radio_button(im_str!("Все"), &mut self.mode, Mode::All);

        match &self.mode {
            Mode::Faces => {
                let mut characters: Vec<_> = state
                    .characters_iter()
                    .filter_map(|(id, character)| Some((*id, character, now - character.last_seen()?)))
                    .collect();
                characters
                    .sort_by_key(|(char_id, _character, last_seen_elapsed)| (*last_seen_elapsed, *char_id));
                let iter = characters.iter().map(|(_id, character, last_seen_elapsed)| (*character, Some(*last_seen_elapsed)));
                self.draw_inner(ui, texture_requester, iter, now);
            }
            Mode::All => {
                let iter = state.characters_iter().map(|(_id, character)| {
                    let last_seen_elapsed = character.last_seen().map(|seen| now-seen);
                    (character, last_seen_elapsed)
                });
                self.draw_inner(ui, texture_requester, iter, now);
            }
        };
    }
    fn padding(&self, _state: &GuiState) -> Option<(u8, u8)> {
        Some((5, 5))
    }
}
