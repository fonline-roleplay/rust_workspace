use crate::critter_info::CritterInfo;
use actix::prelude::*;
use actix_web::Error;
use fo_save_format::ClientSaveData;
use std::{
    io,
    collections::{BTreeMap, HashMap},
    ffi::OsStr,
    sync::Arc,
    time::SystemTime,
};

const CLIENTS_PATH: &'static str = "../save/clients/";

type InnerCritter = Arc<CritterInfo>;
type InnerClients = Arc<BTreeMap<String, ClientRecord>>;

pub struct ClientRecord {
    pub filename: Box<OsStr>,
    pub modified: Option<SystemTime>,
    pub info: Option<InnerCritter>,
}

impl ClientRecord {
    fn new(filename: &OsStr) -> Self {
        Self{
            filename: filename.into(),
            modified: None,
            info: None,
        }
    }
    fn update_info(&mut self, name: String) -> io::Result<()>{
        let mut pathbuf = std::path::PathBuf::new();
        pathbuf.push(CLIENTS_PATH);
        pathbuf.push(&*self.filename);
        pathbuf.set_extension("client");
        self.modified = pathbuf.metadata().and_then(|md| md.modified()).ok();
        let data = std::fs::read(&pathbuf)?;
        let client_data = ClientSaveData::read_bincode(&mut &data[..])?;
        let mut critter_info = CritterInfo::from(&client_data);
        critter_info.name = name;
        self.info = Some(Arc::new(critter_info));
        Ok(())
    }
    fn info(&self) -> io::Result<InnerCritter> {
        //self.update_info(name)?;
        let info = self.info.as_ref().ok_or_else(not_found)?;
        Ok(Arc::clone(info))
    }
}

pub struct CrittersDb {
    hashmap: HashMap<u32, InnerCritter>,
    clients: InnerClients,
}

impl CrittersDb {
    pub fn new() -> Self {
        let mut db = Self {
            hashmap: HashMap::new(),
            clients: Arc::new(BTreeMap::new()),
        };
        db.update_clients().expect("Can't load clients");
        db
    }
    fn update_clients(&mut self) -> io::Result<()> {
        let clients: BTreeMap<String, ClientRecord> = std::fs::read_dir(CLIENTS_PATH)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension() == Some("client".as_ref()))
            .filter_map(|path| {
                path.file_stem().and_then(|stem| {
                    decode_filename(stem).map(|nickname| {
                        let mut record = ClientRecord::new(stem);
                        let _ = record.update_info(nickname.clone());
                        (
                            nickname,
                            record,
                        )
                    })
                })
            })
            .collect();
        self.clients = Arc::new(clients);
        Ok(())
    }
    pub fn client_info(&self, name: &str) -> io::Result<InnerCritter> {
        if let Some(record) = self.clients.get(name) {
            record.info()
        } else {
            Err(not_found())
        }
    }
}

impl Actor for CrittersDb {
    type Context = SyncContext<Self>;
}

pub struct GetCritterInfo {
    pub id: u32,
}

impl Message for GetCritterInfo {
    type Result = Result<Option<InnerCritter>, Error>;
}

impl Handler<GetCritterInfo> for CrittersDb {
    type Result = Result<Option<InnerCritter>, Error>;

    fn handle(&mut self, msg: GetCritterInfo, _: &mut Self::Context) -> Self::Result {
        Ok(self.hashmap.get(&msg.id).cloned())
    }
}

pub struct UpdateCritterInfo(InnerCritter);

impl From<CritterInfo> for UpdateCritterInfo {
    fn from(cr_info: CritterInfo) -> Self {
        UpdateCritterInfo(Arc::new(cr_info))
    }
}

impl Message for UpdateCritterInfo {
    type Result = Result<(), Error>;
}

impl Handler<UpdateCritterInfo> for CrittersDb {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: UpdateCritterInfo, _: &mut Self::Context) -> Self::Result {
        self.hashmap.insert(msg.0.id, msg.0);
        Ok(())
    }
}

impl Default for CrittersDb {
    fn default() -> Self {
        CrittersDb::new()
    }
}

pub struct ListClients;

impl Message for ListClients {
    type Result = Result<InnerClients, Error>;
}

// CP1251 to UTF
const FORWARD_TABLE: &'static [u16] = &[
    1026, 1027, 8218, 1107, 8222, 8230, 8224, 8225, 8364, 8240, 1033, 8249, 1034, 1036, 1035, 1039,
    1106, 8216, 8217, 8220, 8221, 8226, 8211, 8212, 152, 8482, 1113, 8250, 1114, 1116, 1115, 1119,
    160, 1038, 1118, 1032, 164, 1168, 166, 167, 1025, 169, 1028, 171, 172, 173, 174, 1031, 176,
    177, 1030, 1110, 1169, 181, 182, 183, 1105, 8470, 1108, 187, 1112, 1029, 1109, 1111, 1040,
    1041, 1042, 1043, 1044, 1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052, 1053, 1054, 1055, 1056,
    1057, 1058, 1059, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067, 1068, 1069, 1070, 1071, 1072,
    1073, 1074, 1075, 1076, 1077, 1078, 1079, 1080, 1081, 1082, 1083, 1084, 1085, 1086, 1087, 1088,
    1089, 1090, 1091, 1092, 1093, 1094, 1095, 1096, 1097, 1098, 1099, 1100, 1101, 1102, 1103,
]; // 128 entries

#[cfg(windows)]
fn is_ascii(string: &OsStr) -> bool {
    use std::os::windows::ffi::OsStrExt;
    let mut maybe_ascii = false;
    for wide in string.encode_wide() {
        if wide < 0x20 || wide > 0xFF {
            return false;
        }
        if wide >= 0x80 {
            maybe_ascii = true;
        }
    }
    maybe_ascii
}

#[cfg(windows)]
fn from_ascii(string: &OsStr) -> Option<String> {
    use std::convert::TryInto;
    use std::os::windows::ffi::OsStrExt;
    //let mut vec = Vec::with_capacity(string.len()*2);
    let mut new_string = String::with_capacity(string.len() * 2);
    for wide in string.encode_wide() {
        let code = wide.to_ne_bytes()[0];
        if code >= 0x80 {
            let cp1251 = FORWARD_TABLE[(code - 0x80) as usize] as u32;
            new_string.push(cp1251.try_into().ok()?);
        } else if code != 0 {
            new_string.push(code as char);
        }
    }
    Some(new_string)
}

#[cfg(windows)]
fn decode_filename(filename: &OsStr) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    if is_ascii(filename) {
        from_ascii(filename)
    } else {
        filename.to_str().map(String::from)
    }
}

#[cfg(not(windows))]
fn decode_filename(filename: &OsStr) -> Option<String> {
    filename.to_str().map(String::from)
}

impl Handler<ListClients> for CrittersDb {
    type Result = Result<InnerClients, Error>;

    fn handle(&mut self, _msg: ListClients, _: &mut Self::Context) -> Self::Result {
        self.update_clients()?;
        Ok(Arc::clone(&self.clients))
    }
}

pub struct GetClientInfo {
    pub name: String,
}

impl Message for GetClientInfo {
    type Result = Result<InnerCritter, Error>;
}

impl Handler<GetClientInfo> for CrittersDb {
    type Result = Result<InnerCritter, Error>;

    fn handle(&mut self, msg: GetClientInfo, _: &mut Self::Context) -> Self::Result {
        //Ok(self.hashmap.get(&msg.id).cloned())
        if let Some(record) = self.clients.get(&msg.name) {
            Ok(self.client_info(&msg.name)?)
        } else {
            Err(not_found().into())
        }
    }
}

fn not_found() -> io::Error {
    io::ErrorKind::NotFound.into()
}