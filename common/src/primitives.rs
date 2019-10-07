pub type int8 = ::std::os::raw::c_char;
pub type uint8 = ::std::os::raw::c_uchar;
pub type int16 = ::std::os::raw::c_short;
pub type uint16 = ::std::os::raw::c_ushort;
pub type uint64 = u64;
pub type int64 = i64;

pub type ulong = ::std::os::raw::c_ulong;
pub type ushort = ::std::os::raw::c_ushort;
pub type uint = ::std::os::raw::c_uint;
pub type int = ::std::os::raw::c_int;

pub trait Validate {
    fn is_valid(&self) -> bool;
}

#[repr(C)]
pub struct MaybeInvalid<T: Validate>(T);

impl<T: Validate> MaybeInvalid<T> {
    pub fn validate(&self) -> Option<&T> {
        if self.0.is_valid() {
            Some(&self.0)
        } else {
            None
        }
    }
    pub fn validate_mut(&mut self) -> Option<&mut T> {
        if self.0.is_valid() {
            Some(&mut self.0)
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Hex {
    pub x: u16,
    pub y: u16,
}

impl Hex {
    pub fn get_distance(self, other: Hex) -> u32 {
        crate::utils::map::get_distance_hex(self, other, true)
    }
    pub fn get_direction(self, other: Hex) -> u8 {
        crate::utils::map::get_direction(self, other, true)
    }
}
