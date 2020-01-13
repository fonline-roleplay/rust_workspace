#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    invar!(TRAIT_NOT_PRESENT, 0, "ТрейтОтсутствует");
    impl_param!(
        (cfg, ('a), &'a Critter<'a>,
            impl_base!("Трейт"), impl_calc!(), impl_present!("ТрейтПрисутствует", InvarI32, TRAIT_NOT_PRESENT)),
        (SmallFrame,    "XилоеТело",        TRAIT_SMALL_FRAME,  (), ()),
        (NightPerson,   "НочнаяПерсона",    TRAIT_NIGHT_PERSON, (), ()),
    );
}
pub use impl_param::*;

invar!(TRAIT_UNSET, 0, "ТрейтОтсутствует");
invar!(BONUS_ZERO, 0, "НетБонуса");
invar!(BONUS_NIGHT_PERSON_DAY, -1, "ДневнойШтраф");
invar!(BONUS_NIGHT_PERSON_NIGHT, 1, "НочнойБонус");

impl NightPerson {
    pub fn make_bonus(&self) -> impl CrOp {
        let trait_present = self.present();
        let is_night = opaque("СейчасНочь", |data: &Critter| {
            data.time.is_night()
        });
        let night_bonus = if_else(is_night, BONUS_NIGHT_PERSON_NIGHT, BONUS_NIGHT_PERSON_DAY);
        if_else(trait_present, night_bonus, BONUS_ZERO)
    }
}
