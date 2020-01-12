#![deny(dead_code)]
use super::prelude::*;
pub use crate::basic_impl::traits::*;

invar!(TRAIT_UNSET, 0, "ТрейтОтсутствует");
invar!(BONUS_ZERO, 0, "НетБонуса");
invar!(BONUS_NIGHT_PERSON_DAY, -1, "ДневнойШтраф");
invar!(BONUS_NIGHT_PERSON_NIGHT, 1, "НочнойБонус");

impl NightPerson {
    pub fn make_bonus(&self) -> impl CrOp {
        let trait_set = not_equal(self.base(), TRAIT_UNSET);
        let is_night = opaque("СейчасНочь", |data: &Critter| {
            data.time.is_night()
        });
        let night_bonus = if_else(is_night, BONUS_NIGHT_PERSON_NIGHT, BONUS_NIGHT_PERSON_DAY);
        if_else(trait_set, night_bonus, BONUS_ZERO)
    }
}
