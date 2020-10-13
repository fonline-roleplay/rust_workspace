use nom::{branch::permutation, bytes::complete::tag};
use nom_prelude::*;

pub type Arr6<V> = ArrayVec<[V; 6]>;
#[derive(Debug, Default)]
pub struct FoFrmRaw<'a> {
    pub fps: Option<u16>,
    pub count: Option<u16>,
    pub effect: Option<&'a str>,
    pub offset_x: Option<i16>,
    pub offset_y: Option<i16>,
    pub directions: Arr6<Direction<'a>>,
}

impl<'a> FoFrmRaw<'a> {
    fn without_directions(&mut self) -> Result<&mut Self, FoFrmErrorKind> {
        if self.directions.is_empty() {
            Ok(self)
        } else {
            Err(FoFrmErrorKind::UnexpectedToken)
        }
    }
    fn end_direction(&mut self) -> Result<(), FoFrmErrorKind> {
        if let Some(direction) = self.directions.last_mut() {
            direction.end_frame()?;
            let len = direction.frames.len() as u16;
            match &mut self.count {
                Some(count) if *count < len => {
                    return Err(FoFrmErrorKind::WrongNumberOfFrames(
                        direction.frames.len(),
                        *count,
                    ));
                }
                Some(count) if *count > len => {
                    *count = len;
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn new_direction(&mut self, dir: u8) -> Result<(), FoFrmErrorKind> {
        if self.directions.len() != dir as usize {
            return Err(FoFrmErrorKind::WrongDirOrder(self.directions.len(), dir));
        }
        self.end_direction();
        self.directions.push(Default::default());
        Ok(())
    }
    fn last_direction(&mut self) -> &mut Direction<'a> {
        if self.directions.is_empty() {
            self.directions.push(Default::default());
        }
        self.directions.last_mut().unwrap()
    }
}

#[derive(Default, Debug)]
pub struct Direction<'a> {
    pub offset_x: Option<i16>,
    pub offset_y: Option<i16>,
    pub frames: Vec<Frame<'a>>,
}

impl<'a> Direction<'a> {
    fn end_frame(&mut self) -> Result<(), FoFrmErrorKind> {
        match self.frames.last() {
            Some(frame) => {
                frame
                    .frm
                    .ok_or_else(|| FoFrmErrorKind::FrameWithoutPath(self.frames.len()))?;
            }
            None => {}
        }
        Ok(())
    }
    fn new_frame(&mut self) -> Result<&mut Frame<'a>, FoFrmErrorKind> {
        self.end_frame();
        self.frames.push(Default::default());
        Ok(self.frames.last_mut().unwrap())
    }
    fn get_frame(&mut self, frame: u16) -> Result<&mut Frame<'a>, FoFrmErrorKind> {
        let len: u16 = self.frames.len() as u16;
        if len > frame + 1 {
            Err(FoFrmErrorKind::WrongFrameOrder(self.frames.len(), frame))
        } else if len == frame + 1 {
            Ok(self.frames.last_mut().unwrap())
        } else {
            self.new_frame()
        }
    }
}

#[derive(Default, Debug)]
pub struct Frame<'a> {
    pub frm: Option<&'a str>,
    pub next_x: Option<i16>,
    pub next_y: Option<i16>,
}

#[derive(Debug)]
enum LocalError<'a, N: ParseError<&'a str>> {
    //Io(std::io::Error),
    //Canonicalize(PathBuf, std::io::Error),
    //Nom(nom::Err<(String, nom::error::ErrorKind)>),
    Nom(&'a str, nom::Err<N>),
    Verify(&'a str, FoFrmErrorKind),
}

#[derive(Debug)]
pub enum FoFrmError {
    Nom(nom::Err<String>),
    Verify(String, FoFrmErrorKind),
}

type VerboseError<'a> = LocalError<'a, nom::error::VerboseError<&'a str>>;

impl<'a> VerboseError<'a> {
    fn to_owned(self) -> FoFrmError {
        match self {
            LocalError::Nom(i, err) => {
                FoFrmError::Nom(err.map(|err| nom::error::convert_error(i, err)))
            }
            LocalError::Verify(i, err) => {
                let text = i.chars().take(50).collect();
                FoFrmError::Verify(text, err)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Token<'a> {
    Fps(u16),
    Count(u16),
    Effect(&'a str),
    OffsetX(i16),
    OffsetY(i16),
    Dir(u8),
    Frm((u16, &'a str)),
    NextX((u16, i16)),
    NextY((u16, i16)),
    NewLine,
}

fn tokenize<'a, Error: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Token<'a>, Error> {
    use Token::*;
    let uint = unsigned_number;
    let int = integer;
    alt((
        map(kv_eq("fps", uint), Fps),
        map(kv_eq("count", uint), Count),
        map(kv_eq("effect", some_text), Effect),
        map(kv_eq("offs_x", int), OffsetX),
        map(kv_eq("offs_y", int), OffsetY),
        map(section_ext(preceded(tag("dir_"), unsigned_number)), Dir),
        map(kv_eq("frm", some_text), |frm| Frm((0, frm))),
        map(kv_kv_sep(preceded(tag("frm_"), uint), "=", some_text), Frm),
        map(kv_kv_sep(preceded(tag("next_x_"), uint), "=", int), NextX),
        map(kv_kv_sep(preceded(tag("next_y_"), uint), "=", int), NextY),
        map(t_rn, |_| NewLine),
    ))(i)
}

fn parse_fofrm_raw<'a, Error: ParseError<&'a str>>(
    mut i: &'a str,
) -> Result<FoFrmRaw<'a>, LocalError<'a, Error>> {
    let mut fofrm = FoFrmRaw::default();
    loop {
        let (rest, token) = tokenize(i).map_err(|err| LocalError::Nom(i, err))?;
        update_fofrm(&mut fofrm, token).map_err(|err| err.into_local(i))?;
        i = rest;

        if i.is_empty() {
            fofrm.end_direction().map_err(|err| err.into_local(i))?;
            break;
        }
    }
    Ok(fofrm)
}

pub fn parse_verbose<'a>(input: &'a str) -> Result<FoFrmRaw<'a>, FoFrmError> {
    parse_fofrm_raw(input).map_err(VerboseError::to_owned)
}

#[derive(Debug)]
pub enum FoFrmErrorKind {
    RepeatedToken,
    UnexpectedToken,
    WrongDirOrder(usize, u8),
    WrongFrameOrder(usize, u16),
    FrameWithoutPath(usize),
    WrongNumberOfFrames(usize, u16),
}
impl FoFrmErrorKind {
    fn into_local<'a, Error: ParseError<&'a str>>(self, i: &'a str) -> LocalError<'a, Error> {
        LocalError::Verify(i, self)
    }
}

fn update_fofrm<'a>(fofrm: &mut FoFrmRaw<'a>, token: Token<'a>) -> Result<(), FoFrmErrorKind> {
    use Token::*;
    match token {
        Fps(fps) => set_once(&mut fofrm.without_directions()?.fps, fps)?,
        Count(count) => set_once(
            &mut fofrm.without_directions()?.count,
            if count == 0 { 1 } else { count },
        )?,
        Effect(effect) => set_once(&mut fofrm.without_directions()?.effect, effect)?,
        OffsetX(offset_x) => match fofrm.directions.last_mut() {
            Some(direction) => set_once(&mut direction.offset_x, offset_x)?,
            None => set_once(&mut fofrm.offset_x, offset_x)?,
        },
        OffsetY(offset_y) => match fofrm.directions.last_mut() {
            Some(direction) => set_once(&mut direction.offset_y, offset_y)?,
            None => set_once(&mut fofrm.offset_y, offset_y)?,
        },
        Dir(dir) => {
            fofrm.new_direction(dir)?;
        }
        Frm((frame, path)) => {
            let frame = fofrm.last_direction().get_frame(frame)?;
            frame.frm = Some(path);
        }
        NextX((frame, x)) => {
            let frame = fofrm.last_direction().get_frame(frame)?;
            frame.next_x = Some(x);
        }
        NextY((frame, y)) => {
            let frame = fofrm.last_direction().get_frame(frame)?;
            frame.next_y = Some(y);
        }
        NewLine => {}
    }
    Ok(())
}

fn set_once<T>(option: &mut Option<T>, value: T) -> Result<(), FoFrmErrorKind> {
    if option.is_none() {
        *option = Some(value);
        Ok(())
    } else {
        Err(FoFrmErrorKind::RepeatedToken)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_raw_simple() {
        dbg!(parse_verbose(
            "\
            offs_x=0\n\
            offs_y=5\n\
            frm=radcvetg.png\n\
        "
        )
        .unwrap());
    }
    #[test]
    fn test_offset_reverse() {
        dbg!(parse_verbose(
            "\
        frm=manufact_ammo1.png\n\
        \n\
        offs_x=22\n\
        offs_y=10\n\
        "
        )
        .unwrap());
    }

    #[test]
    fn parse_all_fofrm() {
        let fo_data = crate::FoData::init(crate::CLIENT_FOLDER, crate::palette_path()).unwrap();
        use crate::retriever::{fo::FoRetriever, Retriever};
        let retriever = FoRetriever::new(fo_data);
        //let retriever = crate::test_retriever();
        for (path, file_info) in &retriever.data().files {
            if crate::retriever::recognize_type(path) == crate::FileType::FoFrm {
                let bytes = retriever.file_by_info(file_info).unwrap();
                let string = std::str::from_utf8(&bytes).unwrap();
                let fofrm = parse_verbose(string);
                match &fofrm {
                    Ok(fofrm) => {
                        if fofrm.effect.is_none()
                            && fofrm.directions.len() == 1
                            && !fofrm
                                .directions
                                .get(0)
                                .map(|dir| {
                                    dir.frames.iter().any(|frame| {
                                        frame.next_x.is_some()
                                            || frame.next_y.is_some()
                                            || frame
                                                .frm
                                                .map(|frm| {
                                                    let mut ext: String =
                                                        frm.chars().rev().skip(1).take(2).collect();
                                                    ext.make_ascii_lowercase();
                                                    ext == "rf"
                                                })
                                                .unwrap_or(false)
                                    })
                                })
                                .unwrap_or(false)
                        {
                            continue;
                        }
                    }
                    _ => {}
                }
                println!(
                    "Parsing: '{}' from '{:?}': {:#?}",
                    file_info.original_path,
                    file_info.location(retriever.data()),
                    fofrm
                );
            }
        }
    }
}
