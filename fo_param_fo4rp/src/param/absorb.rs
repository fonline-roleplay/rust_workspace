#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    impl_param!(
        {
            lt: ('a), data: &'a Critter<'a>,
            with_args: (
                impl_base!("База"), impl_ext!("Эффект"), impl_calc!(),
                impl_fn!(crate::param::stat::absorb_and_resist::sum_and_armor)
            ),
        },
        
        (Normal,    "ПоглощениеНормальногоУрона",       ST_NORMAL_ABSORB,   ST_NORMAL_ABSORB_EXT,   (0, 999), (|p| p.Armor_DTNormal)),
        (Laser,     "ПоглощениеЛазерногоУрона",         ST_LASER_ABSORB,    ST_LASER_ABSORB_EXT,    (0, 999), (|p| p.Armor_DTLaser)),
        (Fire,      "ПоглощениеОгненногоУрона",         ST_FIRE_ABSORB,     ST_FIRE_ABSORB_EXT,     (0, 999), (|p| p.Armor_DTFire)),
        (Plasma,    "ПоглощениеПлазменногоУрона",       ST_PLASMA_ABSORB,   ST_PLASMA_ABSORB_EXT,   (0, 999), (|p| p.Armor_DTPlasma)),
        (Electro,   "ПоглощениеЭлектрическогоУрона",    ST_ELECTRO_ABSORB,  ST_ELECTRO_ABSORB_EXT,  (0, 999), (|p| p.Armor_DTElectr)),
        (EMP,       "ПоглощениеЭМИУрона",               ST_EMP_ABSORB,      ST_EMP_ABSORB_EXT,      (0, 999), (|p| p.Armor_DTEmp)),
        (Explosion, "ПоглощениеВзрывногоУрона",         ST_EXPLODE_ABSORB,  ST_EXPLODE_ABSORB_EXT,  (0, 999), (|p| p.Armor_DTExplode)),
    );
}
pub use impl_param::*;
