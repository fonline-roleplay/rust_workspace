use crate::{
    engine_types::{
        item::{Item, ItemVec},
        stl::{
            stlp_std_allocator, stlp_std_vector, IntVec, Spinlock, SyncObj, Uint16PairVec,
            Uint16Vec, UintSet,
        },
        ScriptString,
    },
    primitives::*,
};
use fo_defines::CritterParam;
use fo_defines_fo4rp::{fos, param::Param};

impl CritterParam<Param> for Critter {
    fn params_all(&self) -> &[i32] {
        &self.Params
    }
}

#[cfg(feature = "server")]
impl fo_defines::CritterParamMut<Param> for Critter {
    fn params_all_mut(&mut self) -> &mut [i32] {
        &mut self.Params
    }
}
impl CritterParam<Param> for CritterCl {
    fn params_all(&self) -> &[i32] {
        &self.Params
    }
}

pub type CrVec = stlp_std_vector<*mut Critter, stlp_std_allocator>;

impl Critter {
    pub fn hex(&self) -> Hex {
        Hex {
            x: self.HexX,
            y: self.HexY,
        }
    }
    pub fn is_dead(&self) -> bool {
        self.Cond == fos::COND_DEAD
    }
    pub fn is_npc(&self) -> bool {
        self.CritterIsNpc
    }
    pub fn is_player(&self) -> bool {
        !self.CritterIsNpc
    }
    pub fn have_gm_vision(&self) -> bool {
        self.uparam(Param::ST_ACCESS_LEVEL) >= fos::ACCESS_MODER
            && self.uparam(Param::QST_VISION) > 0
    }
}

#[repr(C)]
pub struct Critter {
    pub Id: uint,
    pub HexX: uint16,
    pub HexY: uint16,
    pub WorldX: uint16,
    pub WorldY: uint16,
    pub BaseType: uint,
    pub Dir: uint8,
    pub Cond: uint8,
    pub ReservedCE: uint8,
    pub Reserved0: uint8,
    pub ScriptId: uint,
    pub ShowCritterDist1: uint,
    pub ShowCritterDist2: uint,
    pub ShowCritterDist3: uint,
    pub Reserved00: uint16,
    pub Multihex: int16,
    pub GlobalGroupUid: uint,
    pub LastHexX: uint16,
    pub LastHexY: uint16,
    pub Reserved1: [uint; 4usize],
    pub MapId: uint,
    pub MapPid: uint16,
    pub Reserved2: uint16,
    pub Params: [::std::os::raw::c_int; 1000usize],
    pub Anim1Life: uint,
    pub Anim1Knockout: uint,
    pub Anim1Dead: uint,
    pub Anim2Life: uint,
    pub Anim2Knockout: uint,
    pub Anim2Dead: uint,
    pub Anim2KnockoutEnd: uint,
    pub Reserved3: [uint; 3usize],
    pub Lexems: [::std::os::raw::c_char; 128usize],
    pub Reserved4: [uint; 8usize],
    pub ClientToDelete: bool,
    pub Reserved5: uint8,
    pub Reserved6: uint16,
    pub Temp: uint,
    pub Reserved8: uint16,
    pub HoloInfoCount: uint16,
    pub HoloInfo: [uint; 250usize],
    pub Reserved9: [uint; 10usize],
    pub Scores: [::std::os::raw::c_int; 50usize],
    pub GlobalMapMoveCounter: uint,
    pub UserData: [uint8; 396usize],
    pub HomeMap: uint,
    pub HomeX: uint16,
    pub HomeY: uint16,
    pub HomeDir: uint8,
    pub Reserved11: uint8,
    pub ProtoId: uint16,
    pub Reserved12: uint,
    pub Reserved13: uint,
    pub Reserved14: uint,
    pub Reserved15: uint,
    pub IsDataExt: bool,
    pub Reserved16: uint8,
    pub Reserved17: uint16,
    pub Reserved18: [uint; 8usize],
    pub FavoriteItemPid: [uint16; 4usize],
    pub Reserved19: [uint; 10usize],
    pub EnemyStackCount: uint,
    pub EnemyStack: [uint; 30usize],
    pub Reserved20: [uint; 5usize],
    pub BagCurrentSet: [uint8; 20usize],
    pub BagRefreshTime: int16,
    pub Reserved21: uint8,
    pub BagSize: uint8,
    pub Bag: [Critter__bindgen_ty_1; 50usize],
    pub Reserved22: [uint; 100usize],
    pub DataExt: *mut Critter__bindgen_ty_2,
    pub Sync: SyncObj,
    pub CritterIsNpc: bool,
    pub Flags: uint,
    pub NameStr: ScriptString,
    pub GMapFog: Critter__bindgen_ty_3,
    pub IsRuning: bool,
    pub PrevHexTick: uint,
    pub PrevHexX: uint16,
    pub PrevHexY: uint16,
    pub LockMapTransfers: ::std::os::raw::c_int,
    pub ThisPtr: [*const Critter; 100usize],
    pub AllowedToDownloadMap: uint,
    pub ParamsIsChanged: [bool; 1000usize],
    pub ParamsChanged: IntVec,
    pub ParamLocked: ::std::os::raw::c_int,
    pub VisCr: CrVec,
    pub VisCrSelf: CrVec,
    pub VisCr1: UintSet,
    pub VisCr2: UintSet,
    pub VisCr3: UintSet,
    pub VisItem: UintSet,
    pub VisItemLocker: Spinlock,
    pub ViewMapId: uint,
    pub ViewMapPid: uint16,
    pub ViewMapLook: uint16,
    pub ViewMapHx: uint16,
    pub ViewMapHy: uint16,
    pub ViewMapDir: uint8,
    pub ViewMapLocId: uint,
    pub ViewMapLocEnt: uint,
    pub GroupSelf: *const GlobalMapGroup,
    pub GroupMove: *const GlobalMapGroup,
    pub InvItems: ItemVec,
    pub DefItemSlotHand: *const Item,
    pub DefItemSlotArmor: *const Item,
    pub ItemSlotMain: *const Item,
    pub ItemSlotExt: *const Item,
    pub ItemSlotArmor: *const Item,
    pub FuncId: [::std::os::raw::c_int; 44usize],
    pub KnockoutAp: uint,
    pub NextIntellectCachingTick: uint,
    pub IntellectCacheValue: uint16,
    pub LookCacheValue: uint,
    pub StartBreakTime: uint,
    pub BreakTime: uint,
    pub WaitEndTick: uint,
    pub DisableSend: ::std::os::raw::c_int,
    pub AccessContainerId: uint,
    pub ItemTransferCount: uint,
    pub TryingGoHomeTick: uint,
    pub CrTimeEvents: CritterTimeEventVec,
    pub GlobalIdleNextTick: uint,
    pub ApRegenerationTick: uint,
    pub IsNotValid: bool,
    pub RefCounter: ::std::os::raw::c_int,
}

impl Validate for Critter {
    fn is_valid(&self) -> bool {
        !self.IsNotValid
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Critter__bindgen_ty_1 {
    pub ItemPid: uint,
    pub MinCnt: uint,
    pub MaxCnt: uint,
    pub ItemSlot: uint,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Critter__bindgen_ty_2 {
    pub Reserved23: [uint; 10usize],
    pub GlobalMapFog: [uint8; 2500usize],
    pub Reserved24: uint16,
    pub LocationsCount: uint16,
    pub LocationsId: [uint; 1000usize],
    pub Reserved25: [uint; 40usize],
    pub PlayIp: [uint; 20usize],
    pub PlayPort: [uint16; 20usize],
    pub CurrentIp: uint,
    pub Reserved26: [uint; 29usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Critter__bindgen_ty_3 {
    pub IsAlloc: bool,
    pub Data: *const uint8,
    pub Width: uint,
    pub Height: uint,
    pub WidthB: uint,
}

pub type CritterTimeEventVec = stlp_std_vector<CritterTimeEvent, stlp_std_allocator>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CritterTimeEvent {
    pub FuncNum: uint,
    pub Rate: uint,
    pub NextTime: uint,
    pub Identifier: ::std::os::raw::c_int,
}

#[repr(C)]
#[derive(Debug)]
pub struct GlobalMapGroup {
    pub Group: CrVec,
    pub Rule: *const Critter,
    pub CarId: uint,
    pub CurX: f32,
    pub CurY: f32,
    pub ToX: f32,
    pub ToY: f32,
    pub Speed: f32,
    pub IsSetMove: bool,
    pub TimeCanFollow: uint,
    pub IsMultiply: bool,
    pub ProcessLastTick: uint,
    pub EncounterDescriptor: uint,
    pub EncounterTick: uint,
    pub EncounterForce: bool,
}

pub type CrClVec = stlp_std_vector<*mut CritterCl, stlp_std_allocator>;
#[repr(C)]
pub struct CritterCl {
    pub Id: uint,
    pub Pid: uint16,
    pub HexX: uint16,
    pub HexY: uint16,
    pub Dir: uint8,
    pub Params: [::std::os::raw::c_int; 1000usize],
    pub NameColor: uint,
    pub ContourColor: uint,
    pub LastHexX: Uint16Vec,
    pub LastHexY: Uint16Vec,
    pub Cond: uint8,
    pub Anim1Life: uint,
    pub Anim1Knockout: uint,
    pub Anim1Dead: uint,
    pub Anim2Life: uint,
    pub Anim2Knockout: uint,
    pub Anim2Dead: uint,
    pub Flags: uint,
    pub BaseType: uint,
    pub BaseTypeAlias: uint,
    pub ApRegenerationTick: uint,
    pub Multihex: int16,
    pub Name: ScriptString,
    pub NameOnHead: ScriptString,
    pub Lexems: ScriptString,
    pub Avatar: ScriptString,
    pub PasswordReg: [::std::os::raw::c_char; 31usize],
    pub InvItems: ItemVec,
    pub DefItemSlotHand: *const Item,
    pub DefItemSlotArmor: *const Item,
    pub ItemSlotMain: *const Item,
    pub ItemSlotExt: *const Item,
    pub ItemSlotArmor: *const Item,
    pub ThisPtr: [*const CritterCl; 100usize],
    pub ParamsIsChanged: [bool; 1000usize],
    pub ParamsChanged: IntVec,
    pub ParamLocked: ::std::os::raw::c_int,
    pub IsRuning: bool,
    pub MoveSteps: Uint16PairVec,
}

pub type ClVec = stlp_std_vector<*mut Client, stlp_std_allocator>;

#[repr(C)]
pub struct Client {
    pub _base: Critter,
    pub Name: [::std::os::raw::c_char; 31usize],
    pub PassHash: [::std::os::raw::c_char; 32usize],
    pub Access: uint8,
    pub LanguageMsg: uint,
}

pub type PcVec = stlp_std_vector<*mut Npc, stlp_std_allocator>;

#[repr(C)]
pub struct Npc {
    pub _base: Critter,
    pub NextRefreshBagTick: uint,
    pub AiPlanes: NpcPlaneVec,
    pub Reserved: uint,
}

pub type NpcPlaneVec = stlp_std_vector<*mut NpcPlane, stlp_std_allocator>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NpcPlane {
    pub Type: ::std::os::raw::c_int,
    pub Priority: uint,
    pub Identifier: ::std::os::raw::c_int,
    pub IdentifierExt: uint,
    pub ChildPlane: *const NpcPlane,
    pub IsMove: bool,
    pub __bindgen_anon_1: NpcPlane__bindgen_ty_1,
    pub Move: NpcPlane__bindgen_ty_2,
    pub Assigned: bool,
    pub RefCounter: ::std::os::raw::c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union NpcPlane__bindgen_ty_1 {
    pub Misc: NpcPlane__bindgen_ty_1__bindgen_ty_1,
    pub Attack: NpcPlane__bindgen_ty_1__bindgen_ty_2,
    pub Walk: NpcPlane__bindgen_ty_1__bindgen_ty_3,
    pub Pick: NpcPlane__bindgen_ty_1__bindgen_ty_4,
    pub Buffer: NpcPlane__bindgen_ty_1__bindgen_ty_5,
    _bindgen_union_align: [u32; 8usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NpcPlane__bindgen_ty_1__bindgen_ty_1 {
    pub IsRun: bool,
    pub WaitSecond: uint,
    pub ScriptBindId: ::std::os::raw::c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NpcPlane__bindgen_ty_1__bindgen_ty_2 {
    pub IsRun: bool,
    pub TargId: uint,
    pub MinHp: ::std::os::raw::c_int,
    pub IsGag: bool,
    pub GagHexX: uint16,
    pub GagHexY: uint16,
    pub LastHexX: uint16,
    pub LastHexY: uint16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NpcPlane__bindgen_ty_1__bindgen_ty_3 {
    pub IsRun: bool,
    pub HexX: uint16,
    pub HexY: uint16,
    pub Dir: uint8,
    pub Cut: uint,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NpcPlane__bindgen_ty_1__bindgen_ty_4 {
    pub IsRun: bool,
    pub HexX: uint16,
    pub HexY: uint16,
    pub Pid: uint16,
    pub UseItemId: uint,
    pub ToOpen: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NpcPlane__bindgen_ty_1__bindgen_ty_5 {
    pub Buffer: [uint; 8usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NpcPlane__bindgen_ty_2 {
    pub PathNum: uint,
    pub Iter: uint,
    pub IsRun: bool,
    pub TargId: uint,
    pub HexX: uint16,
    pub HexY: uint16,
    pub Cut: uint,
    pub Trace: uint,
}
