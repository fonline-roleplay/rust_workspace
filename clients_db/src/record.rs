use std::{
    io,
    sync::Arc,
    ffi::{OsStr, OsString},
    time::SystemTime,
    path::PathBuf,
};

use fo_save_format::ClientSaveData;
use crate::{CritterInfo, InnerCritter, not_found};

#[derive(Debug)]
pub struct ClientRecord {
    pub filename: Box<OsStr>,
    pub modified: Option<SystemTime>,
    pub info: Option<InnerCritter>,
}

impl ClientRecord {
    pub fn new(filename: &OsStr) -> Self {
        Self {
            filename: filename.into(),
            modified: None,
            info: None,
        }
    }
    pub fn update_info(&mut self, path: PathBuf, name: String) -> io::Result<()> {
        //let pathbuf = self.file_path(path);
        self.modified = path.metadata().and_then(|md| md.modified()).ok();
        let data = std::fs::read(&path)?;
        let client_data = ClientSaveData::read_bincode(&mut &data[..])?;
        let mut critter_info = CritterInfo::from(&client_data);
        critter_info.name = name;
        self.info = Some(Arc::new(critter_info));
        Ok(())
    }
    pub fn info(&self) -> io::Result<InnerCritter> {
        //self.update_info(name)?;
        let info = self.info.as_ref().ok_or_else(not_found)?;
        Ok(Arc::clone(info))
    }
    pub fn rename_file(&mut self, path: PathBuf, name: String) -> io::Result<()> {
        let from = self.file_path(path.clone());
        let mut to = path;
        to.push(&name);
        to.set_extension("client");
        std::fs::rename(from, to)?;
        self.filename = OsString::from(name).into_boxed_os_str();
        Ok(())
    }
    pub fn file_path(&self, mut pathbuf: PathBuf) -> PathBuf {
        pathbuf.push(&*self.filename);
        pathbuf.set_extension("client");
        pathbuf
    }
}
