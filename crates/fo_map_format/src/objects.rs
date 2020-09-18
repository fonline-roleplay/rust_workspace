use crate::prelude::{complete::*, *};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum MapObjectType {
    MAP_OBJECT_CRITTER = 0,
    MAP_OBJECT_ITEM = 1,
    MAP_OBJECT_SCENERY = 2,
}
impl TryFrom<u8> for MapObjectType {
    type Error = &'static str;
    fn try_from(from: u8) -> Result<Self, &'static str> {
        use MapObjectType::*;
        Ok(match from {
            0 => MAP_OBJECT_CRITTER,
            1 => MAP_OBJECT_ITEM,
            2 => MAP_OBJECT_SCENERY,
            _ => return Err("Unsupported map object type"),
        })
    }
}
impl FromStr for MapObjectType {
    type Err = &'static str;
    fn from_str(from: &str) -> Result<Self, &'static str> {
        u8::from_str(from)
            .map_err(|_| "Parse int error")?
            .try_into()
    }
}

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(
    feature = "serde1",
    skip_serializing_none,
    derive(Serialize, Deserialize, SerDebug)
)]
pub struct Relations {
    pub uid: Option<u32>,
    pub container_uid: Option<u32>,
    pub parent_uid: Option<u32>,
    pub parent_child_index: Option<u32>,
}

impl Relations {
    #[allow(dead_code)]
    fn is_none(&self) -> bool {
        self.uid.is_none()
            && self.container_uid.is_none()
            && self.parent_uid.is_none()
            && self.parent_child_index.is_none()
    }
}

pub fn parse_relations<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Relations, E> {
    Ok(parse_struct!(
        i,
        Relations {
            uid: opt_key_int("UID"),
            container_uid: opt_key_int("ContainerUID"),
            parent_uid: opt_key_int("ParentUID"),
            parent_child_index: opt_key_int("ParentChildIndex"),
        }
    ))
}

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(
    feature = "serde1",
    skip_serializing_none,
    derive(Serialize, Deserialize, SerDebug)
)]
pub struct Light {
    pub color: Option<i32>,
    pub day: Option<u8>,
    pub dir_off: Option<u8>,
    pub distance: Option<u8>,
    pub intensity: Option<i8>,
}
impl Light {
    #[allow(dead_code)]
    fn is_none(&self) -> bool {
        self.color.is_none()
            && self.day.is_none()
            && self.dir_off.is_none()
            && self.distance.is_none()
            && self.intensity.is_none()
    }
}

pub fn parse_light<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Light, E> {
    Ok(parse_struct!(
        i,
        Light {
            color: opt_key_int("LightColor"),
            day: opt_key_int("LightDay"),
            dir_off: opt_key_int("LightDirOff"),
            distance: opt_key_int("LightDistance"),
            intensity: opt_key_int("LightIntensity"),
        }
    ))
}

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(
    feature = "serde1",
    skip_serializing_none,
    derive(Serialize, Deserialize, SerDebug)
)]
pub struct Anim<'a> {
    pub offset_x: Option<i16>,
    pub offset_y: Option<i16>,
    pub anim_stay_begin: Option<u8>,
    pub anim_stay_end: Option<u8>,
    pub anim_wait: Option<u16>,
    pub info_offset: Option<u8>,
    pub pic_map_name: Option<&'a str>,
    pub pic_inv_name: Option<&'a str>,
}
impl<'a> Anim<'a> {
    #[allow(dead_code)]
    fn is_none(&self) -> bool {
        self.offset_x.is_none()
            && self.offset_y.is_none()
            && self.anim_stay_begin.is_none()
            && self.anim_stay_end.is_none()
            && self.anim_wait.is_none()
            && self.info_offset.is_none()
            && self.pic_map_name.is_none()
            && self.pic_inv_name.is_none()
    }
}

pub fn parse_anim<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Anim, E> {
    Ok(parse_struct!(
        i,
        Anim {
            offset_x: opt_key_int("OffsetX"),
            offset_y: opt_key_int("OffsetY"),
            anim_stay_begin: opt_key_int("AnimStayBegin"),
            anim_stay_end: opt_key_int("AnimStayEnd"),
            anim_wait: opt_key_int("AnimWait"),
            info_offset: opt_key_int("InfoOffset"),
            pic_map_name: opt_kv("PicMapName", word),
            pic_inv_name: opt_kv("PicInvName", word),
        }
    ))
}

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(
    feature = "serde1",
    skip_serializing_none,
    derive(Serialize, Deserialize, SerDebug)
)]
pub struct Broken {
    pub flags: Option<u8>,
    pub count: Option<u8>,
    pub deterioration: Option<u16>,
}
impl Broken {
    #[allow(dead_code)]
    fn is_none(&self) -> bool {
        self.flags.is_none() && self.count.is_none() && self.deterioration.is_none()
    }
}

pub fn parse_broken<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Broken, E> {
    Ok(parse_struct!(
        i,
        Broken {
            flags: opt_key_int("Item_BrokenFlags"),
            count: opt_key_int("Item_BrokenCount"),
            deterioration: opt_key_int("Item_Deterioration"),
        }
    ))
}

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(
    feature = "serde1",
    skip_serializing_none,
    derive(Serialize, Deserialize, SerDebug)
)]
pub struct Locker {
    pub door_id: Option<u32>,
    pub condition: Option<u16>,
    pub complexity: Option<u16>,
}
impl Locker {
    #[allow(dead_code)]
    fn is_none(&self) -> bool {
        self.door_id.is_none() && self.condition.is_none() && self.complexity.is_none()
    }
}

pub fn parse_locker<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Locker, E> {
    Ok(parse_struct!(
        i,
        Locker {
            door_id: opt_key_int("Item_LockerDoorId"),
            condition: opt_key_int("Item_LockerCondition"),
            complexity: opt_key_int("Item_LockerComplexity"),
        }
    ))
}

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(
    feature = "serde1",
    skip_serializing_none,
    derive(Serialize, Deserialize, SerDebug)
)]
pub enum Kind<'a> {
    Critter {
        cond: Option<u8>,
        anim1: Option<u32>,
        anim2: Option<u32>,
        #[cfg_attr(
            feature = "serde1",
            serde(borrow, skip_serializing_if = "Vec::is_empty")
        )]
        param: Vec<(&'a str, i32)>,
    },
    Item {
        #[cfg_attr(feature = "serde1", serde(skip_serializing_if = "Anim::is_none"))]
        anim: Anim<'a>,
        count: Option<u32>,
        v9_in_container: Option<bool>,
        slot: Option<u8>,
        #[cfg_attr(feature = "serde1", serde(skip_serializing_if = "Broken::is_none"))]
        broken: Broken,
        ammo_pid: Option<u16>,
        ammo_count: Option<u32>,
        #[cfg_attr(feature = "serde1", serde(skip_serializing_if = "Locker::is_none"))]
        locker: Locker,
        trap_value: Option<u16>,
        #[cfg_attr(feature = "serde1", serde(skip_serializing_if = "slice_has_none"))]
        val: Vec<Option<i32>>,
    },
    Scenery {
        #[cfg_attr(feature = "serde1", serde(skip_serializing_if = "Anim::is_none"))]
        anim: Anim<'a>,
        can_use: Option<bool>,
        can_talk: Option<bool>,
        trigger_num: Option<u32>,
        params_count: Option<u8>,
        #[cfg_attr(feature = "serde1", serde(skip_serializing_if = "slice_has_none"))]
        params: Vec<Option<i32>>,
        to_map_pid: Option<u16>,
        to_entire: Option<u32>,
        to_dir: Option<u8>,
        sprite_cut: Option<u8>,
    },
}
impl Kind<'_> {
    pub fn map_object_type(&self) -> MapObjectType {
        use Kind::*;
        use MapObjectType::*;
        match self {
            Critter { .. } => MAP_OBJECT_CRITTER,
            Item { .. } => MAP_OBJECT_ITEM,
            Scenery { .. } => MAP_OBJECT_SCENERY,
        }
    }
    pub fn anim(&self) -> Option<&Anim> {
        use Kind::*;
        match self {
            Item { anim, .. } => Some(anim),
            Scenery { anim, .. } => Some(anim),
            _ => None,
        }
    }
}
const MAPOBJ_CRITTER_PARAMS: usize = 40;
pub fn param_list<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<(&'a str, i32)>, E> {
    many_m_n(
        0,
        MAPOBJ_CRITTER_PARAMS,
        pair(
            kv_ext(pair(tag("Critter_ParamIndex"), digit1), word),
            kv_ext(pair(tag("Critter_ParamValue"), digit1), integer),
        ),
    )(i)
}

fn parse_kind<'a, E: ParseError<&'a str>>(
    ty: MapObjectType,
) -> impl Fn(&'a str) -> IResult<&'a str, Kind, E> {
    use MapObjectType::*;
    move |i| {
        Ok(match ty {
            MAP_OBJECT_CRITTER => parse_struct!(
                i,
                Kind::Critter {
                    cond: opt_key_int("Critter_Cond"),
                    anim1: opt_key_int("Critter_Anim1"),
                    anim2: opt_key_int("Critter_Anim2"),
                    param: param_list,
                }
            ),
            MAP_OBJECT_ITEM => parse_struct!(
                i,
                Kind::Item {
                    anim: parse_anim,
                    count: opt_key_int("Item_Count"),
                    v9_in_container: opt_kv("Item_InContainer", int_bool),
                    slot: opt_key_int("Item_ItemSlot"),
                    broken: parse_broken,
                    ammo_pid: opt_key_int("Item_AmmoPid"),
                    ammo_count: opt_key_int("Item_AmmoCount"),
                    locker: parse_locker,
                    trap_value: opt_key_int("Item_TrapValue"),
                    val: many_key_index_int("Item_Val", 10),
                }
            ),
            MAP_OBJECT_SCENERY => parse_struct!(
                i,
                Kind::Scenery {
                    anim: parse_anim,
                    can_use: opt_kv("Scenery_CanUse", int_bool),
                    can_talk: opt_kv("Scenery_CanTalk", int_bool),
                    trigger_num: opt_key_int("Scenery_TriggerNum"),
                    params_count: opt_key_int("Scenery_ParamsCount"),
                    params: many_key_index_int("Scenery_Param", 5),
                    to_map_pid: opt_key_int("Scenery_ToMapPid"),
                    to_entire: opt_key_int("Scenery_ToEntire"),
                    to_dir: opt_key_int("Scenery_ToDir"),
                    sprite_cut: opt_key_int("Scenery_SpriteCut"),
                }
            ),
        })
    }
}

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(
    feature = "serde1",
    skip_serializing_none,
    derive(Serialize, Deserialize, SerDebug)
)]
pub struct Object<'a> {
    pub proto_id: u16,
    pub map_x: Option<u16>,
    pub map_y: Option<u16>,
    pub dir: Option<i16>,
    #[cfg_attr(feature = "serde1", serde(skip_serializing_if = "Relations::is_none"))]
    pub relations: Relations,
    #[cfg_attr(feature = "serde1", serde(skip_serializing_if = "Light::is_none"))]
    pub light: Light,
    pub script_name: Option<&'a str>,
    pub script_func: Option<&'a str>,
    #[cfg_attr(feature = "serde1", serde(skip_serializing_if = "slice_has_none"))]
    pub user_data: Vec<Option<i32>>,
    pub kind: Kind<'a>,
    pub ty_str: &'a str,
}

impl<'a> Object<'a> {
    pub fn is_scenery(&self) -> bool {
        if let Kind::Scenery { .. } = self.kind {
            true
        } else {
            false
        }
    }
}

fn object<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Object<'a>, E> {
    let (i, ty, ty_str) = {
        let (new_i, ty) = kv("MapObjType", integer)(i)?;
        let (new_i2, ty_str) = kv("MapObjType", word)(i)?;
        debug_assert_eq!(new_i, new_i2);
        (new_i, ty, ty_str)
    };
    Ok(parse_struct!(
        i,
        Object {
            proto_id: key_int("ProtoId"),
            map_x: opt_key_int("MapX"),
            map_y: opt_key_int("MapY"),
            dir: opt_key_int("Dir"),
            relations: parse_relations,
            light: parse_light,
            script_name: opt_kv("ScriptName", word),
            script_func: opt_kv("FuncName", word),
            user_data: many_key_index_int("UserData", 10),
            kind: parse_kind(ty),
        },
        {
            ty_str: ty_str,
        }
    ))
}

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize, SerDebug))]
pub struct Objects<'a>(#[cfg_attr(feature = "serde1", serde(borrow))] pub Vec<Object<'a>>);

pub fn objects<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Objects<'a>, E> {
    let (i, _) = section("Objects")(i)?;
    map(separated_list(t_rn, object), Objects)(i)
}

impl crate::Offset for Object<'_> {
    fn offset(&self) -> (i32, i32) {
        self.kind
            .anim()
            .map(|anim| {
                (
                    anim.offset_x.unwrap_or(0) as i32,
                    anim.offset_y.unwrap_or(0) as i32,
                )
            })
            .unwrap_or((0, 0))
    }
}
