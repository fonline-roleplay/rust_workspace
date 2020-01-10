use fo_defines::{FoDefines, Say};

#[allow(dead_code)]
pub mod fos;

#[allow(dead_code, non_camel_case_types)]
pub mod param;

pub struct Fo4Rp;

impl FoDefines for Fo4Rp {
    fn decode_say(value: u32) -> Say {
        use fos::*;
        use Say::*;
        const SAY_DIALOGBOX_BUTTON_COUNT: u32 = 20;
        const SAY_DIALOGBOX_BUTTON_LAST: u32 =
            SAY_DIALOGBOX_BUTTON + SAY_DIALOGBOX_BUTTON_COUNT - 1;
        match value {
            SAY_NORM => Normal,
            SAY_NORM_ON_HEAD => NormalOnHead,
            SAY_SHOUT => Shout,
            SAY_SHOUT_ON_HEAD => ShoutOnHead,
            SAY_EMOTE => Emote,
            SAY_EMOTE_ON_HEAD => EmoteOnHead,
            SAY_WHISP => Whisper,
            SAY_WHISP_ON_HEAD => WhisperOnHead,
            SAY_SOCIAL => Social,
            SAY_RADIO => Radio,
            SAY_NETMSG => NetMsg,
            SAY_DIALOG => Dialog,
            SAY_APPEND => Append,
            SAY_ENCOUNTER_ANY => EncounterAny,
            SAY_ENCOUNTER_RT => EncounterRt,
            SAY_ENCOUNTER_TB => EncounterTb,
            SAY_FIX_RESULT => FixResult,
            SAY_DIALOGBOX_TEXT => DialogboxText,
            SAY_DIALOGBOX_BUTTON..=SAY_DIALOGBOX_BUTTON_LAST => {
                DialogboxButton((value - SAY_DIALOGBOX_BUTTON) as u8)
            }
            SAY_SAY_TITLE => SayTitle,
            SAY_SAY_TEXT => SayText,
            SAY_FLASH_WINDOW => FlashWindow,
            _ => Unknown,
        }
    }
}

impl fo_defines::ParamIndex for param::Param {
    fn index(&self) -> usize {
        *self as usize
    }
}
