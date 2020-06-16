pub use fo_defines as defines;
pub use fo_defines_fo4rp as defines_fo4rp;
pub use primitives;

#[cfg(all(windows, feature = "engine_types"))]
pub mod engine_types;

pub mod utils;

#[cfg(all(windows, feature = "dll"))]
pub mod dll {
    pub mod init;
    pub mod param_getters;
}

#[cfg(all(windows, feature = "dll"))]
pub mod engine_functions;

#[cfg(all(windows, feature = "dll"))]
pub use dll::init::console_init;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn TestFuncRust() {
    println!("TestFuncRust!");
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn FloatToBits(float: f32) -> u32 {
    float.to_bits()
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn FloatFromBits(bits: u32) -> f32 {
    f32::from_bits(bits)
}

mod si {
    use std::fmt::{Display, Formatter, Result};
    use crate::engine_types::ScriptString;
    static PREFIXES: &[&str] = &["и", "з", "а", "ф", "п", "н", "мк", "м", "", "к", "М", "Г", "Т", "П", "Э", "З", "И"];

    #[derive(Debug)]
    pub struct SI<'a> {
        value: f32,
        units: &'a str,
        zeros: u8,
    }

    impl<'a> SI<'a> {
        pub fn new(value: f32, units: &'a str, zeros: u8) -> Self {
            Self{value, units, zeros}
        }
    }

    impl<'a> Display for SI<'a> {
        fn fmt(&self, f: &mut Formatter) -> Result {
            let level = (self.value.abs().log10()/3.0).floor();
            let index = (level as i32 + 8).max(0).min(PREFIXES.len() as i32 - 1);
            let prefix = PREFIXES[index as usize];
            let mul = 10.0f32.powf(level*3.0);
            let new_value = self.value/mul;
            write!(f, "{:.zeros$} {}{}", new_value, prefix, self.units, zeros=self.zeros as usize)
        }
    }

    #[no_mangle]
    #[allow(non_snake_case)]
    pub extern "C" fn DisplaySI(value: f32, units: Option<&ScriptString>, zeros: u8) -> *mut ScriptString {
        use std::ffi::CStr;

        let mut units = units.map(|units| units.string()).unwrap_or_else(String::new);
        units.push('\0');
        let si = SI::new(value, &units, zeros);
        let output = si.to_string();
        ScriptString::from_string(&output)
    }
}
