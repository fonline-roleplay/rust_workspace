use crate::backend::{
    Backend, BackendWindow,
    winit_gl::{
        WinitGlWindow, WinitGlBackend, WinitGlError,
    },
};
use tnf_common::message::client_dll_overlay::Message;
use imgui::{im_str, FontGlyphRange, ImFontConfig, ImString};
use crate::avatar_window::AvatarWindow;

pub trait UiLogic {
    fn title(&self) -> Option<ImString>;
    fn size(&self) -> (u32, u32);
    fn draw(&mut self, ui: &imgui::Ui);
}

pub struct Chat {
    messages: Vec<Message>,
    //size: (u32, u32)
}
impl Chat {
    pub fn new() -> Self{
        Chat{messages: vec![],}
    }
    pub fn push_message(&mut self, message: Message) {
        self.messages.push(message);
    }
}
impl UiLogic for Chat {
    fn title(&self) -> Option<ImString> {
        Some(im_str!("FOnline Chat").into())
    }
    fn size(&self) -> (u32, u32) {
        (480, 640)
    }
    fn draw(&mut self, ui: &imgui::Ui) {
        for message in &self.messages { //.rchunks(10).take(1).flatten() {
            let name = im_str!("{}", message.name.as_ref().map(String::as_str).unwrap_or("???"));
            let text = im_str!("{}", message.text);
            ui.small_button(&name);
            ui.text(&text);
        }
    }
}


pub struct UiWindow<B: Backend, L: UiLogic> {
    inner: B::Window,
    back: B,
    logic: L,
    drag: Option<[f32; 2]>,
    hidden: bool,
}

impl<B: Backend, L: UiLogic> UiWindow<B, L> {
    pub fn new(logic: L) -> Result<Self, B::Error> {
        let back = B::new();
        let size = logic.size();
        let mut inner = back.new_window("FOnlineChat".into(), size.0, size.1)?;
        inner.init_gui(|imgui, info|{
            imgui_init_fonts(imgui, info.hidpi_factor);
            let style = imgui.style_mut();
            style.window_rounding = 0.0;
            Ok(())
        })?;
        Ok(UiWindow {
            inner, back, logic, drag: None, hidden: false,
        })
    }
    pub fn draw(&mut self, is_foreground: bool) -> Result<(), B::Error>{
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

        let size = logic.size();

        let mut move_window = None;
        let drag = &mut self.drag;

        let res = inner.draw_gui(|ui, context, textures| {
            ui.window(&title)
                .title_bar(bar)
                .size([size.0 as f32, size.1 as f32], Condition::Always)
                .position([0.0, 0.0], Condition::Always)
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .build(|| {
                    //is window title hovered
                    if bar && ui.is_item_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Left) {
                        *drag = Some(ui.io().mouse_pos);
                    } else if !ui.is_mouse_down(imgui::MouseButton::Left) {
                        *drag = None;
                    } else if let Some(drag) = drag.as_mut() {
                        let pos = ui.io().mouse_pos;
                        move_window = Some((pos[0] - drag[0], pos[1] - drag[1]));
                    }

                    logic.draw(ui);
            });
            true
        });
        if let Some((x, y)) = move_window {
            inner.move_by_f32(x, y);
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
    imgui.fonts().add_font_with_config(
        //include_bytes!("../resources/mplus-1p-regular.ttf"),
        include_bytes!("../resources/fallout_display.ttf"),
        ImFontConfig::new()
            //.merge_mode(true)
            .oversample_h(1)
            .pixel_snap_h(true)
            .size_pixels(font_size)
            .rasterizer_multiply(1.75),
        &FontGlyphRange::cyrillic(),
    );

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
}


impl<B: Backend, L: UiLogic> crate::windowing::OverlayWindow<B> for UiWindow<B, L> {
    fn backend_window(&self) -> &B::Window {
        &self.inner
    }
}