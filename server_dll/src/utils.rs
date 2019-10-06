use tnf_common::{
    engine_types::map::Map,
    primitives::{Hex, MaybeInvalid},
    utils::map::server::{get_hex_in_path, get_hex_in_path_wall},
};

#[no_mangle]
pub extern "C" fn get_hex_coord_wall(
    map: Option<&MaybeInvalid<Map>>,
    hex_x: u16,
    hex_y: u16,
    end_x: &mut u16,
    end_y: &mut u16,
    angle: f32,
    dist: u32,
) {
    if let Some(map) = map.and_then(MaybeInvalid::validate) {
        let end_hex = get_hex_in_path_wall(
            map,
            Hex { x: hex_x, y: hex_y },
            Hex {
                x: *end_x,
                y: *end_y,
            },
            angle,
            dist,
        );
        *end_x = end_hex.x;
        *end_y = end_hex.y;
    }
}

#[no_mangle]
pub extern "C" fn get_hex_coord(
    map: Option<&MaybeInvalid<Map>>,
    hex_x: u16,
    hex_y: u16,
    end_x: &mut u16,
    end_y: &mut u16,
    angle: f32,
    dist: u32,
) {
    if let Some(map) = map.and_then(MaybeInvalid::validate) {
        let end_hex = get_hex_in_path(
            map,
            Hex { x: hex_x, y: hex_y },
            Hex {
                x: *end_x,
                y: *end_y,
            },
            angle,
            dist,
        );
        *end_x = end_hex.x;
        *end_y = end_hex.y;
    }
}
