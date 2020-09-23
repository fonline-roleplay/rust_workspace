use crate::{FileData, FileLocation, FileType, FoData};
use parking_lot::{MappedMutexGuard as Guard, Mutex, MutexGuard};
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

type Archive = zip::ZipArchive<std::io::BufReader<std::fs::File>>;

pub struct Retriever {
    archives: Vec<Mutex<Option<Box<Archive>>>>,
    data: FoData,
}

impl Retriever {
    pub fn new(data: FoData) -> Self {
        let mut archives = Vec::new();
        archives.resize_with(data.archives.len(), Default::default);
        Self { archives, data }
    }
    fn get_archive(&self, archive_index: usize) -> Result<Guard<Archive>, Error> {
        use std::io::BufReader;

        let mut guard = self.archives[archive_index].lock();

        if guard.is_none() {
            let archive = self
                .data
                .archives
                .get(archive_index)
                .ok_or(Error::InvalidArchiveIndex)?;
            let archive_file = std::fs::File::open(&archive.path).map_err(Error::OpenArchive)?;
            let archive_buf_reader = BufReader::with_capacity(1024, archive_file);
            let archive = zip::ZipArchive::new(archive_buf_reader).map_err(Error::Zip)?;
            *guard = Some(Box::new(archive));
        }
        Ok(MutexGuard::map(guard, |option| {
            &mut **option.as_mut().expect("Should be some")
        }))
    }
    pub fn data(&self) -> &FoData {
        &self.data
    }
    pub fn file_by_path(&self, path: &str) -> Result<bytes::Bytes, Error> {
        let file_info = self.data.files.get(path).ok_or(Error::NotFound)?;

        self.file_by_info(&file_info)
    }
    pub fn file_by_info(&self, file_info: &crate::FileInfo) -> Result<bytes::Bytes, Error> {
        use std::io::Read;

        match file_info.location {
            FileLocation::Archive(archive_index) => {
                let mut archive = self.get_archive(archive_index as usize)?;

                let mut file = archive
                    .by_name(&file_info.original_path)
                    .map_err(Error::Zip)?;
                let mut buffer = Vec::with_capacity(file.size() as usize);
                file.read_to_end(&mut buffer).map_err(Error::ArchiveRead)?;
                Ok(buffer.into())
            }
            _ => Err(Error::UnsupportedFileLocation),
        }
    }
}
