use actix::prelude::*;
use actix_web::Error;
use std::collections::HashMap;
use std::sync::Arc;
use crate::critter_info::CritterInfo;
use fo_client_format::ClientSaveData;

type InnerCritter = Arc<CritterInfo>;

pub struct CrittersDb {
    hashmap: HashMap<u32, InnerCritter>,
}

impl CrittersDb {
    pub fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
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
    type Result = Result<Vec<String>, Error>;
}

impl Handler<ListClients> for CrittersDb {
    type Result = Result<Vec<String>, Error>;

    fn handle(&mut self, _msg: ListClients, _: &mut Self::Context) -> Self::Result {
        let clients: Vec<String> = std::fs::read_dir("./save/clients/")?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension() == Some("client".as_ref()))
            .filter_map(|path| {
                path.file_stem()
                    .and_then(std::ffi::OsStr::to_str)
                    .map(String::from)
            })
            .collect();
        Ok(clients)
        //Ok(self.hashmap.get(&msg.id).cloned())
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
        let data = std::fs::read(format!("./save/clients/{}.client", msg.name))?;
        let client_data = ClientSaveData::read_bincode(&mut &data[..])?;
        let mut critter_info = CritterInfo::from(&client_data);
        critter_info.name = msg.name;
        Ok(Arc::new(critter_info))
    }
}
