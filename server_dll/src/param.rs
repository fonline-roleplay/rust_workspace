use tnf_common::{
    defines::param::{CritterParamMut, Param},
    engine_types::{
        critter::Critter,
        game_options::{game_state, critter_change_param},
    },
};

pub fn change_uparam(cr: &mut Critter, param: Param, val: u32) -> bool {
    cr.set_uparam(param, val);
    if let Some(game_options) = game_state() {
        critter_change_param(game_options, cr, param as u32)
    } else {
        false
    }
}