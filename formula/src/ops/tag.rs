use super::{Context, Descriptor, Formula, FormulaData, IntoLineResult, LineResult, Precedence};
use std::{
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
};

#[derive(Copy)]
pub struct Op<I, O, F: Formula<I, O>>(
    pub(super) F,
    pub(super) PhantomData<I>,
    pub(super) PhantomData<O>,
);

impl<I, O, F: Formula<I, O>> Clone for Op<I, O, F> {
    fn clone(&self) -> Self {
        op(self.0.clone())
    }
}
pub type UnOp<O, F> = Op<(), O, F>;
pub fn op<I, O, F: Formula<I, O>>(f: F) -> Op<I, O, F> {
    Op(f, PhantomData, PhantomData)
}
pub fn unop<O, F: Formula<(), O>>(f: F) -> UnOp<O, F> {
    Op(f, PhantomData, PhantomData)
}
impl<I: Copy, O: IntoLineResult, F: Formula<I, O>> Display for Op<I, O, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let info = self.full_info("", None)?;
        f.write_str(&info)
    }
}
impl<I, O, F: Formula<I, O>> Op<I, O, F> {
    pub fn full_info(&self, name: &'static str, input: Option<I>) -> Result<String, fmt::Error>
    where
        O: IntoLineResult,
        I: Copy,
    {
        use fmt::Write;
        let mut desc = Context::new(name);
        self.description(&mut desc, input)?;
        if let Some(input) = input {
            desc.set_value(&self.0, input);
        }
        let mut info = String::with_capacity(128);
        for (prefix, line, val) in desc.backlog.iter() {
            let space = if !prefix.is_empty() {
                writeln!(info, "{}: ", prefix)?;
                "  "
            } else {
                ""
            };
            if let LineResult::NoData = val {
                writeln!(info, "{}{}", space, line)
            } else {
                writeln!(info, "{}{} = {}", space, line, val)
            }?;
        }
        let mut last_param_order = None;
        for ((order, _index, key), value) in &desc.args {
            if Some(order) != last_param_order {
                writeln!(info, "{}:", order)?;
                last_param_order = Some(order);
            }
            if let LineResult::NoData = value {
                writeln!(info, "  {}", key)
            } else {
                writeln!(info, "  {} = {}", key, value)
            }?;
        }
        Ok(info)
    }
}
impl<I, O, F: Formula<I, O>> Debug for Op<I, O, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Op").field(&self.0).finish()
    }
}
/*
pub trait OpFormula<D>: Formula<D> {}

impl<D, F: Formula<D>> OpFormula<D> for Op<D, F> {}
*/
impl<I, O, F: Formula<I, O>> Formula<I, O> for Op<I, O, F> {
    const PRECEDENCE: Precedence = F::PRECEDENCE;
    fn compute(&self, input: I) -> O {
        self.0.compute(input)
    }
    fn description<D: Descriptor>(&self, desc: &mut D, input: Option<I>) -> fmt::Result {
        self.0.description(desc, input)
    }
}

pub trait FormulaCompat<I, O> {
    type Inner: Formula<I, O>;
    fn compat(self) -> Op<I, O, Self::Inner>;
}
impl<I, O, F: Formula<(), O> + Formula<I, O>> FormulaCompat<I, O> for Op<(), O, F> {
    type Inner = F;
    fn compat(self) -> Op<I, O, F> {
        op(self.0)
    }
}
impl<I: FormulaData, O, F: Formula<I, O>> FormulaCompat<I, O> for Op<I, O, F> {
    type Inner = F;
    fn compat(self) -> Op<I, O, F> {
        op(self.0)
    }
}
