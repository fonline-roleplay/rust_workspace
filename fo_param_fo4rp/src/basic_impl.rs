use crate::{critter::Critter, raw_param::RawParam::*};
use fo_param::impl_param;

pub mod stat {
    use super::*;
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, "База", "Эффект"),
        (Strength, "Сила", ST_STRENGTH, ST_STRENGTH_EXT, (1, 10)),
        (Perception, "Восприятие", ST_PERCEPTION, ST_PERCEPTION_EXT, (1, 10)),
        (Endurance, "Выносливость", ST_ENDURANCE, ST_ENDURANCE_EXT, (1, 10)),
        (MaxLife, "МаксЗдоровье", ST_MAX_LIFE, ST_MAX_LIFE_EXT, (1, 9999)),
    );
}

pub mod misc {
    use super::*;
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, "База", "Эффект"),
        (CurrentLife, "ТекущееЗдоровье", ST_CURRENT_HP),
        (DamageEye, "ПовреждёнГлаз", DAMAGE_EYE),
    );
}

pub mod skill {
    use super::*;
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, "Навык", ""),
        (SmallGuns, "ЛегкоеОружие", SK_SMALL_GUNS),
    );
}

pub mod timeout {
    use super::*;
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, "Таймаут", ""),
        (Battle, "Бой", TO_BATTLE),
    );
}

pub mod perk {
    use super::*;
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, "Перк", ""),
        (AdrenalineRush, "ВыбросАдреналина", PE_ADRENALINE_RUSH),
    );
}

pub mod traits {
    use super::*;
    impl_param!(
        (cfg, <'a>, &'a Critter<'a>, "Трейт", ""),
        (NightPerson, "НочнаяПерсона", TRAIT_NIGHT_PERSON),
    );
}
