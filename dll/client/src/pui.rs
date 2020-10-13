use crate::{engine_functions, Client, State};
use tnf_common::engine_types::game_options::{game_state, GameOptions};

use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;
use physical_ui::{nphysics_layer, NPhysicsLayer};
use std::hash::Hash;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RegionKey {
    Custom { ty: u16, id: u32, part: u16 },
}

type Layer = NPhysicsLayer<RegionKey>;

struct AnchoredLayer {
    layer: Box<Layer>,
    shift_x: f32,
    shift_y: f32,
    zoom: f32,
}

/*
enum PuiThreadCommand {
    Update{remove_old: bool},
    Terminate,
}
*/
pub(crate) struct Pui {
    sender: Option<Sender<(Box<Layer>, bool)>>,
    receiver: Receiver<Box<Layer>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Pui {
    pub(crate) fn new() -> Self {
        let layer = nphysics_layer();
        let (sender, thread_receiver) = bounded::<(Box<Layer>, bool)>(1);
        let (thread_sender, receiver) = bounded(1);
        thread_sender
            .send(Box::new(layer))
            .expect("Fresh pui layer send");
        let handle = std::thread::spawn(move || {
            for (mut layer, remove_old) in thread_receiver.iter() {
                layer.update(remove_old);
                if thread_sender.send(layer).is_err() {
                    return;
                }
            }
        });
        Self {
            sender: Some(sender),
            receiver,
            thread: Some(handle),
        }
    }
    fn anchored_layer(&self, client: &Client, timeout_ms: u64) -> Option<AnchoredLayer> {
        let layer = self.recv_layer(client, timeout_ms)?;
        Some(AnchoredLayer::prepare(layer, client))
    }
    fn recv_layer(&self, client: &Client, timeout_ms: u64) -> Option<Box<Layer>> {
        self.receiver
            .recv_timeout(std::time::Duration::from_millis(timeout_ms))
            .ok()
    }
    fn flush(&self, layer: Box<Layer>, remove_old: bool) {
        self.sender
            .as_ref()
            .expect("Pui sender")
            .send((layer, remove_old))
            .expect("Send message to physics engine thread");
    }
}

impl Drop for Pui {
    fn drop(&mut self) {
        drop(self.sender.take());
        if let Some(handle) = self.thread.take() {
            handle.join().expect("Pui thread join");
        }
    }
}

impl AnchoredLayer {
    fn adjust(self, client: &Client) -> Self {
        Self::prepare(self.layer, client)
    }
    fn prepare(layer: Box<Layer>, client: &Client) -> Self {
        let game_options = game_state().expect("Invalid game state");

        let (zero_x, zero_y) = engine_functions::HexMngr_GetHexCurrentPosition(&client, 0, 0)
            .expect("Map is not loaded");
        let (zero_x, zero_y) = (zero_x as f32, zero_y as f32);

        let scr_x = game_options.ScrOx as f32;
        let scr_y = game_options.ScrOy as f32;
        let zoom = game_options.SpritesZoom;

        Self {
            layer,
            shift_x: zero_x + scr_x,
            shift_y: zero_y + scr_y,
            zoom,
        }
    }
}

static LAYER: Mutex<Option<AnchoredLayer>> = parking_lot::const_mutex(None);

#[no_mangle]
pub extern "C" fn PhysicalUI_StartFrame() -> bool {
    Client::with(|client| {
        let mut guard = LAYER.lock();

        let option = match guard.take() {
            Some(layer) => Some(layer.adjust(client)),
            None => client.pui.anchored_layer(client, 10),
        };
        if option.is_some() {
            *guard = option;
            true
        } else {
            false
        }
    })
}

#[no_mangle]
pub extern "C" fn PhysicalUI_UpsertCustom(
    id: u32,
    ty: u16,
    part: u16,
    anchor_x: i32,
    anchor_y: i32,
    width: u16,
    height: u16,
    pos_x: &mut i32,
    pos_y: &mut i32,
) {
    let mut guard = LAYER.lock();
    let anchored_layer = guard.as_mut().expect("PhysicalUI is not prepared!");
    let AnchoredLayer {
        ref mut layer,
        shift_x,
        shift_y,
        zoom,
    } = *anchored_layer;

    let key = RegionKey::Custom { ty, id, part };

    let width = width as f32 * zoom;
    let height = height as f32 * zoom;

    let x = anchor_x as f32 * zoom - shift_x;
    let y = anchor_y as f32 * zoom - shift_y;

    use physical_ui::{Point, Size};
    let Point { x, y } = layer.upsert(key, Size::new(width, height), Point::new(x, y));
    *pos_x = ((x + shift_x) / zoom).round() as i32;
    *pos_y = ((y + shift_y) / zoom).round() as i32;
}

#[no_mangle]
pub extern "C" fn PhysicalUI_EndFrame() {
    Client::with(|client| {
        let mut guard = LAYER.lock();

        let layer = guard.take().expect("PhysicalUI is not prepared!").layer;
        client.pui.flush(layer, true);
    });
}

#[no_mangle]
pub extern "C" fn PhysicalUI_Reset() {
    Client::with(|client| {
        let mut guard = LAYER.lock();

        let option = match guard.take() {
            Some(anchored) => Some(anchored.layer),
            None => client.pui.recv_layer(client, 100),
        };

        if let Some(layer) = option {
            client.pui.flush(layer, true);
        }
    });
}
