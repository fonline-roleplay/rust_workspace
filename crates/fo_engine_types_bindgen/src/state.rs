#[cfg(feature = "r357")]
mod r357 {
    #[cfg(feature = "client")]
    mod client {
        use crate::generated::r357::client::state::GameOptions;
        impl GameOptions {
            fn foo(&self) {}
        }
    }
}

#[cfg(test)]
#[test]
fn state_test() {}
