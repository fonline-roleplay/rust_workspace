#![cfg(windows)]

use tnf_common::{
    dll_main,
};

dll_main!({});

pub mod critters_db;
pub mod webserver;
mod hooks;