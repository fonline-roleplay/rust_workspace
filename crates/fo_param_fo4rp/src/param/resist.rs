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
        (Normal,    "СопротивлениеНормальномуУрону",    ST_NORMAL_RESIST,   ST_NORMAL_RESIST_EXT,   (0, 90),  (DamageType::Normal)),
        (Laser,     "СопротивлениеЛазерномуУрону",      ST_LASER_RESIST,    ST_LASER_RESIST_EXT,    (0, 90),  (DamageType::Laser)),
        (Fire,      "СопротивлениеОгненномуУрону",      ST_FIRE_RESIST,     ST_FIRE_RESIST_EXT,     (0, 90),  (DamageType::Fire)),
        (Plasma,    "СопротивлениеПлазменномуУрону",    ST_PLASMA_RESIST,   ST_PLASMA_RESIST_EXT,   (0, 90),  (DamageType::Plasma)),
        (Electric,  "СопротивлениеЭлектрическомуУрону", ST_ELECTRO_RESIST,  ST_ELECTRO_RESIST_EXT,  (0, 90),  (DamageType::Electric)),
        (EMP,       "СопротивлениеЭМИУрону",            ST_EMP_RESIST,      ST_EMP_RESIST_EXT,     (0, 999),  (DamageType::Emp)),
        (Explosion, "СопротивлениеВзрывномуУрону",      ST_EXPLODE_RESIST,  ST_EXPLODE_RESIST_EXT,  (0, 90),  (DamageType::Explosion)),
    );

    impl_param!(
        {
            lt: ('a), data: &'a Critter<'a>,
            with_args: ( impl_base!("База"), impl_ext!("Эффект"), impl_calc!() ),
        },
        (Radiation, "СопротивлениеРадиации",      ST_RADIATION_RESISTANCE,  ST_RADIATION_RESISTANCE_EXT,  (0, 95)),
        (Poison,    "СопротивлениеЯду",           ST_POISON_RESISTANCE,     ST_POISON_RESISTANCE_EXT,     (0, 95)),
    );
}
pub use impl_param::*;

invar!(
    RADIATION_RESISTANCE_PER_END,
    2,
    "СопротивлениеРадиацииЗаВыносливость"
);
impl Radiation {
    pub fn make_formula(&self) -> impl CrOp {
        self.sum() + stat::Endurance.calc() * RADIATION_RESISTANCE_PER_END
    }
}

invar!(
    POSION_RESISTANCE_PER_END,
    5,
    "СопротивлениеЯдуЗаВыносливость"
);
impl Poison {
    pub fn make_formula(&self) -> impl CrOp {
        self.sum() + stat::Endurance.calc() * POSION_RESISTANCE_PER_END
    }
}

pub fn sum_and_armor<'a>(
    from_stat: &impl HasParamSum<&'a Critter<'a>>,
    damage_type: fo_engine_types::DamageType,
) -> impl CrOp<'a> {
    "ОтСтатов".part(from_stat.sum())
        + opaque("ОтБрони", move |cr: &Critter| {
            use fo_engine_types::{CritterLike, ItemLike};
            cr.armor().map_or(0, |armor| armor.resist_proc(damage_type))
        })
}
