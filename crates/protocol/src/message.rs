use serde::{Deserialize, Serialize};
use std::ffi::CString;

pub mod server_dll_web {
    use super::*;

    pub const HANDSHAKE: u16 = 0xBABA;
    pub const VERSION: u16 = 5;
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum ServerDllToWeb {
        PlayerConnected(u32),
        PlayerAuth(u32),
        Status(ServerStatus),
        DiscordSendMessage{channel: String, text: String},
    }
    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
    pub enum DayTime {
        Morning,
        Day,
        Evening,
        Night,
    }
    impl DayTime {
        pub fn from_hour(hour: u16) -> Self {
            match hour {
                0..=5 => DayTime::Night,
                6..=11 => DayTime::Morning,
                12..=16 => DayTime::Day,
                17..=20 => DayTime::Evening,
                _ => DayTime::Night,
            }
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
    pub struct ServerStatus {
        pub connections: u32,
        pub day_time: DayTime,
    }
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum ServerWebToDll {
        UpdateCharLeaf { id: u32, ver: u32, secret: u32 },
        SendKeyToPlayer(u32, [u32; 3]),
        SendConfig { player_id: u32, url: CString },
        StartGame { player_id: u32 },
        Nop,
    }
}

pub mod client_dll_overlay {
    use super::*;

    pub const HANDSHAKE: u16 = 0xB00B;
    pub const VERSION: u16 = 5;

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
        Debug, Clone, Deserialize, Serialize, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash,
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
        pub masked: bool,
    }
}
