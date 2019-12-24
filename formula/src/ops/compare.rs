use super::{
    biop::{Biop, BiopOutput, TriopOutput},
    tag::{op, Op},
    Descriptor, Formula, Precedence,
};
use std::{
    cmp::{PartialEq, PartialOrd},
    fmt,
    marker::PhantomData,
};
//use derivative::Derivative;

macro_rules! impl_compare (($big:ident, $small:ident, $std_trait:ident, $operation:ident, $sep:expr) => {
    /*#[derive(Derivative)]
    #[derivative(
        Debug(bound = "A: fmt::Debug, B: fmt::Debug"),
        Clone(bound = "A: Clone, B: Clone")
    )]*/
    #[derive(Debug, Clone)]
    pub struct $big<A, B, T>(A, B, PhantomData<T>);

    // remove Debug + Clone bond later
    impl<I: Copy, T: std::fmt::Debug + Clone + $std_trait, A: Formula<I, T>, B: Formula<I, T>> Formula<I, bool>
        for $big<A, B, T>
    {
        const PRECEDENCE: Precedence = Precedence::Bound;
        fn compute(&self, input: I) -> bool {
            let first = self.0.compute(input);
            let second = self.1.compute(input);
            first.$operation(&second)
        }
        fn description<D: Descriptor>(&self, desc: &mut D, input: Option<I>) -> fmt::Result {
            Self::biop(&self.0, &self.1, $sep, desc, input)
        }
    }

    pub fn $small<IA: Copy, IB: Copy, T: PartialEq, A: Formula<IA, T>, B: Formula<IB, T>>(
        first: Op<IA, T, A>,
        second: Op<IB, T, B>,
    ) -> Op<BiopOutput<IA, IB>, bool, $big<A, B, T>>
    where
        (IA, IB): Biop,
        $big<A, B, T>: Formula<BiopOutput<IA, IB>, bool>,
    {
        op($big(first.0, second.0, PhantomData))
    }
});

#[cfg_attr(rustfmt, rustfmt_skip)]
pub mod inner {
    use super::*;
    impl_compare!(Equal, equal, PartialEq, eq, " РАВНО ");
    impl_compare!(NotEqual, not_equal, PartialEq, ne, " НЕ РАВНО ");
    impl_compare!(GreaterThan, greater_than, PartialOrd, gt, " БОЛЬШЕ ЧЕМ ");
    impl_compare!(GreaterOrEqual, greater_or_equal, PartialOrd, ge, " БОЛЬШЕ ИЛИ РАВНО ");
    impl_compare!(LessThan, less_than, PartialOrd, lt, " МЕНЬШЕ ");
    impl_compare!(LessOrEqual, less_or_equal, PartialOrd, le, " МЕНЬШЕ ИЛИ РАВНО ");
}
pub use inner::*;

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::{int, tools::unop};

    #[test]
    fn test_equal() {
        let formula_true = equal(int(10), int(10));
        assert_eq!(formula_true.compute(()), true);
        let formula_false = equal(int(10), int(20));
        assert_eq!(formula_false.compute(()), false);
    }
}
