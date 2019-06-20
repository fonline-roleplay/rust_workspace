#[allow(non_snake_case, non_camel_case_types, unused_variables, dead_code)]
pub mod stl;

#[allow(non_camel_case_types, non_snake_case)] //, dead_code
mod string;
pub use self::string::ScriptString;

#[allow(non_camel_case_types, non_snake_case)] //, dead_code
mod array;
pub use self::array::ScriptArray;

#[allow(non_snake_case)]
pub mod item;

#[allow(non_snake_case, dead_code)]
pub mod critter;

#[allow(non_camel_case_types, non_snake_case, dead_code)]
pub mod game_options;

#[cfg(feature = "server")]
pub mod mutual {
/*
    #[no_mangle]
    #[allow(non_snake_case)]
    pub extern "C" fn SERVER() {
        // FOnline needs this to check if this is correct dll for server
    }
*/
    pub type CritterMutual = crate::engine_types::critter::Critter;
}

#[cfg(feature = "client")]
pub mod mutual {
    #[no_mangle]
    #[allow(non_snake_case)]
    pub extern "C" fn CLIENT() {
        // FOnline needs this to check if this is correct dll for client
    }

    pub type CritterMutual = crate::engine_types::critter::CritterCl;
}

#[cfg(not(any(feature = "server", feature = "client")))]
pub mod mutual {
    pub type CritterMutual = crate::engine_types::critter::Critter;
}
