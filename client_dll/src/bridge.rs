use tnf_common::{
    engine_types::{
        game_options::{self, game_state, Sprite},
        ScriptArray, ScriptString, critter::CritterCl,
    },
    defines::param::{CritterParam, Param},
    bridge::{BridgeClientCell},
    message::client_dll_overlay::{
        OverlayToClientDll as MsgIn,
        ClientDllToOverlay as MsgOut,
        HANDSHAKE, VERSION,
        Avatar, Char, Position,
    },
};

use std::{
    net::SocketAddr,
    convert::identity,
};

use tnf_common::engine_types::game_options::GameOptions;

type BridgeClientToOverlay = BridgeClientCell<MsgIn, MsgOut>;
static BRIDGE: BridgeClientToOverlay = BridgeClientToOverlay::new();

fn is_overlay_running() -> bool {
    let ret = unsafe { winapi::um::winuser::FindWindowA(0 as _, "FOnlineOverlay\0".as_ptr() as _) };
    ret as usize != 0
}

#[no_mangle]
pub extern "C" fn connect_to_overlay(url: &ScriptString) {
    if !is_overlay_running() {
        println!("Spawn new overlay process");
        use std::os::windows::process::CommandExt;
        use winapi::um::winbase;
        let res = std::process::Command::new("FOnlineOverlay")
            .creation_flags(winbase::CREATE_NEW_PROCESS_GROUP | winbase::CREATE_NO_WINDOW)
            .spawn();
        println!("Spawn overlay: {:?}", res);
    } else {
        println!("Reuse old overlay process");
    }

    let url = url.string();
    let addr: SocketAddr = url.parse().expect("malformed socket address");
    BRIDGE.connect(addr, HANDSHAKE, VERSION);
}
/*
#[no_mangle]
pub extern "C" fn update_avatars(array: &ScriptArray) {
    let _res = BRIDGE.with_online(|bridge| {
        let buffer = array.cast_struct().expect("avatar cast");
        let vec = buffer.to_owned();
        bridge.send(MsgOut::UpdateAvatars(vec))
    });
}
*/

fn critter_to_avatar<'a: 'b, 'b>(game_options: &'a GameOptions, critter: &CritterCl, sprites: &mut Option<Vec<&'b Sprite>>) -> Option<Avatar> {
    let ver = critter.uparam(Param::QST_CHAR_VER);
    let secret = critter.uparam(Param::QST_CHAR_SECRET);

    if ver == 0 || secret == 0 {
        return None;
    }

    let hex_x = critter.HexX as i32;
    let hex_y = critter.HexY as i32;

    let sprites = sprites.get_or_insert_with(|| {
        game_options::get_sprites_dot(game_options, 29)
    });

    let sprite = sprites
        .into_iter()
        .filter(|s| s.HexX == hex_x && s.HexY == hex_y)
        .next()?;

    let si = game_options::get_sprite_info(game_options, sprite)?;
    let (x, y) = game_options::sprite_get_top(game_options, sprite, si);

    let char = Char{id: critter.Id, ver, secret};
    let pos = Position{x, y};
    Some(Avatar{char, pos})
}

fn is_player(cr: &CritterCl) -> bool {
    cr.Id < 5_000_000
}

#[no_mangle]
pub extern "C" fn update_avatars(array: &ScriptArray) {
    if let Some(game_options) = game_state() {
        let _res = BRIDGE.with_online(|bridge| {
            let critters = unsafe { array.cast_pointer::<CritterCl>().expect("CritterCl ScriptArray cast") };

            let mut sprites = None;
            let mut avatars = Vec::with_capacity(16);

            for critter in critters.into_iter()
                .filter_map(Option::as_ref)
                .filter(|cr| is_player(*cr))
            {
                if let Some(avatar) = critter_to_avatar(game_options, critter, &mut sprites) {
                    avatars.push(avatar);
                }
            }
            bridge.send(MsgOut::UpdateAvatars(avatars))
        });
    }
}

pub fn finish() {
    let _ = BRIDGE.finish(false);
}