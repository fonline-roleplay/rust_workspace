use crate::prelude::*;

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize, SerDebug))]
pub struct Header<'a> {
    pub version: u32,
    pub max_hex_x: u16,
    pub max_hex_y: u16,
    pub work_hex_x: i32,
    pub work_hex_y: i32,
    pub script_module: Option<&'a str>,
    pub script_func: Option<&'a str>,
    pub no_logout: bool,
    pub time: i32,
    pub day_time: [i32; 4],
    pub day_color: [u8; 12],
}

pub fn header<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Header<'a>, E> {
    let (i, _) = section("Header")(i)?;
    let (i, mut header) = parse_struct!(i, Header{
        version: key_int("Version"),
        max_hex_x: key_int("MaxHexX"),
        max_hex_y: key_int("MaxHexY"),
        work_hex_x: key_int("WorkHexX"),
        work_hex_y: key_int("WorkHexY"),
        script_module: opt_flatten(opt_kv("ScriptModule", optional_str)),
        script_func: opt_flatten(opt_kv("ScriptFunc", optional_str)),
        no_logout: kv("NoLogOut", int_bool),
        time: key_int("Time"),
    }, {
        day_time: [0; 4],
        day_color: [0; 12],
    });

    let (i, (day_time, day_color0, day_color1, day_color2, day_color3)) = tuple((
        kv("DayTime", fixed_list_of_numbers(4)),
        kv("DayColor0", fixed_list_of_numbers(3)),
        kv("DayColor1", fixed_list_of_numbers(3)),
        kv("DayColor2", fixed_list_of_numbers(3)),
        kv("DayColor3", fixed_list_of_numbers(3)),
    ))(i)?;

    header.day_time.copy_from_slice(&day_time);
    header.day_color[0..=2].copy_from_slice(&day_color0);
    header.day_color[3..=5].copy_from_slice(&day_color1);
    header.day_color[6..=8].copy_from_slice(&day_color2);
    header.day_color[9..=11].copy_from_slice(&day_color3);
    Ok((i, header))
}
