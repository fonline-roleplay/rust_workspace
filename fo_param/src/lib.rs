pub mod black_magic;
pub mod param_types;

//use std::concat;

#[cfg(test)]
mod test {
    use formula::prelude::{tools::PartFormula, *};

    use super::{impl_base, impl_calc, impl_ext, impl_param};
    struct Foo<'a>(&'a [i32]);
    impl<'a> super::param_types::ParamGet for &'a Foo<'a> {
        type Index = u16;
        fn get_param(&self, param: Self::Index) -> i32 {
            self.0[param as usize]
        }
    }
    impl_param!(
        { 
            lt: ('a),
            data: &'a Foo<'a>,
            with_args: (impl_base!("База"), impl_ext!("Эффект")),
        },
        (Strength, "Сила", 0, 32),
        (Strength2, "Сила2", 0, 32),
    );

    #[derive(Copy, Clone)]
    struct Bar([i32; 100]);
    impl super::param_types::ParamGet for Bar {
        type Index = u16;
        fn get_param(&self, param: Self::Index) -> i32 {
            self.0[param as usize]
        }
    }
    impl_param!(
        { 
            lt: (), data: Bar,
            with_args: (impl_base!("База"), impl_ext!("Эффект")),
        },
        (Strength3, "Сила", 0, 32),
        (Strength4, "Сила2", 0, 32),
    );
    /*
    impl_param!(
        (cfg, <'a>, &'a Foo<'a>, impl_base!("База"), impl_ext!("Эффект"), impl_calc!()),
        (Strength, "Сила", 0, 32, (1, 10)),
        (Strength2, "Сила2", 0, 32, (1, 10)),
    );
    */
}
