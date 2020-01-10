use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
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

pub trait FoDefines {
    fn decode_say(value: u32) -> Say;
}

pub trait ParamIndex {
    fn index(&self) -> usize;
}

pub trait CritterParam<P: ParamIndex> {
    fn param(&self, p: P) -> i32 {
        self.params_all()[p.index()]
    }
    fn uparam(&self, p: P) -> u32 {
        self.param(p) as u32
    }
    fn bparam(&self, p: P) -> bool {
        self.param(p) != 0
    }
    fn params_range(&self, range: std::ops::Range<P>) -> &[i32] {
        &self.params_all()[range.start.index()..range.end.index()]
    }
    fn params_range_inc(&self, range: std::ops::RangeInclusive<P>) -> &[i32] {
        &self.params_all()[range.start().index()..=range.end().index()]
    }
    fn params_all(&self) -> &[i32];
}

#[cfg(feature = "param_mut")]
pub trait CritterParamMut<P: ParamIndex> {
    fn param_mut(&mut self, p: P) -> &mut i32 {
        &mut self.params_all_mut()[p.index()]
    }
    fn set_uparam(&mut self, p: P, val: u32) {
        *self.param_mut(p) = val as i32;
    }
    fn params_all_mut(&mut self) -> &mut [i32];
}
