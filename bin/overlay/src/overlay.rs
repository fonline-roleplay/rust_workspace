use crate::{
    bridge::{start as bridge_start, BridgeOverlayToClient, MsgIn},
    config::{Config, OverlayMode},
    game_window::GameWindow,
    gui::Gui,
    requester::ImageRequester,
    windows::ext::{OverlaySpawner, self as windows_ext},
    Platform, Viewport, Wgpu, WgpuManager, ControlFlow, Manager, WithLoop
};
use std::{
    time::Duration, ops::{Deref, DerefMut},
};
use winit::{
    event_loop::EventLoopWindowTarget
};

pub(crate) struct Overlay {
    main_view: crate::WindowId,
    gui: Gui,
    game_window: GameWindow,
    bridge: BridgeOverlayToClient,
    requester: ImageRequester,
    visibility: OverlayVisibility,
    min_delay: Duration,
    max_delay: Duration,
    mode: OverlayMode,
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
    pub(crate) fn new(config: &Config, proxy: OverlayEventProxy, main_view: crate::WindowId) -> Self {
        let bridge = bridge_start(proxy);
        let gui = Gui::new();
        let game_window = GameWindow::with_config(&config).expect("Can't find game client window.");
        let requester = ImageRequester::start(config.url().expect("Web site url is not specified").into());
        let visibility = OverlayVisibility {
            was_visible: true,
            now_visible: true,
        };
        let min_delay = config.min_delay();
        let max_delay = config.max_delay();
        let mode = config.mode();

        assert!(min_delay <= max_delay);
        Self {
            main_view,
            bridge,
            gui,
            game_window,
            requester,
            visibility,
            min_delay,
            max_delay,
            mode,
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
                if self.gui.hide.client_asks_to_hide != hide {
                    self.gui.hide.client_asks_to_hide = hide;
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

    /*pub(crate) fn _update_visibility(&mut self, manager: &WgpuManager) {
        let should_be_visible = self.game_window.which_foreground(manager);
        self.visibility.was_visible = self.visibility.now_visible;

        if self.visibility.was_visible != should_be_visible {
            for (_, viewport) in manager.viewports_iter() {
                let window = viewport.window();
                window.set_visible(should_be_visible);
            }
            self.visibility.now_visible = should_be_visible;
        }
    }*/
    pub(crate) fn make_game_foreground(&self) {
        self.game_window.to_foreground();
    }

    pub(crate) fn should_render(&self, platform: &Platform) -> bool {
        if self.visibility.is_changed() || !self.visibility.now_visible {
            return false;
        }
        let elapsed = platform.last_frame().elapsed();
        if self.gui.dirty > 0 && elapsed > self.min_delay {
            return true;
        }
        platform.last_frame().elapsed() > self.max_delay
    }

    pub(crate) fn sleep_or_poll(&self, platform: &Platform, control_flow: &mut ControlFlow) {
        match self.mode {
            OverlayMode::Reparent => {
                std::thread::sleep(Duration::from_millis(5));
                *control_flow = ControlFlow::Poll;
            },
            _ => {
                let now = std::time::Instant::now();
                let wake_up = platform.last_frame() + self.max_delay;
                if wake_up > now {
                    *control_flow = ControlFlow::WaitUntil(wake_up);
                } else {
                    *control_flow = ControlFlow::Poll;
                    //ControlFlow::WaitUntil(now + self.min_delay);
                }
            }
        }
        /*let now = std::time::Instant::now();
        let wake_up = platform.last_frame() + self.max_delay;
        if wake_up > now {
            *control_flow = ControlFlow::WaitUntil(wake_up);
        } else {
            *control_flow = ControlFlow::WaitUntil(now + self.min_delay);
        }*/
        
        /*let now = std::time::Instant::now();
        let wake_up = platform.last_frame() + self.max_delay;
        if wake_up > now {
            std::thread::sleep(self.min_delay.min(wake_up - now));
        }*/

        

        //*control_flow = ControlFlow::Poll;

        /*let now = std::time::Instant::now();
        let wake_up = platform.last_frame() + self.min_delay;
        if wake_up > now {
            *control_flow = ControlFlow::WaitUntil(wake_up);
        } else {
            *control_flow = ControlFlow::Poll;
        }*/
    }

    pub(crate) fn spawning_manager<'a>(&self, manager: &'a mut WgpuManager, event_loop: &'a EventLoopWindowTarget<OverlayEvent>) -> WithLoop<'a, WgpuManager, OverlayEvent, OverlaySpawner> {
        let main_view = manager.viewport(self.main_view).map(|vp| vp.window());
        let spawner = OverlaySpawner::new(self.game_window.clone(), main_view, self.mode);
        manager.with_spawner(event_loop, spawner)
    }

    pub(crate) fn reorder_windows<'a, 'b>(&mut self, manager: &'a mut WgpuManager, iter: impl Iterator<Item = (&'b super::imgui::ImStr, super::WindowId)>) {
        use crate::game_window::Foreground;
        let top = match self.game_window.which_foreground(manager) {
            Foreground::Other => false,
            _ => true
        };

        let main_view = manager.viewport(self.main_view).map(|vp| vp.window()).expect("Expect main view");
        
        if let OverlayMode::AutoSort = self.mode {
            main_view.set_always_on_top(top);
            if top {
                main_view.set_always_on_top(false);
            }
            //windows_ext::_place_window_on_top(main_view);
        }

        if !top && self.mode == OverlayMode::TopMost {
            self.gui.hide.client_is_not_visible = true;
            return;
        }        

        let main_view_hwnd = windows_ext::winapi_hwnd(main_view);

        let mut windows: Vec<_> = iter.filter_map(|(title, wid)| {
            if wid == self.main_view {
                None
            } else {
                manager.viewport(wid).map(|vp| {
                    (
                        windows_ext::winapi_hwnd(vp.window()),
                        title,
                        self.gui.layer_by_title(title),
                    )
                })
            }
        }).collect();
        windows.sort_by_key(|(_hwnd, _title, layer)| *layer);
        /*for (_hwnd, title, layer) in &windows {
            println!("title: {:?}, layer: {:?}", title, layer);
        }*/
        let mut windows: Vec<_> = windows.into_iter().map(|(hwnd, ..)| hwnd).collect();
        //windows.reverse();
        
        if let OverlayMode::AutoSort = self.mode {
            windows.insert(0, self.game_window.hwnd());
            //windows_ext::_place_window_on_top_hwnd(*windows.last().unwrap());
            windows.push(main_view_hwnd);
        }

        windows_ext::defer_order(&windows);

        self.gui.hide.client_is_not_visible = false;
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
