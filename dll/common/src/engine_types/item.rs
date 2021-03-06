use crate::{
    engine_types::stl::{stlp_std_allocator, stlp_std_vector},
    primitives::*,
};

pub type ItemVec = stlp_std_vector<*mut Item, stlp_std_allocator>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Item {
    pub Id: uint,
    pub Proto: *const ProtoItem,
    pub From: ::std::os::raw::c_int,
    pub Accessory: uint8,
    pub ViewPlaceOnMap: bool,
    pub Reserved0: int16,
    pub __bindgen_anon_1: Item__bindgen_ty_1,
    pub Data: Item__Data,
    pub RefCounter: int16,
    pub IsNotValid: bool,
}

impl Validate for Item {
    fn is_valid(&self) -> bool {
        !self.IsNotValid
    }
}

impl Item {
    pub fn proto(&self) -> Option<&ProtoItem> {
        unsafe { std::mem::transmute(self.Proto) }
    }
    pub fn get_deterioration(&self) -> u16 {
        self.Data.Deterioration
    }
    pub fn get_deterioration_proc(&self) -> u16 {
        const MAX_DETERIORATION: u16 = 10000;
        let proc = self.get_deterioration() * 100 / MAX_DETERIORATION;
        proc.min(100)
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union Item__bindgen_ty_1 {
    pub AccHex: Item__bindgen_ty_1__bindgen_ty_1,
    pub AccCritter: Item__bindgen_ty_1__bindgen_ty_2,
    pub AccContainer: Item__bindgen_ty_1__bindgen_ty_3,
    pub AccBuffer: [::std::os::raw::c_char; 8usize],
    _bindgen_union_align: [u32; 2usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Item__bindgen_ty_1__bindgen_ty_1 {
    pub MapId: uint,
    pub HexX: uint16,
    pub HexY: uint16,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Item__bindgen_ty_1__bindgen_ty_2 {
    pub Id: uint,
    pub Slot: uint8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Item__bindgen_ty_1__bindgen_ty_3 {
    pub ContainerId: uint,
    pub StackId: uint,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Item__Data {
    pub SortValue: uint16,
    pub Info: uint8,
    pub Indicator: uint8,
    pub PicMapHash: uint,
    pub PicInvHash: uint,
    pub AnimWaitBase: uint16,
    pub AnimStay: [uint8; 2usize],
    pub AnimShow: [uint8; 2usize],
    pub AnimHide: [uint8; 2usize],
    pub Flags: uint,
    pub Rate: uint8,
    pub LightIntensity: int8,
    pub LightDistance: uint8,
    pub LightFlags: uint8,
    pub LightColor: uint,
    pub ScriptId: uint16,
    pub TrapValue: int16,
    pub Count: uint,
    pub Cost: uint,
    pub ScriptValues: [::std::os::raw::c_int; 10usize],
    pub BrokenFlags: uint8,
    pub BrokenCount: uint8,
    pub Deterioration: uint16,
    pub AmmoPid: uint16,
    pub AmmoCount: uint16,
    pub LockerId: uint,
    pub LockerCondition: uint16,
    pub LockerComplexity: uint16,
    pub HolodiskNumber: uint,
    pub RadioChannel: uint16,
    pub RadioFlags: uint16,
    pub RadioBroadcastSend: uint8,
    pub RadioBroadcastRecv: uint8,
    pub Charge: uint16,
    pub OffsetX: int16,
    pub OffsetY: int16,
    pub Dir: int16,
    pub Reserved: [::std::os::raw::c_char; 2usize],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ProtoItem {
    pub ProtoId: uint16,
    pub Type: ::std::os::raw::c_int,
    pub PicMap: uint,
    pub PicInv: uint,
    pub Flags: uint,
    pub Stackable: bool,
    pub Deteriorable: bool,
    pub GroundLevel: bool,
    pub Corner: ::std::os::raw::c_int,
    pub Dir: ::std::os::raw::c_int,
    pub Slot: uint8,
    pub Weight: uint,
    pub Volume: uint,
    pub Cost: uint,
    pub StartCount: uint,
    pub SoundId: uint8,
    pub Material: uint8,
    pub LightFlags: uint8,
    pub LightDistance: uint8,
    pub LightIntensity: int8,
    pub LightColor: uint,
    pub DisableEgg: bool,
    pub AnimWaitBase: uint16,
    pub AnimWaitRndMin: uint16,
    pub AnimWaitRndMax: uint16,
    pub AnimStay: [uint8; 2usize],
    pub AnimShow: [uint8; 2usize],
    pub AnimHide: [uint8; 2usize],
    pub OffsetX: int16,
    pub OffsetY: int16,
    pub SpriteCut: uint8,
    pub DrawOrderOffsetHexY: int8,
    pub RadioChannel: uint16,
    pub RadioFlags: uint16,
    pub RadioBroadcastSend: uint8,
    pub RadioBroadcastRecv: uint8,
    pub IndicatorStart: uint8,
    pub IndicatorMax: uint8,
    pub HolodiskNum: uint,
    pub StartValue: [::std::os::raw::c_int; 10usize],
    pub BlockLines: [uint8; 50usize],
    pub ChildPids: [uint16; 5usize],
    pub ChildLines: [[uint8; 6usize]; 5usize],
    pub MagicPower: ::std::os::raw::c_int,
    pub Unused: [uint8; 96usize],
    pub Armor_CrTypeMale: uint,
    pub Armor_CrTypeFemale: uint,
    pub Armor_AC: ::std::os::raw::c_int,
    pub Armor_Perk: uint,
    pub Armor_DRNormal: ::std::os::raw::c_int,
    pub Armor_DRLaser: ::std::os::raw::c_int,
    pub Armor_DRFire: ::std::os::raw::c_int,
    pub Armor_DRPlasma: ::std::os::raw::c_int,
    pub Armor_DRElectr: ::std::os::raw::c_int,
    pub Armor_DREmp: ::std::os::raw::c_int,
    pub Armor_DRExplode: ::std::os::raw::c_int,
    pub Armor_DTNormal: ::std::os::raw::c_int,
    pub Armor_DTLaser: ::std::os::raw::c_int,
    pub Armor_DTFire: ::std::os::raw::c_int,
    pub Armor_DTPlasma: ::std::os::raw::c_int,
    pub Armor_DTElectr: ::std::os::raw::c_int,
    pub Armor_DTEmp: ::std::os::raw::c_int,
    pub Armor_DTExplode: ::std::os::raw::c_int,
    pub Armor_Unused: [uint8; 28usize],
    pub Weapon_DmgType: [::std::os::raw::c_int; 3usize],
    pub Weapon_Anim2: [uint; 3usize],
    pub Weapon_DmgMin: [::std::os::raw::c_int; 3usize],
    pub Weapon_DmgMax: [::std::os::raw::c_int; 3usize],
    pub Weapon_Effect: [uint16; 3usize],
    pub Weapon_Remove: [bool; 3usize],
    pub Weapon_ReloadAp: uint,
    pub Weapon_UnarmedCriticalBonus: ::std::os::raw::c_int,
    pub Weapon_CriticalFailture: uint,
    pub Weapon_UnarmedArmorPiercing: bool,
    pub Weapon_Unused: [uint8; 27usize],
    pub Ammo_AcMod: ::std::os::raw::c_int,
    pub Ammo_DrMod: ::std::os::raw::c_int,
    pub Ammo_DmgMult: uint,
    pub Ammo_DmgDiv: uint,
    pub Food_Thirst: uint16,
    pub Food_Restore: uint16,
    pub Food_Flags: uint,
    pub Wait_Time_0: uint16,
    pub Wait_Time_1: uint16,
    pub Wait_Time_2: uint16,
    pub Wait_Time_3: uint16,
    pub Item_UseAp: uint16,
    pub UnusedEnd: [uint8; 166usize],
    pub Weapon_IsUnarmed: bool,
    pub Weapon_UnarmedTree: ::std::os::raw::c_int,
    pub Weapon_UnarmedPriority: ::std::os::raw::c_int,
    pub Weapon_UnarmedMinAgility: ::std::os::raw::c_int,
    pub Weapon_UnarmedMinUnarmed: ::std::os::raw::c_int,
    pub Weapon_UnarmedMinLevel: ::std::os::raw::c_int,
    pub Weapon_Anim1: uint,
    pub Weapon_MaxAmmoCount: uint,
    pub Weapon_Caliber: ::std::os::raw::c_int,
    pub Weapon_DefaultAmmoPid: uint16,
    pub Weapon_MinStrength: ::std::os::raw::c_int,
    pub Weapon_Perk: ::std::os::raw::c_int,
    pub Weapon_ActiveUses: uint,
    pub Weapon_Skill: [::std::os::raw::c_int; 3usize],
    pub Weapon_PicUse: [uint; 3usize],
    pub Weapon_MaxDist: [uint; 3usize],
    pub Weapon_Round: [uint; 3usize],
    pub Weapon_ApCost: [uint; 3usize],
    pub Weapon_Aim: [bool; 3usize],
    pub Weapon_SoundId: [uint8; 3usize],
    pub Ammo_Caliber: ::std::os::raw::c_int,
    pub Door_NoBlockMove: bool,
    pub Door_NoBlockShoot: bool,
    pub Door_NoBlockLight: bool,
    pub Container_Volume: uint,
    pub Container_CannotPickUp: bool,
    pub Container_MagicHandsGrnd: bool,
    pub Container_Changeble: bool,
    pub Locker_Condition: uint16,
    pub Grid_Type: ::std::os::raw::c_int,
    pub Car_Speed: uint,
    pub Car_Passability: uint,
    pub Car_DeteriorationRate: uint,
    pub Car_CrittersCapacity: uint,
    pub Car_TankVolume: uint,
    pub Car_MaxDeterioration: uint,
    pub Car_FuelConsumption: uint,
    pub Car_Entrance: uint,
    pub Car_MovementType: uint,
}
