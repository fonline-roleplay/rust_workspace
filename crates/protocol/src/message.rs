use serde::{Deserialize, Serialize};
use std::ffi::CString;

pub mod server_dll_web {
    use super::*;

    pub const HANDSHAKE: u16 = 0xBABA;
    pub const VERSION: u16 = 2;

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Deserialize, Serialize)]
    pub enum ServerDllToWeb {
        PlayerConnected(u32),
        PlayerAuth(u32),
    }

    #[repr(C)]
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum ServerWebToDll {
        UpdateCharLeaf { id: u32, ver: u32, secret: u32 },
        SendKeyToPlayer(u32, [u32; 3]),
        SendConfig { player_id: u32, url: CString },
        Nop,
    }
}

pub mod client_dll_overlay {
    use super::*;

    pub const HANDSHAKE: u16 = 0xB00B;
    pub const VERSION: u16 = 4;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum OverlayToClientDll {
        Nop,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum ClientDllToOverlay {
        UpdateAvatars(Vec<Avatar>),
        OverlayHide(bool),
        Message(Message),
    }

    #[repr(C)]
    #[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
    pub struct Avatar {
        pub char: Char,
        pub pos: Position,
    }

    #[repr(C)]
    #[derive(
        Debug, Clone, Deserialize, Serialize, Copy, Default, PartialEq, Eq, PartialOrd, Ord,
    )]
    pub struct Char {
        pub id: u32,
        pub ver: u32,
        pub secret: u32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
    pub struct Position {
        pub x: i32,
        pub y: i32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    pub struct Message {
        pub text: String,
        pub say_type: fo_defines::Say,
        pub cr_id: u32,
        pub delay: u32,
        pub name: Option<String>,
    }
}
