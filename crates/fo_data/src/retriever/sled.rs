use std::path::Path;

pub struct SledRetriever {
    db: sled::Db,
    paths: sled::Tree,
    files: sled::Tree,
    palette: Vec<(u8, u8, u8)>,
}

#[derive(Debug)]
pub enum Error {
    Init(sled::Error),
    GetFileIndexByPath(sled::Error),
    PathNotFound,
    GetFileByIndex(sled::Error),
    FileIndexNotFound,
}
type Result<T, E = Error> = std::result::Result<T, E>;

impl SledRetriever {
    pub fn init<P: AsRef<Path>, P2: AsRef<Path>>(db: P, palette: P2) -> Result<Self> {
        let file = std::fs::read(palette).unwrap();
        let (_, palette) = crate::palette::palette_verbose(&file).unwrap();
        SledRetriever::with_pallete(db, palette.colors_multiply(4))
    }
    pub fn with_pallete<P: AsRef<Path>>(path: P, palette: Vec<(u8, u8, u8)>) -> Result<Self> {
        let config = sled::Config::new()
            .path(path)
            .cache_capacity(128 * 1024 * 1024)
            .use_compression(true);
        //.compression_factor(22);
        let db = config.open().map_err(Error::Init)?;
        let paths = db.open_tree("paths").map_err(Error::Init)?;
        let files = db.open_tree("files").map_err(Error::Init)?;
        Ok(Self {
            db,
            paths,
            files,
            palette,
        })
    }
}

impl super::Retriever for SledRetriever {
    type Error = Error;
    fn file_by_path(&self, path: &str) -> Result<bytes::Bytes, Self::Error> {
        let index = self
            .paths
            .get(path)
            .map_err(Error::GetFileIndexByPath)?
            .ok_or(Error::PathNotFound)?;
        let data = self
            .files
            .get(index)
            .map_err(Error::GetFileByIndex)?
            .ok_or(Error::FileIndexNotFound)?;
        Ok(bytes::Bytes::copy_from_slice(data.as_ref()))
    }
}

impl Into<crate::GetImageError> for Error {
    fn into(self) -> crate::GetImageError {
        crate::GetImageError::SledRetrieve(self)
    }
}

impl super::HasPalette for SledRetriever {
    fn palette(&self) -> &[(u8, u8, u8)] {
        &self.palette
    }
}
