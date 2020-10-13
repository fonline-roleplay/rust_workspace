pub mod fo;
#[cfg(feature = "sled-retriever")]
pub mod sled;

use crate::FileType;
use std::path::Path;

pub trait Retriever {
    type Error;
    fn file_by_path(&self, path: &str) -> Result<bytes::Bytes, Self::Error>;
}

pub trait HasPalette {
    fn palette(&self) -> &[(u8, u8, u8)];
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
