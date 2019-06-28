use crate::{
    backend::{Backend, BackendError, BackendWindow},
    bridge::{Avatar, Char},
    image_data::ImageData,
    Rect,
};

pub struct AvatarWindow<B: Backend> {
    inner: B::Window,
    char_texture: Option<(Char, <B::Window as BackendWindow>::Texture)>,
    last_frame: u64,
    hidden: bool,
    pos: Option<(i32, i32)>,
}

impl<B: Backend> AvatarWindow<B> {
    pub fn new(inner: B::Window) -> Self {
        AvatarWindow {
            inner,
            char_texture: None,
            last_frame: 0,
            hidden: true,
            pos: None,
        }
    }
    pub fn maintain(&mut self, frame: u64) -> bool {
        if self.last_frame + 240 * 30 < frame {
            false
        } else if self.last_frame != frame {
            self.hide();
            true
        } else {
            true
        }
    }
    pub fn update(
        &mut self,
        avatar: &Avatar,
        image: &mut ImageData,
        rect: &Rect,
        frame: u64,
    ) -> bool {
        if let Err(err) = self.update_char(avatar.char, image) {
            eprintln!("Update char window: {:?}", err);
            self.hide();
            return false;
        }
        let x = avatar.pos.x - 32;
        let y = avatar.pos.y - 64 - 16;
        let mut appeared = false;
        if (rect.width as i32 - 64 > x && x > 0) && (rect.height as i32 - 64 > y && y > 0) {
            self.set_position(rect.x + x, rect.y + y);
            if self.show() {
                appeared = true;
            }
            self.draw();
        } else {
            //if characters are visible, but out of screen - don't show them. but mark avatar window as used
            self.hide();
        }
        self.last_frame = frame;
        appeared
    }
    fn update_char(&mut self, char: Char, image: &mut ImageData) -> Result<(), BackendError<B>> {
        if let Some((old_char, _)) = &self.char_texture {
            if *old_char == char {
                return Ok(());
            } else {
                let (_, old_texture) = self.char_texture.take().unwrap();
                self.inner.drop_texture(old_texture);
            }
        }
        let texture = self.inner.create_texture(image)?;
        self.char_texture = Some((char, texture));
        Ok(())
    }
    fn draw(&mut self) -> Result<(), BackendError<B>> {
        if let Some((_, texture)) = &self.char_texture {
            let src = Rect::new(0, 0, 128, 128);
            let dst = Rect::new(0, 0, 64, 64);
            self.inner.draw_texture(texture, &src, &dst)?;
        }
        Ok(())
    }
    fn show(&mut self) -> bool {
        if self.hidden {
            self.hidden = false;
            self.inner.show();
            true
        } else {
            false
        }
    }
    fn hide(&mut self) {
        if !self.hidden {
            self.hidden = true;
            self.inner.hide();
        }
    }
    fn set_position(&mut self, x: i32, y: i32) {
        if self.pos != Some((x, y)) {
            self.pos = Some((x, y));
        } else {
            return;
        }
        self.inner.set_position(x, y);
    }
    pub fn backend_window(&self) -> &B::Window {
        &self.inner
    }
}
