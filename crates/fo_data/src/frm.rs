//use array_macro::array;
use debug_helper::impl_debug_for_struct;
use nom::error::context;
use nom::{
    bytes::complete::take,
    combinator::{cut, verify},
    //number::streaming::{be_i16, be_u16, be_u32},
    number::complete::{be_i16, be_u16, be_u32},
    Compare,
    InputLength,
    InputTake,
};
use nom_prelude::*;

pub type Arr6<V> = ArrayVec<[V; 6]>;

#[derive(Debug)]
pub struct Frm<'a> {
    pub version: u32,
    pub fps: u16,
    pub action_frame: u16,
    pub directions: Arr6<Direction<'a>>,
}

pub fn frm_verbose<'a>(
    buf: &'a [u8],
) -> Result<(&'a [u8], Frm<'a>), nom::Err<nom::error::VerboseError<&'a [u8]>>> {
    let (rest, frm) = parse_frm(buf)?;
    Ok((rest, frm))
}

pub fn frm<'a>(buf: &'a [u8]) -> Result<Frm<'a>, ErrorKind> {
    err_to_kind(parse_frm(buf))
}

fn parse_frm<'a, Error: ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], Frm<'a>, Error> {
    let (i, version) = be_u32(i)?;
    assert_eq!(version, 4);
    let (i, fps) = be_u16(i)?;
    let (i, action_frame) = be_u16(i)?;
    let (i, number_of_frames_per_direction) = be_u16(i)?;
    let (i, x_shifts_per_direction): (_, Arr6<_>) = context("x_shifts", count_array(be_i16))(i)?;
    let (i, y_shifts_per_direction): (_, Arr6<_>) = context("y_shifts", count_array(be_i16))(i)?;
    let (i, _memory_offsets_of_first_frame_per_direction) = count(be_u32, 6)(i)?;
    /*println!(
        "memory_offsets: {:?}",
        _memory_offsets_of_first_frame_per_direction
    );*/
    let (i, _size_of_frame_area) = be_u32(i)?;
    let (i, directions_frames): (_, Arr6<_>) = context(
        "directions_frames",
        many_array(1, parse_direction(number_of_frames_per_direction)),
    )(i)?;

    /*let directions = array![|i| {
        Direction {
            shift_x: x_shifts_per_direction[i],
            shift_y: y_shifts_per_direction[i],
            frames: mem_take(&mut directions_frames[i]),
        }
    }; 6];*/
    let directions = itertools::multizip((
        x_shifts_per_direction.iter(),
        y_shifts_per_direction.iter(),
        directions_frames.into_iter(),
    ))
    .map(|(&shift_x, &shift_y, frames)| Direction {
        shift_x,
        shift_y,
        frames,
    })
    .collect();

    Ok((
        i,
        Frm {
            version,
            fps,
            action_frame,
            directions,
        },
    ))
}

#[derive(Default, Debug)]
pub struct Direction<'a> {
    pub shift_x: i16,
    pub shift_y: i16,
    pub frames: Vec<Frame<'a>>,
}

pub fn parse_direction<'a, Error: ParseError<&'a [u8]>>(
    number_of_frames: u16,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Vec<Frame<'a>>, Error> {
    //println!("parse_direction");
    context(
        "parse_direction",
        count_cap(
            context("parse_frame", parse_frame),
            number_of_frames as usize,
        ),
    )
}

pub struct Frame<'a> {
    pub width: u16,
    pub height: u16,
    pub offset_x: i16,
    pub offset_y: i16,
    pub data: &'a [u8],
}

impl<'a> std::fmt::Debug for Frame<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        impl_debug_for_struct!(Frame, f, self,
            .width, .height, .offset_x, .offset_y,
            (.data, "&[...; {}]", self.data.len())
        );
    }
}

pub fn parse_frame<'a, Error: ParseError<&'a [u8]>>(
    i: &'a [u8],
) -> IResult<&'a [u8], Frame<'a>, Error> {
    //println!("parse_frame: {:?}", i);
    let (i, (width, height)) = context("width&height", pair(be_u16, be_u16))(i)?;
    let (i, number_of_pixels) = context(
        "number_of_pixels",
        cut(verify(be_u32, |size| *size == width as u32 * height as u32)),
    )(i)?;
    let (i, (offset_x, offset_y)) = context("frame offsets", pair(be_i16, be_i16))(i)?;
    let (i, data) = context("take_data", take(number_of_pixels))(i)?;
    Ok((
        i,
        Frame {
            width,
            height,
            offset_x,
            offset_y,
            data,
        },
    ))
}

/*
pub fn parse_frm<'a, T: 'a, Input: 'a, Error: ParseError<Input>>(
    tag: T
) -> impl Fn(Input) -> IResult<Input, Input, Error> where
    Input: InputTake + Compare<T>,
    T: InputLength + Clone, {

}
*/
/*
pub fn parse_frm<'a, T: 'a, Input: 'a, Error: ParseError<Input>>(
    input: Input
) -> IResult<Input, Input, Error> where
    Input: InputTake + Compare<T>,
    T: InputLength + Clone, {

}
*/

// std::mem::take is unstable
pub fn mem_take<T: Default>(dest: &mut T) -> T {
    std::mem::replace(dest, T::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_edg1001() {
        let file = std::fs::read("../../test_assets/EDG1001.FRM").unwrap();
        let (rest, frm) = frm_verbose(&file).unwrap();
        println!("frm: {:#?}, rest: {:?}", frm, rest)
    }
    #[test]
    fn parse_hmwarraa() {
        let file = std::fs::read("../../test_assets/HMWARRAA.FRM").unwrap();
        let (rest, frm) = frm_verbose(&file).unwrap();
        println!("frm: {:#?}, rest: {:?}", frm, rest);
    }
}
