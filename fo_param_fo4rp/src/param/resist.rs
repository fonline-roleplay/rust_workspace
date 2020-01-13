#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    impl_param!(
        (cfg, ('a), &'a Critter<'a>, impl_base!("База"), impl_ext!("Эффект"), impl_calc!(),
            impl_fn!(crate::param::stat::absorb_and_resist::sum_and_armor)
        ),
          
        (Normal,    "СопротивлениеНормальномуУрону",    ST_NORMAL_RESIST,   ST_NORMAL_RESIST_EXT,   (0, 90),  (|p| p.Armor_DRNormal)),
        (Laser,     "СопротивлениеЛазерномуУрону",      ST_LASER_RESIST,    ST_LASER_RESIST_EXT,    (0, 90),  (|p| p.Armor_DRLaser)),
        (Fire,      "СопротивлениеОгненномуУрону",      ST_FIRE_RESIST,     ST_FIRE_RESIST_EXT,     (0, 90),  (|p| p.Armor_DRFire)),
        (Plasma,    "СопротивлениеПлазменномуУрону",    ST_PLASMA_RESIST,   ST_PLASMA_RESIST_EXT,   (0, 90),  (|p| p.Armor_DRPlasma)),
        (Electro,   "СопротивлениеЭлектрическомуУрону", ST_ELECTRO_RESIST,  ST_ELECTRO_RESIST_EXT,  (0, 90),  (|p| p.Armor_DRElectr)),
        (EMP,       "СопротивлениеЭМИУрону",            ST_EMP_RESIST,      ST_EMP_RESIST_EXT,     (0, 999),  (|p| p.Armor_DREmp)),
        (Explosion, "СопротивлениеВзрывномуУрону",      ST_EXPLODE_RESIST,  ST_EXPLODE_RESIST_EXT,  (0, 90),  (|p| p.Armor_DRExplode)),
    );

    impl_param!(
        (cfg, ('a), &'a Critter<'a>, impl_base!("База"), impl_ext!("Эффект"), impl_calc!()),
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
