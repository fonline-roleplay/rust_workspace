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
    messages: Vec<(Message, Color)>,
    filter: Option<(u32, String, Color)>,
    //size: (u32, u32)
}
impl Chat {
    pub fn new() -> Self {
        Chat {
            messages: vec![],
            filter: None,
        }
    }
    pub fn push_message(&mut self, message: Message) {
        let color = random_color(message.cr_id);
        self.messages.push((message, color));
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
        //ui.text(im_str!(""));
        ui.text(im_str!("Фильтр: "));
        ui.same_line(0.0);
        let filter = &mut self.filter;
        if let Some((_, name, color)) = filter.as_ref() {
            if color.small_button(ui, &im_str!("{}##filter", name)) {
                *filter = None;
            }
        } else {
            ui.text(im_str!("нет"));
        }

        let mut size = ui.get_content_region_avail();
        let messages = &self.messages;
        ChildWindow::new("Сообщения")
            .size(size.into())
            .border(true)
            .build(ui, || {
                for (i, (message, color)) in messages.into_iter().enumerate() {
                    //.rchunks(10).take(1).flatten() {
                    if let Some((cr_id, _, _)) = filter.as_ref() {
                        if message.cr_id != *cr_id {
                            continue;
                        }
                    }
                    ui.spacing();
                    let name = message.name.as_ref().map(String::as_str).unwrap_or("???");
                    let label = im_str!("{}##name_{}", name, i);
                    let text = im_str!("{}", message.text);
                    {
                        if color.small_button(ui, &label) {
                            *filter = Some((message.cr_id, name.to_string(), *color));
                        }
                    }
                    ui.text(&text);
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
    let font_size = (13.0 * hidpi_factor) as f32;
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
        data: include_bytes!("../resources/fallout_display.ttf"),
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
