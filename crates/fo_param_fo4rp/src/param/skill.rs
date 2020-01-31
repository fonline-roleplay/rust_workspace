#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    impl_param!(
        {
            lt: ('a), data: &'a Critter<'a>,
            with_args: ( impl_base!("Навык") ),
            no_args: ( impl_calc!() ),
        },
        (SmallGuns, "ЛегкоеОружие",     SK_SMALL_GUNS),
    );
}
pub use impl_param::*;
