use crate::{
    bridge::{Avatar, Char},
    image_data::ImageData,
    SdlError,
};
use sdl2::{
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    surface::Surface,
    video::{Window, WindowContext, WindowPos},
    VideoSubsystem,
};

pub struct AvatarWindow {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    char_texture: Option<(Char, Texture)>,
    last_frame: u64,
    hidden: bool,
    pos_x: WindowPos,
    pos_y: WindowPos,
}

impl AvatarWindow {
    pub fn new(video_subsystem: &VideoSubsystem) -> Result<Self, SdlError> {
        use sdl2::sys::SDL_WindowFlags;
        let window = video_subsystem
            .window("FOnlineOverlay", 64, 64)
            .set_window_flags(
                SDL_WindowFlags::SDL_WINDOW_BORDERLESS as u32
                    | SDL_WindowFlags::SDL_WINDOW_ALWAYS_ON_TOP as u32
                    | SDL_WindowFlags::SDL_WINDOW_SKIP_TASKBAR as u32,
            )
            .hidden()
            .build()
            .map_err(SdlError::WindowBuild)?;
        let canvas = window
            .into_canvas()
            .build()
            .map_err(SdlError::CanvasBuild)?;
        let texture_creator = canvas.texture_creator();
        Ok(AvatarWindow {
            canvas,
            texture_creator,
            char_texture: None,
            last_frame: 0,
            hidden: true,
            pos_x: WindowPos::Undefined,
            pos_y: WindowPos::Undefined,
        })
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
        if (rect.width() as i32 - 64 > x && x > 0) && (rect.height() as i32 - 64 > y && y > 0) {
            self.set_position(rect.x() + x, rect.y() + y);
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
    fn update_char(&mut self, char: Char, image: &mut ImageData) -> Result<(), SdlError> {
        if let Some((old_char, _)) = &self.char_texture {
            if *old_char == char {
                return Ok(());
            } else {
                let (_, old_texture) = self.char_texture.take().unwrap();
                unsafe {
                    old_texture.destroy();
                }
            }
        }
        let surface = image.surface()?;
        let texture = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .map_err(SdlError::TextureFromSurface)?;
        self.char_texture = Some((char, texture));
        Ok(())
    }
    fn draw(&mut self) -> Result<(), SdlError> {
        if let Some((_, texture)) = &self.char_texture {
            let src = Rect::new(0, 0, 128, 128);
            let dst = Rect::new(0, 0, 64, 64);
            self.canvas
                .copy(texture, src, dst)
                .map_err(SdlError::TextureCopy)?;
            self.canvas.present();
        }
        Ok(())
    }
    fn show(&mut self) -> bool {
        if self.hidden {
            self.hidden = false;
            self.window_mut().show();
            true
        } else {
            false
        }
    }
    fn hide(&mut self) {
        if !self.hidden {
            self.hidden = true;
            self.window_mut().hide();
        }
    }
    fn set_position(&mut self, x: i32, y: i32) {
        let x = WindowPos::Positioned(x);
        let y = WindowPos::Positioned(y);
        if self.pos_x != x {
            self.pos_x = x;
        } else if self.pos_y != y {
            self.pos_y = y;
        } else {
            return;
        }
        let window = self.window_mut();
        window.set_position(x, y);
    }
    pub fn window(&self) -> &Window {
        self.canvas.window()
    }
    fn window_mut(&mut self) -> &mut Window {
        self.canvas.window_mut()
    }
}
