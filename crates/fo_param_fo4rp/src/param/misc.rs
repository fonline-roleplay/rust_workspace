#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    impl_param!(
        {
            lt: ('a), data: &'a Critter<'a>,
            with_args: ( impl_base!(""), impl_calc!()),
        },
        (LifeCurrent,               "ТекущееЗдоровье",      ST_CURRENT_HP,      ()),
        (ActionPointsCurrent,       "ТекущиеОД",            ST_CURRENT_AP,      (-9999, 9999)),
        (ActionPointsRegen,         "РегенерацияОД",        ST_APREGEN,         ()),
        (ArmorClassTurnBased,       "КлассБрониПошаговый",  ST_TURN_BASED_AC,   ()),
        (MoveActionPointsMax,       "МаксОДПеремещения",    ST_MAX_MOVE_AP,     (0, 9999)),
        (MoveActionPointsCurrent,   "ТекущиеОДПеремещения", ST_MOVE_AP,         (0, 9999)),
    );
}
pub use impl_param::*;

invar!(AP_DIVIDER, 100, "ДелительОД");
impl ActionPointsCurrent {
    pub fn make_formula(&self) -> impl CrOp {
        self.base() / AP_DIVIDER
    }
}

invar!(APREGEN_PER_AGI, 50, "РегенОДзаЛовкость");
invar!(APREGEN_PER_END, 20, "РегенОДзаВыносливость");
invar!(APREGEN_BASE, 20, "БазовыйРегенОД");
impl ActionPointsRegen {
    pub fn make_formula(&self) -> impl CrOp {
        self.base()
            + stat::Agility.base() * APREGEN_PER_AGI
            + stat::Endurance.base() * APREGEN_PER_END
            + APREGEN_BASE
    }
}
