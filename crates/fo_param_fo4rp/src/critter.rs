use crate::raw_param::RawParam;
use fo_engine_types_bindgen::generated::r357::server::item::Item;
use fo_param::param_types::ParamGet;
use formula::prelude::{Formula, FormulaCompat, FormulaData};

#[derive(Clone, Copy, Debug)]
pub struct Time {
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
}
impl Time {
    pub fn is_night(&self) -> bool {
        let full_minute = self.hour * 60 + self.minute;
        full_minute <= 6 * 60 || 18 * 60 < full_minute
    }
}

pub struct Critter<'a> {
    pub param: &'a [i32; RawParam::PARAMS_COUNT as usize],
    pub time: Time,
    pub full_second: u32,
    armor: Option<Item>,
}

impl<'a> Critter<'a> {
    pub fn new(param: &'a [i32; RawParam::PARAMS_COUNT as usize], time: Time) -> Self {
        Critter {
            param,
            time,
            full_second: 0,
            armor: None,
        }
    }
    pub fn armor(&self) -> Option<&Item> {
        self.armor.as_ref()
    }
}

impl<'a> ParamGet for &'a Critter<'a> {
    type Index = RawParam;
    fn get_param(&self, param: RawParam) -> i32 {
        self.param[param as usize]
    }
}

impl<'a> FormulaData for &'a Critter<'a> {}

pub trait CrOp<'a>: Send + Sync + FormulaCompat<&'a Critter<'a>, i32> {}
impl<'a, T: Send + Sync + FormulaCompat<&'a Critter<'a>, i32>> CrOp<'a> for T {}
