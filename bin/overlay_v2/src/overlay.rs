use crate::{
    bridge::{start as bridge_start, BridgeOverlayToClient, MsgIn},
    config::Config,
    game_window::GameWindow,
    gui::Gui,
    requester::ImageRequester,
    windows::ext::OverlaySpawner,
    Platform, Viewport, Wgpu, WgpuManager, ControlFlow
};
use std::{
    time::Duration, ops::{Deref, DerefMut},
};

pub(crate) struct Overlay {
    gui: Gui,
    game_window: GameWindow,
    bridge: BridgeOverlayToClient,
    requester: ImageRequester,
    visibility: OverlayVisibility,
    desired_sleep: Duration,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct OverlayVisibility {
    was_visible: bool,
    now_visible: bool,
}
impl OverlayVisibility {
    pub(crate) fn is_changed(&self) -> bool {
        self.was_visible != self.now_visible
    }
}

#[derive(Debug)]
pub enum OverlayEvent {
    BridgeMessage(MsgIn),
}
pub type OverlayEventProxy = winit::event_loop::EventLoopProxy<OverlayEvent>;

impl Overlay {
    pub(crate) fn new(config: &Config, proxy: OverlayEventProxy) -> Self {
        let bridge = bridge_start(proxy);
        let gui = Gui::new();
        let game_window = GameWindow::with_config(&config).expect("Can't find game client window.");
        let requester = ImageRequester::start(config.url().expect("Web site url is not specified").into());
        let visibility = OverlayVisibility {
            was_visible: true,
            now_visible: true,
        };
        let desired_sleep = config.desired_sleep();
        Self {
            bridge,
            gui,
            game_window,
            requester,
            visibility,
            desired_sleep,
        }
    }

    pub(crate) fn frame(&mut self, ui: &super::imgui::Ui, wgpu: &mut Wgpu) {
        let rect = self.game_window.rect();

        self.update_bridge();

        self.requester.update();
        let mut texture_requester = self.requester.with_renderer(wgpu);

        self.gui.state.update_game_rect(rect);
        self.gui.frame(ui, &mut texture_requester);
    }

    pub fn handle_event(&mut self, event: OverlayEvent) {
        let dirty = match event {
            OverlayEvent::BridgeMessage(msg) => self.handle_bridge_message(msg),
        };
        if dirty {
            self.gui.dirty = self.gui.dirty.max(1);
        }
    }

    fn handle_bridge_message(&mut self, event: MsgIn) -> bool {
        match event {
            MsgIn::UpdateAvatars(avatars) => self.gui.update_avatars(avatars),
            MsgIn::OverlayHide(hide) => {
                if self.gui.hide != hide {
                    self.gui.hide = hide;
                    true
                } else {
                    false
                }
            }
            MsgIn::Message(mut msg) => {
                use fo_defines::Say;
                match msg.say_type {
                    Say::Normal | Say::Shout | Say::Emote | Say::Whisper | Say::Radio => {
                        //log message
                    }
                    _ => return false,
                };
                match msg.say_type {
                    Say::Normal | Say::NormalOnHead => {
                        auto_emote(&mut msg.text);
                    }
                    _ => {}
                }
                // TODO: hide while texting
                /*
                if msg.text.chars().count() > 15 {
                    if let Some(avatar_window) = self.windowing.get_window_mut(msg.cr_id) {
                        avatar_window.hide_for_ms(msg.delay);
                    }
                }
                */

                //if let Some(chat) = &mut self.gui.chat {
                self.gui.push_message(msg);
                //}
                true
            }
        }
    }

    fn update_bridge(&mut self) {
        if self.bridge.is_online() {
            // TODO: Should we handle ping error somehow?
            let _ = self.bridge.ping();
        } else {
            self.gui.update_avatars(Vec::new());
        }
    }

    pub(crate) fn _update_visibility(&mut self, manager: &WgpuManager) {
        let should_be_visible = self.game_window._is_game_foreground(manager);
        self.visibility.was_visible = self.visibility.now_visible;

        if self.visibility.was_visible != should_be_visible {
            for (_, viewport) in manager.viewports_iter() {
                let window = viewport.window();
                window.set_visible(should_be_visible);
            }
            self.visibility.now_visible = should_be_visible;
        }
    }
    pub(crate) fn make_game_foreground(&self) {
        self.game_window.to_foreground();
    }

    pub(crate) fn should_render(&self, platform: &Platform) -> bool {
        if self.visibility.is_changed() || !self.visibility.now_visible {
            return false;
        }
        if self.gui.dirty > 0 {
            return true;
        }
        platform.last_frame().elapsed() > self.desired_sleep
    }

    pub(crate) fn sleep_or_poll(&self, platform: &Platform, control_flow: &mut ControlFlow) {
        let now = std::time::Instant::now();
        let wake_up = platform.last_frame() + self.desired_sleep;
        if wake_up > now {
            *control_flow = ControlFlow::WaitUntil(wake_up);
        } else {
            *control_flow = ControlFlow::Poll;
        }
    }

    pub(crate) fn window_spawner(&self) -> OverlaySpawner {
        OverlaySpawner::new(self.game_window.clone())
    }

    pub(crate) fn should_exit(&self) -> bool {
        !self.game_window.is_alive()
    }

    pub(crate) fn finish(self) {
        self.requester.finish();
        if let Err(err) = self.bridge.finish(true) {
            eprintln!("Bridge finish err: {:?}", err);
        }
    }
}

pub(crate) struct OverlayWrapper(Option<Overlay>);

impl Deref for OverlayWrapper {
    type Target = Overlay;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("Invaild overlay state")
    }
}
impl DerefMut for OverlayWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect("Invaild overlay state")
    }
}
impl Drop for OverlayWrapper {
    fn drop(&mut self) {
        self.0.take().expect("Invaild overlay state").finish();
    }
}

fn auto_emote(text: &mut String) {
    let emoted = text.replace("**", "*");
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
