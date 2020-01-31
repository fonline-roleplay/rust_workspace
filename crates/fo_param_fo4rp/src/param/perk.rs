#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    invar!(PERK_NOT_PRESENT, 0, "ПеркОтсутствует");
    impl_param!(
        {
            lt: ('a), data: &'a Critter<'a>,
            with_args: ( impl_base!("Перк") ),
            no_args: ( impl_calc!(), impl_present!("ТрейтПрисутствует", InvarI32, PERK_NOT_PRESENT) ),
        },
        (AdrenalineRush, "ВыбросАдреналина", PE_ADRENALINE_RUSH),
    );
}
pub use impl_param::*;
