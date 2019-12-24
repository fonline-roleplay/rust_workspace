pub mod biop;
pub mod boxed;
pub mod compare;
pub mod cond;
pub mod uniforms;
use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
};
pub mod tag;
use tag::{op, Op};

// Трейт, который имплементируется для всех типов, которые могут участвовать в просчете формулы.
pub trait Formula<I, O>: Debug + Clone {
    // Порядок важности оператора, нужно для добавления скобок вокруг операций сложения в пользовательском выводе
    // По-умолчанию как у не оператора, а обычного числа, первостепенный
    const PRECEDENCE: Precedence = Precedence::Num;
    fn compute(&self, input: I) -> O;
    fn description<D: Descriptor>(&self, desc: &mut D, input: Option<I>) -> fmt::Result;
    // Статичный метод оборачивающий группу операторов в скобки, если это требуется при пользовательском форматировании
    fn braces<O2, A: Formula<I, O2>, D: Descriptor>(
        around: &A,
        desc: &mut D,
        input: Option<I>,
    ) -> fmt::Result {
        if Self::PRECEDENCE as u8 >= A::PRECEDENCE as u8 {
            around.description(desc, input)
        } else {
            desc.buffer().push('(');
            around.description(desc, input)?;
            desc.buffer().push(')');
            Ok(())
        }
    }
    fn biop<O2, O3, A: Formula<I, O2>, B: Formula<I, O3>, D: Descriptor>(
        a: &A,
        b: &B,
        separator: &'static str,
        desc: &mut D,
        input: Option<I>,
    ) -> fmt::Result
    where
        I: Copy,
    {
        Self::braces(a, desc, input)?;
        desc.buffer().push_str(separator);
        Self::braces(b, desc, input)
    }
}

pub trait Descriptor {
    fn local(&self, name: &'static str) -> Self;
    fn consume(&mut self, local_desc: Self);
    fn buffer(&mut self) -> &mut String;
    fn set_value<I, O: IntoLineResult, F: Formula<I, O>>(&mut self, fragment: &F, input: I);
    fn compute_param<I, O: IntoLineResult, F: Formula<I, O>>(
        &mut self,
        formula: &F,
        input: I,
        order: ArgSortOrder,
        index: u16,
        key: &'static str,
    );
    fn add_param<V: IntoLineResult>(&mut self, key: (ArgSortOrder, u16, &'static str), value: V);
}

//struct Void;
pub trait FormulaData {}
impl<I: FormulaData, O, F: Formula<(), O>> Formula<I, O> for Op<(), O, F> {
    const PRECEDENCE: Precedence = <Self as Formula<(), O>>::PRECEDENCE;
    fn compute(&self, _input: I) -> O {
        <Self as Formula<(), O>>::compute(self, ())
    }
    fn description<D: Descriptor>(&self, desc: &mut D, _input: Option<I>) -> fmt::Result {
        //<T as Formula<()>>::description(self, ctx)
        todo!()
    }
}

#[derive(Debug)]
pub enum Precedence {
    Num,
    Pow,
    Mul,
    Add,
    BitAnd,
    BitOr,
    Bound,
}

#[derive(PartialEq)]
pub enum LineResult {
    Int32(i32),
    Bool(bool),
    Other(String),
    NoData,
}
impl Display for LineResult {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use fmt::Write;
        match self {
            LineResult::Int32(val) => write!(f, "{}", val),
            LineResult::Bool(val) => f.write_str(if *val { "Да" } else { "Нет" }),
            LineResult::Other(val) => f.write_str(&val),
            LineResult::NoData => Ok(()),
        }
    }
}
pub trait IntoLineResult {
    fn into_line_result(self) -> LineResult;
}
impl IntoLineResult for i32 {
    fn into_line_result(self) -> LineResult {
        LineResult::Int32(self)
    }
}
impl IntoLineResult for bool {
    fn into_line_result(self) -> LineResult {
        LineResult::Bool(self)
    }
}
impl<O: Display> IntoLineResult for &O {
    fn into_line_result(self) -> LineResult {
        LineResult::Other(format!("{}", self))
    }
}
impl IntoLineResult for LineResult {
    fn into_line_result(self) -> LineResult {
        self
    }
}

pub struct Context {
    backlog: Vec<(&'static str, String, LineResult)>,
    args: BTreeMap<(ArgSortOrder, u16, &'static str), LineResult>,
}
impl Context {
    fn new(name: &'static str) -> Self {
        Context {
            backlog: vec![(name, String::new(), LineResult::NoData)],
            args: Default::default(),
        }
    }
}
impl Descriptor for Context {
    fn local(&self, name: &'static str) -> Self {
        Context::new(name)
    }
    fn consume(&mut self, local_desc: Self) {
        self.backlog.extend(local_desc.backlog.into_iter());
        for (key, value) in local_desc.args {
            self.add_param(key, value);
        }
    }
    fn buffer(&mut self) -> &mut String {
        &mut self.backlog[0].1
    }
    fn set_value<I, O: IntoLineResult, F: Formula<I, O>>(&mut self, fragment: &F, input: I) {
        let val = fragment.compute(input);
        self.backlog[0].2 = val.into_line_result();
    }
    fn compute_param<I, O: IntoLineResult, F: Formula<I, O>>(
        &mut self,
        formula: &F,
        input: I,
        order: ArgSortOrder,
        index: u16,
        key: &'static str,
    ) {
        let value = formula.compute(input);
        self.add_param((order, index, key), value);
    }
    fn add_param<V: IntoLineResult>(&mut self, key: (ArgSortOrder, u16, &'static str), value: V) {
        let value = value.into_line_result();
        use std::collections::btree_map::Entry;
        match self.args.entry(key) {
            Entry::Occupied(mut occupied) => {
                if occupied.get() != &value {
                    panic!("Collision between param names");
                } else {
                    occupied.insert(value);
                }
            }
            Entry::Vacant(vacant) => {
                vacant.insert(value);
            }
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub enum ArgSortOrder {
    Invar,
    Stat,
}
impl Display for ArgSortOrder {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use fmt::Write;
        use ArgSortOrder::*;
        f.write_str(match self {
            Invar => "Константы",
            Stat => "Статы",
        })
    }
}

#[derive(Debug, Clone)]
pub struct Cut<F> {
    name: &'static str,
    fragment: F,
}
impl<I: Copy, O: IntoLineResult, F: Formula<I, O>> Formula<I, O> for Cut<F> {
    fn compute(&self, input: I) -> O {
        self.fragment.compute(input)
    }
    fn description<D: Descriptor>(&self, desc: &mut D, input: Option<I>) -> fmt::Result {
        let mut local_desc = desc.local(self.name);
        self.fragment.description(&mut local_desc, input)?;
        desc.buffer().push_str(self.name);
        if let Some(input) = input {
            local_desc.set_value(&self.fragment, input);
        }
        desc.consume(local_desc);
        Ok(())
    }
}

pub fn cut<I: Copy, O: IntoLineResult, F: Formula<I, O>>(
    name: &'static str,
    Op(fragment, _, _): Op<I, O, F>,
) -> Op<I, O, Cut<F>> {
    op(Cut { name, fragment })
}

pub trait PartFormula {
    fn part<I: Copy, O: IntoLineResult, F: Formula<I, O>>(
        &self,
        fragment: Op<I, O, F>,
    ) -> Op<I, O, Cut<F>>;
}
impl PartFormula for &'static str {
    fn part<I: Copy, O: IntoLineResult, F: Formula<I, O>>(
        &self,
        fragment: Op<I, O, F>,
    ) -> Op<I, O, Cut<F>> {
        cut(self, fragment)
    }
}
