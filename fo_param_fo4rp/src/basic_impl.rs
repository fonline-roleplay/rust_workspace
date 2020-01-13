use crate::{critter::Critter, raw_param::RawParam::*};
use fo_param::{impl_base, impl_calc, impl_ext, impl_param, impl_present};
use formula::prelude::invar;

type InvarI32 = formula::prelude::tools::Invar<i32>;

pub mod stat {
    use super::*;
    impl_param!(
        (cfg, ('a), &'a Critter<'a>, impl_base!("База"), impl_ext!("Эффект"), impl_calc!()),
        (Strength, "Сила", ST_STRENGTH, ST_STRENGTH_EXT, (1, 10)),
        (Perception, "Восприятие", ST_PERCEPTION, ST_PERCEPTION_EXT, (1, 10)),
        (Endurance, "Выносливость", ST_ENDURANCE, ST_ENDURANCE_EXT, (1, 10)),
        (Charisma, "Обаяние", ST_CHARISMA, ST_CHARISMA_EXT, (1, 10)),
        (Intellect, "Интеллект", ST_INTELLECT, ST_INTELLECT_EXT, (1, 10)),
        (Agility, "Ловкость", ST_AGILITY, ST_AGILITY_EXT, (1, 10)),
        (Luck, "Удача", ST_LUCK, ST_LUCK_EXT, (1, 10)),
        (LifeMax, "МаксЗдоровье", ST_MAX_LIFE, ST_MAX_LIFE_EXT, (1, 9999)),
        (ActionPointsMax, "МаксОД", ST_ACTION_POINTS, ST_ACTION_POINTS_EXT, (1, 9999)),
        (WeightMax, "МаксВес", ST_CARRY_WEIGHT, ST_CARRY_WEIGHT_EXT, (0, 2_000_000_000)),
        (Sequence, "ПорядокДействий", ST_SEQUENCE, ST_SEQUENCE_EXT, (0, 9999)),
        (MeleeDamage, "РукопашныйУрон", ST_MELEE_DAMAGE, ST_MELEE_DAMAGE_EXT, (1, 9999)),
        (HealingRate, "ТемпЛечения", ST_HEALING_RATE, ST_HEALING_RATE_EXT, (0, 9999)),
        (CriticalChance, "ШансНаКрит", ST_CRITICAL_CHANCE, ST_CRITICAL_CHANCE_EXT, (0, 100)),
        (CriticalMax, "ЛучшийКрит", ST_MAX_CRITICAL, ST_MAX_CRITICAL_EXT, (-100, 100)),
        (ArmorClass, "КлассБрони", ST_ARMOR_CLASS, ST_ARMOR_CLASS_EXT, (0, 90)),
    );
}

pub mod misc {
    use super::*;
    impl_param!(
        (cfg, ('a), &'a Critter<'a>, impl_base!(""), impl_calc!()),
        (LifeCurrent, "ТекущееЗдоровье", ST_CURRENT_HP, ()),
        (ActionPointsCurrent, "ТекущиеОД", ST_CURRENT_AP, (-9999, 9999)),
        (ActionPointsRegen, "РегенерацияОД", ST_APREGEN, ()),
        (MoveActionPointsMax, "МаксОДПеремещения", ST_MAX_MOVE_AP, (0, 9999)),
        (MoveActionPointsCurrent, "ТекущиеОДПеремещения", ST_MOVE_AP, (0, 9999)),
    );
}

pub mod damage {
    use super::*;
    invar!(DAMAGE_NOT_PRESENT, 0, "ПовреждениеОтсутствует");
    impl_param!(
        (cfg, ('a), &'a Critter<'a>, impl_base!("Состояние"), impl_calc!(), impl_present!("Повреждён", InvarI32, DAMAGE_NOT_PRESENT)),
        (Eye, "Глаз", DAMAGE_EYE, (), ()),
    );
}

pub mod skill {
    use super::*;
    impl_param!(
        (cfg, ('a), &'a Critter<'a>, impl_base!("Навык"), impl_calc!()),
        (SmallGuns, "ЛегкоеОружие", SK_SMALL_GUNS, ()),
    );
}

pub mod timeout {
    use super::*;
    impl_param!(
        (cfg, ('a), &'a Critter<'a>, impl_base!("Таймаут"), impl_calc!()),
        (Battle, "Бой", TO_BATTLE, ()),
    );
}

pub mod perk {
    use super::*;
    invar!(PERK_NOT_PRESENT, 0, "ПеркОтсутствует");
    impl_param!(
        (cfg, ('a), &'a Critter<'a>, impl_base!("Перк"), impl_calc!(), impl_present!("ТрейтПрисутствует", InvarI32, PERK_NOT_PRESENT)),
        (AdrenalineRush, "ВыбросАдреналина", PE_ADRENALINE_RUSH, (), ()),
    );
}

pub mod traits {
    use super::*;
    invar!(TRAIT_NOT_PRESENT, 0, "ТрейтОтсутствует");
    impl_param!(
        (cfg, ('a), &'a Critter<'a>, impl_base!("Трейт"), impl_calc!(), impl_present!("ТрейтПрисутствует", InvarI32, TRAIT_NOT_PRESENT)),
        (SmallFrame, "XилоеТело", TRAIT_SMALL_FRAME, (), ()),
        (NightPerson, "НочнаяПерсона", TRAIT_NIGHT_PERSON, (), ()),
    );
}
