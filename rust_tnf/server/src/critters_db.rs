use actix::prelude::*;
use actix_web::Error;
use std::collections::HashMap;
use tnf_common::engine_types::critter::CritterInfo;

pub struct CrittersDb {
    hashmap: HashMap<u32, CritterInfo>,
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
    type Result = Result<Option<CritterInfo>, Error>;
}

impl Handler<GetCritterInfo> for CrittersDb {
    type Result = Result<Option<CritterInfo>, Error>;

    fn handle(&mut self, msg: GetCritterInfo, _: &mut Self::Context) -> Self::Result {
        Ok(self.hashmap.get(&msg.id).cloned())
    }
}

pub struct UpdateCritterInfo(CritterInfo);

impl From<CritterInfo> for UpdateCritterInfo {
    fn from(cr_info: CritterInfo) -> Self {
        UpdateCritterInfo(cr_info)
    }
}

impl Message for UpdateCritterInfo {
    type Result = Result<(), Error>;
}

impl Handler<UpdateCritterInfo> for CrittersDb {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: UpdateCritterInfo, _: &mut Self::Context) -> Self::Result {
        self.hashmap.insert(msg.0.Id, msg.0);
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
