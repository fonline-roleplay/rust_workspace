mod display;
mod lexer;
mod logic_node;

use std::collections::BTreeMap;

#[derive(Debug, Default)]
struct RecipeBook<'a> {
    recipes: BTreeMap<u32, Recipe<'a>>,
}

#[derive(PartialEq, Debug)]
struct Recipe<'a> {
    name: &'a str,
    description: Option<&'a str>,
    params_to_see: Option<LogicChain<'a>>,
    params_to_craft: Option<LogicChain<'a>>,
    ingredients: LogicChain<'a>,
    tools: Option<LogicChain<'a>>,
    output: LogicChain<'a>,
    side_effect: SideEffect<'a>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct KeyValue<'a> {
    key: &'a str,
    value: u32,
}

#[derive(PartialEq, Debug)]
struct LogicChain<'a> {
    first: KeyValue<'a>,
    rest: Vec<(Logical, KeyValue<'a>)>,
}

#[derive(PartialEq, Debug)]
enum Logical {
    And,
    Or,
}

#[derive(PartialEq, Debug)]
enum SideEffect<'a> {
    Script { module: &'a str, function: &'a str },
    Experience(u32),
}
