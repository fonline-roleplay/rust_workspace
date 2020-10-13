use std::os::raw::{c_char, c_int};
use std::ptr::{null, null_mut};

use crate::{Client, State};
use tnf_common::{
    engine_types::{
        game_options::{game_state, Field, GameOptions, Sprite},
        ScriptArray,
    },
    primitives::*,
};

pub enum Sprites {}
pub enum AnyFrames {}

use fo_engine_functions::*;

ffi_module!(ClientApi, "../../../ffi/API_Client.rs");
pub(crate) use ffi::ClientApi;

fn _unwrap_or_abort<T, E: std::fmt::Display>(res: Result<T, E>) -> T {
    match res {
        Ok(ok) => ok,
        Err(err) => {
            println!("Abort: {}", err);
            std::process::abort()
        }
    }
}

#[no_mangle]
pub extern "C" fn test_send_run_script() {
    Client::with(|client| unsafe {
        client.api.Net_SendRunScript(
            true,
            "test@unsafe_test_api\0".as_ptr() as _,
            0,
            0,
            0,
            null(),
            null(),
            0,
        );
    })
}

#[no_mangle]
pub extern "C" fn add_map_sprite(
    hex_x: u16,
    hex_y: u16,
    anim_id: u32,
    spr_index: i32,
    offs_x: i32,
    offs_y: i32,
    draw_order_type: i32,
    draw_offs_y: i32,
) -> Option<&'static mut Sprite> {
    Client::with(|client| {
        let game_options = game_state().expect("Invalid game state");
        let field = game_options.get_field(hex_x, hex_y)?;

        let sprite_id = unsafe { client.api.Client_AnimGetCurSpr(anim_id) };
        if sprite_id == 0 {
            return None;
        }
        unsafe {
            let sprite = client.api.Sprites_InsertSprite(
                client.api.HexMngr_GetDrawTree(),
                draw_order_type,
                hex_x as i32,
                hex_y as i32 + draw_offs_y,
                0,
                field.ScrX + offs_x,
                field.ScrY + offs_y,
                sprite_id,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            );
            std::mem::transmute(sprite)
        }
    })
}

/*
#define RES_NONE                   ( 0 )
#define RES_IFACE                  ( 1 )
#define RES_CRITTERS               ( 2 )
#define RES_ITEMS                  ( 3 )
#define RES_SCRIPT                 ( 4 )
#define RES_SPLASH                 ( 5 )
#define RES_GLOBAL_MAP             ( 6 )
#define RES_IFACE_EXT              ( 7 )
*/
const RES_ITEMS: i32 = 3; //used for tiles and items

#[no_mangle]
pub extern "C" fn change_tile(
    name_hash: u32,
    hex_x: u16,
    hex_y: u16,
    offset_x: i16,
    offset_y: i16,
    layer: u8,
    is_roof: bool,
) -> bool {
    Client::with(|client| -> Option<()> {
        let game_options = game_state().expect("Invalid game state");
        let field = game_options.get_field_mut(hex_x, hex_y)?;

        let anim = unsafe { client.api.ResMngr_GetAnim(name_hash, 0, RES_ITEMS, true) };
        if anim.is_null() {
            return None;
        }
        unsafe {
            client
                .api
                .Field_ChangeTile(field, anim, offset_x, offset_y, layer, is_roof)
        };
        Some(())
    })
    .is_some()
}
/*
#[no_mangle]
pub extern "C" fn max_roof_num() -> u16 {
    let game_options = game_state().expect("Invalid game state");
    game_options.max_roof_num().unwrap_or(0)
}
*/
/*
#[no_mangle]
pub extern "C" fn update_roof_num(hex_x: u16, hex_y: u16, mut roof_num: u16) -> u16 {
    let game_options = game_state().expect("Invalid game state");
    if let Some(field) = game_options.get_field_mut(hex_x, hex_y) {
        if field.RoofNum != 0 {
            return roof_num;
        }
    } else {
        return roof_num;
    }

    let skip_size = game_options.MapRoofSkipSize as i16;
    let neighbors = [
        (0, -skip_size),
        (-skip_size, 0),
        (0, skip_size),
        (skip_size, 0),
    ];

    let other_roof_nums: Vec<_> = neighbors
        .iter()
        .filter_map(|&(ox, oy)| game_options.get_field_offset(hex_x, hex_y, ox, oy))
        .filter_map(|field| Some(field.RoofNum as u16).filter(|&num| num != 0))
        .collect();

    let new_roof_num = if other_roof_nums.is_empty() {
        roof_num += 1;
        roof_num
    } else {
        other_roof_nums[0]
    };

    fn fill_square(game_options: &mut GameOptions, hex_x: u16, hex_y: u16, new_roof_num: u16) {
        let skip_size = game_options.MapRoofSkipSize as i16;
        for oy in 0..skip_size {
            for ox in 0..skip_size {
                if let Some(field) = game_options.get_field_offset_mut(hex_x, hex_y, ox, oy) {
                    field.RoofNum = new_roof_num as _;
                }
            }
        }
    }
    fill_square(game_options, hex_x, hex_y, new_roof_num);

    return roof_num;
}
*/
#[no_mangle]
pub extern "C" fn regroup_roofs() {
    let game_options = game_state().expect("Invalid game state");
    game_options.regroup_roofs();
}

#[no_mangle]
pub extern "C" fn GetAllItems(items: *mut ScriptArray) -> size_t {
    Client::with(|client| unsafe { client.api.HexMngr_GetAllItems_ScriptArray(items) })
}

pub(crate) fn HexMngr_GetHexCurrentPosition(
    client: &Client,
    hex_x: u16,
    hex_y: u16,
) -> Option<(i32, i32)> {
    let mut out_x = 0;
    let mut out_y = 0;
    if unsafe {
        client
            .api
            .HexMngr_GetHexCurrentPosition(hex_x, hex_y, &mut out_x, &mut out_y)
    } {
        Some((out_x, out_y))
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn option_is_bool() {
        assert_eq!(1, std::mem::size_of_val(&Some(())));
        let some: bool = unsafe { std::mem::transmute(Some(())) };
        let none: bool = unsafe { std::mem::transmute(Option::<()>::None) };
        assert_eq!(false, none);
        assert_eq!(true, some);
    }
}
