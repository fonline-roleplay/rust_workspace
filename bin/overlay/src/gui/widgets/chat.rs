use crate::imgui::{self, im_str, ChildWindow, ImStr, ImString, StyleColor, Ui};
use std::borrow::Cow;

use super::{
    super::{
        state::{self, color::{rgb_to_rgb_arr, Color}}, CharId, Message, AVATARS_SIZES, DEFAULT_AVATAR_SIZE_INDEX,
        helpers::rich_text,
    },
    GuiState, TextureRequester, UiLogic,
};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum MessageSender {
    Char(CharId),
    Radio,
}
impl MessageSender {
    const RADIO: Color = Color {
        normal: [0.3, 0.7, 0.5, 1.0],
        lighter: [0.4, 0.8, 0.6, 1.0],
        darker: [0.2, 0.6, 0.4, 1.0],
    };
    fn info(self, state: &mut GuiState) -> MessageSenderInfo<'_> {
        match self {
            MessageSender::Char(char_id) => {
                let character = state.character(char_id);
                MessageSenderInfo {
                    color: character.color(),
                    name: character.name(),
                    avatar: Some(character.avatar())
                }
            }
            MessageSender::Radio => {
                MessageSenderInfo {
                    color: &Self::RADIO,
                    name: im_str!("Рация"),
                    avatar: None,
                }
            }
        }
    }
}
struct MessageSenderInfo<'a> {
    color: &'a Color,
    name: &'a ImStr,
    avatar: Option<Option<state::Char>>,
}

pub(crate) struct Chat {
    messages: Vec<(MessageSender, String, SayType)>,
    filter: ChatFilter,
    avatars_sizes_index: usize,
    stick_to_bottom: bool,
    //size: (u32, u32)
}

const AVATAR_SIZE: [f32; 2] = [32.0; 2];

struct ChatFilter {
    add: bool,
    ids: Vec<MessageSender>,
}
impl ChatFilter {
    fn has_sender(&self, sender: MessageSender) -> bool {
        self.ids.iter().any(|filter| sender == *filter)
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
    pub fn push_message(&mut self, mut message: Message) {
        //use fo_defines::Say;

        //remove_colors(&mut message.text);
        /*match message.say_type {
            Say::Normal => {
                if auto_emote(&mut message.text) {
                    message.say_type = Say::Emote
                }
            }
            Say::NormalOnHead => {
                if auto_emote(&mut message.text) {
                    message.say_type = Say::EmoteOnHead
                }
            }
            _ => {}
        }*/

        let say_type = SayType(message.say_type);
        let sender = match message.say_type {
            fo_defines::Say::Radio => MessageSender::Radio,
            _ => MessageSender::Char(message.cr_id)
        };
        
        self.messages
            //.push((sender, message.text, say_type));
            .push((sender, say_type.format_rich(&message.text), say_type));
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

fn auto_emote(text: &mut String) -> bool {
    let emoted = text.replace("**", "*");
    text.clear();
    let mut emotes = 0;
    for (i, chunk) in emoted.split("*").enumerate() {
        if chunk.len() == 0 {
            continue;
        }
        let odd = i % 2 == 1;
        if odd {
            text.push_str("**");
        }
        emotes += 1;
        text.push_str(chunk);
        if odd {
            text.push_str("**");
        }
    }
    emotes == 1 && text.starts_with("**") && text.ends_with("**")
}

fn remove_colors(text: &mut String) {
    let mut is_color = false;
    text.retain(|ch| {
        is_char_part_of_color(ch, &mut is_color)
    });
}

fn push_str_wihout_colors(buffer: &mut String, text: &str) {
    let mut is_color = false;
    for ch in text.chars() {
        if is_char_part_of_color(ch, &mut is_color) {
            buffer.push(ch);
        }
    };
}

fn is_char_part_of_color(ch: char, is_color: &mut bool) -> bool {
    if *is_color {
        if ch.is_whitespace() {
            *is_color = false;
        }
        false
    } else if ch == '|' {
        *is_color = true;
        false
    } else {
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_remove_colors() {
        let mut string = "(Джек Гримстоун): |4294506899 Ладно. |4294506899 Пойдем.".into();
        remove_colors(&mut string);
        assert_eq!(&string, "(Джек Гримстоун): Ладно. Пойдем.");
    }
}

#[derive(Clone, Copy, PartialEq)]
struct SayType(fo_defines::Say);

fn draw_rich_text(drawer: &mut rich_text::Drawer, style: SayType, text: &str) {
    let style = drawer.ui().push_style_color(StyleColor::Text, style.color());
    drawer.draw_line(&text);
    style.pop(drawer.ui());
}

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
    fn _text_wrapped<T: AsRef<ImStr>>(self, ui: &Ui, text: T) {
        let style = ui.push_style_color(StyleColor::Text, self.color());
        ui.text_wrapped(text.as_ref());
        style.pop(ui);
    }
    fn text<T: AsRef<ImStr>>(self, ui: &Ui, text: T) {
        let style = ui.push_style_color(StyleColor::Text, self.color());
        ui.text(text.as_ref());
        style.pop(ui);
    }
    fn format_rich(self, text: &str) -> String {
        use fo_defines::Say::*;
        match self.0 {
            Shout => format!("!!!{}| !!!", text.to_uppercase()),
            Emote => format!("**{}| **", text.trim_matches('*')),
            Whisper => format!("...{}| ...", text.to_lowercase()),
            Radio => format!("..{}| ..", text),
            _ => format!("{}| ", text),
        }
    }
    fn _format_to_imstring(self, text: &str) -> ImString {
        use fo_defines::Say::*;
        match self.0 {
            Shout => im_str!("!!!{}!!!", text.to_uppercase()),
            Emote => im_str!("**{}**", text.trim_matches('*')),
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
                str.push_str(text.trim_matches('*'));
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
        im_str!("FOnline Chat",)
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

        // Filters
        if !filter.ids.is_empty() {
            let mut delete = None;
            for (i, sender) in filter.ids.iter().enumerate() {
                let info = sender.info(state);
                if info.color.small_button(ui, &im_str!("{}##filter", info.name))
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
                let mut last_msg: Option<(MessageSender, SayType, u32)> = None;
                let mut drawer = rich_text::Drawer::new(ui, 4.0);
                for (i, (sender, text, say_type)) in messages.into_iter().enumerate() {
                    //let character = state.character(*char_id);
                    if !filter.add && !ui.io().key_alt && !filter.ids.is_empty() {
                        if !filter.has_sender(*sender) {
                            continue;
                        }
                    }
                    match &mut last_msg {
                        Some((last_id, last_say_type, times))
                            if (*sender == *last_id
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
                        let info = sender.info(state);

                        ui.spacing();
                        ui.columns(2, im_str!("columns"), false);
                        ui.set_current_column_width(40.0);
                        
                        // If sender can have avatar
                        if let Some(avatar) = info.avatar {
                            // If sender actually have avatar and it's loaded
                            if let Some(texture_id) = avatar.and_then(|avatar| texture_requester.texture_for_char(avatar)) {
                                let avatar = imgui::Image::new(texture_id, AVATAR_SIZE);
                                avatar.build(ui);
                            } else {
                                ui.button(im_str!("?"), AVATAR_SIZE);
                            }
                            ui.same_line(0.0);
                        }
                        ui.next_column();
                        let label = im_str!("{}##name_{}", info.name, i);
                        {
                            if info.color.small_button(ui, &label) {
                                if !filter.has_sender(*sender) {
                                    filter.ids.push(*sender);
                                }
                            }
                        }
                        {
                            if let Some(text) = say_type.action() {
                                ui.same_line(0.0);
                                SayType::unknown().text(ui, text);
                            }
                        }
                        last_msg = Some((*sender, *say_type, 1));
                    }
                    //say_type.text_wrapped(ui, &text);
                    draw_rich_text(&mut drawer, *say_type, &text);

                    if let Some(log) = &mut copy_text {
                        let info = sender.info(state);

                        log.push_str(info.name.to_str());
                        log.push_str(": ");
                        push_str_wihout_colors(log, text.as_str());
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
