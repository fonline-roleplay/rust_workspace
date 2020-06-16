use super::{Entry, Line, Lst};
use nom_prelude::{complete::*, *};

fn _lst_err_kind(input: &str) -> Result<Lst<'_>, ErrorKind> {
    err_to_kind(lst(input))
}

pub(crate) fn tokenize_lst(input: &str, exhaustive: bool) -> Result<Lst<'_>, String> {
    let (rest, res) = nom_err_to_string(input, lst(input))?;
    if !exhaustive || rest.is_empty() {
        Ok(res)
    } else {
        Err("Failed to exhaust input to the end.".into())
    }
}

fn lst<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Lst<'a>, E> {
    map(separated_list_first_unchecked(t_rn, line), |lines| Lst {
        lines,
    })(i)
}

fn line<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Line<'a>, E> {
    alt((
        map(preceded(char('*'), unsigned_number), Line::Section),
        map(preceded(char('#'), optional_text), Line::Comment),
        map(entry, Line::Entry),
        map(space0, |_| Line::Break),
    ))(i)
}

fn entry<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Entry<'a>, E> {
    let (i, (add, name)) = tuple((map_res(digit1, u32::from_str), preceded(space1, word)))(i)?;
    Ok((i, Entry { add, name }))
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

    const STRENGTH: Entry = Entry {
        add: 0,
        name: "ST_STRENGTH",
    };

    #[test]
    fn lex_entry() {
        for str in &[
            "0 ST_STRENGTH",
            "0      ST_STRENGTH",
            "0\tST_STRENGTH",
            "0\t ST_STRENGTH",
            "0 \tST_STRENGTH",
        ] {
            assert_eq!(lex(entry, str), STRENGTH);
        }
    }

    fn line_entry(add: u32, name: &str) -> Line<'_> {
        Line::Entry(Entry { add, name })
    }

    #[test]
    fn lex_line() {
        assert_eq!(lex(line, "*200"), Line::Section(200));
        assert_eq!(lex(line, "0      ST_STRENGTH"), Line::Entry(STRENGTH));
        assert_eq!(lex(line, ""), Line::Break);
        assert_eq!(lex(line, "# Deprecated"), Line::Comment("Deprecated"));
    }

    #[test]
    fn lex_lst() {
        let input = "\
            \n\
            0      ST_STRENGTH\n\
            1      ST_PERCEPTION\n\
            2      ST_ENDURANCE\n\
            \n\
            *200\n\
            0      SK_SMALL_GUNS\r\n\
            1      SK_BIG_GUNS\r\n\
            2      SK_ENERGY_WEAPONS\r\n\
            3      SK_UNARMED\r\n\
            # Deprecated\n\
            # Deprecated again\r\n\
            0      BT_MEN\
        ";
        let should_be = Lst {
            lines: vec![
                Line::Break,
                line_entry(0, "ST_STRENGTH"),
                line_entry(1, "ST_PERCEPTION"),
                line_entry(2, "ST_ENDURANCE"),
                Line::Break,
                Line::Section(200),
                line_entry(0, "SK_SMALL_GUNS"),
                line_entry(1, "SK_BIG_GUNS"),
                line_entry(2, "SK_ENERGY_WEAPONS"),
                line_entry(3, "SK_UNARMED"),
                Line::Comment("Deprecated"),
                Line::Comment("Deprecated again"),
                line_entry(0, "BT_MEN"),
            ],
        };
        let parsed = lex(lst, input);
        compare(parsed.lines.iter(), should_be.lines.iter());
        assert_eq!(parsed, should_be);
    }

    fn compare<T: PartialEq + std::fmt::Debug>(
        mut left: impl Iterator<Item = T>,
        mut right: impl Iterator<Item = T>,
    ) {
        loop {
            let left_item = left.next();
            let right_item = right.next();
            if left_item.is_none() && right_item.is_none() {
                break;
            }
            assert_eq!(left_item, right_item);
        }
    }
}
