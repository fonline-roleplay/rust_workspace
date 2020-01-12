#![deny(dead_code)]
use super::prelude::*;
pub use crate::basic_impl::stat::*;

invar!(PERK_UNSET, 0, "ПеркОтсутствует");
invar!(TIMEOUT_READY, 0, "Закончился");
invar!(BONUS_ZERO, 0, "НетБонуса");
invar!(BONUS_RUSH, 1, "БонусЗаВыбросАдреналина");

impl Strength {
    pub fn make_formula(&self) -> impl CrOp {
        let low_life = less_or_equal(
            misc::CurrentLife.base(),
            (MaxLife.base() + Strength.base() + Endurance.base() * int(2)) / int(2),
        );
        let rush_condition = not_equal(perk::AdrenalineRush.base(), PERK_UNSET)
            & not_equal(timeout::Battle.base(), TIMEOUT_READY)
            & "МалоЗдоровья".part(low_life);
        let rush_bonus = "ОтВыбросаАдреналина".part(if_else(
            "ВыбросАдреналинаДействует".part(rush_condition),
            BONUS_RUSH,
            BONUS_ZERO,
        ));

        self.sum() + rush_bonus
    }
}

invar!(NO_DAMAGE, 0, "ПоврежденияНет");
invar!(DAMAGED_PERCEPTION, 1, "ПовреждённоеВосприятие");

impl Perception {
    pub fn make_formula(&self) -> impl CrOp {
        let maybe_damaged = if_else(
            equal(misc::DamageEye.base(), NO_DAMAGE),
            self.sum(),
            DAMAGED_PERCEPTION,
        );
        "ОтВосприятия".part(maybe_damaged)
            + "ОтНочнойПерсоны".part(traits::NightPerson.make_bonus().compat())
    }
}

// Original
//invar!(HP_PER_STR, 1, "ЗдоровьеЗаСилу");
//invar!(HP_PER_END, 2, "ЗдоровьеЗаВыносливость");

// Roleplay
invar!(HP_PER_STR, 4, "ЗдоровьеЗаСилу");
invar!(HP_PER_END, 8, "ЗдоровьеЗаВыносливость");

impl MaxLife {
    pub fn make_formula(&self) -> impl CrOp {
        self.sum()
            + "ОтСилы".part(Strength.base() * HP_PER_STR)
            + "ОтВыносливости".part(Endurance.base() * HP_PER_END)
    }
}
