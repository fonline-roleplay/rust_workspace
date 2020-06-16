use angelscript::*;

#[derive(AngelScript)]
#[repr(C)]
pub struct Foo {
    bar: u32,
    pub baz: i8,
    pub bah: f32,
    boob: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct ASEngine {
        obj: Option<(String, u32)>,
        fields: Vec<(String, String, usize)>,
    }

    impl Engine for ASEngine {
        fn register_object_type<T>(
            &mut self,
            obj: &str,
            flags: u32,
        ) -> Result<(), i32> {
            self.obj = Some((obj.to_owned(), flags));
            Ok(())
        }

        fn register_object_property<T>(
            &mut self,
            obj: &str,
            declaration: &str,
            byte_offset: usize,
        ) -> Result<(), i32> {
            self.fields.push((obj.to_owned(), declaration.to_owned(), byte_offset));
            Ok(())
        }
    }

    #[test]
    fn it_works() {
        let mut engine = ASEngine::default();
        Foo::register(&mut engine);
        assert_eq!(&engine.obj, &Some(("Foo".to_owned(), 0)));
        assert_eq!(&engine.fields, &[
            ("Foo".to_owned(), "int8 baz".to_owned(), 4),
            ("Foo".to_owned(), "float bah".to_owned(), 8),
        ]);
    }
}
