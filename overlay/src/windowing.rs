use crate::{avatar_window::AvatarWindow, SdlError};
use sdl2::{
    //event::{Event, WindowEvent},
    //keyboard::Keycode,
    EventPump,
    VideoSubsystem,
};
use std::collections::btree_map::{BTreeMap, Entry};

pub struct Windowing {
    pub windows: BTreeMap<u32, AvatarWindow>,
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: VideoSubsystem,
    pub event_pump: EventPump,
}
impl Windowing {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();

        let drivers: Vec<_> = sdl2::video::drivers().collect();
        println!("Available drivers: {:?}", drivers);

        let video_subsystem = sdl_context.video().unwrap();

        println!(
            "Current driver: {:?}",
            video_subsystem.current_video_driver()
        );

        let event_pump = sdl_context.event_pump().expect("event pump");

        Windowing {
            windows: BTreeMap::new(),
            sdl_context,
            video_subsystem,
            event_pump,
        }
    }
    pub fn window_for_char(&mut self, char: u32) -> Result<&mut AvatarWindow, SdlError> {
        use std::collections::btree_map::Entry;
        /*match self.windows.entry(char) {
            Entry::Occupied(mut window) => {
                return Ok(window.get_mut());
            },
            Entry::Vacant(vacant) => {
                let window = AvatarWindow::new(&self.video_subsystem, char)?;
                return Ok(vacant.insert(window));
            }
        }*/
        if let Entry::Vacant(vacant) = self.windows.entry(char) {
            let window = AvatarWindow::new(&self.video_subsystem)?;
            vacant.insert(window);
        }
        Ok(self.windows.get_mut(&char).unwrap())
    }
}
