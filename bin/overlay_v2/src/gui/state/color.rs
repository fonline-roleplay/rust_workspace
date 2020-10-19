use crate::imgui::{ImStr, StyleColor, Ui};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Color {
    pub(crate) normal: [f32; 4],
    pub(crate) darker: [f32; 4],
    pub(crate) lighter: [f32; 4],
}

impl Color {
    pub(crate) fn small_button(&self, ui: &Ui, label: &ImStr) -> bool {
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
    pub(crate) fn button(&self, ui: &Ui, label: &ImStr, size: [f32; 2]) -> bool {
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

pub(crate) fn rgb_to_rgb_arr(rgb: u32) -> [f32; 4] {
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

pub(super) fn random_color(seed: u32) -> Color {
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

fn pseudo_random(val: u32) -> u32 {
    val.wrapping_add(987456)
        .wrapping_mul(3464972813)
        .wrapping_add(654731)
}
