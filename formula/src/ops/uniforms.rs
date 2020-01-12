use super::{
    tag::{unop, Op, UnOp},
    ArgSortOrder, Context, Descriptor, Formula, IntoLineResult,
};
use std::{fmt, marker::PhantomData};
// Число-константа со значением и именем
#[derive(Debug, Clone, Copy)]
pub struct Invar<O> {
    value: O,
    name: &'static str,
}
// Удобный конструктор для Invar
pub const fn invar(value: i32, name: &'static str) -> UnOp<i32, Invar<i32>> {
    Op(Invar { value, name }, PhantomData, PhantomData)
}
// Макрос для удобного объявления констант типа Invar
#[macro_export(local_inner_macros)]
macro_rules! invar (($const_name:ident, $value:expr, $name:expr) => {
    const $const_name: $crate::prelude::tools::UnOp<i32, $crate::prelude::tools::Invar<i32>> = $crate::prelude::invar($value, $name);
});

// имплементация трейта Formula - для просчета результата формулы
impl<I, O: IntoLineResult + Copy + fmt::Debug> Formula<I, O> for Invar<O> {
    fn compute(&self, _input: I) -> O {
        // это константа, считать нечего, возвращаем свое постоянное значение
        self.value
    }
    fn description<D: Descriptor>(&self, desc: &mut D, input: Option<I>) -> fmt::Result {
        desc.buffer().push_str(self.name);
        desc.compute_param(self, (), ArgSortOrder::Invar, 0, self.name);
        Ok(())
    }
}

// Не уверен нужно ли, когда уже есть Const
impl<I> Formula<I, i32> for i32 {
    fn compute(&self, _input: I) -> i32 {
        *self
    }
    fn description<D: Descriptor>(&self, desc: &mut D, _input: Option<I>) -> fmt::Result {
        use std::fmt::Write;
        write!(desc.buffer(), "{}", self)
    }
}
pub fn int(int: i32) -> UnOp<i32, i32> {
    unop(int)
}

// Не уверен нужно ли, когда уже есть Const
impl<I> Formula<I, bool> for bool {
    fn compute(&self, _input: I) -> bool {
        *self
    }
    fn description<D: Descriptor>(&self, desc: &mut D, _input: Option<I>) -> fmt::Result {
        use std::fmt::Write;
        write!(desc.buffer(), "{}", if *self { "Да" } else { "Нет" })
    }
}
