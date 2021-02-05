use crate::imgui::{Ui, StyleColor, ColorStackToken, StyleVar};
use super::{UiExt, rgb_to_rgb_arr};


pub(crate) struct Drawer<'a> {
    ui: &'a Ui<'a>,
    buffer: String,
    color_token: Option<ColorStackToken>,
    word_count: u32,
    avail: f32,
    space: f32,
    spacing: [f32; 2],
}

impl<'a> Drawer<'a> {
    pub(crate) fn new(ui: &'a Ui<'a>, vertical_spacing: f32) -> Self {
        let color_token: Option<ColorStackToken> = None;
        let buffer = String::with_capacity(1024);
        let space = ui.calc_string_size( " ", true, -1.0)[0];
        let avail = 0.0;
        let spacing = [0.0, vertical_spacing];
        Self {
            ui, buffer, color_token, word_count: 0, avail, space, spacing
        }
    }
    fn update_avail(&mut self) {
        self.avail = self.ui.content_region_avail()[0];
    }
    fn flush(&mut self, new_line: bool) {
        if !self.buffer.is_empty() {
            self.ui.text(&self.buffer);
            self.buffer.clear();
    
            if !new_line {
                self.ui.same_line(0.0);
            }
        } else if new_line {
            self.ui.new_line();
        }
        self.update_avail();
    }
    fn pop(&mut self) {
        if let Some(token) = self.color_token.take() {
            token.pop(&self.ui);
        }
    }
    fn process(&mut self, mut text: &str) {
        while !text.is_empty() {
            Fragment::new(&mut text).draw(self)
        }
    }
    fn set_color(&mut self, color: u32) {
        self.pop();
        self.color_token = Some(self.ui.push_style_color(StyleColor::Text, rgb_to_rgb_arr(color)));
    }

    pub(crate) fn draw_line(&mut self, line: &str) {
        let spacing = self.ui.push_style_var(StyleVar::ItemSpacing(self.spacing));
        self.update_avail();
        let mut iter = line.char_indices();
        let mut start = None;
        let mut end = None;
        loop {
            if let Some(next) = iter.next() {
                let first = start.get_or_insert(next.0);
                let ch = next.1;
                if ch.is_whitespace() {
                    self.process(&line[*first..=next.0]);
                    start = None;
                    end = None;
                } else {
                    end = Some(next.0);
                }
                continue;
            }
            if let (Some(start), Some(end)) = (start, end) {
                self.process(&line[start..=end]);
            }
            break;
        }
        
            
        self.flush(true);
        self.pop();
        spacing.pop(&self.ui);
    }
    pub(crate) fn ui(&self) -> &Ui<'a> {
        &self.ui
    }
}

enum Fragment<'a> {
    Word(&'a str),
    Color(u32),
    ResetColor,
    Whitespace,
}

impl<'a> Fragment<'a> {
    fn new(text: &mut &'a str) -> Self {
        if text.is_empty() {
            return Fragment::Whitespace;
        }
        let has_color = text.rfind('|');
        let ret;
        match has_color {
            Some(del) if del==0 => {
                let (string, radix) = if text.starts_with("|0x") {
                    (text.get(3..), 16)
                } else {
                    (text.get(1..), 10)
                };
                ret = if let Some(color) = string.and_then(|string| u32::from_str_radix(string.trim_end(), radix).ok()) {
                    Fragment::Color(color)
                } else {
                    Fragment::ResetColor
                };
                *text = &mut "";
            }
            Some(del) => {
                ret = Fragment::Word(&text[..del]);
                *text = &mut &text[del..];
            }
            None => {
                ret = Fragment::Word(text);
                *text = &mut "";
            }
        }
        ret
    }
    fn draw(&self, drawer: &mut Drawer) {
        match self {
            Fragment::Color(color) => {
                drawer.flush(false);
                drawer.set_color(*color);
            }
            Fragment::ResetColor => {
                drawer.flush(false);
                drawer.pop();
            }
            Fragment::Word(word) => {
                let need = drawer.ui.calc_string_size( word, true, -1.0)[0];
                if drawer.word_count != 0 && drawer.avail < need {
                    drawer.flush(true);
                }
                drawer.avail -= need; // + drawer.space;
                /*if !drawer.buffer.is_empty() {
                    drawer.buffer.push(' ');
                }*/
                drawer.buffer.push_str(word);
                drawer.word_count += 1;
            }
            Fragment::Whitespace => {}
        }
    }
}
