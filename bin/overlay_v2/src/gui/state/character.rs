use super::{
    color::{random_color, Color},
    Char, CharId, Instant,
};
use crate::imgui::{im_str, ImStr, ImString};

#[derive(Debug)]
pub(crate) struct Character {
    pub(super) avatar: Option<Char>,
    pub(super) name: Option<ImString>,
    pub(super) last_seen: Option<Instant>,
    pub(super) last_heard: Option<Instant>,
    pub(super) color: Color,
}
impl Character {
    pub(super) fn new(char_id: CharId) -> Self {
        let color = random_color(char_id);
        Self {
            avatar: None,
            name: None,
            last_seen: None,
            last_heard: None,
            color,
        }
    }
    pub(in crate::gui) fn name(&self) -> &ImStr {
        self.name.as_deref().unwrap_or(im_str!("???"))
    }
    pub(in crate::gui) fn color(&self) -> &Color {
        &self.color
    }
    pub(in crate::gui) fn avatar(&self) -> Option<Char> {
        self.avatar
    }
    pub(in crate::gui) fn last_seen(&self) -> Option<Instant> {
        self.last_seen
    }
    pub(in crate::gui) fn last_heard(&self) -> Option<Instant> {
        self.last_heard
    }
}
