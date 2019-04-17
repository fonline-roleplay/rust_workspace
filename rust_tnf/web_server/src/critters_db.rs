use crate::critter_info::CritterInfo;
use crate::fix_encoding::{decode_filename, os_str_debug};
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
        Self {
            filename: filename.into(),
            modified: None,
            info: None,
        }
    }
    fn update_info(&mut self, name: String) -> io::Result<()> {
        let pathbuf = self.file_path();
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
    fn rename_file(&mut self, name: String) -> io::Result<()> {
        let from = self.file_path();
        let mut to = from.with_file_name(&name);
        to.set_extension("client");
        std::fs::rename(from, to)?;
        self.filename = OsString::from(name).into_boxed_os_str();
        Ok(())
    }
    fn file_path(&self) -> PathBuf {
        let mut pathbuf = PathBuf::new();
        pathbuf.push(CLIENTS_PATH);
        pathbuf.push(&*self.filename);
        pathbuf.set_extension("client");
        pathbuf
    }
}

pub struct CrittersDb {
    hashmap: HashMap<u32, InnerCritter>,
    clients: InnerClients,
}

impl CrittersDb {
    pub fn new() -> Self {
        let mut db = Self::default();
        db.update_clients(true).expect("Can't load clients");
        db
    }
    pub fn fix_clients(dry_ran: bool) {
        let mut db = Self::default();
        db.update_clients(false).expect("Can't fix clients");
        let clients = if let Ok(clients) = Arc::try_unwrap(db.clients) {
            clients
        } else {
            unreachable!();
        };
        print!("Fixing clients...");
        for (name, mut record) in clients {
            match record.filename.to_str() {
                Some(string) if string == name => {
                    println!("{:?} == {:?}, skipping", name, string);
                }
                _ => {
                    print!(
                        "{:?} != {:?}, fixing... ",
                        name,
                        os_str_debug(&record.filename)
                    );
                    if dry_ran {
                        println!("dry run");
                    } else {
                        match record.rename_file(name) {
                            Ok(()) => println!("OK"),
                            Err(err) => println!("ERROR: {:?}", err),
                        }
                    }
                }
            }
        }
    }
    fn update_clients(&mut self, load_clients_info: bool) -> io::Result<()> {
        let mut clients: BTreeMap<String, ClientRecord> = BTreeMap::new();

        for (key, value) in std::fs::read_dir(CLIENTS_PATH)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension() == Some("client".as_ref()))
            .filter_map(|path| {
                path.file_stem().and_then(|stem| {
                    decode_filename(stem).map(|nickname| {
                        let mut record = ClientRecord::new(stem);
                        if load_clients_info {
                            let _ = record.update_info(nickname.clone());
                        }
                        (nickname, record)
                    })
                })
            })
        {
            match clients.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                Entry::Occupied(entry) => {
                    let (old_key, old_value) = entry.remove_entry();
                    eprintln!(
                        "Two clients with the same name {:?}, ignoring both: {:?} == {:?}",
                        old_key,
                        os_str_debug(&value.filename),
                        os_str_debug(&old_value.filename),
                    );
                }
            };
        }
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
        Self {
            hashmap: HashMap::new(),
            clients: Arc::new(BTreeMap::new()),
        }
    }
}

pub struct ListClients;

impl Message for ListClients {
    type Result = Result<InnerClients, Error>;
}

impl Handler<ListClients> for CrittersDb {
    type Result = Result<InnerClients, Error>;

    fn handle(&mut self, _msg: ListClients, _: &mut Self::Context) -> Self::Result {
        self.update_clients(true)?;
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
