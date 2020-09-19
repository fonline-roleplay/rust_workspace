use crate::{FileData, FileLocation, FileType, FoData};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    NotFound,
    InvalidArchiveIndex,
    OpenArchive(std::io::Error),
    Zip(zip::result::ZipError),
    UnsupportedFileLocation,
    ArchiveRead(std::io::Error),
}

pub fn recognize_type(path: &str) -> FileType {
    move || -> Option<_> {
        let ext = Path::new(path).extension()?.to_str()?.to_ascii_lowercase();
        Some(match ext.as_str() {
            "png" => FileType::Png,
            "frm" => FileType::Frm,
            "gif" => FileType::Gif,
            "fofrm" => FileType::FoFrm,
            _ => FileType::Unsupported(ext),
        })
    }()
    .unwrap_or(FileType::Unknown)
}

pub fn retrieve_file(data: &FoData, path: &str) -> Result<bytes::Bytes, Error> {
    let file = data.files.get(path).ok_or(Error::NotFound)?;
    /*if let Some(data) = file.cache.get() {
        return Ok(data);
    }*/
    let data = file.retrieve(data)?;
    //file.cache.set(data.into()).expect("Can't be not empty");
    //Ok(file.cache.get().expect("Can't be empty"))
    Ok(data)
}
