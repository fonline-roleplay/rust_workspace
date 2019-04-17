use std::{borrow::Cow, ffi::OsStr};

// CP1251 to UTF
const FORWARD_TABLE: &'static [u16] = &[
    1026, 1027, 8218, 1107, 8222, 8230, 8224, 8225, 8364, 8240, 1033, 8249, 1034, 1036, 1035, 1039,
    1106, 8216, 8217, 8220, 8221, 8226, 8211, 8212, 152, 8482, 1113, 8250, 1114, 1116, 1115, 1119,
    160, 1038, 1118, 1032, 164, 1168, 166, 167, 1025, 169, 1028, 171, 172, 173, 174, 1031, 176,
    177, 1030, 1110, 1169, 181, 182, 183, 1105, 8470, 1108, 187, 1112, 1029, 1109, 1111, 1040,
    1041, 1042, 1043, 1044, 1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052, 1053, 1054, 1055, 1056,
    1057, 1058, 1059, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067, 1068, 1069, 1070, 1071, 1072,
    1073, 1074, 1075, 1076, 1077, 1078, 1079, 1080, 1081, 1082, 1083, 1084, 1085, 1086, 1087, 1088,
    1089, 1090, 1091, 1092, 1093, 1094, 1095, 1096, 1097, 1098, 1099, 1100, 1101, 1102, 1103,
]; // 128 entries

#[cfg(windows)]
fn is_ascii(string: &OsStr) -> bool {
    use std::os::windows::ffi::OsStrExt;
    let mut maybe_ascii = false;
    for wide in string.encode_wide() {
        if wide < 0x20 || wide > 0xFF {
            return false;
        }
        if wide >= 0x80 {
            maybe_ascii = true;
        }
    }
    maybe_ascii
}

#[cfg(windows)]
fn from_ascii(string: &OsStr) -> Option<String> {
    use std::convert::TryInto;
    use std::os::windows::ffi::OsStrExt;
    //let mut vec = Vec::with_capacity(string.len()*2);
    let mut new_string = String::with_capacity(string.len() * 2);
    for wide in string.encode_wide() {
        let code = wide.to_ne_bytes()[0];
        if code >= 0x80 {
            let cp1251 = FORWARD_TABLE[(code - 0x80) as usize] as u32;
            new_string.push(cp1251.try_into().ok()?);
        } else if code != 0 {
            new_string.push(code as char);
        }
    }
    Some(new_string)
}

#[cfg(windows)]
pub fn decode_filename(filename: &OsStr) -> Option<String> {
    if is_ascii(filename) {
        from_ascii(filename)
    } else {
        filename.to_str().map(String::from)
    }
}

#[cfg(not(windows))]
pub fn decode_filename(filename: &OsStr) -> Option<String> {
    let string = match filename.to_str() {
        Some(string) => string,
        None => {
            return from_ascii(filename);
        }
    };

    if is_ascii(string) {
        from_utfish_ascii(string)
    } else {
        Some(string.to_owned())
    }
}

#[cfg(not(windows))]
fn from_utfish_ascii(string: &str) -> Option<String> {
    use std::convert::TryInto;
    let mut new_string = String::with_capacity(string.len() * 2);
    for ch in string.chars() {
        let code = ch as u32;
        if code >= 0x80 && code <= 0xFF {
            let cp1251 = FORWARD_TABLE[(code - 0x80) as usize] as u32;
            new_string.push(cp1251.try_into().ok()?);
        } else if code != 0 {
            new_string.push(ch);
        }
    }
    Some(new_string)
}

#[cfg(not(windows))]
fn from_ascii(string: &OsStr) -> Option<String> {
    use std::convert::TryInto;
    use std::os::unix::ffi::OsStrExt;

    let mut new_string = String::with_capacity(string.len() * 2);
    for &code in string.as_bytes() {
        if code >= 0x80 {
            let cp1251 = FORWARD_TABLE[(code - 0x80) as usize] as u32;
            new_string.push(cp1251.try_into().ok()?);
        } else if code != 0 {
            new_string.push(code as char);
        }
    }
    Some(new_string)
}

#[cfg(not(windows))]
fn is_ascii(string: &str) -> bool {
    let mut maybe_ascii = false;
    for ch in string.chars() {
        let ch = ch as u32;
        if ch < 0x20 || ch > 0xFF {
            return false;
        }
        if ch >= 0x80 {
            maybe_ascii = true;
        }
    }
    maybe_ascii
}
/*
#[cfg(not(windows))]
fn is_true_ascii() {
    use std::os::unix::ffi::OsStrExt;
    let mut maybe_ascii = false;
    println!("{:?}", string);
    let mut last_cyr = false;
    let mut maybe_utf8 = false;
    let mut utf_len = 0;
    for &ch in string.as_bytes() {
        if ch < 0x20 {
            return false;
        }

        if ch&0b11000000 == 0b11000000 {
            utf_len+=1;
        } else {
            if utf_len > 1 {
                return true;
            }
            utf_len = 0;
        }

        if last_cyr && ch < 0b11000000 {
            maybe_utf8 = true;
        }

        last_cyr = if ch == 208 || ch == 209 {
            true
        } else {
            false
        };

        if ch >= 0x80 {
            maybe_ascii = true;
        }
        print!(" {}", ch)
    }
    if maybe_utf8 {
        return false;
    }
    return maybe_ascii
}*/

pub fn os_str_debug<'a>(os_str: &'a OsStr) -> Cow<'a, str> {
    match os_str.to_str() {
        Some(string) => Cow::Borrowed(string),
        None => Cow::Owned(os_str_debug_inner(os_str)),
    }
}
#[cfg(not(windows))]
fn os_str_debug_inner(os_str: &OsStr) -> String {
    use std::os::unix::ffi::OsStrExt;
    format!("{:X?}", os_str.as_bytes())
}
#[cfg(windows)]
fn os_str_debug_inner(os_str: &OsStr) -> String {
    use std::os::windows::ffi::OsStrExt;
    let mut vec = Vec::with_capacity(os_str.len());
    vec.extend(string.encode_wide());
    format!("{:X?}", &vec[..])
}
