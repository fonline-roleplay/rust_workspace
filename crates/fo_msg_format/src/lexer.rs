use super::{Entry, Line, Msg};
use nom_prelude::{complete::*, *};

pub(crate) fn tokenize_msg(input: &str, exhaustive: bool) -> Result<Msg<'_>, String> {
    let (rest, res) = nom_err_to_string(input, msg(input))?;
    if !exhaustive || rest.is_empty() {
        Ok(res)
    } else {
        Err(format!(
            "Failed to exhaust input to the end: {}",
            rest.chars().take(20).collect::<String>()
        ))
    }
}

fn msg<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Msg<'a>, E> {
    map(separated_list_first_unchecked(t_rn, line), |lines| Msg {
        lines,
    })(i)
}

fn line<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Line<'a>, E> {
    alt((
        map(comment, Line::Comment),
        //map(char('#'), |_| Line::Comment("")),
        map(preceded(space0, entry_with_apply), Line::Entry),
        map(space0, |_| Line::Break),
    ))(i)
}

fn comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    preceded(pair(space0, char('#')), optional_text)(i)
}

fn entry_with_tuple<'a, E: ParseError<&'a str>>(
    ref mut i: &'a str,
) -> IResult<&'a str, Entry<'a>, E> {
    map(
        tuple((
            curly_delimited(unsigned_number),
            curly_delimited(not_closing_curly),
            curly_delimited(not_closing_curly),
            opt(comment),
        )),
        |(index, secondary, value, comment)| Entry {
            index,
            secondary,
            value,
            comment,
        },
    )(i)
}

fn entry_with_macro<'a, E: ParseError<&'a str>>(
    ref mut i: &'a str,
) -> IResult<&'a str, Entry<'a>, E> {
    Ok(parse_struct!(
        i,
        Entry {
            index: curly_delimited(unsigned_number),
            secondary: curly_delimited(not_closing_curly),
            value: curly_delimited(not_closing_curly),
            comment: opt(comment),
        }
    ))
}

fn entry_with_apply<'a, E: ParseError<&'a str>>(
    ref mut i: &'a str,
) -> IResult<&'a str, Entry<'a>, E> {
    let entry = Entry {
        index: apply(i, curly_delimited(unsigned_number))?,
        secondary: apply(i, curly_delimited(not_closing_curly))?,
        value: apply(i, curly_delimited(not_closing_curly))?,
        comment: apply(i, opt(comment))?,
    };
    Ok((i, entry))
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

    fn with_all_entry_impls(sample: &str, correct: &Entry) {
        assert_eq!(&lex(entry_with_tuple, sample), correct);
        assert_eq!(&lex(entry_with_macro, sample), correct);
        assert_eq!(&lex(entry_with_apply, sample), correct);
    }

    #[test]
    fn test_all_entry_impls() {
        const SAMPLE: &str = "{1}{foo}{bar}";
        const CORRECT: Entry = Entry {
            index: 1,
            secondary: "foo",
            value: "bar",
            comment: None,
        };
        with_all_entry_impls(SAMPLE, &CORRECT);
    }

    fn new_entry<'a>(index: u32, secondary: &'a str, value: &'a str) -> Entry<'a> {
        Entry {
            index,
            secondary,
            value,
            comment: None,
        }
    }
    fn entry_line<'a>(index: u32, secondary: &'a str, value: &'a str) -> Line<'a> {
        Line::Entry(Entry {
            index,
            secondary,
            value,
            comment: None,
        })
    }

    #[test]
    fn lex_entry() {
        let samples = &[
            (
                "{4294967295}{             zxc}{zxc              zxc}",
                new_entry(4294967295, "             zxc", "zxc              zxc"),
            ),
            ("{0}{}{}", new_entry(0, "", "")),
            ("{1}{\n}{\n}", new_entry(1, "\n", "\n")),
            (
                "{2}{\n foo \n   \n}{\n\n\n   bar}",
                new_entry(2, "\n foo \n   \n", "\n\n\n   bar"),
            ),
        ];
        for (sample, correct) in samples {
            with_all_entry_impls(sample, correct);
        }
    }

    #[test]
    fn lex_msg() {
        const SAMPLE: &str = "\
            \n\
            # Transit Name, (pid + 1) * 10 + 8 pm added\n\
            \n\
            # Map 0, Global, base 10\n\
            {10}{}{Global map}\n\
            {15}{}{20car}\n\
            {15}{}{23world}\n\
            {15}{}{03 - A Way To Anywhere.ogg}\
        ";
        let correct = Msg {
            lines: vec![
                Line::Break,
                Line::Comment("Transit Name, (pid + 1) * 10 + 8 pm added"),
                Line::Break,
                Line::Comment("Map 0, Global, base 10"),
                entry_line(10, "", "Global map"),
                entry_line(15, "", "20car"),
                entry_line(15, "", "23world"),
                entry_line(15, "", "03 - A Way To Anywhere.ogg"),
            ],
        };
        assert_eq!(lex(msg, SAMPLE), correct);
    }
}
