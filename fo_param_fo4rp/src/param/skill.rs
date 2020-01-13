#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    impl_param!(
        (cfg, ('a), &'a Critter<'a>,    impl_base!("Навык"),    impl_calc!()),
        (SmallGuns, "ЛегкоеОружие",     SK_SMALL_GUNS,          ()),
    );
}
pub use impl_param::*;
