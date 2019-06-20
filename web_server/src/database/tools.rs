use std::convert::TryInto;

pub fn slice_to_u32(slice: &[u8]) -> Option<u32> {
    let bytes: &[u8;4] = slice.try_into().ok()?;
    Some(u32::from_be_bytes(*bytes))
}

pub fn increment(old: Option<&[u8]>) -> Option<Vec<u8>> {
    let number = match old {
        Some(slice) => {
            if let Some(number) = slice_to_u32(slice) {
                number + 1
            } else {
                eprintln!("Attempt to increment value with length of {}", slice.len());
                return Some(slice.to_vec());
            }
        },
        None => 0,
    };
    Some(number.to_be_bytes().to_vec())
}
