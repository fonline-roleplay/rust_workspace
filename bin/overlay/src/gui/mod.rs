mod state;
mod widgets;
mod helpers;

use super::imgui::{self, Condition, StyleVar, Window, ImString};
use crate::{bridge::{Avatar}, requester::TextureRequester};
use protocol::message::client_dll_overlay::Message;
use state::GuiState;
use widgets::{Widgets, UiLogic};
use std::collections::hash_map::{HashMap, Entry};

const AVATARS_SIZES: [u16; 7] = [32, 48, 64, 80, 96, 112, 128];
const DEFAULT_AVATAR_SIZE_INDEX: usize = 2; // 64

type CharId = u32;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[allow(dead_code)]
pub(super) enum Layer {
    BottomMost,
    Bottom,
    Middle,
    Top,
    TopMost,
}
type Layers = HashMap<ImString, Layer>;

struct GuiBundle<'a, 'b, 'c> {
    ui: &'a imgui::Ui<'c>,
    texture_requester: &'a mut TextureRequester<'b>,
    state: &'a mut GuiState,
    layers: &'a mut Layers,
}
impl GuiBundle<'_, '_, '_> {
    fn render<L: UiLogic>(
        &mut self,
        logic: &mut L,
    ) {
        let GuiBundle{ui, state, texture_requester, ..} = self;

        if !logic.visible(state) {
            return;
        }

        let title = logic.title();
        let window = Window::new(&title).title_bar(L::TITLE_BAR);
        let (window, size) = match logic.fixed_size(state) {
            Some(fixed) => {
                let size = [fixed.0 as f32, fixed.1 as f32];
                (
                    window
                        .size(size, Condition::Always)
                        .resizable(false)
                        .collapsible(false),
                    size,
                )
            }
            None => {
                let size = [L::INITIAL_SIZE.0 as f32, L::INITIAL_SIZE.1 as f32];
                (
                    window
                        .size(size, Condition::FirstUseEver)
                        .resizable(true)
                        .collapsible(true),
                    size,
                )
            }
        };
        let window = match (&state.game_rect, logic.fixed_position(state)) {
            (Some(rect), Some(fixed)) => {
                let pos = fixed.apply(rect, size);
                window.position(pos, Condition::Always).movable(false)
            }
            (_, None) => window.movable(true),
            _ => return,
        };
        let style = logic
            .padding(state)
            .map(|(x, y)| ui.push_style_var(StyleVar::WindowPadding([x as f32, y as f32])));
        window.scroll_bar(false).build(ui, || {
            logic.draw(ui, state, texture_requester);
        });

        if let Some(style) = style {
            style.pop(ui);
        }

        match self.layers.entry(title) {
            Entry::Occupied(occupied) => {
                panic!("Two windows with same title: {:?}", occupied.key());
            }
            Entry::Vacant(vacant) => {
                vacant.insert(logic.layer());
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct Hider {
    pub(crate) client_asks_to_hide: bool,
    pub(crate) client_is_not_visible: bool,
}
impl Hider {
    fn should_hide_gui(&self) -> bool {
        self.client_asks_to_hide || self.client_is_not_visible
    }
}

pub struct Gui {
    widgets: Widgets,
    layers: Layers,
    pub(crate) state: GuiState,
    pub(crate) hide: Hider,
    pub(crate) dirty: i8,
    //message_generator: MessageGenerator,
}
impl Gui {
    pub fn new() -> Self {
        Self {
            widgets: Widgets::new(),
            layers: Layers::new(),
            state: GuiState::new(),
            hide: Default::default(),
            dirty: 3,
            //message_generator: MessageGenerator::new(1),
        }
    }
    pub fn frame(&mut self, ui: &imgui::Ui, texture_requester: &mut TextureRequester) {
        if self.hide.should_hide_gui() {
            self.dirty = 0;
            return;
        }
        self.layers.clear();

        /*if let Some(msg) = self.message_generator.message() {
            self.chat.push_message(msg);
        }*/

        let bundle = GuiBundle{
            ui, texture_requester, state: &mut self.state, layers: &mut self.layers
        };

        self.widgets.frame(bundle);

        let active = false; //ui.is_mouse_dragging(imgui::MouseButton::Left);
        self.dirty = (self.dirty - 1).max(if active { 1 } else { 0 });
    }
    // return true if avatars changed
    pub(crate) fn update_avatars(&mut self, avatars: Vec<Avatar>) -> bool {
        if self.widgets.avatars != avatars {
            self.state.update_avatars(&avatars);
            self.widgets.avatars = avatars;
            true
        } else {
            false
        }
    }
    pub(crate) fn push_message(&mut self, mut message: Message) {
        self.state.push_message(&mut message);
        self.widgets.chat.push_message(message);
    }
    pub(super) fn layer_by_title(&self, title: &imgui::ImStr) -> Layer {
        self.layers.get(title).copied().unwrap_or(Layer::TopMost)
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    use super::*;


    /*#[test]
    fn test_imstr() {
        use imgui::im_str;
        use std::ops::Deref;
        use std::hash::Hash;
        use std::collections::hash_map::DefaultHasher;
        //use std::hash::

        let im_str1 = im_str!("FOnline Chat\0");
        let im_string2 = im_str!("FOnline Chat{}", "");
        let im_str2 = im_string2.deref();

        let mut state = DefaultHasher::new();

        assert_eq!(im_str1, im_str2);
        assert_eq!(im_str1.hash(&mut state), im_str2.hash(&mut state));
    }*/
    /*
    use std::time::Instant;
    struct MessageGenerator {
        last_message_time: Instant,
        texti: usize,
        chari: usize,
        //sayi: usize,
        every_secs: u64,
    }

    use protocol::message::client_dll_overlay::Message;
    impl MessageGenerator {
        fn new(every_secs: u64) -> Self {
            Self {
                last_message_time: Instant::now(),
                texti: 0,
                chari: 0,
                every_secs,
            }
        }
        fn text(&mut self) -> String {
            let texts = ["foo", "bar", "baz", "foobar"];
            let ret = texts[self.texti % texts.len()].into();
            self.texti += 1;
            ret
        }
        fn char(&mut self) -> (CharId, Option<String>) {
            let chars = ["Anuri", "Frank", "VVish", "Sjaman"];
            let cr_id = self.chari % chars.len();
            self.chari += 1;
            (cr_id as CharId, Some(chars[cr_id].into()))
        }
        /*fn say_type(&mut self) -> String {
            let says = [];
            let ret = chars[self.chari % chars.len()].into();
            self.chari += 1;
            ret
        }*/
        fn message(&mut self) -> Option<Message> {
            if self.last_message_time.elapsed().as_secs() < self.every_secs {
                return None;
            }
            self.last_message_time = Instant::now();

            let (cr_id, name) = self.char();
            Some(Message {
                text: self.text(),
                say_type: fo_defines::Say::Normal,
                cr_id,
                delay: 0,
                name,
            })
        }
    }
    */
}
