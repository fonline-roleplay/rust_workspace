#![deny(dead_code)]
use super::prelude::*;
pub use crate::basic_impl::misc::*;

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
