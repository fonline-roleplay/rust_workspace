use crate::engine_types::stl::{stlp_std_allocator, stlp_std_priv__STLP_alloc_proxy};
use std::mem::ManuallyDrop;

#[repr(C)]
union stlp_std_priv__String_base__Buffers<_Tp: Copy> {
    _M_end_of_storage: *mut _Tp,
    _M_static_buf: [_Tp; 4 * ::std::mem::size_of::<*const ::std::os::raw::c_void>()],
    _phantom_0: ::std::marker::PhantomData<::std::cell::UnsafeCell<_Tp>>,
}

type stlp_std_priv__String_base_allocator_type<_Alloc> = _Alloc;
type stlp_std_priv__String_base__AllocProxy<_Tp, _Alloc> = stlp_std_priv__STLP_alloc_proxy<
    *mut _Tp,
    _Tp,
    stlp_std_priv__String_base_allocator_type<_Alloc>,
>;

#[repr(C)]
struct stlp_std_priv__String_base<_Tp: Copy, _Alloc> {
    _M_buffers: stlp_std_priv__String_base__Buffers<_Tp>,
    _M_finish: *mut _Tp,
    _M_start_of_storage: stlp_std_priv__String_base__AllocProxy<_Tp, _Alloc>,
    _phantom_0: ::std::marker::PhantomData<::std::cell::UnsafeCell<_Tp>>,
    _phantom_1: ::std::marker::PhantomData<::std::cell::UnsafeCell<_Alloc>>,
}

/*
impl<_Tp: std::fmt::Debug+Copy, _Alloc: std::fmt::Debug> stlp_std_priv__String_base<_Tp, _Alloc> {
    fn is_using_static_buf(&self) -> bool {
        unsafe{ self._M_start_of_storage._M_data as *const _ == &self._M_buffers._M_static_buf as *const _ }
    }
    pub fn print(&self) {
        unsafe {
            let end_of_storage = self._M_buffers._M_end_of_storage;
            let static_buf = self._M_buffers._M_static_buf;
            let finish = self._M_finish;
            let start_of_storage = &self._M_start_of_storage;
            println!("Start of storage: {:?}", start_of_storage);
            println!("End of storage: {:?}", end_of_storage);
            println!("Finish: {:?}", finish);
            println!("Static buffer: {:?}", static_buf);
            let using_st = self.is_using_static_buf();
            println!("using_static: {:?}", using_st);
        }
    }
}
*/

#[repr(C)]
struct stlp_std_basic_string<_CharT: Copy, _Alloc> {
    _base: stlp_std_priv__String_base<_CharT, _Alloc>,
    _phantom_0: ::std::marker::PhantomData<::std::cell::UnsafeCell<_CharT>>,
    _phantom_1: ::std::marker::PhantomData<::std::cell::UnsafeCell<_Alloc>>,
}
type stlp_std_string = stlp_std_basic_string<::std::os::raw::c_char, stlp_std_allocator>;

#[repr(C)]
struct ScriptString__bindgen_vtable(::std::os::raw::c_void);
#[repr(C)]
pub struct ScriptStringInner {
    vtable_: *const ScriptString__bindgen_vtable,
    buffer: stlp_std_string,
    refCount: ::std::os::raw::c_int,
}

#[repr(C)]
pub struct ScriptString {
    inner: ManuallyDrop<ScriptStringInner>,
}

impl ScriptString {
    pub fn string(&self) -> String {
        cp1251_to_utf8(self.inner.buffer._base._M_start_of_storage._M_data)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_std_string_size() {
        assert_eq!(
            ::std::mem::size_of::<stlp_std_basic_string<::std::os::raw::c_char, stlp_std_allocator>>(
            ),
            24usize,
            concat!(
                "Size of template specialization: ",
                stringify ! ( stlp_std_basic_string < :: std :: os :: raw :: c_char , stlp_std_allocator > )
            )
        );
        assert_eq!(
            ::std::mem::align_of::<stlp_std_basic_string<::std::os::raw::c_char, stlp_std_allocator>>(
            ),
            4usize,
            concat!(
                "Alignment of template specialization: ",
                stringify ! ( stlp_std_basic_string < :: std :: os :: raw :: c_char , stlp_std_allocator > )
            )
        );
    }
}

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

fn cp1251_to_utf8(ptr: *mut ::std::os::raw::c_char) -> String {
    let c_str = unsafe { ::std::ffi::CString::from_raw(ptr) };
    let string: String = c_str
        .as_bytes()
        .iter()
        .map(|&code| {
            use std::convert::TryInto;
            if code >= 0x80 {
                (FORWARD_TABLE[(code - 0x80) as usize] as u32)
                    .try_into()
                    .unwrap_or(' ')
            } else {
                code as char
            }
        })
        .collect();
    ::std::mem::forget(c_str);
    string
}
