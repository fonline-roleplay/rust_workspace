#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    impl_param!(
        {
            lt: ('a), data: &'a Critter<'a>,
            with_args: ( impl_base!("База"), impl_ext!("Эффект"), impl_calc!()),
        },
        (Strength,        "Сила",               ST_STRENGTH,        ST_STRENGTH_EXT,        (1, 10)),
        (Perception,      "Восприятие",         ST_PERCEPTION,      ST_PERCEPTION_EXT,      (1, 10)),
        (Endurance,       "Выносливость",       ST_ENDURANCE,       ST_ENDURANCE_EXT,       (1, 10)),
        (Charisma,        "Обаяние",            ST_CHARISMA,        ST_CHARISMA_EXT,        (1, 10)),
        (Intellect,       "Интеллект",          ST_INTELLECT,       ST_INTELLECT_EXT,       (1, 10)),
        (Agility,         "Ловкость",           ST_AGILITY,         ST_AGILITY_EXT,         (1, 10)),
        (Luck,            "Удача",              ST_LUCK,            ST_LUCK_EXT,            (1, 10)),
        (LifeMax,         "МаксЗдоровье",       ST_MAX_LIFE,        ST_MAX_LIFE_EXT,        (1, 9999)),
        (ActionPointsMax, "МаксОД",             ST_ACTION_POINTS,   ST_ACTION_POINTS_EXT,   (1, 9999)),
        (WeightMax,       "МаксВес",            ST_CARRY_WEIGHT,    ST_CARRY_WEIGHT_EXT,    (0, 2_000_000_000)),
        (Sequence,        "ПорядокДействий",    ST_SEQUENCE,        ST_SEQUENCE_EXT,        (0, 9999)),
        (MeleeDamage,     "РукопашныйУрон",     ST_MELEE_DAMAGE,    ST_MELEE_DAMAGE_EXT,    (1, 9999)),
        (HealingRate,     "ТемпЛечения",        ST_HEALING_RATE,    ST_HEALING_RATE_EXT,    (0, 9999)),
        (CriticalChance,  "ШансНаКрит",         ST_CRITICAL_CHANCE, ST_CRITICAL_CHANCE_EXT, (0, 100)),
        (CriticalMax,     "ЛучшийКрит",         ST_MAX_CRITICAL,    ST_MAX_CRITICAL_EXT,    (-100, 100)),
        (ArmorClass,      "КлассБрони",         ST_ARMOR_CLASS,     ST_ARMOR_CLASS_EXT,     (0, 90)),
    );
}
pub use impl_param::*;

invar!(TIMEOUT_READY, 0, "Закончился");
invar!(BONUS_ZERO, 0, "НетБонуса");
invar!(BONUS_RUSH, 1, "БонусЗаВыбросАдреналина");

impl Strength {
    pub fn make_formula(&self) -> impl CrOp {
        let low_life = less_or_equal(
            misc::LifeCurrent.base(),
            (LifeMax.base() + Strength.base() + Endurance.base() * int(2)) / int(2),
        );
        let rush_condition = perk::AdrenalineRush.present()
            & not_equal(timeout::Battle.calc(), TIMEOUT_READY)
            & "МалоЗдоровья".part(low_life);
        let rush_bonus = "ОтВыбросаАдреналина".part(if_else(
            "ВыбросАдреналинаДействует".part(rush_condition),
            BONUS_RUSH,
            BONUS_ZERO,
        ));

        self.sum() + rush_bonus
    }
}

invar!(DAMAGED_PERCEPTION, 1, "ПовреждённоеВосприятие");

impl Perception {
    pub fn make_formula(&self) -> impl CrOp {
        let maybe_damaged = if_else(damage::Eye.present(), DAMAGED_PERCEPTION, self.sum());
        "ОтВосприятия".part(maybe_damaged)
            + "ОтНочнойПерсоны".part(traits::NightPerson.make_bonus().compat())
    }
}

impl Endurance {
    // default .sum() formula
}

impl Charisma {
    // default .sum() formula
}

impl Intellect {
    pub fn make_formula(&self) -> impl CrOp {
        "ОтИнтеллекта".part(self.sum())
            + "ОтНочнойПерсоны".part(traits::NightPerson.make_bonus().compat())
    }
}

impl Agility {
    // default .sum() formula
}

impl Luck {
    // default .sum() formula
}

// Original
//invar!(HP_PER_STR, 1, "ЗдоровьеЗаСилу");
//invar!(HP_PER_END, 2, "ЗдоровьеЗаВыносливость");

// Roleplay
invar!(HP_PER_STR, 4, "ЗдоровьеЗаСилу");
invar!(HP_PER_END, 8, "ЗдоровьеЗаВыносливость");

impl LifeMax {
    pub fn make_formula(&self) -> impl CrOp {
        self.sum()
            + "ОтСилы".part(Strength.base() * HP_PER_STR)
            + "ОтВыносливости".part(Endurance.base() * HP_PER_END)
    }
}

invar!(APPOINTS_BASE, 100, "БазовыеОД");

impl ActionPointsMax {
    pub fn make_formula(&self) -> impl CrOp {
        self.sum() + APPOINTS_BASE // + Agility.base() / 2
    }
}

invar!(SMALL_FRAME_CW_MALUS_DIV, 4, "ХилоеТелоШтрафМаксВесаОтСилы");
invar!(CW_PER_STR, 10, "МаксВесЗаСилу");
invar!(CW_BASE, 15, "БазовыйМаксВес");

impl WeightMax {
    pub fn make_formula(&self) -> impl CrOp {
        let small_frame = traits::SmallFrame.base() * SMALL_FRAME_CW_MALUS_DIV;
        let from_strength = Strength.base() * (CW_PER_STR - small_frame);
        "ОтМаксВеса".part(max(self.sum(), int(0)))
            + int(1000) * (CW_BASE + "ОтСилы".part(from_strength))
    }
}

invar!(SEQUENCE_PER_PERCEPTION, 2, "ПорядокДействийЗаВосприятие");
impl Sequence {
    pub fn make_formula(&self) -> impl CrOp {
        self.sum() + Perception.calc() * SEQUENCE_PER_PERCEPTION
    }
}

invar!(MELEE_DAMAGE_BASE, 3, "БазовыйРукопашныйУрон");
invar!(MELEE_DAMAGE_PER_STR, 2, "РукопашныйУронЗаСилу");
impl MeleeDamage {
    pub fn make_formula(&self) -> impl CrOp {
        self.sum() + MELEE_DAMAGE_BASE + "ОтСилы".part(Strength.base() * MELEE_DAMAGE_PER_STR)
    }
}

impl HealingRate {
    pub fn make_formula(&self) -> impl CrOp {
        let from_endurance = "ОтВыносливости".part(max(int(1), Endurance.calc() / int(3)));
        self.sum() + from_endurance
    }
}

impl CriticalChance {
    pub fn make_formula(&self) -> impl CrOp {
        self.sum() + Luck.calc()
    }
}
impl CriticalMax {
    // default .sum() formula
}

invar!(AC_PER_AGILITY, 5, "ОчковБрониЗаЛовкость");
impl ArmorClass {
    pub fn make_formula(&self) -> impl CrOp {
        let armor_ac = opaque("ОчкиНадетойБрони", |data: &Critter| {
            data.armor()
                .and_then(|armor| armor.proto())
                .map_or(0, |armor| armor.Armor_AC)
        });
        self.sum() + Agility.calc() * AC_PER_AGILITY + misc::ArmorClassTurnBased.base() - armor_ac
    }
}

pub mod absorb_and_resist {
    use super::*;
    use formula::prelude::tools::Op;
    use tnf_common::engine_types::item::ProtoItem;

    pub fn sum_and_armor<'a, F: Copy + Fn(&ProtoItem) -> i32>(
        from_stat: &impl HasParamSum<&'a Critter<'a>>,
        from_armor: F,
    ) -> impl CrOp<'a> {
        "ОтСтатов".part(from_stat.sum()) + opaque("ОтБрони", move |cr| armor(cr, from_armor))
    }
    fn armor(cr: &Critter, val: impl Fn(&ProtoItem) -> i32) -> i32 {
        if let Some(armor) = cr.armor() {
            if let Some(proto) = armor.proto() {
                return val(proto) * (100 - armor.get_deterioration_proc()) as i32 / 100;
            }
        }
        0
    }
}
