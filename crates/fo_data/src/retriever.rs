use crate::{FileData, FileLocation, FileType, FoData};
use std::io::{BufReader, Read};
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
            _ => FileType::Unsupported(ext),
        })
    }()
    .unwrap_or(FileType::Unknown)
}

pub fn retrieve_file<'a>(data: &'a FoData, path: &str) -> Result<bytes::Bytes, Error> {
    let file = data.files.get(path).ok_or(Error::NotFound)?;
    /*if let Some(data) = file.cache.get() {
        return Ok(data);
    }*/
    let data = match file.location {
        FileLocation::Archive(archive_index) => {
            let archive_path = data
                .archives
                .get(archive_index as usize)
                .ok_or(Error::InvalidArchiveIndex)?;
            let archive_file = std::fs::File::open(archive_path).map_err(Error::OpenArchive)?;
            let archive_buf_reader = BufReader::with_capacity(1024, archive_file);
            let mut archive = zip::ZipArchive::new(archive_buf_reader).map_err(Error::Zip)?;
            let mut file = archive.by_name(&file.original_path).map_err(Error::Zip)?;
            let mut buffer = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buffer);
            buffer
        }
        _ => return Err(Error::UnsupportedFileLocation),
    };
    //file.cache.set(data.into()).expect("Can't be not empty");
    //Ok(file.cache.get().expect("Can't be empty"))
    Ok(data.into())
}
