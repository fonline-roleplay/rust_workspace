pub(crate) mod character;
pub(crate) mod color;

use super::{CharId, Message, AVATARS_SIZES, DEFAULT_AVATAR_SIZE_INDEX};
pub(super) use crate::{
    bridge::{Avatar, Char},
    Rect,
};
pub(super) use character::Character;
use std::{
    collections::hash_map::HashMap,
    time::Instant,
};

pub(crate) struct GuiState {
    pub(super) avatar_size: u16,
    pub(super) game_rect: Option<Rect>,
    pub(super) characters: HashMap<CharId, Character>,
}
impl GuiState {
    pub(super) fn new() -> Self {
        Self {
            avatar_size: AVATARS_SIZES[DEFAULT_AVATAR_SIZE_INDEX],
            game_rect: None,
            characters: Default::default(),
        }
    }
    pub(super) fn set_avatars_size(&mut self, size: u16) {
        self.avatar_size = size;
    }
    pub(crate) fn update_game_rect(&mut self, rect: Option<Rect>) {
        self.game_rect = rect;
    }
    pub(super) fn character(&mut self, char_id: CharId) -> &mut Character {
        self.characters
            .entry(char_id)
            .or_insert_with(|| Character::new(char_id))
    }
    pub(super) fn update_avatars(&mut self, avatars: &[Avatar]) {
        let now = Instant::now();
        for Avatar { char, .. } in avatars {
            let character = self.character(char.id);
            character.avatar = Some(*char);
            character.last_seen = Some(now);
        }
    }
    pub(super) fn push_message(&mut self, message: &mut Message) {
        if message.say_type > fo_defines::Say::WhisperOnHead {
            return;
        }
        let character = self.character(message.cr_id);
        if let Some(name) = message.name.take() {
            character.name = Some(name.into());
        }
        character.last_heard = Some(Instant::now());
    }
    pub(crate) fn characters_iter(&self) -> impl Iterator<Item = (&CharId, &Character)> {
        self.characters.iter()
    }
}
