use nom_prelude::{complete::*, *};
use std::path::{Path, PathBuf};

const DATAFILES_CFG: &str = "DataFiles.cfg";

#[derive(Debug)]
pub enum Error {
    Io(PathBuf, std::io::Error),
    Canonicalize(PathBuf, std::io::Error),
    Metadata(PathBuf, std::io::Error),
    //Nom(nom::Err<(String, nom::error::ErrorKind)>),
    Nom(nom::Err<String>),
}
trait PathError<T, E>: Sized {
    fn path_err<E2>(self, path: &Path, fun: fn(PathBuf, E) -> E2) -> Result<T, E2>;
}
impl<T, E> PathError<T, E> for Result<T, E> {
    fn path_err<E2>(self, path: &Path, fun: fn(PathBuf, E) -> E2) -> Result<T, E2> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(fun(path.into(), err)),
        }
    }
}

fn datafile_path(parent_folder: &Path) -> Result<PathBuf, Error> {
    let datafiles = parent_folder.join(DATAFILES_CFG);
    datafiles
        .canonicalize()
        .path_err(&parent_folder, Error::Canonicalize)
}

fn changetime(path: &Path) -> Result<crate::ChangeTime, Error> {
    let metadata = path.metadata().path_err(path, Error::Metadata)?;
    //map_err(|err| Error::Metadata(datafiles, err) )?;
    metadata.modified().path_err(path, Error::Metadata)
}

pub fn datafiles_changetime<P: AsRef<Path>>(parent_folder: P) -> Result<crate::ChangeTime, Error> {
    let datafiles = datafile_path(parent_folder.as_ref())?;
    changetime(parent_folder.as_ref())
}

pub fn parse_datafile<P: AsRef<Path>>(parent_folder: P) -> Result<Vec<crate::FoArchive>, Error> {
    let datafiles = datafile_path(parent_folder.as_ref())?;
    let file = std::fs::read_to_string(&datafiles).path_err(&datafiles, Error::Io)?;
    //parse_datafile_inner::<(&str, nom::error::ErrorKind)>(&file)
    parse_datafile_inner::<nom::error::VerboseError<_>>(&file)
        //.map_err(|err| Error::Nom(owned_err(err)))
        .map_err(|err| Error::Nom(err.map(|err| nom::error::convert_error(&file, err))))
        .and_then(|(rest, vec)| {
            let res: Result<Vec<crate::FoArchive>, Error> = vec
                .into_iter()
                .map(|path| datapath(parent_folder.as_ref(), path).and_then(gather_metadata))
                .collect();
            res
        })
}

fn gather_metadata(path: PathBuf) -> Result<crate::FoArchive, Error> {
    let changed = changetime(&path)?;
    Ok(crate::FoArchive { changed, path })
}

fn parse_datafile_inner<'a, E: std::fmt::Debug + ParseError<&'a str>>(
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

fn alt_line<'a, E: std::fmt::Debug + ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Option<&'a str>, E> {
    alt((
        map(comment, |_| None),
        map(include, |_| None),
        map(line, Some),
        map(t_rn, |_| None),
    ))(i)
}

fn comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    preceded(char('#'), alt((line, end_of_line)))(i)
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

fn datapath(parent: &Path, datapath: &str) -> Result<PathBuf, Error> {
    let mut buf = PathBuf::from(parent);
    buf.extend(Path::new(datapath).components());
    buf.canonicalize()
        .map_err(move |err| Error::Canonicalize(buf, err))
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
