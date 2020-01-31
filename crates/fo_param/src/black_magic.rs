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
    (
        {
            lt: $lt:tt, data: $data:ty,
            with_args: ($ ($impl:ident!($ ($shared:tt)*)),*),
            $(no_args: ($($impl2:ident!($($shared2:tt)*)),*),)?
            $(all_decls: $all_decls:tt,)?
        },
    ) => {

    };
    (
        {
            lt: $lt:tt, data: $data:ty,
            with_args: ($ ($impl:ident!($ ($shared:tt)*)),*),
            $(no_args: ($($impl2:ident!($($shared2:tt)*)),*),)?
            with_decls: ($($impl3:ident!($($shared3:tt)*)),*),
            all_decls: $all_decls:tt,
        },
    ) => {
        $(
            $impl3!(
                lt: $lt, data: $data,
                shared: ($($shared3)*),
                args: $all_decls
            );
        )*
    };
    (
        {
            lt: $lt:tt, data: $data:ty,
            with_args: ($ ($impl:ident!($ ($shared:tt)*)),*),
            $(no_args: ($($impl2:ident!($($shared2:tt)*)),+),)?
            $(with_decls: ($($impl3:ident!($($shared3:tt)*)),*),)?
            $(all_decls: ($($all_decls:ident),+),)?
        },
        ($decl:ident,  $name:expr, $($args:tt),*),
        $($rest:tt)*
    ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $decl;
        $(
            $impl!(
                lt: $lt, data: $data,
                decl: $decl, name: $name,
                shared: ($($shared)*),
                args: $args
            );
        )*
        $($(
            $impl2!(
                lt: $lt, data: $data,
                decl: $decl, name: $name,
                shared: ($($shared2)*),
                args: ()
            );
        )+)?
        impl_param!(
            {
                lt: $lt, data: $data,
                with_args: ($ ($impl!($ ($shared)*)),*),
                $(no_args: ($($impl2!($($shared2)*)),*),)?
                $(with_decls: ($($impl3!($($shared3)*)),*),)?
                all_decls: ($($($all_decls),+,)? $decl ),
            },
            $($rest)*
        );
    };
);

#[macro_export]
macro_rules! impl_base(
    {
        lt: ($($lt:tt)?), data: $data:ty,
        decl: $decl:ident, name: $name:expr,
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
        lt: ($($lt:tt)?), data: $data:ty,
        decl: $decl:ident, name: $name:expr,
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
        lt: ($($lt:tt)?), data: $data:ty,
        decl: $decl:ident, name: $name:expr,
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
        lt: ($($lt:tt)?), data: $data:ty,
        decl: $decl:ident, name: $name:expr,
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

#[macro_export]
macro_rules! impl_fn(
    {
        lt: ($($lt:tt)?), data: $data:ty,
        decl: $decl:ident, name: $name:expr,
        shared: ($fun:path),
        args: ($($args:expr),*)
    } => {
        impl $decl {
            #[forbid(dead_code)]
            pub fn make_formula$(<$lt>)?(&$($lt)? self) -> impl formula::prelude::FormulaCompat<$data, i32> {
                $fun(self, $($args),*)
            }
        }
    }
);

#[macro_export]
macro_rules! impl_boxed_formulas(
    {
        lt: ($($lt:tt)?), data: $data:ty,
        shared: ($struct_name:ident::$fun_name:ident),
        args: ($($args:ident),*)
    } => {
        /*impl $struct_name {
            #[deny(dead_code)]
            pub(crate) fn $fun_name$(<$lt>)?() -> Vec<(
                <$data as $crate::param_types::ParamGet>::Index,
                formula::prelude::tools::Op<$data, i32, formula::prelude::tools::BoxedFormula<$data, i32>>
            )>
                $(where $lt: 'static)?
            {
                vec![$(
                    (
                        <$args as $crate::param_types::HasParamBase<$data>>::INDEX,
                        formula::prelude::tools::boxed($args.calc())
                    )
                ),*]
            }
        }*/
        /*impl$(<$lt>)? $crate::param_types::HasBoxedFormulas<$data> for $struct_name
        {
            fn make_boxed_formulas(&self) -> Vec<(
                <$data as $crate::param_types::ParamGet>::Index,
                formula::prelude::tools::BoxedFormula<$data, i32>
            )> {
                vec![$(
                    (
                        <$args as $crate::param_types::HasParamBase<$data>>::INDEX,
                        formula::prelude::tools::boxed($args.calc().0)
                    )
                ),*]
            }
        }*/
    }
);
