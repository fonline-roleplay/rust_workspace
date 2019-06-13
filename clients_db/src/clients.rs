use std::{
    io,
    collections::{BTreeMap, btree_map::Entry},
    sync::Arc,
    path::PathBuf,
};

use crate::{
    ClientRecord, fix_encoding::{os_str_debug, decode_filename}, InnerCritter, not_found,
};

#[derive(Default, Debug)]
pub struct ClientsDb {
    clients: BTreeMap<String, ClientRecord>,
}

impl ClientsDb {
    pub fn new(path: &PathBuf) -> Self {
        let mut db = ClientsDb {
            ..Default::default()
        };
        db.update_clients(&path, true).expect("Can't load clients");
        db
    }
    pub fn clients(&self) -> &BTreeMap<String, ClientRecord> {
        &self.clients
    }
    pub fn list_names(path: PathBuf) -> BTreeMap<u32, String> {
        let mut db = ClientsDb {
            ..Default::default()
        };
        db.update_clients(&path, true).expect("Can't load clients");
        db.clients.iter().map(|(name, record)| {
            (record.info.as_ref().expect("client info").id, name.clone())
        }).collect()
    }
    pub fn list_ids(path: PathBuf) -> BTreeMap<u32, PathBuf> {
        let mut db = ClientsDb {
            ..Default::default()
        };
        db.update_clients(&path, true).expect("Can't load clients");
        db.clients.values().map(|record| {
            (record.info.as_ref().expect("client info").id, record.file_path(path.clone()))
        }).collect()
    }
    pub fn fix_clients(path: PathBuf, dry_ran: bool) {
        let mut db = ClientsDb {
            ..Default::default()
        };
        db.update_clients(&path, false).expect("Can't fix clients");
        print!("Fixing clients...");
        for (name, mut record) in db.clients {
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
                        match record.rename_file(path.clone(), name) {
                            Ok(()) => println!("OK"),
                            Err(err) => println!("ERROR: {:?}", err),
                        }
                    }
                }
            }
        }
    }
    pub fn update_clients(&mut self, path: &PathBuf, load_clients_info: bool) -> io::Result<()> {
        let mut clients: BTreeMap<String, ClientRecord> = BTreeMap::new();

        for (key, value) in std::fs::read_dir(&path)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension() == Some("client".as_ref()))
            .filter_map(|path| {
                path.file_stem().and_then(|stem| {
                    decode_filename(stem).map(|nickname| {
                        let mut record = ClientRecord::new(stem);
                        if load_clients_info {
                            let _ = record.update_info(path.clone(), nickname.clone());
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
        self.clients = clients;
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
