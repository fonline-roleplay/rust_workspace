use crate::backend::{
    Backend, BackendWindow,
    winit_gl::{
        WinitGlWindow, WinitGlBackend, WinitGlError,
    },
};
use tnf_common::message::client_dll_overlay::Message;
use imgui::{FontGlyphRange, ImFontConfig};

pub struct ChatWindow {
    inner: WinitGlWindow,
    back: WinitGlBackend,
    messages: Vec<Message>,
    size: (u32, u32),
}

impl ChatWindow {
    pub fn new() -> Result<Self, WinitGlError> {
        let back = WinitGlBackend::new();
        let size = (480, 640);
        let mut inner = back.new_window("FOnlineChat".into(), size.0, size.1)?;
        inner.init_gui(|imgui, info|{
            imgui_init_fonts(imgui, info.hidpi_factor);
            let style = imgui.style_mut();
            style.window_rounding = 0.0;
            Ok(())
        })?;
        Ok(ChatWindow {
            inner, back, messages: vec![], size,
        })
    }
    pub fn draw(&mut self) -> Result<(), WinitGlError>{
        use imgui::{Condition, StyleVar, im_str};

        let size = self.size;
        let messages = &self.messages;
        self.inner.draw_gui(|ui, context, textures| {
            ui
                .window(im_str!("FOnline Chat - тест"))
                    .size([size.0 as f32, size.1 as f32], Condition::Always)
                    .position([0.0, 0.0], Condition::Always)
                    .build(|| {
                        for message in messages.rchunks(10).take(1).flatten() {
                            ui.text(im_str!("({}): {}", message.cr_id, message.text));
                        }
                });
            true
        })
    }
    pub fn push_message(&mut self, message: Message) {
        self.messages.push(message);
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
