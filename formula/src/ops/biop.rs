use super::{
    tag::{op, Op},
    Context, Descriptor, Formula, FormulaData, Precedence,
};
use std::fmt;

pub trait Biop {
    type Output;
}
pub type BiopOutput<DA, DB> = <(DA, DB) as Biop>::Output;
pub type TriopOutput<DA, DB, DC> = <(DA, DB, DC) as Biop>::Output;

impl<A: FormulaData> Biop for (A, ()) {
    type Output = A;
}
impl<B: FormulaData> Biop for ((), B) {
    type Output = B;
}
impl<A: FormulaData> Biop for (A, A) {
    type Output = A;
}
impl Biop for ((), ()) {
    type Output = ();
}

impl<A: FormulaData> Biop for (A, (), ()) {
    type Output = A;
}
impl<A: FormulaData> Biop for (A, A, ()) {
    type Output = A;
}
impl<A: FormulaData> Biop for (A, A, A) {
    type Output = A;
}
impl<A: FormulaData> Biop for ((), A, A) {
    type Output = A;
}
impl<A: FormulaData> Biop for ((), (), A) {
    type Output = A;
}
impl<A: FormulaData> Biop for (A, (), A) {
    type Output = A;
}
impl<A: FormulaData> Biop for ((), A, ()) {
    type Output = A;
}
impl Biop for ((), (), ()) {
    type Output = ();
}

macro_rules! impl_biop(
    ($name:ident, $precedence:expr, $sep:expr, $compute:ident, $compute_func:ident) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name<A, B>(A, B);
        impl<I: Copy, O: $compute, A: Formula<I, O>, B: Formula<I, O>> Formula<I, O> for $name<A, B> {
            const PRECEDENCE: Precedence = $precedence;
            fn compute(&self, input: I) -> O {
                let a = || self.0.compute(input);
                let b = || self.1.compute(input);
                $compute::$compute_func(a, b)
            }
            fn description<D: Descriptor>(&self, desc: &mut D, input: Option<I>) -> fmt::Result {
                Self::biop(&self.0, &self.1, $sep, desc, input)
            }
        }
    };
    ($name:ident, $big:ident, $small:ident, $precedence:expr, $sep:expr, $compute:ident, $compute_func:ident) => {
        impl_biop!($name, $precedence, $sep, $compute, $compute_func);
        impl<IA, IB, O: $compute, A: Formula<IA, O>, B: Formula<IB, O>> std::ops::$big<Op<IB, O, B>> for Op<IA, O, A>
        where
            (IA, IB): Biop,
            $name<A, B>: Formula<BiopOutput<IA, IB>, O>,
        {
            type Output = Op<BiopOutput<IA, IB>, O, $name<A, B>>;
            fn $small(self, rhs: Op<IB, O, B>) -> Self::Output {
                op($name(self.0, rhs.0))
            }
        }
    }
);

pub trait Saturating: Sized {
    fn compute_add(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self;
    fn compute_sub(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self;
    fn compute_mul(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self;
    fn compute_div(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self;
}
impl Saturating for i32 {
    fn compute_add(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self {
        i32::saturating_add(a(), b())
    }
    fn compute_sub(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self {
        i32::saturating_sub(a(), b())
    }
    fn compute_mul(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self {
        i32::saturating_mul(a(), b())
    }
    fn compute_div(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self {
        i32::checked_div(a(), b()).expect("Division by zero or overflowing")
    }
}

// new binary operator with overloading of operator
impl_biop!(
    Add, //name of new combinator
    Add, //name of trait from std::ops
    add, //method of trait from std::ops
    Precedence::Add,
    " + ",       //separator
    Saturating,  //trait to bound output type
    compute_add  //method of trait to bound output type
);

impl_biop!(
    Sub,
    Sub,
    sub,
    Precedence::Add,
    " - ",
    Saturating,
    compute_sub
);

impl_biop!(
    Mul,
    Mul,
    mul,
    Precedence::Mul,
    " x ",
    Saturating,
    compute_mul
);

impl_biop!(
    Div,
    Div,
    div,
    Precedence::Mul,
    " / ",
    Saturating,
    compute_div
);

pub trait SaturatingPow: Sized {
    fn saturating_pow(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self;
}
impl SaturatingPow for i32 {
    fn saturating_pow(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self {
        use std::convert::TryInto;
        match b().try_into() {
            Ok(exp) => i32::saturating_pow(a(), exp),
            Err(_) => 0,
        }
    }
}

// new binary operator without overloading of operator
impl_biop!(
    Pow, //name of new combinator
    Precedence::Pow,
    "^",            //separator
    SaturatingPow,  //trait to bound output type
    saturating_pow  //method of trait to bound output type
);

pub fn pow<I: Copy, O: SaturatingPow, A: Formula<I, O>, B: Formula<I, O>>(
    base: Op<I, O, A>,
    exponent: Op<I, O, B>,
) -> Op<I, O, Pow<A, B>> {
    op(Pow(base.0, exponent.0))
}

pub trait Boolish: Sized {
    fn logical_and(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self;
    fn logical_or(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self;
}
impl Boolish for bool {
    fn logical_and(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self {
        a() && b()
    }
    fn logical_or(a: impl Fn() -> Self, b: impl Fn() -> Self) -> Self {
        a() || b()
    }
}

impl_biop!(
    And,
    BitAnd,
    bitand,
    Precedence::BitAnd,
    " И ",
    Boolish,
    logical_and
);
impl_biop!(
    Or,
    BitOr,
    bitor,
    Precedence::BitOr,
    " ИЛИ ",
    Boolish,
    logical_or
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::tools::unop;

    #[test]
    fn test_and() {
        assert_eq!(false, (unop(false) & unop(false)).compute(()));
        assert_eq!(false, (unop(true) & unop(false)).compute(()));
        assert_eq!(false, (unop(false) & unop(true)).compute(()));
        assert_eq!(true, (unop(true) & unop(true)).compute(()));
    }
    #[test]
    fn and_isnt_associative() {
        #[derive(Debug, Clone)]
        struct NeverCalled;
        impl Formula<(), bool> for NeverCalled {
            fn compute(&self, _input: ()) -> bool {
                panic!("This should not be called!");
            }
            fn description<D: Descriptor>(
                &self,
                _desc: &mut D,
                _input: Option<()>,
            ) -> std::fmt::Result {
                unimplemented!()
            }
        }

        let formula = unop(false) & unop(NeverCalled);
        assert_eq!(false, formula.compute(()));
    }
}
