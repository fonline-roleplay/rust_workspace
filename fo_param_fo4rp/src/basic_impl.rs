use crate::{critter::Critter, raw_param::RawParam::*};
use fo_param::{impl_base, impl_calc, impl_ext, impl_param, impl_present};
use formula::prelude::invar;

type InvarI32 = formula::prelude::tools::Invar<i32>;

pub mod stat {
    use super::*;
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, impl_base!("База"), impl_ext!("Эффект"), impl_calc!()),
        (Strength, "Сила", ST_STRENGTH, ST_STRENGTH_EXT, (1, 10)),
        (Perception, "Восприятие", ST_PERCEPTION, ST_PERCEPTION_EXT, (1, 10)),
        (Endurance, "Выносливость", ST_ENDURANCE, ST_ENDURANCE_EXT, (1, 10)),
        (MaxLife, "МаксЗдоровье", ST_MAX_LIFE, ST_MAX_LIFE_EXT, (1, 9999)),
    );
}

pub mod misc {
    use super::*;
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, impl_base!("База"), impl_calc!()),
        (CurrentLife, "ТекущееЗдоровье", ST_CURRENT_HP, ()),
    );
}

pub mod damage {
    use super::*;
    invar!(DAMAGE_NOT_PRESENT, 0, "ПовреждениеОтсутствует");
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, impl_base!("Состояние"), impl_calc!(), impl_present!("Повреждён", InvarI32, DAMAGE_NOT_PRESENT)),
        (Eye, "Глаз", DAMAGE_EYE, (), ()),
    );
}

pub mod skill {
    use super::*;
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, impl_base!("Навык"), impl_calc!()),
        (SmallGuns, "ЛегкоеОружие", SK_SMALL_GUNS, ()),
    );
}

pub mod timeout {
    use super::*;
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, impl_base!("Таймаут"), impl_calc!()),
        (Battle, "Бой", TO_BATTLE, ()),
    );
}

pub mod perk {
    use super::*;
    invar!(PERK_NOT_PRESENT, 0, "ПеркОтсутствует");
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, impl_base!("Перк"), impl_calc!(), impl_present!("ТрейтПрисутствует", InvarI32, PERK_NOT_PRESENT)),
        (AdrenalineRush, "ВыбросАдреналина", PE_ADRENALINE_RUSH, (), ()),
    );
}

pub mod traits {
    use super::*;
    invar!(TRAIT_NOT_PRESENT, 0, "ТрейтОтсутствует");
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, impl_base!("Трейт"), impl_calc!(), impl_present!("ТрейтПрисутствует", InvarI32, TRAIT_NOT_PRESENT)),
        (NightPerson, "НочнаяПерсона", TRAIT_NIGHT_PERSON, (), ()),
    );
}
