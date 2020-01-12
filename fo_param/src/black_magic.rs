use crate::param_types::{HasParamBase, HasParamSum, ParamBase, ParamExt, ParamGet};
use formula::prelude::{tools::*, *};

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
    ((cfg, $lt:tt, $data:ty, $($impl:ident!($($shared:tt)*)),*),) => {

    };
    (
        (cfg, $lt:tt, $data:ty, $($impl:ident!($($shared:tt)*)),*),
        ($decl:ident,  $name:expr, $($args:tt),*),
        $($rest:tt)*
    ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $decl;
        $(
            $impl!(
                cfg: ($lt, $data),
                decl: $decl,
                name: $name,
                shared: ($($shared)*),
                args: $args
            );
        )*
        impl_param!((cfg, $lt, $data, $($impl!($($shared)*)),*), $($rest)*);
    };
);

#[macro_export]
macro_rules! impl_base(
    {
        cfg: (($($lt:tt)?), $data:ty),
        decl: $decl:ident,
        name: $name:expr,
        shared: ($base:expr),
        args: $index:expr
    } => {
        impl$(<$lt>)? $crate::param_types::HasParamBase<$data> for $decl {
            const INDEX: <$data as $crate::param_types::ParamGet>::Index = $index;
            const NAME: &'static str = concat!($name, $base);
        }
    }
);

#[macro_export]
macro_rules! impl_ext(
    {
        cfg: (($($lt:tt)?), $data:ty),
        decl: $decl:ident,
        name: $name:expr,
        shared: ($ext:expr),
        args: $index_ext:expr
    } => {
        impl$(<$lt>)? $crate::param_types::HasParamExt<$data> for $decl {
            const INDEX_EXT: <$data as $crate::param_types::ParamGet>::Index = $index_ext;
            const NAME_EXT: &'static str = concat!($name, $ext);
        }
    }
);

#[macro_export]
macro_rules! impl_calc(
    {
        cfg: (($($lt:tt)?), $data:ty),
        decl: $decl:ident,
        name: $name:expr,
        shared: (),
        args: ($($min:expr, $max:expr)?)
    } => {
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
    }
);

#[macro_export]
macro_rules! impl_present(
    {
        cfg: (($($lt:tt)?), $data:ty),
        decl: $decl:ident,
        name: $name:expr,
        shared: ($present:expr, $not_present_type:ty, $not_present:expr),
        args: ()
    } => {
        impl$(<$lt>)? $crate::param_types::HasParamPresent<$data> for $decl {
            type Formula = $not_present_type;
            const NOT_PRESENT: formula::prelude::tools::Op<(), i32, Self::Formula> = $not_present;
            const CUT: &'static str = concat!($name, $present);
        }
    }
);
