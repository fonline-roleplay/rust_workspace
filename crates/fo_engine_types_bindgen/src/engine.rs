#[cfg(feature = "r357")]
pub mod r357 {
    #[cfg(feature = "client")]
    pub mod client {
        use crate::generated::r357::client::{critter, item};
        use fo_engine_types::impl_engine;

        pub struct Client;
        impl_engine!(
            impl Engine for Client {
                type Item = item::Item;
                type ItemProto = item::ProtoItem;
                type Critter = critter::CritterCl;
                type Param = u32;
            }
        );
    }

    #[cfg(feature = "server")]
    pub mod server {
        use crate::generated::r357::server::{critter, item};
        use fo_engine_types::impl_engine;

        pub struct Server;
        impl_engine!(
            impl Engine for Server {
                type Item = item::Item;
                type ItemProto = item::ProtoItem;
                type Critter = critter::Critter;
                type Param = u32;
            }
        );
    }
}

#[cfg(feature = "r476")]
pub mod r476 {
    #[cfg(feature = "client")]
    pub mod client {
        use crate::generated::r476::client::{critter, item};
        use fo_engine_types::impl_engine;

        pub struct Client;
        impl_engine!(
            impl Engine for Client {
                type Item = item::Item;
                type ItemProto = item::ProtoItem;
                type Critter = critter::CritterCl;
                type Param = u32;
            }
        );
    }

    #[cfg(feature = "server")]
    pub mod server {
        use crate::generated::r476::server::{critter, item};
        use fo_engine_types::impl_engine;

        pub struct Server;
        impl_engine!(
            impl Engine for Server {
                type Item = item::Item;
                type ItemProto = item::ProtoItem;
                type Critter = critter::Critter;
                type Param = u32;
            }
        );
    }
}

#[cfg(test)]
#[test]
fn engine_test() {}
