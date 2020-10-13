use crate::{FileData, FileLocation, FoData};
use parking_lot::{MappedMutexGuard as Guard, Mutex, MutexGuard};

#[derive(Debug)]
pub enum Error {
    NotFound,
    InvalidArchiveIndex,
    OpenArchive(std::io::Error),
    Zip(zip::result::ZipError),
    UnsupportedFileLocation,
    ArchiveRead(std::io::Error),
}

type Archive = zip::ZipArchive<std::io::BufReader<std::fs::File>>;

pub struct FoRetriever {
    archives: Vec<Mutex<Option<Box<Archive>>>>,
    data: FoData,
}

impl FoRetriever {
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

impl super::Retriever for FoRetriever {
    type Error = Error;
    fn file_by_path(&self, path: &str) -> Result<bytes::Bytes, Self::Error> {
        let file_info = self.data.files.get(path).ok_or(Error::NotFound)?;

        self.file_by_info(&file_info)
    }
}

impl Into<crate::GetImageError> for Error {
    fn into(self) -> crate::GetImageError {
        crate::GetImageError::FoRetrieve(self)
    }
}

impl super::HasPalette for FoRetriever {
    fn palette(&self) -> &[(u8, u8, u8)] {
        &self.data().palette
    }
}
