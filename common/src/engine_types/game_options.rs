use crate::{
    engine_types::{
        critter::{CrClVec, Critter, CritterCl},
        item::{Item, ItemVec},
        mutual::CritterMutual,
        stl::{stlp_std_allocator, stlp_std_vector},
        ScriptString,
    },
    primitives::*,
};

#[no_mangle]
pub static mut FOnline: *mut GameOptions = 0usize as *mut GameOptions;

// not very safe
pub fn game_state<'a>() -> Option<&'a GameOptions> {
    let state = unsafe { FOnline };
    if state as usize == 0usize {
        eprintln!("GameOptions is null!");
        None
    } else {
        unsafe { Some(&*state) }
    }
}

#[cfg(feature = "server")]
pub fn critter_change_param(state: &GameOptions, cr: &mut Critter, param: u32) -> bool {
    if let Some(func) = state.CritterChangeParameter {
        unsafe { func(cr as *mut Critter, param) };
        true
    } else {
        false
    }
}

#[cfg(feature = "client")]
pub fn get_drawind_sprites(state: &GameOptions) -> Option<&[Option<&Sprite>]> {
    state.GetDrawingSprites.and_then(|f| {
        let mut count = 0;
        unsafe {
            let ptr = f(&mut count) as *mut Option<&Sprite>;
            if ptr as usize == 0 || count == 0 {
                None
            } else {
                Some(std::slice::from_raw_parts(ptr, count as usize))
            }
        }
    })
}

#[cfg(feature = "client")]
pub fn get_sprite_info<'a>(state: &'a GameOptions, sprite: &Sprite) -> Option<&'a SpriteInfo> {
    state.GetSpriteInfo.and_then(|f| unsafe {
        let spr_id = if sprite.PSprId as usize != 0 {
            *sprite.PSprId
        } else {
            sprite.SprId
        };
        let ptr = f(spr_id);
        if ptr as usize != 0 {
            Some(&*ptr)
        } else {
            None
        }
    })
}

#[cfg(feature = "client")]
pub fn sprite_get_pos(state: &GameOptions, sprite: &Sprite, si: &SpriteInfo) -> (i32, i32) {
    let offs_x = if sprite.OffsX as usize == 0 {
        0
    } else {
        unsafe { *sprite.OffsX }
    };
    let offs_y = if sprite.OffsY as usize == 0 {
        0
    } else {
        unsafe { *sprite.OffsY }
    };
    let x = ((sprite.ScrX - si.Width as i32 / 2 + si.OffsX as i32 + offs_x as i32 + state.ScrOx)
        as f32
        / state.SpritesZoom) as i32;
    let y = ((sprite.ScrY - si.Height as i32 + si.OffsY as i32 + offs_y as i32 + state.ScrOy)
        as f32
        / state.SpritesZoom) as i32;
    (x, y)
}

#[cfg(feature = "client")]
pub fn sprite_get_top(state: &GameOptions, sprite: &Sprite, si: &SpriteInfo) -> (i32, i32) {
    let offs_x = if sprite.OffsX as usize == 0 {
        0
    } else {
        unsafe { *sprite.OffsX }
    };
    let offs_y = if sprite.OffsY as usize == 0 {
        0
    } else {
        unsafe { *sprite.OffsY }
    };
    let x = ((sprite.ScrX + offs_x as i32 + state.ScrOx) as f32 / state.SpritesZoom) as i32;
    let y = ((sprite.ScrY - si.Height as i32 + si.OffsY as i32 + offs_y as i32 + state.ScrOy)
        as f32
        / state.SpritesZoom) as i32;
    (x, y)
}
/*
pub fn get_sprites_hex(state: &GameOptions, hex_x: i32, hex_y: i32) -> Option<impl Iterator<Item=&Sprite>> {
    let all_sprites = get_drawind_sprites(state)?;
    let sprites = all_sprites.iter()
        .filter_map(|s| *s)
        .filter(|s| s.Valid)
        .filter(|s| s.HexX == hex_x && s.HexY == hex_y);
    Some(sprites)
}
*/
#[cfg(feature = "client")]
pub fn get_sprites_dot(state: &GameOptions, dot: i32) -> Vec<&Sprite> {
    if let Some(all_sprites) = get_drawind_sprites(state) {
        all_sprites
            .into_iter()
            .filter_map(|s| *s)
            .filter(|s| s.Valid)
            .filter(|s| s.DrawOrderType == dot)
            .collect()
    } else {
        Vec::new()
    }
}

#[repr(C)]
pub struct GameOptions {
    pub YearStart: uint16,
    pub YearStartFTLo: uint,
    pub YearStartFTHi: uint,
    pub Year: uint16,
    pub Month: uint16,
    pub Day: uint16,
    pub Hour: uint16,
    pub Minute: uint16,
    pub Second: uint16,
    pub FullSecondStart: uint,
    pub FullSecond: uint,
    pub TimeMultiplier: uint16,
    pub GameTimeTick: uint,
    pub DisableTcpNagle: bool,
    pub DisableZlibCompression: bool,
    pub FloodSize: uint,
    pub NoAnswerShuffle: bool,
    pub DialogDemandRecheck: bool,
    pub FixBoyDefaultExperience: uint,
    pub SneakDivider: uint,
    pub LevelCap: uint,
    pub LevelCapAddExperience: bool,
    pub LookNormal: uint,
    pub LookMinimum: uint,
    pub GlobalMapMaxGroupCount: uint,
    pub CritterIdleTick: uint,
    pub TurnBasedTick: uint,
    pub DeadHitPoints: ::std::os::raw::c_int,
    pub Breaktime: uint,
    pub TimeoutTransfer: uint,
    pub TimeoutBattle: uint,
    pub ApRegeneration: uint,
    pub RtApCostCritterWalk: uint,
    pub RtApCostCritterRun: uint,
    pub RtApCostMoveItemContainer: uint,
    pub RtApCostMoveItemInventory: uint,
    pub RtApCostPickItem: uint,
    pub RtApCostDropItem: uint,
    pub RtApCostReloadWeapon: uint,
    pub RtApCostPickCritter: uint,
    pub RtApCostUseItem: uint,
    pub RtApCostUseSkill: uint,
    pub RtAlwaysRun: bool,
    pub TbApCostCritterMove: uint,
    pub TbApCostMoveItemContainer: uint,
    pub TbApCostMoveItemInventory: uint,
    pub TbApCostPickItem: uint,
    pub TbApCostDropItem: uint,
    pub TbApCostReloadWeapon: uint,
    pub TbApCostPickCritter: uint,
    pub TbApCostUseItem: uint,
    pub TbApCostUseSkill: uint,
    pub TbAlwaysRun: bool,
    pub ApCostAimEyes: uint,
    pub ApCostAimHead: uint,
    pub ApCostAimGroin: uint,
    pub ApCostAimTorso: uint,
    pub ApCostAimArms: uint,
    pub ApCostAimLegs: uint,
    pub RunOnCombat: bool,
    pub RunOnTransfer: bool,
    pub GlobalMapWidth: uint,
    pub GlobalMapHeight: uint,
    pub GlobalMapZoneLength: uint,
    pub GlobalMapMoveTime: uint,
    pub BagRefreshTime: uint,
    pub AttackAnimationsMinDist: uint,
    pub WhisperDist: uint,
    pub ShoutDist: uint,
    pub LookChecks: ::std::os::raw::c_int,
    pub LookDir: [uint; 5usize],
    pub LookSneakDir: [uint; 5usize],
    pub LookWeight: uint,
    pub CustomItemCost: bool,
    pub RegistrationTimeout: uint,
    pub AccountPlayTime: uint,
    pub LoggingVars: bool,
    pub ScriptRunSuspendTimeout: uint,
    pub ScriptRunMessageTimeout: uint,
    pub TalkDistance: uint,
    pub NpcMaxTalkers: uint,
    pub MinNameLength: uint,
    pub MaxNameLength: uint,
    pub DlgTalkMinTime: uint,
    pub DlgBarterMinTime: uint,
    pub MinimumOfflineTime: uint,
    pub StartSpecialPoints: ::std::os::raw::c_int,
    pub StartTagSkillPoints: ::std::os::raw::c_int,
    pub SkillMaxValue: ::std::os::raw::c_int,
    pub SkillModAdd2: ::std::os::raw::c_int,
    pub SkillModAdd3: ::std::os::raw::c_int,
    pub SkillModAdd4: ::std::os::raw::c_int,
    pub SkillModAdd5: ::std::os::raw::c_int,
    pub SkillModAdd6: ::std::os::raw::c_int,
    pub AbsoluteOffsets: bool,
    pub SkillBegin: uint,
    pub SkillEnd: uint,
    pub TimeoutBegin: uint,
    pub TimeoutEnd: uint,
    pub KillBegin: uint,
    pub KillEnd: uint,
    pub PerkBegin: uint,
    pub PerkEnd: uint,
    pub AddictionBegin: uint,
    pub AddictionEnd: uint,
    pub KarmaBegin: uint,
    pub KarmaEnd: uint,
    pub DamageBegin: uint,
    pub DamageEnd: uint,
    pub TraitBegin: uint,
    pub TraitEnd: uint,
    pub ReputationBegin: uint,
    pub ReputationEnd: uint,
    pub ReputationLoved: ::std::os::raw::c_int,
    pub ReputationLiked: ::std::os::raw::c_int,
    pub ReputationAccepted: ::std::os::raw::c_int,
    pub ReputationNeutral: ::std::os::raw::c_int,
    pub ReputationAntipathy: ::std::os::raw::c_int,
    pub ReputationHated: ::std::os::raw::c_int,
    pub MapHexagonal: bool,
    pub MapHexWidth: ::std::os::raw::c_int,
    pub MapHexHeight: ::std::os::raw::c_int,
    pub MapHexLineHeight: ::std::os::raw::c_int,
    pub MapTileOffsX: ::std::os::raw::c_int,
    pub MapTileOffsY: ::std::os::raw::c_int,
    pub MapRoofOffsX: ::std::os::raw::c_int,
    pub MapRoofOffsY: ::std::os::raw::c_int,
    pub MapRoofSkipSize: ::std::os::raw::c_int,
    pub MapCameraAngle: f32,
    pub MapSmoothPath: bool,
    pub MapDataPrefix: ScriptString,
    pub Quit: bool,
    pub OpenGLDebug: bool,
    pub AssimpLogging: bool,
    pub MouseX: ::std::os::raw::c_int,
    pub MouseY: ::std::os::raw::c_int,
    pub ScrOx: ::std::os::raw::c_int,
    pub ScrOy: ::std::os::raw::c_int,
    pub ShowTile: bool,
    pub ShowRoof: bool,
    pub ShowItem: bool,
    pub ShowScen: bool,
    pub ShowWall: bool,
    pub ShowCrit: bool,
    pub ShowFast: bool,
    pub ShowPlayerNames: bool,
    pub ShowNpcNames: bool,
    pub ShowCritId: bool,
    pub ScrollKeybLeft: bool,
    pub ScrollKeybRight: bool,
    pub ScrollKeybUp: bool,
    pub ScrollKeybDown: bool,
    pub ScrollMouseLeft: bool,
    pub ScrollMouseRight: bool,
    pub ScrollMouseUp: bool,
    pub ScrollMouseDown: bool,
    pub ShowGroups: bool,
    pub HelpInfo: bool,
    pub DebugInfo: bool,
    pub DebugNet: bool,
    pub DebugSprites: bool,
    pub FullScreen: bool,
    pub VSync: bool,
    pub FlushVal: ::std::os::raw::c_int,
    pub BaseTexture: ::std::os::raw::c_int,
    pub Light: ::std::os::raw::c_int,
    pub Host: ScriptString,
    pub Port: uint,
    pub ProxyType: uint,
    pub ProxyHost: ScriptString,
    pub ProxyPort: uint,
    pub ProxyUser: ScriptString,
    pub ProxyPass: ScriptString,
    pub Name: ScriptString,
    pub ScrollDelay: ::std::os::raw::c_int,
    pub ScrollStep: uint,
    pub ScrollCheck: bool,
    pub FoDataPath: ScriptString,
    pub FixedFPS: ::std::os::raw::c_int,
    pub MsgboxInvert: bool,
    pub ChangeLang: ::std::os::raw::c_int,
    pub DefaultCombatMode: uint8,
    pub MessNotify: bool,
    pub SoundNotify: bool,
    pub AlwaysOnTop: bool,
    pub TextDelay: uint,
    pub DamageHitDelay: uint,
    pub ScreenWidth: ::std::os::raw::c_int,
    pub ScreenHeight: ::std::os::raw::c_int,
    pub MultiSampling: ::std::os::raw::c_int,
    pub MouseScroll: bool,
    pub IndicatorType: ::std::os::raw::c_int,
    pub DoubleClickTime: uint,
    pub RoofAlpha: uint8,
    pub HideCursor: bool,
    pub DisableLMenu: bool,
    pub DisableMouseEvents: bool,
    pub DisableKeyboardEvents: bool,
    pub HidePassword: bool,
    pub PlayerOffAppendix: ScriptString,
    pub CombatMessagesType: ::std::os::raw::c_int,
    pub DisableDrawScreens: bool,
    pub Animation3dSmoothTime: uint,
    pub Animation3dFPS: uint,
    pub RunModMul: ::std::os::raw::c_int,
    pub RunModDiv: ::std::os::raw::c_int,
    pub RunModAdd: ::std::os::raw::c_int,
    pub MapZooming: bool,
    pub SpritesZoom: f32,
    pub SpritesZoomMax: f32,
    pub SpritesZoomMin: f32,
    pub EffectValues: [f32; 10usize],
    pub AlwaysRun: bool,
    pub AlwaysRunMoveDist: ::std::os::raw::c_int,
    pub AlwaysRunUseDist: ::std::os::raw::c_int,
    pub KeyboardRemap: ScriptString,
    pub CritterFidgetTime: uint,
    pub Anim2CombatBegin: uint,
    pub Anim2CombatIdle: uint,
    pub Anim2CombatEnd: uint,
    pub ClientPath: ScriptString,
    pub ServerPath: ScriptString,
    pub ShowCorners: bool,
    pub ShowCuttedSprites: bool,
    pub ShowDrawOrder: bool,
    pub SplitTilesCollection: bool,
    pub CritterChangeParameter:
        ::std::option::Option<unsafe extern "C" fn(cr: *mut Critter, index: uint)>,
    pub CritterTypes: *mut CritterType,
    pub ClientMap: *mut Field,
    pub ClientMapLight: *mut uint8,
    pub ClientMapWidth: uint,
    pub ClientMapHeight: uint,
    pub GetDrawingSprites:
        ::std::option::Option<unsafe extern "C" fn(count: *mut uint) -> *mut *mut Sprite>,
    pub GetSpriteInfo: ::std::option::Option<unsafe extern "C" fn(sprId: uint) -> *mut SpriteInfo>,
    pub GetSpriteColor: ::std::option::Option<
        unsafe extern "C" fn(
            sprId: uint,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            affectZoom: bool,
        ) -> uint,
    >,
    pub IsSpriteHit: ::std::option::Option<
        unsafe extern "C" fn(
            sprite: *mut Sprite,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            checkEgg: bool,
        ) -> bool,
    >,
    pub GetNameByHash:
        ::std::option::Option<unsafe extern "C" fn(hash: uint) -> *const ::std::os::raw::c_char>,
    pub GetHashByName:
        ::std::option::Option<unsafe extern "C" fn(name: *const ::std::os::raw::c_char) -> uint>,
    pub ScriptLoadModule: ::std::option::Option<
        unsafe extern "C" fn(moduleName: *const ::std::os::raw::c_char) -> bool,
    >,
    pub ScriptBind: ::std::option::Option<
        unsafe extern "C" fn(
            moduleName: *const ::std::os::raw::c_char,
            funcDecl: *const ::std::os::raw::c_char,
            temporaryId: bool,
        ) -> uint,
    >,
    pub ScriptPrepare: ::std::option::Option<unsafe extern "C" fn(bindId: uint) -> bool>,
    pub ScriptSetArgInt8: ::std::option::Option<unsafe extern "C" fn(value: int8)>,
    pub ScriptSetArgInt16: ::std::option::Option<unsafe extern "C" fn(value: int16)>,
    pub ScriptSetArgInt: ::std::option::Option<unsafe extern "C" fn(value: ::std::os::raw::c_int)>,
    pub ScriptSetArgInt64: ::std::option::Option<unsafe extern "C" fn(value: int64)>,
    pub ScriptSetArgUInt8: ::std::option::Option<unsafe extern "C" fn(value: uint8)>,
    pub ScriptSetArgUInt16: ::std::option::Option<unsafe extern "C" fn(value: uint16)>,
    pub ScriptSetArgUInt: ::std::option::Option<unsafe extern "C" fn(value: uint)>,
    pub ScriptSetArgUInt64: ::std::option::Option<unsafe extern "C" fn(value: uint64)>,
    pub ScriptSetArgBool: ::std::option::Option<unsafe extern "C" fn(value: bool)>,
    pub ScriptSetArgFloat: ::std::option::Option<unsafe extern "C" fn(value: f32)>,
    pub ScriptSetArgDouble: ::std::option::Option<unsafe extern "C" fn(value: f64)>,
    pub ScriptSetArgObject:
        ::std::option::Option<unsafe extern "C" fn(value: *mut ::std::os::raw::c_void)>,
    pub ScriptSetArgAddress:
        ::std::option::Option<unsafe extern "C" fn(value: *mut ::std::os::raw::c_void)>,
    pub ScriptRunPrepared: ::std::option::Option<unsafe extern "C" fn() -> bool>,
    pub ScriptGetReturnedInt8: ::std::option::Option<unsafe extern "C" fn() -> int8>,
    pub ScriptGetReturnedInt16: ::std::option::Option<unsafe extern "C" fn() -> int16>,
    pub ScriptGetReturnedInt:
        ::std::option::Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub ScriptGetReturnedInt64: ::std::option::Option<unsafe extern "C" fn() -> int64>,
    pub ScriptGetReturnedUInt8: ::std::option::Option<unsafe extern "C" fn() -> uint8>,
    pub ScriptGetReturnedUInt16: ::std::option::Option<unsafe extern "C" fn() -> uint16>,
    pub ScriptGetReturnedUInt: ::std::option::Option<unsafe extern "C" fn() -> uint>,
    pub ScriptGetReturnedUInt64: ::std::option::Option<unsafe extern "C" fn() -> uint64>,
    pub ScriptGetReturnedBool: ::std::option::Option<unsafe extern "C" fn() -> bool>,
    pub ScriptGetReturnedFloat: ::std::option::Option<unsafe extern "C" fn() -> f32>,
    pub ScriptGetReturnedDouble: ::std::option::Option<unsafe extern "C" fn() -> f64>,
    pub ScriptGetReturnedObject:
        ::std::option::Option<unsafe extern "C" fn() -> *mut ::std::os::raw::c_void>,
    pub ScriptGetReturnedAddress:
        ::std::option::Option<unsafe extern "C" fn() -> *mut ::std::os::raw::c_void>,
    pub GetUseApCost: ::std::option::Option<
        unsafe extern "C" fn(cr: *mut CritterMutual, item: *mut Item, mode: uint8) -> uint,
    >,
    pub GetAttackDistantion: ::std::option::Option<
        unsafe extern "C" fn(cr: *mut CritterMutual, item: *mut Item, mode: uint8) -> uint,
    >,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CritterType {
    pub Enabled: bool,
    pub Name: [::std::os::raw::c_char; 64usize],
    pub SoundName: [::std::os::raw::c_char; 64usize],
    pub Alias: uint,
    pub Multihex: uint,
    pub AnimType: ::std::os::raw::c_int,
    pub CanWalk: bool,
    pub CanRun: bool,
    pub CanAim: bool,
    pub CanArmor: bool,
    pub CanRotate: bool,
    pub Anim1: [bool; 37usize],
}

#[repr(C)]
#[derive(Debug)]
pub struct Field {
    pub Crit: *mut CritterCl,
    pub DeadCrits: CrClVec,
    pub ScrX: ::std::os::raw::c_int,
    pub ScrY: ::std::os::raw::c_int,
    pub Tiles: Field_TileVec,
    pub Roofs: Field_TileVec,
    pub Items: ItemVec,
    pub RoofNum: int16,
    pub ScrollBlock: bool,
    pub IsWall: bool,
    pub IsWallSAI: bool,
    pub IsWallTransp: bool,
    pub IsScen: bool,
    pub IsExitGrid: bool,
    pub IsNotPassed: bool,
    pub IsNotRaked: bool,
    pub Corner: uint8,
    pub IsNoLight: bool,
    pub LightValues: [uint8; 3usize],
    pub IsMultihex: bool,
}

pub type Field_TileVec = stlp_std_vector<Field_Tile, stlp_std_allocator>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Field_Tile {
    pub Anim: *mut ::std::os::raw::c_void,
    pub OffsX: int16,
    pub OffsY: int16,
    pub Layer: uint8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Sprite {
    pub DrawOrderType: ::std::os::raw::c_int,
    pub DrawOrderPos: uint,
    pub TreeIndex: uint,
    pub SprId: uint,
    pub PSprId: *mut uint,
    pub HexX: ::std::os::raw::c_int,
    pub HexY: ::std::os::raw::c_int,
    pub ScrX: ::std::os::raw::c_int,
    pub ScrY: ::std::os::raw::c_int,
    pub OffsX: *mut int16,
    pub OffsY: *mut int16,
    pub CutType: ::std::os::raw::c_int,
    pub Parent: *mut Sprite,
    pub Child: *mut Sprite,
    pub CutX: f32,
    pub CutW: f32,
    pub CutTexL: f32,
    pub CutTexR: f32,
    pub Alpha: *mut uint8,
    pub Light: *mut uint8,
    pub EggType: ::std::os::raw::c_int,
    pub ContourType: ::std::os::raw::c_int,
    pub ContourColor: uint,
    pub Color: uint,
    pub FlashMask: uint,
    pub ValidCallback: *mut bool,
    pub Valid: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SpriteInfo {
    pub Surface: *const ::std::os::raw::c_void,
    pub SurfaceUV: [f32; 4usize],
    pub Width: uint16,
    pub Height: uint16,
    pub OffsX: int16,
    pub OffsY: int16,
    pub Effect: *const ::std::os::raw::c_void,
    pub Anim3d: *const ::std::os::raw::c_void,
}
