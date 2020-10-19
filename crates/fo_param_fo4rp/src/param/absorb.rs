#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    use fo_engine_types::DamageType;
    impl_param!(
        {
            lt: ('a), data: &'a Critter<'a>,
            with_args: (
                impl_base!("База"), impl_ext!("Эффект"), impl_calc!(),
                impl_fn!(super::sum_and_armor)
            ),
        },

        (Normal,    "ПоглощениеНормальногоУрона",       ST_NORMAL_ABSORB,   ST_NORMAL_ABSORB_EXT,   (0, 999), (DamageType::Normal)),
        (Laser,     "ПоглощениеЛазерногоУрона",         ST_LASER_ABSORB,    ST_LASER_ABSORB_EXT,    (0, 999), (DamageType::Laser)),
        (Fire,      "ПоглощениеОгненногоУрона",         ST_FIRE_ABSORB,     ST_FIRE_ABSORB_EXT,     (0, 999), (DamageType::Fire)),
        (Plasma,    "ПоглощениеПлазменногоУрона",       ST_PLASMA_ABSORB,   ST_PLASMA_ABSORB_EXT,   (0, 999), (DamageType::Plasma)),
        (Electric,  "ПоглощениеЭлектрическогоУрона",    ST_ELECTRO_ABSORB,  ST_ELECTRO_ABSORB_EXT,  (0, 999), (DamageType::Electric)),
        (EMP,       "ПоглощениеЭМИУрона",               ST_EMP_ABSORB,      ST_EMP_ABSORB_EXT,      (0, 999), (DamageType::Emp)),
        (Explosion, "ПоглощениеВзрывногоУрона",         ST_EXPLODE_ABSORB,  ST_EXPLODE_ABSORB_EXT,  (0, 999), (DamageType::Explosion)),
    );
}
pub use impl_param::*;

pub fn sum_and_armor<'a>(
    from_stat: &impl HasParamSum<&'a Critter<'a>>,
    damage_type: fo_engine_types::DamageType,
) -> impl CrOp<'a> {
    "ОтСтатов".part(from_stat.sum())
        + opaque("ОтБрони", move |cr: &Critter| {
            use fo_engine_types::{CritterLike, ItemLike};
            cr.armor().map_or(0, |armor| armor.absorb_proc(damage_type))
        })
}
