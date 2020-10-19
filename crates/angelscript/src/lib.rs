pub use as_derive::*;
pub use memoffset;

pub trait Engine {
    fn register_object_type<T>(&mut self, obj: &str, flags: u32) -> Result<(), i32>;

    fn register_object_property<T>(
        &mut self,
        obj: &str,
        declaration: &str,
        byte_offset: usize,
    ) -> Result<(), i32>;
}

pub trait AngelScript {
    fn register<E: Engine>(engine: &mut E);
}
