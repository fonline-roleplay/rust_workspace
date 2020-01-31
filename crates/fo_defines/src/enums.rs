#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

#[repr(C)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Say {
    Normal,
    NormalOnHead,
    Shout,
    ShoutOnHead,
    Emote,
    EmoteOnHead,
    Whisper,
    WhisperOnHead,
    Social,
    Radio,
    NetMsg,
    Dialog,
    Append,
    EncounterAny,
    EncounterRt,
    EncounterTb,
    FixResult,
    DialogboxText,
    DialogboxButton(u8),
    SayTitle,
    SayText,
    FlashWindow,
    Unknown,
}

#[repr(C)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageType {
    Uncalled,
    Normal,
    Laser,
    Fire,
    Plasma,
    Electric,
    Emp,
    Explosion,
    Unknown,
}
