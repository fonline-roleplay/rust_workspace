use derivative::Derivative;
use formula::prelude::{tools::*, *};
use std::{
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
};

pub trait ParamGet {
    type Index: Into<u16>;
    fn get_param(&self, param: Self::Index) -> i32;
}

#[derive(Derivative)]
#[derivative(Debug(bound = "P: Debug"), Clone(bound = "P: Clone"))]
pub struct ParamBase<I: ParamGet, P: HasParamBase<I>>(P, PhantomData<I>);
impl<I: ParamGet, P: HasParamBase<I>> Formula<I, i32> for ParamBase<I, P> {
    fn compute(&self, input: I) -> i32 {
        input.get_param(P::INDEX)
    }
    fn description<D: Descriptor>(&self, desc: &mut D, input: Option<I>) -> fmt::Result {
        let name = P::NAME;
        desc.buffer().push_str(name);
        if let Some(input) = input {
            desc.compute_param(self, input, ArgSortOrder::Stat, P::INDEX.into(), name);
        }
        Ok(())
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = "P: Debug"), Clone(bound = "P: Clone"))]
pub struct ParamExt<I: ParamGet, P: HasParamExt<I>>(P, PhantomData<I>);
impl<I: ParamGet, P: HasParamExt<I>> Formula<I, i32> for ParamExt<I, P> {
    fn compute(&self, input: I) -> i32 {
        input.get_param(P::INDEX_EXT)
    }
    fn description<D: Descriptor>(&self, desc: &mut D, input: Option<I>) -> fmt::Result {
        let name = P::NAME_EXT;
        desc.buffer().push_str(name);
        if let Some(input) = input {
            desc.compute_param(self, input, ArgSortOrder::Stat, P::INDEX_EXT.into(), name);
        }
        Ok(())
    }
}

pub trait HasParamBase<I: ParamGet>: Debug + Copy + Sized {
    const INDEX: I::Index;
    const NAME: &'static str;
    fn base(self) -> Op<I, i32, ParamBase<I, Self>> {
        op(ParamBase(self, PhantomData))
    }
}

pub trait HasParamExt<I: ParamGet>: Debug + Copy + Sized {
    const INDEX_EXT: I::Index;
    const NAME_EXT: &'static str;
    fn ext(self) -> Op<I, i32, ParamExt<I, Self>> {
        op(ParamExt(self, PhantomData))
    }
}

pub trait HasParamSum<I: ParamGet>: HasParamBase<I> + HasParamExt<I> {
    type Formula: Formula<I, i32>;
    fn sum(self) -> Op<I, i32, Self::Formula>;
}

impl<I: ParamGet + FormulaData + Copy, P: HasParamBase<I> + HasParamExt<I>> HasParamSum<I> for P {
    type Formula = Add<ParamBase<I, P>, ParamExt<I, P>>;
    fn sum(self) -> tools::Op<I, i32, Self::Formula> {
        self.base() + self.ext()
    }
}

pub trait HasParamPresent<I: ParamGet + FormulaData + Copy>: HasParamBase<I> {
    type Formula: Formula<I, i32> + Formula<(), i32>;
    const NOT_PRESENT: Op<(), i32, Self::Formula>;
    const CUT: &'static str;
    fn present(self) -> Op<I, bool, Cut<NotEqual<ParamBase<I, Self>, Self::Formula, i32>>> {
        Self::CUT.part(not_equal(self.base(), Self::NOT_PRESENT))
    }
}
