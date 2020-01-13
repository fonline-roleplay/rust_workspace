#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    invar!(DAMAGE_NOT_PRESENT, 0, "ПовреждениеОтсутствует");
    impl_param!(
        (cfg, ('a), &'a Critter<'a>,
            impl_base!("Состояние"), impl_calc!(), impl_present!("Повреждён", InvarI32, DAMAGE_NOT_PRESENT)),
        (Eye, "Глаз", DAMAGE_EYE, (), ()),
    );
}
pub use impl_param::*;
