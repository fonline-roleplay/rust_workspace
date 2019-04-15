#![cfg(windows)]

use tnf_common::dll_main;

dll_main!({});

mod critter_info;
mod critters_db;
mod hooks;
mod templates;
mod webserver;
