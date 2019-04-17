use fo_client_format::ClientSaveData;
use tnf_common::{
    defines::{fos, param::CritterParam},
    engine_types::critter::Critter,
};

pub struct CritterInfo {
    pub id: u32,
    pub hex_x: u16,
    pub hex_y: u16,
    pub dir: u8,
    pub cond: u8,
    pub map_id: u32,
    pub map_pid: u16,
    pub params: [i32; 1000],
    pub name: String,
}

impl From<&Critter> for CritterInfo {
    fn from(cr: &Critter) -> Self {
        CritterInfo {
            id: cr.Id,
            hex_x: cr.HexX,
            hex_y: cr.HexY,
            dir: cr.Dir,
            cond: cr.Cond,
            map_id: cr.MapId,
            map_pid: cr.MapPid,
            params: cr.Params.clone(),
            name: cr.NameStr.string(),
        }
    }
}

impl From<&ClientSaveData> for CritterInfo {
    fn from(cr: &ClientSaveData) -> Self {
        CritterInfo {
            id: cr.data.Id,
            hex_x: cr.data.HexX,
            hex_y: cr.data.HexY,
            dir: cr.data.Dir,
            cond: cr.data.Cond,
            map_id: cr.data.MapId,
            map_pid: cr.data.MapPid,
            params: cr.data.Params.clone(),
            name: Default::default(),
        }
    }
}

impl CritterParam for CritterInfo {
    fn params_all(&self) -> &[i32] {
        &self.params
    }
}

const COND: [&'static str; 4] = ["INVALID", "ALIVE", "KO", "DYING"];
const DEAD_MAP_PID: u16 = 170;

impl CritterInfo {
    pub fn cond(&self) -> &'static str {
        if self.map_pid == DEAD_MAP_PID {
            "DEAD"
        } else {
            COND[self.cond.min(fos::COND_DEAD) as usize]
        }
    }
}