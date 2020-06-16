pub use dlopen::wrapper::{Container, WrapperApi};
pub use dlopen_derive::WrapperApi;
pub use once_cell::sync::Lazy;

#[macro_export]
macro_rules! dynamic_ffi {
    ($api:ident, $(pub fn $fun:ident($($arg:ident: $typ:ty$ (,)?)*) $(-> $ret:ty)? ;)*) => {
        #[derive(WrapperApi)]
        pub struct $api {
            $($fun: unsafe extern "C" fn($($arg: $typ,)*) $(-> $ret)? ,)*
        }
    }
}

#[macro_export]
macro_rules! ffi_module {
    ($name:ident, $typ:ident, $file:expr) => {
        #[allow(bad_style)]
        mod ffi {
            //use fo_engine_functions::*;
            use super::*;
            include!($file);
        }
        use ffi::$typ;

        static $name: Lazy<Container<$typ>> =
            Lazy::new(|| unsafe { Container::load_self() }.expect("Can't load api"));

    }
}
