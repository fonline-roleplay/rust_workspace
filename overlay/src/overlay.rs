use crate::{
    backend::{Backend, BackendWindow},
    bridge::{Avatar, BridgeOverlayToClient, Char, MsgIn},
    downloader::{DownloaderError, ImageRequester},
    image_data::ImageData,
    windowing::Windowing,
    ui_window::{UiWindow, Chat},
    GameWindow, Rect,
};
use std::{collections::BTreeMap, time::Duration};

pub struct Overlay<B: Backend> {
    rect: Rect,
    game_window: GameWindow,
    avatars: Vec<Avatar>,
    hide: bool,
    dirty: bool,
    is_foreground: bool,
    images: BTreeMap<Char, AvatarImage>,
    frame: u64,
    bridge: BridgeOverlayToClient,
    requester: ImageRequester,
    requester_free: bool,
    windowing: Windowing<B>,
    //parent: Option<UiWindow<B>>,
    chat: Option<UiWindow<B, Chat>>,
}

impl<B: Backend> Overlay<B> {
    pub fn new(
        game_window: GameWindow,
        bridge: BridgeOverlayToClient,
        requester: ImageRequester,
    ) -> Self {
        eprintln!("err test");
        let chat = {
            let logic = Chat::new();
            match UiWindow::new(logic) {
                Ok(chat) => Some(chat),
                Err(err) => {
                    eprintln!("Can't create chat window: {:?}", err);
                    None
                }
            }
        };
        Overlay {
            rect: game_window.rect().expect("game window rect"),
            game_window,
            avatars: vec![],
            images: BTreeMap::new(),
            frame: 0,
            hide: false,
            dirty: true,
            is_foreground: true,
            bridge,
            requester,
            requester_free: true,
            windowing: Windowing::new(),
            chat,
        }
    }

    fn update(&mut self) {
        //let loop_frame = self.frame % 360;
        /*let angle = (loop_frame as f32).to_radians();
        //dbg!(angle);
        for (i, avatar) in self.avatars.iter_mut().enumerate() {
            let x = i as i32 % 10;
            let y = i as i32 / 10;

            *avatar = (x*100 + (angle.cos() * 50.0) as i32, y*100 + (angle.sin() * 50.0) as i32)
        }*/
        self.frame += 1;
    }
    fn update_game_window(&mut self) -> bool {
        if let Some(new_rect) = self.game_window.rect() {
            //.window_pos() {
            if new_rect != self.rect {
                self.rect = new_rect;
                if !self.hide {
                    self.dirty = true;
                }
            }
            let is_foreground = self.is_game_foreground();
            if is_foreground != self.is_foreground {
                self.is_foreground = is_foreground;
                self.dirty = true;
            }
            true
        } else {
            false
        }
    }
    pub fn run(mut self) {
        while self.game_loop() {
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 240));
            self.update();
        }
        let _ = self.requester.finish().join();
        self.bridge.finish(true);
    }
    fn game_loop(&mut self) -> bool {
        if !self.update_game_window() {
            println!("Window closed");
            return false;
        }

        let exit = self.windowing.poll_events();
        if exit {
            return false;
        }

        //check new messages from client, send ping
        self.update_bridge();

        //check if new image is downloaded, mark overlay dirty is so
        self.update_downloader();

        if self.dirty {
            self.redraw();
        }
        self.windowing.maintain(self.frame, self.dirty);
        self.dirty = false;

        if self.frame % 4 == 0 {
            if let Some(chat) = self.chat.as_mut() {
                if let Err(err) = chat.draw(self.is_foreground) {
                    eprintln!("Error drawing chat window: {:?}", err);
                    self.chat = None;
                }
            }
        }

        true
    }

    fn update_bridge(&mut self) {
        if self.bridge.is_online() {
            let _ = self.bridge.ping();

            let mut last_avatars = None;

            for msg in self.bridge.receive() {
                match msg {
                    MsgIn::UpdateAvatars(avatars) => {
                        last_avatars = Some(avatars);
                    }
                    MsgIn::OverlayHide(hide) => {
                        self.hide = hide;
                        self.dirty = true;
                    }
                    MsgIn::Message(mut msg) => {
                        use tnf_common::defines::fos;
                        match msg.say_type as u32 {
                            fos::SAY_NORM | fos::SAY_SHOUT | fos::SAY_EMOTE | fos::SAY_WHISP => {
                                //log message
                            }
                            _ => continue,
                        };
                        match msg.say_type as u32 {
                            fos::SAY_NORM | fos::SAY_NORM_ON_HEAD => {
                                auto_emote(&mut msg.text);
                            }
                            _ => {}
                        }
                        if msg.text.chars().count() > 15 {
                            if let Some(avatar_window) = self.windowing.get_window_mut(msg.cr_id) {
                                avatar_window.hide_for_ms(msg.delay);
                            }
                        }
                        if let Some(chat) = &mut self.chat {
                            chat.logic().push_message(msg);
                        }
                    }
                }
            }

            if let Some(avatars) = last_avatars {
                if self.avatars != avatars {
                    self.avatars = avatars;
                    if !self.hide {
                        self.dirty = true;
                    }
                }
            }
        }
    }

    fn update_downloader(&mut self) {
        self.requester_free = self.requester.is_free();
        if self.requester_free {
            if let Some((for_char, new_image)) = self.requester.receive() {
                match new_image {
                    Ok(image) => {
                        self.images
                            .insert(for_char, AvatarImage::Image(ImageData::new(image)));
                        self.dirty = true;
                    }
                    Err(err) => {
                        self.images.insert(for_char, AvatarImage::Error(err));
                    }
                }
            }
        }
    }

    fn redraw(&mut self) {
        if self.is_foreground && !self.hide && !self.avatars.is_empty() {
            let mut popup_game_window = false;

            for avatar in &self.avatars {
                match self.images.get_mut(&avatar.char) {
                    Some(image) => {
                        match image {
                            AvatarImage::Image(image) => {
                                //visible_avatars.push((image, avatar.pos));
                                match self.windowing.window_for_char(avatar.char.id) {
                                    Ok(window) => {
                                        if window.update(avatar, image, &self.rect, self.frame) {
                                            popup_game_window = true;
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!("{:?}", err);
                                    }
                                }
                            }
                            AvatarImage::Error(_) => {
                                // TODO: Implement error recovery
                            }
                        }
                    }
                    None => {
                        if self.requester_free {
                            self.requester.send(avatar.char);
                            self.requester_free = false;
                        }
                    }
                }
            }

            /*if popup_game_window {
                self.game_window.to_foreground();
            }*/
        }
    }

    fn is_game_foreground(&self) -> bool {
        use winapi::um::winuser;
        use crate::windowing::OverlayWindow;

        let game_window = self.game_window.raw();
        let focus = unsafe { winuser::GetForegroundWindow() };
        if focus as usize == 0 {
            return false;
        }
        if focus == game_window {
            return true;
        }
        for window in self.windowing.windows.values() {
            let handle = window.backend_window().window_handle() as usize;
            if handle == focus as usize {
                return true;
            }
        }
        if let Some(chat) = self.chat.as_ref() {
            let handle = chat.backend_window().window_handle() as usize;
            if handle == focus as usize {
                return true;
            }
        }

        false
    }
}

enum AvatarImage {
    Image(ImageData),
    Error(DownloaderError),
}

fn auto_emote(text: &mut String) {
    let mut emoted = text.replace("**", "*");
    text.clear();
    for (i, chunk) in emoted.split("*").enumerate() {
        if chunk.len() == 0 {
            continue;
        }
        let odd = i % 2 == 1;
        if odd {
            text.push_str("**");
        }
        text.push_str(chunk);
        if odd {
            text.push_str("**");
        }
    }
}
