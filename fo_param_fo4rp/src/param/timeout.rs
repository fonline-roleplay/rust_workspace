#![deny(dead_code)]
use super::prelude::*;

mod impl_param {
    use crate::param::impl_prelude::*;
    impl_param!(
        (cfg, ('a), &'a Critter<'a>,    impl_base!("Таймаут"),  impl_calc!()),
        (Battle,    "Бой",              TO_BATTLE,              ()),
    );
}
pub use impl_param::*;
