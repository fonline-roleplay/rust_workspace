pub(super) mod rich_text;
use crate::imgui::{sys, Ui};
use super::state::color::rgb_to_rgb_arr;

pub(super) trait UiExt {
    fn calc_string_size(
        &self,
        string: &str,
        hide_text_after_double_hash: bool,
        wrap_width: f32,
    ) -> [f32; 2];
}

impl UiExt for Ui<'_> {
    fn calc_string_size(
        &self,
        string: &str,
        hide_text_after_double_hash: bool,
        wrap_width: f32,
    ) -> [f32; 2] {
        use std::os::raw::c_char;
    
        let mut out = sys::ImVec2::zero();
        let text = string.as_ptr() as *const c_char;
        unsafe {
            let text_end = text.add(string.len());
            sys::igCalcTextSize(
                &mut out,
                text,
                text_end,
                hide_text_after_double_hash,
                wrap_width,
            )
        };
        out.into()
    }
}