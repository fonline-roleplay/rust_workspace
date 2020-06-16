use std::os::raw::{c_char, c_int};
use std::ptr::{null, null_mut};

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

ffi_module!(CLIENT_API, ClientApi, "../../../ffi/API_Client.rs");

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
    unsafe {
        CLIENT_API.Net_SendRunScript(
            true,
            "test@unsafe_test_api\0".as_ptr() as _,
            0,
            0,
            0,
            null(),
            null(),
            0,
        );
    }
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
    let game_options = game_state().expect("Invalid game state");
    let field = game_options.get_field(hex_x, hex_y)?;

    let api = &*CLIENT_API;
    let sprite_id = unsafe { api.Client_AnimGetCurSpr(anim_id) };
    if sprite_id == 0 {
        return None;
    }
    unsafe {
        let sprite = api.Sprites_InsertSprite(
            api.HexMngr_GetDrawTree(),
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
) -> Option<()> {
    let game_options = game_state().expect("Invalid game state");
    let field = game_options.get_field_mut(hex_x, hex_y)?;

    let api = &*CLIENT_API;
    let anim = unsafe { api.ResMngr_GetAnim(name_hash, 0, RES_ITEMS, true) };
    if anim.is_null() {
        return None;
    }
    unsafe { api.Field_ChangeTile(field, anim, offset_x, offset_y, layer, is_roof) };
    Some(())
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
    unsafe { CLIENT_API.HexMngr_GetAllItems_ScriptArray(items) }
}

pub fn HexMngr_GetHexCurrentPosition(hex_x: u16, hex_y: u16) -> (i32, i32) {
    let mut out_x = 0;
    let mut out_y = 0;
    unsafe { CLIENT_API.HexMngr_GetHexCurrentPosition(hex_x, hex_y, &mut out_x, &mut out_y) };
    (out_x, out_y)
}

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use physical_ui::{nphysics_layer, NPhysicsLayer};
use std::hash::Hash;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RegionKey {
    Custom { ty: u16, id: u32, part: u16 },
}

static NPHYSICS_LAYER: Lazy<Mutex<NPhysicsLayer<RegionKey>>> =
    Lazy::new(|| Mutex::new(nphysics_layer()));

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
    let game_options = game_state().expect("Invalid game state");
    let mut layer = NPHYSICS_LAYER.lock();
    //.expect("Lock NPHYSICS_LAYER mutex in PhysicalUI_UpsertCustom");
    let key = RegionKey::Custom { ty, id, part };

    let (zero_x, zero_y) = HexMngr_GetHexCurrentPosition(0, 0);
    //dbg!(zero_x, zero_y);
    let (zero_x, zero_y) = (zero_x as f32, zero_y as f32);

    //dbg!(game_options.ScrOx, game_options.ScrOy);
    let scr_x = game_options.ScrOx as f32;
    let scr_y = game_options.ScrOy as f32;
    let scr_z = game_options.SpritesZoom;
    //dbg!(scr_x, scr_y, scr_z);

    let width = width as f32 * scr_z;
    let height = height as f32 * scr_z;

    //let x = (anchor_x + scr_x) as f32 * scr_z;
    //let y = (anchor_y + scr_y) as f32 * scr_z;
    let x = anchor_x as f32 * scr_z - scr_x - zero_x;
    let y = anchor_y as f32 * scr_z - scr_y - zero_y;

    use physical_ui::{Point, Size};
    let Point { x, y } = layer.upsert(key, Size::new(width, height), Point::new(x, y));
    //*pos_x = ((x / scr_z).round() as i32) - scr_x;
    //*pos_y = ((y / scr_z).round() as i32) - scr_y;
    *pos_x = ((x + scr_x + zero_x) / scr_z).round() as i32;
    *pos_y = ((y + scr_y + zero_y) / scr_z).round() as i32;
}

#[no_mangle]
pub extern "C" fn PhysicalUI_Update(remove_old: bool) {
    std::thread::spawn(move || {
        let mut layer = NPHYSICS_LAYER.lock();
        //.expect("Lock NPHYSICS_LAYER mutex in PhysicalUI_Update");
        layer.update(remove_old);
    });
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
