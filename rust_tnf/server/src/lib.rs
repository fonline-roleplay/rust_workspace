#![cfg(windows)]

use tnf_common::dll_main;

dll_main!({});

pub mod critters_db;
mod hooks;
pub mod webserver;
