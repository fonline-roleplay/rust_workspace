use crate::imgui::{self, im_str, ChildWindow, ImStr, ImString, StyleColor, Ui};
use std::borrow::Cow;

use super::{
    super::{
        state::color::rgb_to_rgb_arr, CharId, Message, AVATARS_SIZES, DEFAULT_AVATAR_SIZE_INDEX,
    },
    GuiState, TextureRequester, UiLogic,
};

pub(crate) struct Chat {
    messages: Vec<(CharId, ImString, SayType)>,
    filter: ChatFilter,
    avatars_sizes_index: usize,
    stick_to_bottom: bool,
    //size: (u32, u32)
}

const AVATAR_SIZE: [f32; 2] = [32.0; 2];

struct ChatFilter {
    add: bool,
    ids: Vec<CharId>,
}
impl ChatFilter {
    fn has_char_id(&self, char_id: CharId) -> bool {
        self.ids.iter().any(|filter| char_id == *filter)
    }
}

impl Chat {
    pub(crate) fn new() -> Self {
        Chat {
            messages: vec![],
            filter: ChatFilter {
                add: false,
                ids: vec![],
            },
            avatars_sizes_index: DEFAULT_AVATAR_SIZE_INDEX,
            stick_to_bottom: true,
        }
    }
    pub fn push_message(&mut self, message: Message) {
        let say_type = SayType(message.say_type);
        self.messages
            .push((message.cr_id, say_type.format(&message.text), say_type));
    }
    fn stick_bottom(&mut self, ui: &Ui) {
        let scroll = ui.scroll_y();
        let scroll_max = ui.scroll_max_y();
        if scroll == scroll_max {
            //} && ui.io().mouse_wheel <= 0.0 {
            self.stick_to_bottom = true;
        }
        if ui.io().mouse_wheel > 0.0 {
            self.stick_to_bottom = false;
        }
        if self.stick_to_bottom {
            ui.set_scroll_here_y_with_ratio(1.0);
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct SayType(fo_defines::Say);

impl SayType {
    fn unknown() -> Self {
        SayType(fo_defines::Say::Unknown)
    }
    fn color(self) -> [f32; 4] {
        use fo_defines::Say::*;
        match self.0 {
            Normal => rgb_to_rgb_arr(0xF8F993),
            Shout => rgb_to_rgb_arr(0xFF0000),
            Emote => rgb_to_rgb_arr(0xFF00FF),
            Whisper => rgb_to_rgb_arr(0x00FFFF),
            Radio => rgb_to_rgb_arr(0xFFFFFE),
            _ => rgb_to_rgb_arr(0x555555),
        }
    }
    fn action(self) -> Option<&'static ImStr> {
        use fo_defines::Say::*;
        match self.0 {
            Normal => Some(im_str!("(говорит)")),
            Shout => Some(im_str!("(кричит)")),
            Whisper => Some(im_str!("(шепчет)")),
            _ => None,
        }
    }
    fn text_wrapped<T: AsRef<ImStr>>(self, ui: &Ui, text: T) {
        let style = ui.push_style_color(StyleColor::Text, self.color());
        ui.text_wrapped(text.as_ref());
        style.pop(ui);
    }
    fn format(self, text: &str) -> ImString {
        use fo_defines::Say::*;
        match self.0 {
            Shout => im_str!("!!!{}!!!", text.to_uppercase()),
            Emote => im_str!("**{}**", text),
            Whisper => im_str!("...{}...", text.to_lowercase()),
            Radio => im_str!("..{}..", text),
            _ => im_str!("{}", text),
        }
    }
    fn _format_imstr(self, text: &ImStr) -> Cow<ImStr> {
        use fo_defines::Say::*;
        match self.0 {
            Shout => im_str!("!!!{}!!!", text.to_str().to_uppercase()),
            Emote => im_str!("**{}**", text),
            Whisper => im_str!("...{}...", text.to_str().to_lowercase()),
            Radio => im_str!("..{}..", text),
            _ => return text.into(),
        }
        .into()
    }
    fn _push_str(self, str: &mut String, text: &str) {
        use fo_defines::Say::*;
        match self.0 {
            Shout => {
                str.push_str("!!!");
                str.push_str(&text.to_uppercase());
                str.push_str("!!!");
            }
            Emote => {
                str.push_str("**");
                str.push_str(text);
                str.push_str("**");
            }
            Whisper => {
                str.push_str("...");
                str.push_str(&text.to_lowercase());
                str.push_str("...");
            }
            Radio => {
                str.push_str("..");
                str.push_str(text);
                str.push_str("..");
            }
            _ => str.push_str(text),
        }
    }
}

impl UiLogic for Chat {
    const INITIAL_SIZE: (u32, u32) = (480, 640);
    const TITLE_BAR: bool = true;
    fn title(&self) -> ImString {
        im_str!("FOnline Chat").into()
    }
    fn draw(
        &mut self,
        ui: &imgui::Ui,
        state: &mut super::GuiState,
        texture_requester: &mut TextureRequester,
    ) {
        //WIP: Remove
        ui.same_line(8.0);

        let filter = &mut self.filter;
        let messages = &mut self.messages;

        let settings = im_str!("Настройки");
        if ui.small_button(settings) {
            ui.open_popup(settings);
        }
        //let debug = ui.small_button(im_str!("Дебаг"));

        let avatars_sizes_index = &mut self.avatars_sizes_index;

        ui.popup_modal(settings)
            .save_settings(false)
            .always_auto_resize(true)
            .build(|| {
                if ui.small_button(&im_str!("Очистить")) {
                    messages.clear();
                    ui.close_current_popup();
                }
                if ui.small_button(&im_str!(
                    "Размер аватарок: {}",
                    AVATARS_SIZES[*avatars_sizes_index]
                )) {
                    *avatars_sizes_index = (*avatars_sizes_index + 1) % AVATARS_SIZES.len();
                    state.set_avatars_size(AVATARS_SIZES[*avatars_sizes_index]);
                }
                if ui.small_button(im_str!("Закрыть")) {
                    ui.close_current_popup();
                }
            });

        ui.same_line(0.0);
        let mut copy_text = if ui.small_button(&im_str!("Копировать")) {
            Some(String::with_capacity(1024))
        } else {
            None
        };

        ui.same_line(0.0);
        ui.text(im_str!("Фильтр: "));
        ui.same_line(0.0);

        if !filter.ids.is_empty() {
            let mut delete = None;
            for (i, char_id) in filter.ids.iter().enumerate() {
                let character = state.character(*char_id);
                if character
                    .color()
                    .small_button(ui, &im_str!("{}##filter", character.name()))
                {
                    //*filter = None;
                    delete = Some(i);
                }
                ui.same_line(0.0);
            }
            if let Some(delete) = delete {
                filter.ids.remove(delete);
            }
            if ui.small_button(if filter.add {
                im_str!("ok")
            } else {
                im_str!("+")
            }) {
                filter.add = !filter.add;
            }
        } else {
            ui.text(im_str!("нет"));
        }

        let mut size = ui.content_region_avail();
        size[1] -= 5.0;
        ChildWindow::new("Сообщения")
            .size(size.into())
            .border(true)
            .build(ui, || {
                let filter = &mut self.filter;
                let messages = &mut self.messages;

                // last message: Critter id, message type, messages under same header
                let mut last_msg: Option<(CharId, SayType, u32)> = None;
                for (i, (char_id, text, say_type)) in messages.into_iter().enumerate() {
                    //let character = state.character(*char_id);
                    if !filter.add && !ui.io().key_alt && !filter.ids.is_empty() {
                        if !filter.has_char_id(*char_id) {
                            continue;
                        }
                    }
                    match &mut last_msg {
                        Some((last_id, last_say_type, times))
                            if (*char_id == *last_id
                                && *say_type == *last_say_type
                                && *times < 10) =>
                        {
                            *times += 1;
                        }
                        Some(..) => {
                            ui.columns(1, im_str!("columns"), false);
                            ui.separator();
                            last_msg = None;
                        }
                        None => {}
                    }
                    if last_msg.is_none() {
                        let character = state.character(*char_id);

                        ui.spacing();
                        ui.columns(2, im_str!("columns"), false);
                        ui.set_current_column_width(40.0);
                        {
                            if let Some(texture_id) = texture_requester.texture_for_cr_id(*char_id)
                            {
                                let avatar = imgui::Image::new(texture_id, AVATAR_SIZE);
                                avatar.build(ui);
                            } else {
                                ui.button(im_str!("?"), AVATAR_SIZE);
                            }
                            ui.same_line(0.0);
                        }
                        ui.next_column();
                        let label = im_str!("{}##name_{}", character.name(), i);
                        {
                            if character.color().small_button(ui, &label) {
                                if !filter.has_char_id(*char_id) {
                                    filter.ids.push(*char_id);
                                }
                            }
                        }
                        {
                            if let Some(text) = say_type.action() {
                                ui.same_line(0.0);
                                SayType::unknown().text_wrapped(ui, text);
                            }
                        }
                        last_msg = Some((*char_id, *say_type, 1));
                    }
                    say_type.text_wrapped(ui, &text);

                    if let Some(log) = &mut copy_text {
                        let character = state.character(*char_id);
                        log.push_str(character.name().to_str());
                        log.push_str(": ");
                        log.push_str(text.to_str());
                        //say_type.push_str(log, text.to_str());
                        log.push('\n');
                    }
                }
                self.stick_bottom(ui);
            });
        if let Some(log) = copy_text {
            use clipboard::{ClipboardContext, ClipboardProvider};
            let mut ctx: ClipboardContext = ClipboardProvider::new().expect("clipboard provider");
            let _res = ctx.set_contents(log);
        }
    }
    fn padding(&self, _state: &GuiState) -> Option<(u8, u8)> {
        Some((5, 5))
    }
}
