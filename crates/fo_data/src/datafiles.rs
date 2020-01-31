use nom_prelude::{complete::*, *};
use std::path::{Path, PathBuf};

const DATAFILES_CFG: &str = "DataFiles.cfg";

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Canonicalize(PathBuf, std::io::Error),
    Nom(nom::Err<(String, nom::error::ErrorKind)>),
}

pub fn parse_datafile<P: AsRef<Path>>(parent_folder: P) -> Result<Vec<PathBuf>, Error> {
    let datafiles = parent_folder.as_ref().join(DATAFILES_CFG);
    let datafiles = datafiles
        .canonicalize()
        .map_err(move |err| Error::Canonicalize(datafiles.into(), err))?;
    let file = std::fs::read_to_string(&datafiles).map_err(Error::Io)?;
    parse_datafile_inner::<(&str, nom::error::ErrorKind)>(&file)
        .map_err(|err| Error::Nom(owned_err(err)))
        .and_then(|(rest, vec)| {
            vec.into_iter()
                .map(datapath(parent_folder.as_ref()))
                .collect()
        })
}

fn parse_datafile_inner<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<&'a str>, E> {
    fold_many0(alt_line, Vec::new(), push_some)(i)
}

fn _debug<'a, E: ParseError<&'a str>, F, O: std::fmt::Debug>(
    f: F,
) -> impl Fn(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    move |i| {
        let (rest, val) = f(i)?;
        println!(
            "In: {:?}, Out: {:?}, Rest: {:?}",
            i.chars().take(40).collect::<String>(),
            &val,
            rest.chars().take(40).collect::<String>(),
        );
        Ok((rest, val))
    }
}

fn alt_line<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Option<&'a str>, E> {
    alt((
        map(t_rn, |_| None),
        map(comment, |_| None),
        map(include, |_| None),
        map(line, Some),
    ))(i)
}

fn comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    preceded(char('#'), alt((line, t_rn)))(i)
}

fn include<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    preceded(tag("include "), line)(i)
}

fn push_some<T>(mut acc: Vec<T>, item: Option<T>) -> Vec<T> {
    if let Some(item) = item {
        acc.push(item);
    }
    acc
}

fn datapath<'a>(parent: &'a Path) -> impl Fn(&'a str) -> Result<PathBuf, Error> {
    move |datapath| {
        let mut buf = PathBuf::from(parent);
        buf.extend(Path::new(datapath).components());
        buf.canonicalize()
            .map_err(move |err| Error::Canonicalize(buf, err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_datafile() {
        let datafiles = parse_datafile("../../CL4RP").unwrap();
        dbg!(datafiles);
    }
}
