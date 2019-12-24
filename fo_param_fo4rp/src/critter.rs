use crate::raw_param::RawParam;
use fo_param::param_types::ParamGet;
use formula::prelude::{Formula, FormulaCompat, FormulaData};

pub struct Critter<'a> {
    pub param: &'a [i32; RawParam::PARAMS_COUNT as usize],
}

impl<'a> Critter<'a> {
    pub fn new(param: &'a [i32; RawParam::PARAMS_COUNT as usize]) -> Self {
        Critter { param }
    }
}

impl<'a> ParamGet for &'a Critter<'a> {
    type Index = RawParam;
    fn get_param(&self, param: RawParam) -> i32 {
        self.param[param as usize]
    }
}

impl<'a> FormulaData for &'a Critter<'a> {}

pub trait CrOp<'a>: FormulaCompat<&'a Critter<'a>, i32> {}
impl<'a, T: FormulaCompat<&'a Critter<'a>, i32>> CrOp<'a> for T {}
