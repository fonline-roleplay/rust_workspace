#[cfg(not(target_arch = "i686"))]
compile_error!("Only i686 arch is supported.");

#[cfg(feature = "bindings")]
macro_rules! exports {
    (common) => {
        pub mod critter;
        use critter::*;
        pub mod item;
        use item::*;

        pub mod sprite;
        use sprite::*;
        pub mod field;
        use field::*;
        pub mod state;

        use super::angelscript::*;
        use super::num::*;
        use super::opaque::*;
        use super::stl::*;

        stl!(common);
    };
    (server) => {
        exports!(common);
        pub mod map;
        use map::*;
        stl!(server);
    };
    (client) => {
        exports!(common);
        stl!(client);
    };
}

#[cfg(feature = "bindings")]
macro_rules! stl {
    (num) => {
        use std::os::raw::c_int as int;
        pub type UintPair = std_pair<uint, uint>;
        pub type Uint16Pair = std_pair<uint16, uint16>;

        pub type UintVec = std_vector<uint>;
        pub type Uint16Vec = std_vector<uint16>;
        pub type IntVec = std_vector<int>;
        pub type UintPairVec = std_vector<UintPair>;
        pub type Uint16PairVec = std_vector<Uint16Pair>;

        pub type IntSet = std_set<int>;
        pub type UintSet = std_set<uint>;
    };
    (common) => {
        pub type ItemVec = std_vector<*mut Item>;
    };
    (server) => {
        pub type NpcPlaneVec = std_vector<*mut NpcPlane>;
        pub type CrVec = std_vector<*mut Critter>;
        pub type ClVec = std_vector<*mut Client>;
        pub type PcVec = std_vector<*mut Npc>;
        pub type MapObjectVec = std_vector<*mut MapObject>;
        pub type MapVec = std_vector<*mut Map>;
        pub type LocVec = std_vector<*mut Location>;

        pub type CritterTimeEventVec = std_vector<CritterTimeEvent>;
        pub type EntiresVec = std_vector<MapEntire>;
        pub type SceneryToClientVec = std_vector<SceneryToClient>;
        pub type ProtoMap_TileVec = std_vector<ProtoMap_Tile>;

        pub type CrMap = std_map<uint, *mut Critter>;
    };
    (client) => {
        pub type CrClVec = std_vector<*mut CritterCl>;
        pub type Field_TileVec = std_vector<Field_Tile>;
    };
}

#[cfg(all(feature = "bindings", feature = "r357"))]
macro_rules! exports_r357 {
    () => {
        #[cfg(feature = "server")]
        pub mod server {
            exports!(server);
        }

        #[cfg(feature = "client")]
        pub mod client {
            exports!(client);
        }
        pub mod angelscript;
        pub mod num;
        pub mod opaque;
        pub mod stl {
            use std::cell::UnsafeCell;
            use std::marker::PhantomData;
            pub struct std_vector<V>([u32; 3], PhantomData<UnsafeCell<V>>);
            pub struct std_map<K, V>(
                [u32; 6],
                PhantomData<UnsafeCell<K>>,
                PhantomData<UnsafeCell<V>>,
            );
            pub struct std_set<V>([u32; 6], PhantomData<UnsafeCell<V>>);
            pub struct std_pair<A, B>(A, B);
            pub struct std_string([u32; 6]);

            pub type stlp_std_string = std_string;
            pub type stlp_std_vector<V> = std_vector<V>;

            use super::num::*;
            stl!(num);
        }
        use stl::*;
    };
}

#[cfg(all(feature = "bindings", feature = "r476"))]
macro_rules! exports_r476 {
    () => {
        #[cfg(feature = "server")]
        pub mod server {
            exports!(server);
        }

        #[cfg(feature = "client")]
        pub mod client {
            exports!(client);
        }
        pub mod angelscript;
        pub mod num;
        pub mod opaque;
        pub mod stl {
            use std::cell::UnsafeCell;
            use std::marker::PhantomData;
            pub struct std_vector<V>([u32; 4], PhantomData<UnsafeCell<V>>);
            pub struct std_map<K, V>(
                [u32; 4],
                PhantomData<UnsafeCell<K>>,
                PhantomData<UnsafeCell<V>>,
            );
            pub struct std_set<V>([u32; 4], PhantomData<UnsafeCell<V>>);
            pub struct std_pair<A, B>(A, B);
            pub struct std_string([u32; 7]);

            use super::num::*;
            stl!(num);
        }
        use stl::*;
    };
}

#[cfg(feature = "bindings")]
pub mod generated {
    #[allow(bad_style, dead_code)]
    #[cfg(feature = "r357")]
    pub mod r357 {
        exports_r357!();
    }

    #[allow(bad_style, dead_code)]
    #[cfg(feature = "r476")]
    pub mod r476 {
        exports_r476!();
    }
}

#[cfg(feature = "impl_engine")]
pub mod engine;

#[cfg(feature = "impl_state")]
pub mod state;
