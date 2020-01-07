use crate::{
    avatar_window::AvatarWindow,
    backend::{
        winit_gl::{WinitGlBackend, WinitGlError, WinitGlWindow},
        Backend, BackendRef, BackendWindow, WindowRef,
    },
    windowing::{Windowing, WindowingExt},
    Rect,
};
use imgui::{
    im_str, ChildWindow, FontConfig, FontGlyphRanges, FontSource, ImStr, ImString, StyleColor,
    StyleVar, Ui, Window,
};
use std::rc::Rc;
use tnf_common::message::client_dll_overlay::Message;

pub trait UiLogic {
    const INITIAL_SIZE: (u32, u32);
    const FIXED: bool;
    const TITLE_BAR: bool;
    fn title(&self) -> ImString;
    fn draw(&mut self, ui: &imgui::Ui, windowing: &mut impl WindowingExt);
    fn sticky_pos(&self) -> Option<(i32, i32)> {
        None
    }
}

pub struct Parent {
    show_avatars: bool,
    show_ui: bool,
}

pub struct Bar {
    pub client_size: (u32, u32),
    pub show_chat: bool,
    pub show_faces: bool,
    button: ToggleButton,
}
impl Bar {
    pub fn new(client_size: (u32, u32)) -> Self {
        Bar {
            client_size,
            show_chat: true,
            show_faces: true,
            button: ToggleButton::new(),
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

impl UiLogic for Bar {
    const INITIAL_SIZE: (u32, u32) = (90, 35);
    const FIXED: bool = true;
    const TITLE_BAR: bool = false;
    fn title(&self) -> ImString {
        im_str!("FOnline Bar").into()
    }
    fn draw(&mut self, ui: &imgui::Ui, windowing: &mut impl WindowingExt) {
        //WIP: Remove
        ui.same_line(4.0);
        let size = [0.0, 24.0];
        if self
            .button
            .toggle(ui, im_str!("Чат"), size, &mut self.show_chat)
        {}
        ui.same_line(0.0);
        if self
            .button
            .toggle(ui, im_str!("Лица"), size, &mut self.show_faces)
        {}
    }
    fn sticky_pos(&self) -> Option<(i32, i32)> {
        Some((self.client_size.0 as i32 - Self::INITIAL_SIZE.0 as i32, 0))
    }
}

pub struct Chat {
    messages: Vec<(Message, SayType, Color)>,
    filter: ChatFilter,
    avatars_sizes_index: usize,
    //size: (u32, u32)
}

struct ChatFilter {
    add: bool,
    names: Vec<(u32, String, Color)>,
}
impl ChatFilter {
    fn has_cr_id(&self, cr_id: u32) -> bool {
        self.names.iter().any(|entry| cr_id == entry.0)
    }
}

const AVATARS_SIZES: [u16; 8] = [16, 32, 48, 64, 80, 96, 112, 128];

impl Chat {
    pub fn new() -> Self {
        Chat {
            messages: vec![],
            filter: ChatFilter {
                add: false,
                names: vec![],
            },
            avatars_sizes_index: 3,
        }
    }
    pub fn push_message(&mut self, message: Message) {
        let color = random_color(message.cr_id);
        let say_type = message.say_type.into();
        self.messages.push((message, say_type, color));
    }
}

fn stick_bottom(ui: &Ui) {
    let scroll = ui.scroll_y();
    let scroll_max = ui.scroll_max_y();
    if scroll == scroll_max && ui.io().mouse_wheel <= 0.0 {
        ui.set_scroll_here_y_with_ratio(1.0);
    }
}

fn pseudo_random(val: u32) -> u32 {
    val.wrapping_add(987456)
        .wrapping_mul(3464972813)
        .wrapping_add(654731)
}

#[derive(Clone, Copy)]
struct Color {
    normal: [f32; 4],
    darker: [f32; 4],
    lighter: [f32; 4],
}

#[derive(Clone, Copy, PartialEq)]
enum SayType {
    Normal,
    Shout,
    Emote,
    Whisper,
    Radio,
    Unknown,
}
impl From<i32> for SayType {
    fn from(from: i32) -> Self {
        use tnf_common::defines::fos;
        use SayType::*;
        match from as u32 {
            fos::SAY_NORM => Normal,
            fos::SAY_SHOUT => Shout,
            fos::SAY_EMOTE => Emote,
            fos::SAY_WHISP => Whisper,
            fos::SAY_RADIO => Radio,
            _ => Unknown,
        }
    }
}
impl SayType {
    fn color(self) -> [f32; 4] {
        use SayType::*;
        match self {
            Normal => rgb_to_rgb_arr(0xF8F993),
            Shout => rgb_to_rgb_arr(0xFF0000),
            Emote => rgb_to_rgb_arr(0xFF00FF),
            Whisper => rgb_to_rgb_arr(0x00FFFF),
            Radio => rgb_to_rgb_arr(0xFFFFFE),
            Unknown => rgb_to_rgb_arr(0x555555),
        }
    }
    fn action(self) -> Option<&'static ImStr> {
        use SayType::*;
        match self {
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
        use SayType::*;
        match self {
            Shout => im_str!("!!!{}!!!", text.to_uppercase()),
            Emote => im_str!("**{}**", text),
            Whisper => im_str!("...{}...", text.to_lowercase()),
            Radio => im_str!("..{}..", text),
            _ => im_str!("{}", text),
        }
    }
    fn push_str(self, str: &mut String, text: &str) {
        use SayType::*;
        match self {
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

impl Color {
    fn small_button(&self, ui: &Ui, label: &ImStr) -> bool {
        let token = ui.push_style_colors(
            [
                (StyleColor::Button, self.normal),
                (StyleColor::ButtonHovered, self.lighter),
                (StyleColor::ButtonActive, self.darker),
            ]
            .iter(),
        );
        let res = ui.small_button(label);
        token.pop(ui);
        res
    }
    fn button(&self, ui: &Ui, label: &ImStr, size: [f32; 2]) -> bool {
        let token = ui.push_style_colors(
            [
                (StyleColor::Button, self.normal),
                (StyleColor::ButtonHovered, self.lighter),
                (StyleColor::ButtonActive, self.darker),
            ]
            .iter(),
        );
        let res = ui.button(label, size);
        token.pop(ui);
        res
    }
}

fn rgb_to_rgb_arr(rgb: u32) -> [f32; 4] {
    let rgb = rgb.to_be_bytes();
    [
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
        rgb[3] as f32 / 255.0,
        1.0,
    ]
}

fn int_hsl_to_rgb_arr(h: u16, s: u8, l: u8) -> [f32; 4] {
    let hsl = colorsys::Hsl::new(h as f64, s as f64, l as f64, None);
    let rgb: colorsys::Rgb = hsl.into();
    [
        rgb.get_red() as f32 / 255.0,
        rgb.get_green() as f32 / 255.0,
        rgb.get_blue() as f32 / 255.0,
        1.0,
    ]
}

fn random_color(seed: u32) -> Color {
    let rand = pseudo_random(seed);
    let bytes = rand.to_le_bytes();
    let h = u16::from_le_bytes([bytes[0], bytes[1]]) % 360;
    let s = (bytes[2] % 56) + 42;
    let l = (bytes[2] % 25) + 25;
    let normal = int_hsl_to_rgb_arr(h, s, l);
    let darker = int_hsl_to_rgb_arr(h, s, l - 10);
    let lighter = int_hsl_to_rgb_arr(h, s, l + 10);
    Color {
        normal,
        darker,
        lighter,
    }
}

impl UiLogic for Chat {
    const INITIAL_SIZE: (u32, u32) = (480, 640);
    const FIXED: bool = false;
    const TITLE_BAR: bool = true;
    fn title(&self) -> ImString {
        im_str!("FOnline Chat").into()
    }
    fn draw(&mut self, ui: &imgui::Ui, windowing: &mut impl WindowingExt) {
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

        ui.popup_modal(settings).always_auto_resize(true).build(|| {
            if ui.small_button(&im_str!("Очистить")) {
                messages.clear();
                ui.close_current_popup();
            }
            if ui.small_button(&im_str!(
                "Размер аватарок: {}",
                AVATARS_SIZES[*avatars_sizes_index]
            )) {
                *avatars_sizes_index = (*avatars_sizes_index + 1) % AVATARS_SIZES.len();
                windowing.set_avatars_size(AVATARS_SIZES[*avatars_sizes_index]);
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

        if !filter.names.is_empty() {
            let mut delete = None;
            for (i, entry) in filter.names.iter().enumerate() {
                let (_, name, color) = entry;
                if color.small_button(ui, &im_str!("{}##filter", name)) {
                    //*filter = None;
                    delete = Some(i);
                }
                ui.same_line(0.0);
            }
            if let Some(delete) = delete {
                filter.names.remove(delete);
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
                // last message: Critter id, message type, messages under same header
                let mut last_msg: Option<(u32, SayType, u32)> = None;
                for (i, (message, say_type, color)) in messages.into_iter().enumerate() {
                    //.rchunks(10).take(1).flatten() {
                    if !filter.add && !ui.io().key_alt && !filter.names.is_empty() {
                        if !filter.has_cr_id(message.cr_id) {
                            continue;
                        }
                    }
                    match &mut last_msg {
                        Some((cr_id, last_say_type, times))
                            if (message.cr_id == *cr_id
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
                    let name = message.name.as_ref().map(String::as_str).unwrap_or("???");
                    if last_msg.is_none() {
                        ui.spacing();
                        ui.columns(2, im_str!("columns"), false);
                        ui.set_current_column_width(40.0);
                        {
                            if let Some(texture_id) = windowing.texture_for_char(message.cr_id) {
                                let avatar = imgui::Image::new(texture_id, [32.0; 2]);
                                avatar.build(ui);
                            } else {
                                ui.button(im_str!("?"), [32.0; 2]);
                            }
                            ui.same_line(0.0);
                        }
                        ui.next_column();
                        let label = im_str!("{}##name_{}", name, i);
                        {
                            if color.small_button(ui, &label) {
                                if !filter.has_cr_id(message.cr_id) {
                                    filter.names.push((message.cr_id, name.to_string(), *color));
                                }
                            }
                        }
                        {
                            if let Some(text) = say_type.action() {
                                ui.same_line(0.0);
                                SayType::Unknown.text_wrapped(ui, text);
                            }
                        }
                        last_msg = Some((message.cr_id, *say_type, 1));
                    }

                    say_type.text_wrapped(ui, say_type.format(&message.text));
                    if let Some(log) = &mut copy_text {
                        log.push_str(name);
                        log.push_str(": ");
                        say_type.push_str(log, &message.text);
                        log.push('\n');
                    }
                }
                stick_bottom(ui);
            });
        /*if debug {
            texture_for_char.debug();
        }*/
        if let Some(log) = copy_text {
            use clipboard::{ClipboardContext, ClipboardProvider};
            let mut ctx: ClipboardContext = ClipboardProvider::new().expect("clipboard provider");
            let _res = ctx.set_contents(log);
        }
    }
}

pub struct UiWindow<B: Backend, L: UiLogic> {
    inner: WindowRef<B>,
    back: BackendRef<B>,
    logic: L,
    drag: Option<[f32; 2]>,
    resize: Option<[f32; 2]>,
    hidden: bool,
    last_size: (u32, u32),
}

impl<B: Backend, L: UiLogic> UiWindow<B, L> {
    pub fn new(logic: L, back: BackendRef<B>) -> Result<Self, B::Error> {
        let size = L::INITIAL_SIZE;
        let inner = {
            let mut back_ref = back.borrow_mut();
            let mut inner = back_ref.new_popup("FOnlineChat".into(), size.0, size.1)?;
            {
                let mut window = inner.borrow_mut();
                window.init_gui(&mut *back_ref, |imgui, info| {
                    //imgui_init_fonts(imgui, info.hidpi_factor);
                    let style = imgui.style_mut();
                    /*
                    dbg!(&style.window_border_size);
                    dbg!(&style.display_window_padding);
                    dbg!(&style.display_safe_area_padding);
                    dbg!(&style.window_padding);
                    */

                    style.window_rounding = 0.0;
                    //style.window_border_size = 0.0;
                    //style.window_padding[0] = 0.0;
                    //style.display_window_padding = [0.0, 0.0];
                    //style.display_safe_area_padding = [0.0, 0.0];
                    imgui
                        .io_mut()
                        .config_flags
                        .set(imgui::ConfigFlags::NO_MOUSE_CURSOR_CHANGE, true);
                    Ok(())
                })?;
                window.show();
            }
            inner
        };
        Ok(UiWindow {
            inner,
            back,
            logic,
            drag: None,
            hidden: false,
            last_size: size,
            resize: None,
        })
    }
    pub fn draw(
        &mut self,
        is_foreground: bool,
        windowing: &mut Windowing<B>,
        rect: &Rect,
    ) -> Result<(), B::Error> {
        use imgui::{Condition, StyleVar};

        let mut inner = self.inner.borrow_mut();

        if !is_foreground {
            if !self.hidden {
                inner.hide();
                self.hidden = true;
                self.drag = None;
            }
            return Ok(());
        } else {
            if self.hidden {
                inner.show();
                self.hidden = false;
            }
            //inner.to_foreground();
        }

        let logic = &mut self.logic;

        let title = logic.title();
        let bar = L::TITLE_BAR;

        let size = &mut self.last_size;

        let mut move_window = None;
        let mut resize_window = None;
        let drag = &mut self.drag;
        let resize = &mut self.resize;

        let fixed = L::FIXED;

        let res = inner.draw_gui(|ui, context, textures| {
            /*for tex_id in windowing.char_textures.values() {
                textures
            }*/
            windowing.char_textures.clear();

            std::mem::swap(textures, &mut windowing.textures);
            //windowing.textures = imgui::Textures::new();
            //windowing.char_textures.clear();
            let style = ui.push_style_var(StyleVar::WindowPadding([0.0, 5.0]));
            Window::new(&title)
                .title_bar(bar)
                //.size([size.0 as f32, size.1 as f32], Condition::Once)
                .size([size.0 as f32, size.1 as f32], Condition::Always)
                .position([0.0, 0.0], Condition::Always)
                //.resizable(!fixed)
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .scroll_bar(false)
                .build(ui, || {
                    style.pop(ui);
                    if !fixed {
                        //is window title hovered
                        if bar
                            && ui.is_item_hovered()
                            && ui.is_mouse_clicked(imgui::MouseButton::Left)
                        {
                            *drag = Some(ui.io().mouse_pos);
                        } else if !ui.is_mouse_down(imgui::MouseButton::Left) {
                            *drag = None;
                        } else if let Some(drag) = drag.as_mut() {
                            let pos = ui.io().mouse_pos;
                            move_window = Some((pos[0] - drag[0], pos[1] - drag[1]));
                        } /*else {
                              let new_size = ui.window_size();
                              let width = (new_size[0] as u32 / 32 + 1).max(1) * 32;
                              let height = (new_size[1] as u32 / 32 + 1).max(2) * 32;
                              resize_window = Some((width, height));
                          }*/
                    }
                    logic.draw(ui, windowing);
                    if !fixed && drag.is_none() {
                        let [width, height] = ui.window_size();
                        //ui.new_line();
                        //ui.same_line(width - 20.0);
                        ui.set_cursor_pos([width - 10.0, height - 10.0]);
                        ui.button(&im_str!("##resize"), [10.0, 10.0]);
                        if ui.is_item_hovered() && ui.is_item_clicked(imgui::MouseButton::Left) {
                            *resize = Some(ui.io().mouse_pos);
                        } else if !ui.is_mouse_down(imgui::MouseButton::Left) {
                            *resize = None
                        } else if let Some(resize) = resize.as_mut() {
                            let new_size = ui.io().mouse_pos;
                            let width = (new_size[0] as i32 / 32 + 1).max(1).min(64) as u32 * 32;
                            let height = (new_size[1] as i32 / 32 + 1).max(2).min(64) as u32 * 32;
                            resize_window = Some((width, height));
                        }
                    }
                });
            //println!("windows: {:?}", windowing.windows.len());
            //println!("tex: {:?}", windowing.textures);
            //println!("char_tex: {:?}", windowing.char_textures);
            std::mem::swap(textures, &mut windowing.textures);
            //windowing.textures = imgui::Textures::new();
            //windowing.textures = imgui::Textures::new();
            //windowing.char_textures.clear();
            windowing.char_textures.values().cloned().collect()
        });

        if let Some((x, y)) = move_window {
            inner.move_by_f32(x, y);
        } else if let Some(new_size) = resize_window {
            if new_size.0 != size.0 || new_size.1 != size.1 {
                *size = new_size;
                inner.set_size(size.0, size.1);
            }
        } else if let Some((x, y)) = self.logic.sticky_pos() {
            inner.set_position(rect.x + x, rect.y + y);
        }
        res
    }
    pub fn logic(&mut self) -> &mut L {
        &mut self.logic
    }
}
/*
fn imgui_init_fonts(imgui: &mut imgui::Context, hidpi_factor: f64) {
    dbg!(hidpi_factor);
    let font_size = (16.0 * hidpi_factor) as f32;
    /*
        imgui.fonts().add_default_font_with_config(
            ImFontConfig::new()
                .oversample_h(1)
                .pixel_snap_h(true)
                .size_pixels(font_size),
        );
    */
    let config = FontConfig {
        oversample_h: 1,
        pixel_snap_h: true,
        //size_pixels: font_size,
        //rasterizer_multiply: 1.75,
        glyph_ranges: FontGlyphRanges::cyrillic(),
        ..Default::default()
    };
    let font = FontSource::TtfData {
        config: Some(config),
        data: include_bytes!("../resources/clacon.ttf"),
        //data: include_bytes!("../resources/fallout_display.ttf"),
        size_pixels: font_size,
    };
    imgui.fonts().add_font(&[font]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
}
*/
impl<B: Backend, L: UiLogic> crate::windowing::OverlayWindow<B> for UiWindow<B, L> {
    fn backend_window(&self) -> &WindowRef<B> {
        &self.inner
    }
}
