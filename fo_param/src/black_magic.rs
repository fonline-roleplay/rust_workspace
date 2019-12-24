use crate::param_types::{HasParamBase, HasParamSum, ParamBase, ParamExt, ParamGet};
use formula::prelude::{tools::*, *};

// Black magic
pub trait CalcBase<I: ParamGet> {
    type This: HasParamBase<I>;
    fn make_formula(self) -> Op<I, i32, ParamBase<I, Self::This>>;
}

impl<I: ParamGet, T: HasParamBase<I>> CalcBase<I> for &&T {
    type This = T;
    fn make_formula(self) -> Op<I, i32, ParamBase<I, Self::This>> {
        self.base()
    }
}

pub trait CalcSum<I: ParamGet> {
    type This: HasParamSum<I>;
    fn make_formula(self) -> Op<I, i32, <Self::This as HasParamSum<I>>::Formula>;
}

impl<I: ParamGet, T: HasParamSum<I>> CalcSum<I> for &T {
    type This = T;
    fn make_formula(self) -> Op<I, i32, <Self::This as HasParamSum<I>>::Formula> {
        self.sum()
    }
}

#[macro_export]
macro_rules! impl_param(
        ((cfg, $(<$lt:tt>)?, $data:ty, $base:expr, $ext:expr), ) => {

        };
        ((cfg, $(<$lt:tt>)?, $data:ty, $base:expr, $ext:expr), ($decl:ident,  $name:expr, $index:expr $(, ($min:expr, $max:expr))?), $($rest:tt)*) => {
           #[derive(Debug, Clone, Copy)]
            pub struct $decl;
            impl $decl {
                #[allow(dead_code)]
                pub fn calc$(<$lt>)?(&$($lt)? self) -> formula::prelude::tools::Op<$data, i32, impl formula::prelude::Formula<$data, i32>> {
                    use $crate::black_magic::{CalcSum, CalcBase};
                    use formula::prelude::{clamp, int, FormulaCompat};
                    let res = self.make_formula().compat();
                    $(let res = clamp(res, int($min), int($max));)?
                    res
                }
            }
            impl$(<$lt>)? $crate::param_types::HasParamBase<$data> for $decl {
                const INDEX: <$data as $crate::param_types::ParamGet>::Index = $index;
                const NAME: &'static str = concat!($name, $base);
            }
            impl_param!((cfg, $(<$lt>)?, $data, $base, $ext), $($rest)*);
        };
        (
            (cfg, $(<$lt:tt>)?, $data:ty, $base:expr, $ext:expr),
            ($decl:ident, $name:expr, $index:expr, $index_ext:expr $(, ($min:expr, $max:expr))?),
            $($rest:tt)*
        ) => {
            impl_param!((cfg, $(<$lt>)?, $data, $base, $ext), ($decl, $name, $index $(, ($min, $max))?),);
            impl$(<$lt>)? $crate::param_types::HasParamExt<$data> for $decl {
                const INDEX_EXT: <$data as $crate::param_types::ParamGet>::Index = $index_ext;
                const NAME_EXT: &'static str = concat!($name, $ext);
            }
            impl_param!((cfg, $(<$lt>)?, $data, $base, $ext), $($rest)*);
        };
    );
