use crate::{
    engine_types::{
        critter::{ClVec, CrVec, PcVec},
        item::ItemVec,
        stl::{
            stlp_std_allocator, stlp_std_vector, Mutex, SyncObj, Uint16Vec, UintPairVec, UintVec,
        },
        string::stlp_std_string,
    },
    primitives::*,
};

/*
// Map blocks
#define FH_BLOCK                     BIN8( 00000001 )
#define FH_NOTRAKE                   BIN8( 00000010 )
#define FH_WALL                      BIN8( 00000100 )
#define FH_SCEN                      BIN8( 00001000 )
#define FH_SCEN_GRID                 BIN8( 00010000 )
#define FH_TRIGGER                   BIN8( 00100000 )
#define FH_CRITTER                   BIN8( 00000001 )
#define FH_DEAD_CRITTER              BIN8( 00000010 )
#define FH_ITEM                      BIN8( 00000100 )
#define FH_BLOCK_ITEM                BIN8( 00010000 )
#define FH_NRAKE_ITEM                BIN8( 00100000 )
#define FH_WALK_ITEM                 BIN8( 01000000 )
#define FH_GAG_ITEM                  BIN8( 10000000 )
#define FH_NOWAY                     BIN16( 00010001, 00000001 )
#define FH_NOSHOOT                   BIN16( 00100000, 00000010 )
*/
const FH_CRITTER: u8 = 0b00000001;
const FH_DEAD_CRITTER: u8 = 0b00000010;
const FH_NOWAY: u16 = bin16(0b00010001, 0b00000001);
const FH_NOSHOOT: u16 = bin16(0b00100000, 0b00000010);

const fn bin16(a: u16, b: u16) -> u16 {
    (a << 8) | b
}

impl Map {
    pub fn get_max_hex(&self) -> Hex {
        let proto = self.proto().expect("Map prototype");
        let header = &proto.Header;
        Hex {
            x: header.MaxHexX,
            y: header.MaxHexY,
        }
    }
    pub fn is_hex_critter(&self, hex: Hex) -> bool {
        (self.get_hex_flags(hex) & (FH_CRITTER | FH_DEAD_CRITTER)) != 0
    }
    pub fn is_hex_passed(&self, hex: Hex) -> bool {
        (self.get_hex_flags_with_proto(hex) & FH_NOWAY) == 0
    }
    pub fn is_hex_raked(&self, hex: Hex) -> bool {
        (self.get_hex_flags_with_proto(hex) & FH_NOSHOOT) == 0
    }

    pub fn get_hex_flags(&self, hex: Hex) -> u8 {
        let max_hex = self.get_max_hex();
        assert!(hex.x < max_hex.x && hex.y < max_hex.y);
        let index = hex.y as isize * max_hex.x as isize + hex.x as isize;
        unsafe { *self.HexFlags.offset(index) }
    }

    pub fn get_hex_flags_with_proto(&self, hex: Hex) -> u16 {
        let max_hex = self.get_max_hex();
        assert!(hex.x < max_hex.x && hex.y < max_hex.y);
        let index = hex.y as isize * max_hex.x as isize + hex.x as isize;
        let map_flags = unsafe { *self.HexFlags.offset(index) } as u16;
        let proto_flags =
            unsafe { *self.proto().expect("Map prototype").HexFlags.offset(index) } as u16;
        (map_flags << 8) | proto_flags
    }
    pub fn proto(&self) -> Option<&ProtoMap> {
        unsafe { std::mem::transmute(self.Proto) }
    }
    pub fn proto_id(&self) -> u16 {
        //self.proto().expect("Map prototype").Pid
        self.Data.MapPid
    }
}

pub type MapVec = stlp_std_vector<*mut Map, stlp_std_allocator>;

#[repr(C)]
pub struct Map {
    pub Sync: SyncObj,
    pub DataLocker: Mutex,
    pub HexFlags: *const uint8,
    pub MapCritters: CrVec,
    pub MapPlayers: ClVec,
    pub MapNpcs: PcVec,
    pub HexItems: ItemVec,
    pub MapLocation: *const Location,
    pub Data: Map__bindgen_ty_1,
    pub Proto: *const ProtoMap,
    pub NeedProcess: bool,
    pub FuncId: [uint; 12usize],
    pub LoopEnabled: [uint; 5usize],
    pub LoopLastTick: [uint; 5usize],
    pub LoopWaitTick: [uint; 5usize],
    pub IsTurnBasedOn: bool,
    pub TurnBasedEndTick: uint,
    pub TurnSequenceCur: ::std::os::raw::c_int,
    pub TurnSequence: UintVec,
    pub IsTurnBasedTimeout: bool,
    pub TurnBasedBeginSecond: uint,
    pub NeedEndTurnBased: bool,
    pub TurnBasedRound: uint,
    pub TurnBasedTurn: uint,
    pub TurnBasedWholeTurn: uint,
    pub IsNotValid: bool,
    pub RefCounter: int16,
}

impl Validate for Map {
    fn is_valid(&self) -> bool {
        !self.IsNotValid
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Map__bindgen_ty_1 {
    pub MapId: uint,
    pub MapPid: uint16,
    pub MapRain: uint8,
    pub IsTurnBasedAviable: bool,
    pub MapTime: ::std::os::raw::c_int,
    pub ScriptId: uint,
    pub MapDayTime: [::std::os::raw::c_int; 4usize],
    pub MapDayColor: [uint8; 12usize],
    pub Reserved: [uint; 20usize],
    pub UserData: [::std::os::raw::c_int; 100usize],
}

#[cfg(feature = "server")]
#[repr(C)]
pub struct ProtoMap {
    pub Header: ProtoMap__bindgen_ty_1,
    pub MObjects: MapObjectVec,
    pub LastObjectUID: uint,
    pub Tiles: ProtoMap_TileVec,

    WallsToSend: SceneryToClientVec,
    SceneriesToSend: SceneryToClientVec,
    HashTiles: uint,
    HashWalls: uint,
    HashScen: uint,
    CrittersVec: MapObjectVec,
    ItemsVec: MapObjectVec,
    SceneriesVec: MapObjectVec,
    GridsVec: MapObjectVec,
    HexFlags: *const uint8,

    pub MapEntires: EntiresVec,
    pub PathType: ::std::os::raw::c_int,
    pub Name: stlp_std_string,
    pub Pid: uint16,
}

#[cfg(not(feature = "server"))]
#[repr(C)]
pub struct ProtoMap {
    pub Header: ProtoMap__bindgen_ty_1,
    pub MObjects: MapObjectVec,
    pub LastObjectUID: uint,
    pub Tiles: ProtoMap_TileVec,
    pub MapEntires: EntiresVec,
    pub PathType: ::std::os::raw::c_int,
    pub Name: stlp_std_string,
    pub Pid: uint16,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ProtoMap__bindgen_ty_1 {
    pub Version: uint,
    pub MaxHexX: uint16,
    pub MaxHexY: uint16,
    pub WorkHexX: ::std::os::raw::c_int,
    pub WorkHexY: ::std::os::raw::c_int,
    pub ScriptModule: [::std::os::raw::c_char; 65usize],
    pub ScriptFunc: [::std::os::raw::c_char; 65usize],
    pub Time: ::std::os::raw::c_int,
    pub NoLogOut: bool,
    pub DayTime: [::std::os::raw::c_int; 4usize],
    pub DayColor: [uint8; 12usize],
    pub HeaderSize: uint16,
    pub Packed: bool,
    pub UnpackedDataLen: uint,
}

pub type EntiresVec = stlp_std_vector<MapEntire, stlp_std_allocator>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MapEntire {
    pub Number: uint,
    pub HexX: uint16,
    pub HexY: uint16,
    pub Dir: uint8,
}

pub type MapObjectVec = stlp_std_vector<*mut MapObject, stlp_std_allocator>;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MapObject {
    pub MapObjType: uint8,
    pub ProtoId: uint16,
    pub MapX: uint16,
    pub MapY: uint16,
    pub Dir: int16,
    pub UID: uint,
    pub ContainerUID: uint,
    pub ParentUID: uint,
    pub ParentChildIndex: uint,
    pub LightRGB: uint,
    pub LightDay: uint8,
    pub LightDirOff: uint8,
    pub LightDistance: uint8,
    pub LightIntensity: int8,
    pub ScriptName: [::std::os::raw::c_char; 26usize],
    pub FuncName: [::std::os::raw::c_char; 26usize],
    pub Reserved: [uint; 7usize],
    pub UserData: [::std::os::raw::c_int; 10usize],
    pub __bindgen_anon_1: MapObject__bindgen_ty_1,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union MapObject__bindgen_ty_1 {
    pub MCritter: MapObject__bindgen_ty_1__bindgen_ty_1,
    pub MItem: MapObject__bindgen_ty_1__bindgen_ty_2,
    pub MScenery: MapObject__bindgen_ty_1__bindgen_ty_3,
    _bindgen_union_align: [u32; 63usize],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MapObject__bindgen_ty_1__bindgen_ty_1 {
    pub Cond: uint8,
    pub Anim1: uint,
    pub Anim2: uint,
    pub ParamIndex: [int16; 40usize],
    pub ParamValue: [::std::os::raw::c_int; 40usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MapObject__bindgen_ty_1__bindgen_ty_2 {
    pub OffsetX: int16,
    pub OffsetY: int16,
    pub AnimStayBegin: uint8,
    pub AnimStayEnd: uint8,
    pub AnimWait: uint16,
    pub InfoOffset: uint8,
    pub PicMapHash: uint,
    pub PicInvHash: uint,
    pub Count: uint,
    pub ItemSlot: uint8,
    pub BrokenFlags: uint8,
    pub BrokenCount: uint8,
    pub Deterioration: uint16,
    pub AmmoPid: uint16,
    pub AmmoCount: uint,
    pub LockerDoorId: uint,
    pub LockerCondition: uint16,
    pub LockerComplexity: uint16,
    pub TrapValue: int16,
    pub Val: [::std::os::raw::c_int; 10usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MapObject__bindgen_ty_1__bindgen_ty_3 {
    pub OffsetX: int16,
    pub OffsetY: int16,
    pub AnimStayBegin: uint8,
    pub AnimStayEnd: uint8,
    pub AnimWait: uint16,
    pub InfoOffset: uint8,
    pub PicMapHash: uint,
    pub PicInvHash: uint,
    pub CanUse: bool,
    pub CanTalk: bool,
    pub TriggerNum: uint,
    pub ParamsCount: uint8,
    pub Param: [::std::os::raw::c_int; 5usize],
    pub ToMapPid: uint16,
    pub ToEntire: uint,
    pub ToDir: uint8,
}

pub type ProtoMap_TileVec = stlp_std_vector<ProtoMap_Tile, stlp_std_allocator>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ProtoMap_Tile {
    pub NameHash: uint,
    pub HexX: uint16,
    pub HexY: uint16,
    pub OffsX: int8,
    pub OffsY: int8,
    pub Layer: uint8,
    pub IsRoof: bool,
}

#[repr(C)]
pub struct Location {
    pub Sync: SyncObj,
    pub LocMaps: MapVec,
    pub Data: Location__bindgen_ty_1,
    pub Proto: *const ProtoLocation,
    pub GeckCount: ::std::os::raw::c_int,
    pub IsNotValid: bool,
    pub RefCounter: int16,
}

impl Validate for Location {
    fn is_valid(&self) -> bool {
        !self.IsNotValid
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Location__bindgen_ty_1 {
    pub LocId: uint,
    pub LocPid: uint16,
    pub WX: uint16,
    pub WY: uint16,
    pub Radius: uint16,
    pub Visible: bool,
    pub GeckVisible: bool,
    pub AutoGarbage: bool,
    pub ToGarbage: bool,
    pub Color: uint,
    pub Reserved3: [uint; 59usize],
}

#[repr(C)]
pub struct ProtoLocation {
    pub IsInit: bool,
    pub LocPid: uint16,
    pub Name: stlp_std_string,
    pub MaxPlayers: uint,
    pub ProtoMapPids: Uint16Vec,
    pub AutomapsPids: Uint16Vec,
    pub Entrance: UintPairVec,
    pub ScriptBindId: ::std::os::raw::c_int,
    pub Radius: uint16,
    pub Visible: bool,
    pub AutoGarbage: bool,
    pub GeckVisible: bool,
}

pub type SceneryToClientVec = stlp_std_vector<SceneryToClient, stlp_std_allocator>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SceneryToClient {
    pub ProtoId: uint16,
    pub Flags: uint8,
    pub Reserved0: uint8,
    pub MapX: uint16,
    pub MapY: uint16,
    pub OffsetX: int16,
    pub OffsetY: int16,
    pub LightColor: uint,
    pub LightDistance: uint8,
    pub LightFlags: uint8,
    pub LightIntensity: int8,
    pub InfoOffset: uint8,
    pub AnimStayBegin: uint8,
    pub AnimStayEnd: uint8,
    pub AnimWait: uint16,
    pub PicMapHash: uint,
    pub Dir: int16,
    pub Reserved1: uint16,
}
