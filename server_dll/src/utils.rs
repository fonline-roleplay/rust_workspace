use crate::config::{config, CheckLook, CritterRates, MovingRates, SenseRates};
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

    let config = &config().check_look;

    let smart = check_look_smart(config, map, cr, opponent);

    /*let old = check_look_old(config, map, cr, opponent);
    if old != smart {
        println!("old != smart: {:?} != {:?}", old, smart);
    }*/

    /*let mut config_default = CheckLook::default();
    config_default.npc_fast.enable = config.npc_fast.enable;
    let smart_default = check_look_smart(&config_default, map, cr, opponent);

    if smart != smart_default {
        println!("smart != smart_default: {:?} != {:?}", smart, smart_default);
    }*/

    smart
}

fn check_look_smart(config: &CheckLook, map: &Map, cr: &Critter, opponent: &Critter) -> bool {
    if map.proto_id() == config.map_utility_start
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

    let cr_perception = param_getters::getParam_Perception(cr, 0) as u32;
    assert!(cr_perception >= 1 && cr_perception <= 10);

    fn basic_dist(rates: &CritterRates, perception: u32) -> u32 {
        rates.basic_bonus + perception * rates.basic_perception_rate
    }

    let self_is_npc = cr.is_npc();

    if self_is_npc {
        if cr.is_dead() {
            return false;
        }
        let npc_fast = &config.npc_fast;
        if npc_fast.enable && cr.ProtoId >= npc_fast.fast_from && cr.ProtoId <= npc_fast.fast_to {
            return basic_dist(&config.senses[npc_fast.sense_index].npc, cr_perception) >= dist;
        }
    }

    let start_dir = cr_hex.get_direction(opp_hex);
    let mut look_dir = i8::abs(start_dir as i8 - cr.Dir as i8); //Направление

    if look_dir > 3 {
        look_dir = 6 - look_dir
    }
    assert!(look_dir >= 0 && look_dir <= 3);

    fn moving_rate(cr: &Critter, rates: &MovingRates) -> f32 {
        if cr.IsRuning {
            rates.running
        //} else if cr.is_walking() {
        //    rates.walking
        } else {
            rates.still
        }
    }

    fn sense_mul(rates: &SenseRates, cr: &Critter, opponent: &Critter, look_dir: i8) -> f32 {
        rates.dir_rate[look_dir as usize]
            * moving_rate(cr, &rates.self_moving)
            * moving_rate(opponent, &rates.target_moving)
    }

    let senses: Vec<(f32, f32)> = config
        .senses
        .iter()
        .map(|sense| {
            let critter_rates = if self_is_npc {
                &sense.npc
            } else {
                &sense.player
            };
            let basic_dist = basic_dist(critter_rates, cr_perception);
            let sense_mul = sense_mul(sense, cr, opponent, look_dir);
            let wall_mul = sense.wall_rate[cr_perception as usize - 1];
            let clear_dist = basic_dist as f32 * sense_mul;
            //dbg!(clear_dist, wall_mul);
            (clear_dist, wall_mul)
        })
        .collect();

    let max_dist = senses
        .iter()
        .map(|(dist, _wall_mul)| *dist as u32)
        .max()
        .expect("At least one sense");

    //dbg!(dist, max_dist);
    if dist > max_dist {
        return false;
    }

    let end_hex = get_hex_in_path(map, cr_hex, opp_hex, 0.0, dist);
    if dist > cr_hex.get_distance(end_hex) {
        for (basic_dist, wall_mull) in senses {
            //dbg!(basic_dist * wall_mull, dist);
            if (basic_dist * wall_mull) as u32 >= dist {
                return true;
            }
        }
        false
    } else {
        true
    }
}

fn _check_look_old(config: &CheckLook, map: &Map, cr: &Critter, opponent: &Critter) -> bool {
    if map.proto_id() == config.map_utility_start
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
        let cfg_npc = &config.npc_fast;
        if cfg_npc.enable && cr.ProtoId >= cfg_npc.fast_from && cr.ProtoId <= cfg_npc.fast_to {
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

    //dbg!(dist, max_view, tmp_max_hear);

    // new optimization: return early if distance larger than max_view and max_hear
    if dist > max_view && dist > tmp_max_hear {
        return false;
    }

    let end_hex = get_hex_in_path(map, cr_hex, opp_hex, 0.0, dist);
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
    //dbg!(max_hear);
    if dist > max_hear {
        is_hear = false;
    }

    return is_view || is_hear;
}
