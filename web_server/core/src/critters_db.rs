use actix_web::Error;
use arc_swap::ArcSwapAny;
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

#[derive(Clone)]
pub struct CrittersDb {
    clients: ArcSwapAny<InnerClients>,
    path: PathBuf,
}

impl CrittersDb {
    pub fn new(path: PathBuf) -> Self {
        let clients = Arc::new(ClientsDb::new(&path));
        CrittersDb {
            clients: ArcSwapAny::from(clients),
            path,
        }
    }
    pub fn list_clients(&self) -> InnerClients {
        let clients = Arc::new(ClientsDb::new(&self.path));
        self.clients.store(Arc::clone(&clients));
        clients
    }
    pub fn client_info(&self, name: &str) -> io::Result<InnerCritter> {
        self.clients.load().client_info(name)
    }
}

fn not_found() -> io::Error {
    io::ErrorKind::NotFound.into()
}
