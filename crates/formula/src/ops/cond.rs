use super::{
    biop::{Biop, BiopOutput, TriopOutput},
    format_braces,
    tag::{op, Op},
    Descriptor, Formula, Precedence,
};
use std::{cmp::PartialOrd, fmt};

#[derive(Debug, Clone, Copy)]
pub struct IfElse<C, A, B>(C, A, B);

impl<I: Copy, C: Formula<I, bool>, O, A: Formula<I, O>, B: Formula<I, O>> Formula<I, O>
    for IfElse<C, A, B>
{
    const PRECEDENCE: Precedence = Precedence::Bound;
    fn compute(&self, input: I) -> O {
        let first = self.0.compute(input);
        if first {
            self.1.compute(input)
        } else {
            self.2.compute(input)
        }
    }
    fn description<D: Descriptor>(&self, desc: &mut D, input: Option<I>) -> fmt::Result {
        use std::fmt::Write;
        desc.buffer().write_str("ЕСЛИ ")?;
        Self::braces(&self.0, desc, input)?;
        desc.buffer().write_str(" ТО ")?;
        //Self::biop(&self.1, &self.2, " ИНАЧЕ ", desc, input)
        format_braces(Precedence::_ComplexStart, &self.1, desc, input)?;
        desc.buffer().push_str(" ИНАЧЕ ");
        format_braces(Precedence::_ComplexStart, &self.2, desc, input)
    }
}

pub fn if_else<
    IA: Copy,
    IB: Copy,
    IC: Copy,
    C: Formula<IC, bool>,
    O,
    A: Formula<IA, O>,
    B: Formula<IB, O>,
>(
    condition: Op<IC, bool, C>,
    first: Op<IA, O, A>,
    second: Op<IB, O, B>,
) -> Op<TriopOutput<IC, IA, IB>, O, IfElse<C, A, B>>
where
    (IC, IA, IB): Biop,
    IfElse<C, A, B>: Formula<TriopOutput<IC, IA, IB>, O>,
{
    op(IfElse(condition.0, first.0, second.0))
}

macro_rules! min_max (
    ($big:ident, $small:ident, $text:expr) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $big<A, B>(A, B);

        impl<I: Copy, O: Ord, A: Formula<I, O>, B: Formula<I, O>> Formula<I, O> for $big<A, B> {
            const PRECEDENCE: Precedence = Precedence::Bound;
            fn compute(&self, input: I) -> O {
                let a = self.0.compute(input);
                let b = self.1.compute(input);
                std::cmp::$small(a, b)
            }
            fn description<D: Descriptor>(&self, desc: &mut D, input: Option<I>) -> fmt::Result {
                use std::fmt::Write;
                desc.buffer().write_str($text)?;
                Self::biop(&self.0, &self.1, " И ", desc, input)
            }
        }

        pub fn $small<IA: Copy, IB: Copy, O: PartialOrd, A: Formula<IA, O>, B: Formula<IB, O>>(
            a: Op<IA, O, A>,
            b: Op<IB, O, B>,
        ) -> Op<BiopOutput<IA, IB>, O, $big<A, B>>
        where
            (IA, IB): Biop,
            $big<A, B>: Formula<BiopOutput<IA, IB>, O>,
        {
            op($big(a.0, b.0))
        }
});

min_max!(Max, max, "НАИБОЛЬШЕЕ СРЕДИ ");
min_max!(Min, min, "НАИМЕНЬШЕЕ СРЕДИ ");

#[derive(Debug, Clone, Copy)]
pub struct Clamp<C, A, B>(C, A, B);

impl<I: Copy, C: Formula<I, O>, O: PartialOrd, A: Formula<I, O>, B: Formula<I, O>> Formula<I, O>
    for Clamp<C, A, B>
{
    const PRECEDENCE: Precedence = Precedence::Bound;
    fn compute(&self, input: I) -> O {
        let value = self.0.compute(input);
        let min = self.1.compute(input);
        if value < min {
            min
        } else {
            let max = self.2.compute(input);
            if value > max {
                max
            } else {
                value
            }
        }
    }
    fn description<D: Descriptor>(&self, desc: &mut D, input: Option<I>) -> fmt::Result {
        use std::fmt::Write;
        desc.buffer().write_str("В РАМКАХ (")?;
        Self::biop(&self.1, &self.2, " < X < ", desc, input)?;
        desc.buffer().write_str(") ОГРАНИЧИТЬ ")?;
        Self::braces(&self.0, desc, input)
    }
}

pub fn clamp<
    IA: Copy,
    IB: Copy,
    IC: Copy,
    C: Formula<IC, O>,
    O: PartialOrd,
    A: Formula<IA, O>,
    B: Formula<IB, O>,
>(
    value: Op<IC, O, C>,
    min: Op<IA, O, A>,
    max: Op<IB, O, B>,
) -> Op<TriopOutput<IC, IA, IB>, O, Clamp<C, A, B>>
where
    (IC, IA, IB): Biop,
    Clamp<C, A, B>: Formula<TriopOutput<IC, IA, IB>, O>,
{
    op(Clamp(value.0, min.0, max.0))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::{int, tools::unop};

    #[test]
    fn test_if_else() {
        let formula_true = if_else(unop(true), int(10), int(20));
        assert_eq!(10, formula_true.compute(()));
        let formula_false = if_else(unop(false), int(10), int(20));
        assert_eq!(20, formula_false.compute(()));
    }

    #[test]
    fn test_clamp() {
        let formula = clamp(int(5), int(-5), int(10));
        assert_eq!(5, formula.compute(()));
        let formula = clamp(int(15), int(-5), int(10));
        assert_eq!(10, formula.compute(()));
        let formula = clamp(int(-10), int(-5), int(10));
        assert_eq!(-5, formula.compute(()));
    }

    #[test]
    fn test_min_max() {
        let formula = max(int(5), int(-5));
        assert_eq!(5, formula.compute(()));
        let formula = max(int(-5), int(5));
        assert_eq!(5, formula.compute(()));

        let formula = min(int(5), int(-5));
        assert_eq!(-5, formula.compute(()));
        let formula = min(int(-5), int(5));
        assert_eq!(-5, formula.compute(()));
    }
}
