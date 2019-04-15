use crate::{
    defines::param::CritterParam,
    engine_types::{
        item::{Item, ItemVec},
        primitives::{
            int16, stlp_std_allocator, stlp_std_vector, uint, uint16, uint8, IntVec, Spinlock,
            SyncObj, Uint16PairVec, Uint16Vec, UintSet,
        },
        ScriptString,
    },
};

mod critter_info {
    use super::*;

    macro_rules! copycat {
        (; $cr:ident $field:ident $conv:ident
        ) => {
            $cr.$field.$conv()
        };
        (; $cr:ident $field:ident
        ) => {
            $cr.$field.clone()
        };
        (
            $tyfrom:path >> $tyto:ident;
            $($visy:vis $field:ident : $typ:ty
                $(=> $conv:ident)?
            ,)*
        ) => {
            #[allow(non_snake_case, dead_code)]
            #[derive(Clone)]
            pub struct $tyto {
                $($visy $field : $typ),*
            }

            impl $tyto {
                pub fn new(cr: &$tyfrom) -> Self {
                    Self {
                        $( $field: copycat!(; cr $field $($conv)?) ),*
                    }
                }
            }
        }
    }

    copycat!( super::Critter >> CritterInfo;
        pub Id: uint,
        pub HexX: uint16,
        pub HexY: uint16,
        pub Dir: uint8,
        pub MapId: uint,
        pub Params: [i32; 1000],
        pub NameStr: String => string ,
    );
}
pub use critter_info::CritterInfo;

impl CritterParam for CritterInfo {
    fn params_all(&self) -> &[i32] {
        &self.Params
    }
}
impl CritterParam for Critter {
    fn params_all(&self) -> &[i32] {
        &self.Params
    }
}
impl CritterParam for CritterCl {
    fn params_all(&self) -> &[i32] {
        &self.Params
    }
}

pub type CrVec = stlp_std_vector<*mut Critter, stlp_std_allocator>;

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
