use tnf_common::{
    defines::CritterParamMut,
    defines_fo4rp::param::Param,
    engine_types::{
        critter::Critter,
        game_options::{critter_change_param, game_state},
    },
};
//use crate::engine_functions::inv_vec_push_box;

pub fn change_uparams(cr: &mut Critter, params: &[(Param, u32)]) -> bool {
    let mut check = true;
    if let Some(game_options) = game_state() {
        for (param, _val) in params {
            if !critter_change_param(game_options, cr, *param as u32) {
                check = false;
            }
        }
    } else {
        check = false;
    }

    for (param, val) in params {
        cr.set_uparam(*param, *val);
    }
    /*
    for (param, _val) in params {
        if !cr.ParamsIsChanged[*param as usize] {
            inv_vec_push_box(&mut cr.ParamsChanged, *param as i32);
            cr.ParamsIsChanged[*param as usize] = true;
        }
    }*/
    check
}
