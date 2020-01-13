pub mod absorb;
pub mod damage;
pub mod misc;
pub mod perk;
pub mod resist;
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

mod impl_prelude {
    pub use crate::{critter::Critter, raw_param::RawParam::*};
    pub use fo_param::{impl_base, impl_calc, impl_ext, impl_fn, impl_param, impl_present};
    pub use formula::prelude::invar;

    pub type InvarI32 = formula::prelude::tools::Invar<i32>;
}
