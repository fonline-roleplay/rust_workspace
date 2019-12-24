use super::{
    biop::{Biop, BiopOutput, TriopOutput},
    tag::{op, Op},
    Descriptor, Formula, Precedence,
};
use std::{cmp::PartialOrd, fmt, marker::PhantomData};

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
        Self::biop(&self.1, &self.2, " ИНАЧЕ ", desc, input)
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
}
