#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    impl_param!(
        {
                lt: ('a), data: &'a Critter<'a>,
                with_args: ( impl_base!("Таймаут") ),
                no_args: ( impl_calc!(), impl_fn!(crate::param::timeout::impl_timeout) ),
        },
        (Death,             "Смерть",           TO_DEATH),
        (Battle,            "Бой",              TO_BATTLE),
        (Transfer,          "Перемещение",      TO_TRANSFER),
        (RemoveFromGame,    "ВыходИзИгры",      TO_REMOVE_FROM_GAME),
        (Replication,       "Перерождение",     TO_REPLICATION),
        (Tiredness,         "Усталость",        TO_TIREDNESS),
        (Sneak,             "Скрытность",       TO_SNEAK),
        (Healing,           "Лечение",          TO_HEALING),
        (Stealing,          "Кража",            TO_STEALING),
        (Agressor,          "Агрессия",         TO_AGGRESSOR),
        (HairGrow,          "РостВолос",        TO_HAIR_GROW),
        (Say,               "Речь",             TO_SAY),
        (Dead,              "Мертв",            TO_DEAD),
    );
}
pub use impl_param::*;

pub mod of_skill {
    mod impl_param {
        use crate::param::impl_prelude::*;
        impl_param!(
            {
                lt: ('a), data: &'a Critter<'a>,
                with_args: ( impl_base!("СкиллТаймаут") ),
                no_args: ( impl_calc!(), impl_fn!(crate::param::timeout::impl_timeout) ),
            },
            (FirstAid,      "ПерваяПомощь",   TO_SK_FIRST_AID),
            (Doctor,        "Доктор",         TO_SK_DOCTOR),
            (Repair,        "Ремонт",         TO_SK_REPAIR),
            (Science,       "Наука",          TO_SK_SCIENCE),
            (LockPick,      "Взлом",          TO_SK_LOCKPICK),
            (Steal,         "Воровство",      TO_SK_STEAL),
            (Outdoorsman,   "Выживание",      TO_SK_OUTDOORSMAN),
        );
    }
    pub use impl_param::*;
}

pub fn impl_timeout<'a>(timeout: &impl HasParamBase<&'a Critter<'a>>) -> impl CrOp<'a> {
    max(
        timeout.base()
            - opaque(
                "СекундОтСотворенияМира",
                |data: &Critter| data.full_second as i32,
            ),
        int(0),
    )
}
