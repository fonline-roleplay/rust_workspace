use super::{KeyValue, LogicChain, Logical, Recipe, SideEffect};
use nom_prelude::{complete::*, *};

fn recipe<'a, E: ParseError<&'a str>>(ref mut i: &'a str) -> IResult<&'a str, Recipe<'a>, E> {
    let entry = Recipe {
        name: apply(i, terminated(not_a_dog, a_dog))?,
        description: apply(i, terminated(opt(not_a_dog), a_dog))?,
        params_to_see: apply(i, optional_dog_logic_chain)?,
        params_to_craft: apply(i, optional_dog_logic_chain)?,
        ingredients: apply(i, dog_logic_chain)?,
        tools: apply(i, optional_dog_logic_chain)?,
        output: apply(i, dog_logic_chain)?,
        side_effect: apply(i, side_effect)?,
    };
    Ok((i, entry))
}

fn logic_chain<'a, E: ParseError<&'a str>>(
    ref mut i: &'a str,
) -> IResult<&'a str, LogicChain<'a>, E> {
    let chain = LogicChain {
        first: apply(i, key_value)?,
        rest: apply(i, many0(pair(logical, key_value)))?,
    };
    Ok((i, chain))
}

fn key_value<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, KeyValue, E> {
    map(
        space0_delimited(separated_pair(word, space1, unsigned_number)),
        |(key, value)| KeyValue { key, value },
    )(i)
}

fn logical<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Logical, E> {
    space0_delimited(alt((
        map(char('&'), |_| Logical::And),
        map(char('|'), |_| Logical::Or),
    )))(i)
}

fn side_effect<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, SideEffect<'a>, E> {
    alt((
        map(
            preceded(
                pair(tag("script"), space1),
                separated_pair(not_a_dog, a_dog, word),
            ),
            |(module, function)| SideEffect::Script { module, function },
        ),
        map(preceded(pair(tag("exp"), space1), unsigned_number), |exp| {
            SideEffect::Experience(exp)
        }),
    ))(i)
}

fn not_a_dog<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    take_till1(|ch| ch == '@')(i)
}

fn a_dog<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, char, E> {
    char('@')(i)
}

fn dog_logic_chain<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, LogicChain<'a>, E> {
    terminated(map_parser(not_a_dog, logic_chain), a_dog)(i)
}

fn optional_dog_logic_chain<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Option<LogicChain<'a>>, E> {
    terminated(opt(map_parser(not_a_dog, logic_chain)), a_dog)(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex<'a, T: 'a, F>(fun: F, str: &'a str) -> T
    where
        F: FnOnce(&'a str) -> IResult<&'a str, T, nom::error::VerboseError<&'a str>>,
    {
        fun(str).unwrap().1
    }

    #[test]
    fn test_side_effect_script() {
        const SAMPLE: &str = "script fix_boy@fix_Tribal";
        let correct = SideEffect::Script {
            module: "fix_boy",
            function: "fix_Tribal",
        };
        assert_eq!(correct, lex(side_effect, SAMPLE));
    }

    #[test]
    fn test_side_effect_exp() {
        const SAMPLE: &str = "exp 100";
        let correct = SideEffect::Experience(100);
        assert_eq!(correct, lex(side_effect, SAMPLE));
    }

    #[test]
    fn test_zaplatka() {
        const SAMPLE: &str = "\
            PID_ZAPLATKA_CRAFT_BASIC@Разделитель раздела для создания простейших вещей.\
            @@@PID_ZAPLATKA_CRAFT_BASIC 1@@PID_ZAPLATKA_CRAFT_BASIC 1@script fix_boy@fix_Tribal\
        ";
        let correct = Recipe {
            name: "PID_ZAPLATKA_CRAFT_BASIC",
            description: Some("Разделитель раздела для создания простейших вещей."),
            params_to_see: None,
            params_to_craft: None,
            ingredients: LogicChain {
                first: KeyValue {
                    key: "PID_ZAPLATKA_CRAFT_BASIC",
                    value: 1,
                },
                rest: vec![],
            },
            tools: None,
            output: LogicChain {
                first: KeyValue {
                    key: "PID_ZAPLATKA_CRAFT_BASIC",
                    value: 1,
                },
                rest: vec![],
            },
            side_effect: SideEffect::Script {
                module: "fix_boy",
                function: "fix_Tribal",
            },
        };
        assert_eq!(correct, lex(recipe, SAMPLE));
    }

    #[test]
    fn test_jet() {
        const SAMPLE: &str = "\
            PID_EMPTY_JET@Пустая банка для Джета. Расплавьте пластиковую бутылку и полученную жидкость влейте в форму, \
            после чего из различного мусора соберите простейший клапан и закрепите на еще горячем пластике.\
            @@SK_REPAIR 100|SK_SCIENCE 100@PID_BOTTLE_EMPTY 5&PID_CRAFT_L_LINT 5&PID_CRAFT_M_JUNK 1@PID_KNIFE 1&PID_LIGHTER 1@PID_EMPTY_JET 5\
            @script fix_boy@fix_FreeHands
        ";
        let correct = Recipe {
            name: "PID_EMPTY_JET",
            description: Some("Пустая банка для Джета. Расплавьте пластиковую бутылку и полученную жидкость влейте в форму, \
            после чего из различного мусора соберите простейший клапан и закрепите на еще горячем пластике."),
            params_to_see: None,
            params_to_craft: Some(LogicChain {
                first: KeyValue {
                    key: "SK_REPAIR",
                    value: 100,
                },
                rest: vec![
                    (Logical::Or, KeyValue {
                        key: "SK_SCIENCE",
                        value: 100,
                    }),
                ],
            }),
            ingredients: LogicChain {
                first: KeyValue {
                    key: "PID_BOTTLE_EMPTY",
                    value: 5,
                },
                rest: vec![
                    (Logical::And, KeyValue {
                        key: "PID_CRAFT_L_LINT",
                        value: 5,
                    }),
                    (Logical::And, KeyValue {
                        key: "PID_CRAFT_M_JUNK",
                        value: 1,
                    }),
                ],
            },
            tools: Some(LogicChain {
                first: KeyValue {
                    key: "PID_KNIFE",
                    value: 1,
                },
                rest: vec![
                    (Logical::And, KeyValue {
                        key: "PID_LIGHTER",
                        value: 1,
                    }),
                ],
            }),
            output: LogicChain {
                first: KeyValue {
                    key: "PID_EMPTY_JET",
                    value: 5,
                },
                rest: vec![],
            },
            side_effect: SideEffect::Script {
                module: "fix_boy",
                function: "fix_FreeHands",
            },
        };
        assert_eq!(correct, lex(recipe, SAMPLE));
    }

    fn assert_eq_display(original: &str, new: &str) {
        let logic_chain = lex(logic_chain, original);
        assert_eq!(new, logic_chain.to_string());
        let logic_nodes = logic_chain.logic_nodes();
        assert_eq!(new, logic_nodes.to_string());
    }

    #[test]
    fn test_logic_chain_display() {
        assert_eq_display("SK_REPAIR 100", "SK_REPAIR: 100");
        assert_eq_display(
            "SK_REPAIR 100|SK_SCIENCE 100",
            "SK_REPAIR: 100 or SK_SCIENCE: 100",
        );
        assert_eq_display(
            "SK_REPAIR 100|SK_SCIENCE 100&SK_DOCTOR 100",
            "(SK_REPAIR: 100 or SK_SCIENCE: 100) and SK_DOCTOR: 100",
        );
        assert_eq_display(
            "SK_REPAIR 100&SK_SCIENCE 100|SK_DOCTOR 100",
            "SK_REPAIR: 100 and (SK_SCIENCE: 100 or SK_DOCTOR: 100)",
        );
        assert_eq_display(
            "SK_REPAIR 100|SK_SCIENCE 100&SK_DOCTOR 100|SK_OUTDOORSMAN 100",
            "(SK_REPAIR: 100 or SK_SCIENCE: 100) and (SK_DOCTOR: 100 or SK_OUTDOORSMAN: 100)",
        );
    }

    #[test]
    fn lex_forp_crafts() {
        for dir in &["../../../FO4RP/text/eng", "../../../FO4RP/text/russ"] {
            for file in std::fs::read_dir(dir).unwrap() {
                let path = file.unwrap().path();
                if path.file_name().unwrap() == "FOCRAFT.MSG" {
                    let mut book = crate::RecipeBook::default();
                    let path_str = path.to_str().unwrap();
                    let res = fo_msg_format::parse_cp1251_file(&path).expect(path_str);
                    for (key, value) in res.iter_firsts() {
                        let recipe = lex(recipe, value);
                        book.recipes.insert(key, recipe);
                    }
                    dbg!(book);
                }
            }
        }
    }
}
