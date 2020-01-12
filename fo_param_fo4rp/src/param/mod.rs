pub mod misc;
pub mod perk;
pub mod skill;
pub mod stat;
pub mod timeout;
pub mod traits;

mod prelude {
    pub use crate::{
        critter::{CrOp, Critter},
        param::*,
    };
    pub use fo_param::param_types::*;
    pub use formula::prelude::{tools::PartFormula, *};
}

#[deny(dead_code)]
pub mod damage {
    use super::prelude::*;
    pub use crate::basic_impl::damage::*;
}
