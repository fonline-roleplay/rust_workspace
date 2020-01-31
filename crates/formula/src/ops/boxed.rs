use super::{
    tag::{op, Op},
    Context, Descriptor, Formula, Precedence,
};
use std::fmt::{self, Debug, Formatter};

pub trait DynFormula<I, O>: Send + Sync {
    fn debug(&self, fmt: &mut Formatter<'_>) -> fmt::Result;
    fn description(&self, desc: &mut Context, input: Option<I>) -> fmt::Result;
    fn compute(&self, input: I) -> O;
    //fn clone(&self) -> BoxedFormula<I, O>;
}
impl<I, O, F: 'static + Send + Sync + Formula<I, O>> DynFormula<I, O> for F {
    fn debug(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, fmt)
    }
    fn description(&self, desc: &mut Context, input: Option<I>) -> fmt::Result {
        self.description(desc, input)
    }
    fn compute(&self, input: I) -> O {
        self.compute(input)
    }
    /*fn clone(&self) -> BoxedFormula<I, O> {
        let cloned = Clone::clone(self);
        BoxedFormula(Box::new(cloned))
    }*/
}

pub struct BoxedFormula<I, O>(Box<dyn DynFormula<I, O>>);

pub fn boxed<I, O, F: 'static + DynFormula<I, O>>(formula: F) -> BoxedFormula<I, O> {
    BoxedFormula(Box::new(formula))
}
impl<I, O> Clone for BoxedFormula<I, O> {
    fn clone(&self) -> Self {
        //DynFormula::clone(&*self.0)
        unimplemented!()
    }
}
impl<I, O> Debug for BoxedFormula<I, O> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.debug(f)
    }
}
impl<I, O> Formula<I, O> for BoxedFormula<I, O> {
    const PRECEDENCE: Precedence = Precedence::Bound;
    fn compute(&self, input: I) -> O {
        self.0.compute(input)
    }
    fn description<D: Descriptor>(&self, desc: &mut D, input: Option<I>) -> fmt::Result {
        self.0.description(desc.context(), input)
    }
}
