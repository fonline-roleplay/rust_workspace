use actix::prelude::*;
use actix_web::Error;
use fo_save_format::ClientSaveData;
use std::{
    collections::{btree_map::Entry, BTreeMap, HashMap},
    ffi::{OsStr, OsString},
    io,
    path::PathBuf,
    sync::Arc,
    time::SystemTime,
};

use clients_db::{
    fix_encoding::{decode_filename, os_str_debug},
    ClientRecord, ClientsDb, CritterInfo,
};

type InnerCritter = Arc<CritterInfo>;
type InnerClients = Arc<ClientsDb>;

pub struct CrittersDb {
    hashmap: HashMap<u32, InnerCritter>,
    clients: InnerClients,
    path: PathBuf,
}

impl CrittersDb {
    pub fn new(path: PathBuf) -> Self {
        let clients = Arc::new(ClientsDb::new(&path));
        CrittersDb {
            hashmap: Default::default(),
            path,
            clients,
        }
    }
    /*fn update_clients(&mut self, load_clients_info: bool) -> io::Result<()> {

    }*/
    pub fn client_info(&self, name: &str) -> io::Result<InnerCritter> {
        self.clients.client_info(name)
    }
}

impl Actor for CrittersDb {
    type Context = SyncContext<Self>;
}

pub struct GetCritterInfo {
    pub id: u32,
}

impl Message for GetCritterInfo {
    type Result = Result<Option<InnerCritter>, ()>;
}

impl Handler<GetCritterInfo> for CrittersDb {
    type Result = <GetCritterInfo as Message>::Result;

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
    type Result = Result<(), ()>;
}

impl Handler<UpdateCritterInfo> for CrittersDb {
    type Result = <UpdateCritterInfo as Message>::Result;

    fn handle(&mut self, msg: UpdateCritterInfo, _: &mut Self::Context) -> Self::Result {
        self.hashmap.insert(msg.0.id, msg.0);
        Ok(())
    }
}

pub struct ListClients;

impl Message for ListClients {
    type Result = Result<InnerClients, ()>;
}

impl Handler<ListClients> for CrittersDb {
    type Result = <ListClients as Message>::Result;

    fn handle(&mut self, _msg: ListClients, _: &mut Self::Context) -> Self::Result {
        //self.clients.update_clients(&self.path, true)?;
        self.clients = Arc::new(ClientsDb::new(&self.path));
        Ok(Arc::clone(&self.clients))
    }
}

pub struct GetClientInfo {
    pub name: String,
}

impl Message for GetClientInfo {
    type Result = Result<InnerCritter, io::Error>;
}

impl Handler<GetClientInfo> for CrittersDb {
    type Result = <GetClientInfo as Message>::Result;

    fn handle(&mut self, msg: GetClientInfo, _: &mut Self::Context) -> Self::Result {
        //Ok(self.hashmap.get(&msg.id).cloned())
        if let Some(record) = self.clients.clients().get(&msg.name) {
            Ok(self.client_info(&msg.name)?)
        } else {
            Err(not_found().into())
        }
    }
}

fn not_found() -> io::Error {
    io::ErrorKind::NotFound.into()
}
