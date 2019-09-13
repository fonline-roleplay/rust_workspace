use crate::avatar_window::AvatarWindow;
use crate::backend::{
    winit_gl::{WinitGlBackend, WinitGlError, WinitGlWindow},
    Backend, BackendWindow,
};
use imgui::{
    im_str, ChildWindow, FontConfig, FontGlyphRanges, FontSource, ImStr, ImString, StyleColor, Ui,
    Window,
};
use tnf_common::message::client_dll_overlay::Message;

pub trait UiLogic {
    const INITIAL_SIZE: (u32, u32);
    fn title(&self) -> Option<ImString>;
    fn draw(&mut self, ui: &imgui::Ui);
}

pub struct Parent {
    show_avatars: bool,
    show_ui: bool,
}

pub struct Chat {
    messages: Vec<(Message, SayType, Color)>,
    filter: ChatFilter,
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

impl Chat {
    pub fn new() -> Self {
        Chat {
            messages: vec![],
            filter: ChatFilter {
                add: false,
                names: vec![],
            },
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

#[derive(Clone, Copy)]
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
    fn title(&self) -> Option<ImString> {
        Some(im_str!("FOnline Chat").into())
    }
    fn draw(&mut self, ui: &imgui::Ui) {
        let filter = &mut self.filter;
        let messages = &mut self.messages;

        let settings = im_str!("Настройки");
        if ui.small_button(settings) {
            ui.open_popup(settings);
        }

        ui.popup_modal(settings).always_auto_resize(true).build(|| {
            if ui.small_button(&im_str!("Очистить")) {
                messages.clear();
                ui.close_current_popup();
            }
            if ui.small_button(im_str!("Закрыть")) {
                ui.close_current_popup();
            }
        });

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
        ChildWindow::new("Сообщения")
            .size(size.into())
            .border(true)
            .build(ui, || {
                for (i, (message, say_type, color)) in messages.into_iter().enumerate() {
                    //.rchunks(10).take(1).flatten() {
                    if !filter.add && !ui.io().key_alt && !filter.names.is_empty() {
                        if !filter.has_cr_id(message.cr_id) {
                            continue;
                        }
                    }
                    ui.spacing();
                    let name = message.name.as_ref().map(String::as_str).unwrap_or("???");
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
                    say_type.text_wrapped(ui, say_type.format(&message.text));
                    ui.separator();
                }
                stick_bottom(ui);
            });
    }
}

pub struct UiWindow<B: Backend, L: UiLogic> {
    inner: B::Window,
    back: B,
    logic: L,
    drag: Option<[f32; 2]>,
    hidden: bool,
    last_size: (u32, u32),
}

impl<B: Backend, L: UiLogic> UiWindow<B, L> {
    pub fn new(logic: L) -> Result<Self, B::Error> {
        let back = B::new();
        let size = L::INITIAL_SIZE;
        let mut inner = back.new_window("FOnlineChat".into(), size.0, size.1)?;
        inner.init_gui(|imgui, info| {
            imgui_init_fonts(imgui, info.hidpi_factor);
            let style = imgui.style_mut();
            style.window_rounding = 0.0;
            Ok(())
        })?;
        Ok(UiWindow {
            inner,
            back,
            logic,
            drag: None,
            hidden: false,
            last_size: size,
        })
    }
    pub fn draw(&mut self, is_foreground: bool) -> Result<(), B::Error> {
        use imgui::{Condition, StyleVar};

        let inner = &mut self.inner;

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
            inner.to_foreground();
        }

        self.back.poll_events(|event| {
            inner.handle_event(&event);
        });

        let logic = &mut self.logic;

        let title = logic.title();
        let bar = title.is_some();
        let title = title.unwrap_or_else(ImString::default);

        let size = &mut self.last_size;

        let mut move_window = None;
        let mut resize_window = None;
        let drag = &mut self.drag;

        let res = inner.draw_gui(|ui, context, textures| {
            Window::new(&title)
                .title_bar(bar)
                .size([size.0 as f32, size.1 as f32], Condition::Once)
                .position([0.0, 0.0], Condition::Always)
                .resizable(true)
                .movable(false)
                .collapsible(false)
                .build(ui, || {
                    //is window title hovered
                    if bar && ui.is_item_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Left)
                    {
                        *drag = Some(ui.io().mouse_pos);
                    } else if !ui.is_mouse_down(imgui::MouseButton::Left) {
                        *drag = None;
                    } else if let Some(drag) = drag.as_mut() {
                        let pos = ui.io().mouse_pos;
                        move_window = Some((pos[0] - drag[0], pos[1] - drag[1]));
                    } else {
                        let new_size = ui.get_window_size();
                        resize_window = Some((new_size[0] as u32, new_size[1] as u32));
                    }

                    logic.draw(ui);
                });
            true
        });
        if let Some((x, y)) = move_window {
            inner.move_by_f32(x, y);
        } else if let Some(new_size) = resize_window {
            if new_size.0 != size.0 || new_size.1 != size.1 {
                *size = new_size;
                inner.set_size(size.0, size.1);
            }
        }
        res
    }
    pub fn logic(&mut self) -> &mut L {
        &mut self.logic
    }
}

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

impl<B: Backend, L: UiLogic> crate::windowing::OverlayWindow<B> for UiWindow<B, L> {
    fn backend_window(&self) -> &B::Window {
        &self.inner
    }
}
