use tnf_common::{
    dll::param_getters,
    engine_types::{critter::Critter, map::Map},
    primitives::{Hex, MaybeInvalid},
    utils::map::{
        get_distance_hex,
        server::{get_hex_in_path, get_hex_in_path_wall},
    },
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
/*
#[no_mangle]
pub extern "C" fn test_hex_flags(
    map: Option<&MaybeInvalid<Map>>,
    hex_x: u16,
    hex_y: u16,
    raked: bool,
    passed: bool,
) {
    if let Some(map) = map.and_then(MaybeInvalid::validate) {
        let hex = Hex { x: hex_x, y: hex_y };
        let flags = map.get_hex_flags_with_proto(hex);
        let mut wrong = false;
        if raked != map.is_hex_raked(hex) {
            wrong = true;
            print!("Raked - should be {}, but {}; ", raked, !raked);
        }
        if passed != map.is_hex_passed(hex) {
            wrong = true;
            print!("Passed - should be {}, but {}; ", passed, !passed);
        }
        if wrong {
            println!("Hex: {:?}, flags: {:016b}", hex, flags);
        }
    }
}
*/

macro_rules! validate {
    ($this:expr, $default:expr) => {
        match $this.and_then(MaybeInvalid::validate) {
            Some(this) => this,
            None => return $default,
        }
    };
}
/*
#[no_mangle]
pub extern "C" fn is_gM(Critter& player)
{
if( !player.IsPlayer() ) return false;

if( !isLoadedGMs )
LoadGMs( player, 0, 0, 0 );

if( player.StatBase[ ST_ACCESS_LEVEL ] < ACCESS_MODER && ( player.GetAccess() >= ACCESS_MODER || isPocketGM( player.Id ) ) )
player.StatBase[ ST_ACCESS_LEVEL ] = ACCESS_MODER;

return player.StatBase[ ST_ACCESS_LEVEL ] >= ACCESS_MODER && ( checkVision ? player.ParamBase[ QST_VISION ] > 0 : true );
}*/

const MAP_UTILITY_START: u16 = 92;

#[no_mangle]
pub extern "C" fn check_look(
    map: Option<&MaybeInvalid<Map>>,
    cr: Option<&MaybeInvalid<Critter>>,
    opponent: Option<&MaybeInvalid<Critter>>,
) -> bool {
    // Consider remove this
    let map = validate!(map, false);
    let cr = validate!(cr, false);
    let opponent = validate!(opponent, false);

    if map.proto_id() == MAP_UTILITY_START
        && opponent.is_player()
        && cr.is_player()
        && !cr.have_gm_vision()
    {
        return false;
    }

    let cr_hex = cr.hex();
    let opp_hex = opponent.hex();
    let dist = cr_hex.get_distance(opp_hex);

    use tnf_common::defines::param::{CritterParam, Param};
    let cr_vision = cr.uparam(Param::QST_VISION);
    let cr_perception = param_getters::getParam_Perception(cr, 0) as u32;
    //cr.uparam(Param::ST_PERCEPTION);

    let opp_invis = opponent.uparam(Param::QST_INVIS);

    if cr_vision >= dist && opp_invis <= dist {
        return true;
    }
    if opp_invis != 0 && (opp_invis - 1) < dist {
        // && ( !( cr.IsPlayer() ) || cr.IsPlayer() && !isGM( cr ) ) )
        return false;
    }
    if opp_invis > dist || cr_vision >= dist {
        return true;
    }

    if cr.is_npc() {
        // упрощенный расчет для нпц, учитывает только дистанцию
        if cr.is_dead() {
            return false;
        }
        if cr.ProtoId >= 2200 {
            // ???
            return (10 + cr_perception * 5) >= dist;
        }
    }

    let max_view = 10 + cr_perception * 5;
    let mut max_hear = 5 + cr_perception * 2;
    if cr.is_npc() {
        max_hear += 20;
    }

    let mut is_view = true;
    let mut is_hear = true;

    let start_dir = cr_hex.get_direction(opp_hex);
    let mut look_dir = i8::abs(start_dir as i8 - cr.Dir as i8); //Направление

    if look_dir > 3 {
        look_dir = 6 - look_dir
    }

    let (view_mul, mut hear_mul) = match look_dir {
        0 => (1.0, 0.8),
        1 => (0.8, 1.0),
        2 => (0.5, 0.8),
        3 => (0.4, 0.8),
        _ => unreachable!(),
    };

    if opponent.IsRuning {
        hear_mul *= 3.0;
    }
    if cr.IsRuning {
        hear_mul *= 0.8;
    }

    let max_view = (max_view as f32 * view_mul) as u32;
    let tmp_max_hear = (max_hear as f32 * hear_mul) as u32;

    // new optimization: return early if distance larger than max_view and max_hear
    if dist > max_view && dist > tmp_max_hear {
        return false;
    }

    let end_hex = get_hex_in_path(map, cr_hex, opp_hex, 0.0, max_view);
    if dist > cr_hex.get_distance(end_hex) {
        is_view = false;
        hear_mul *= match cr_perception {
            1..=4 => 0.1,
            5..=8 => 0.3,
            9..=10 => 0.4,
            _ => 1.0,
        };
    }
    if dist > max_view {
        is_view = false;
    }

    let max_hear = (max_hear as f32 * hear_mul) as u32;
    if dist > max_hear {
        is_hear = false;
    }

    return is_view || is_hear;
}
