#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    invar!(DAMAGE_NOT_PRESENT, 0, "ПовреждениеОтсутствует");
    impl_param!(
        {
            lt: ('a), data: &'a Critter<'a>,
            with_args: ( impl_base!("Состояние") ),
            no_args: ( impl_calc!(), impl_present!("Повреждён", InvarI32, DAMAGE_NOT_PRESENT) ),
        },
        (Eye, "Глаз", DAMAGE_EYE),
    );
}
pub use impl_param::*;
