use crate::{critter::Critter, param, raw_param::RawParam};
use fo_param::param_types::{HasBoxedFormulas, ParamGet};
use formula::prelude::{
    tools::{self, BoxedFormula, Op},
    Formula,
};
use once_cell::sync::Lazy;
/*
static FORMULAS: Lazy<Vec<Option<BoxedFormula<&Critter, i32>>>> = Lazy::new(|| {
    let mut formulas = vec![None; 1000];

    let stats = param::stat::StatFormulas.make_boxed_formulas();
    for (k, v) in stats {
        let index = k as usize;
        if let Some(old) = formulas[index].replace(v) {
            panic!("Expected 0..1000 range of Params: {}", index);
        }
    }
    formulas
});
*/

pub fn param<'a>(data: &'a Critter<'a>, index: RawParam) -> i32 {
    /*let mut formulas = vec![None; 1000];

    let stats = param::stat::StatFormulas.make_boxed_formulas();
    for (k, v) in stats {
        let index = k as usize;
        if let Some(old) = formulas[index].replace(v) {
            panic!("Expected 0..1000 range of Params: {}", index);
        }
    }

    let formula = formulas[index as usize].as_ref().unwrap();
    formula.compute(data)*/

    use crate::param::stat;
    match index {
        RawParam::ST_STRENGTH => stat::Strength.calc().compute(data),
        RawParam::ST_PERCEPTION => stat::Perception.calc().compute(data),
        RawParam::ST_ENDURANCE => stat::Endurance.calc().compute(data),
        _ => data.get_param(index),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_get_param() {
        /*use crate::critter::*;
        let mut params = [0i32; 1000];
        let time = Time {
            hour: 23,
            minute: 30,
            second: 0,
        };

        let cr = Critter::new(&params, time);
        let res = param(&cr, RawParam::ST_STRENGTH);*/
    }
}
