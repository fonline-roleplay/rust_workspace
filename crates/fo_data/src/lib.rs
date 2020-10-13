//mod converter;
mod converter;
pub mod crawler;
pub mod datafiles;
mod fofrm;
mod frm;
mod palette;
mod retriever;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
pub type PathMap<K, V> = BTreeMap<K, V>;
pub type ChangeTime = std::time::SystemTime;
pub use crate::{
    converter::{Converter, GetImageError, RawImage},
    retriever::{fo::FoRetriever, HasPalette, Retriever},
};

#[cfg(feature = "sled-retriever")]
pub use retriever::sled::SledRetriever;

#[derive(Debug, Serialize, Deserialize)]
pub enum FileLocation {
    Archive(u16),
    Local,
}
impl Default for FileLocation {
    fn default() -> Self {
        FileLocation::Local
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FileInfo {
    location: FileLocation,
    original_path: String,
    compressed_size: u64,
}
impl FileInfo {
    fn location<'a>(&self, data: &'a FoData) -> Option<&'a std::path::PathBuf> {
        use std::convert::TryInto;
        match self.location {
            FileLocation::Archive(index) => data
                .archives
                .get(index as usize)
                .map(|archive| &archive.path),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FoArchive {
    changed: ChangeTime,
    path: std::path::PathBuf,
}

pub struct FileData {
    pub data_type: DataType,
    pub data: bytes::Bytes,
    pub dimensions: (u32, u32),
    pub offset: (i16, i16),
}

#[derive(Debug, PartialEq)]
pub enum FileType {
    Png,
    Frm,
    Gif,
    FoFrm,
    Unsupported(String),
    Unknown,
}

#[derive(Debug, Hash)]
pub enum DataType {
    Png,
    Rgba,
}

#[derive(Debug)]
pub enum DataInitError {
    LoadPalette(palette::Error),
    Datafiles(datafiles::Error),
    GatherPaths(crawler::Error),
    CacheSerialize(bincode::Error),
    CacheDeserialize(bincode::Error),
    CacheIO(std::io::Error),
    CacheStale,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FoData {
    changed: ChangeTime,
    archives: Vec<FoArchive>,
    files: PathMap<String, FileInfo>,
    //cache: HashMap<(String, OutputType), FileData>,
    palette: Vec<(u8, u8, u8)>,
}

const CACHE_PATH: &str = "fo_data.bin";
impl FoData {
    pub fn stub() -> Self {
        FoData {
            changed: ChangeTime::now(),
            archives: Default::default(),
            files: Default::default(),
            palette: Default::default(),
        }
    }
    fn recover_from_cache<P: AsRef<Path>>(client_root: P) -> Result<Self, DataInitError> {
        type Error = DataInitError;
        let cache_file = std::fs::File::open(CACHE_PATH).map_err(Error::CacheIO)?;
        let cache_changed = cache_file
            .metadata()
            .map_err(Error::CacheIO)?
            .modified()
            .map_err(Error::CacheIO)?;
        let reader = std::io::BufReader::new(cache_file);
        let fo_data: FoData = bincode::deserialize_from(reader).map_err(Error::CacheDeserialize)?;
        let datafiles_changetime =
            datafiles::datafiles_changetime(client_root).map_err(Error::Datafiles)?;
        let cache_changed = cache_changed.min(fo_data.changed);
        if datafiles_changetime > cache_changed {
            return Err(Error::CacheStale);
        }
        for archive in &fo_data.archives {
            if archive.changed > cache_changed {
                return Err(Error::CacheStale);
            }
        }
        Ok(fo_data)
    }
    pub fn init<P: AsRef<Path>, P2: AsRef<Path>>(
        client_root: P,
        palette_path: P2,
    ) -> Result<Self, DataInitError> {
        type Error = DataInitError;
        match Self::recover_from_cache(&client_root) {
            Err(err) => println!("FoData recovery failed: {:?}", err),
            ok => return ok,
        }

        let palette = palette::load_palette(palette_path).map_err(Error::LoadPalette)?;
        let palette = palette.colors_multiply(4);
        let archives = datafiles::parse_datafile(client_root).map_err(Error::Datafiles)?;
        let files = crawler::gather_paths(&archives).map_err(Error::GatherPaths)?;
        let changed = ChangeTime::now();
        let fo_data = FoData {
            changed,
            archives,
            files,
            palette,
        };
        {
            let cache_file = std::fs::File::create(CACHE_PATH).map_err(Error::CacheIO)?;
            let mut writer = std::io::BufWriter::new(cache_file);
            bincode::serialize_into(&mut writer, &fo_data).map_err(Error::CacheSerialize)?;
        }
        Ok(fo_data)
    }
    pub fn count_archives(&self) -> usize {
        self.archives.len()
    }
    pub fn count_files(&self) -> usize {
        self.files.len()
    }
    pub fn into_retriever(self) -> FoRetriever {
        FoRetriever::new(self)
    }
    pub fn files(&self) -> impl ExactSizeIterator<Item = (&str, &FileInfo)> {
        self.files.iter().map(|(path, info)| (path.as_str(), info))
    }
}

#[cfg(test)]
mod test_stuff {
    use std::path::{Path, PathBuf};

    pub const CLIENT_FOLDER: &str = "../../../CL4RP";
    pub const TEST_ASSETS_FOLDER: &str = "../../../test_assets";
    pub fn test_assets() -> PathBuf {
        Path::new(TEST_ASSETS_FOLDER).to_owned()
    }
    pub fn palette_path() -> PathBuf {
        Path::new(TEST_ASSETS_FOLDER).join("COLOR.PAL")
    }

    #[cfg(not(feature = "sled-retriever"))]
    pub fn test_retriever() -> crate::FoRetriever {
        let fo_data = crate::FoData::init(CLIENT_FOLDER, palette_path()).unwrap();
        fo_data.into_retriever()
    }

    #[cfg(feature = "sled-retriever")]
    pub fn test_retriever() -> &'static crate::SledRetriever {
        static RETRIEVER: once_cell::sync::Lazy<SledRetriever> = once_cell::sync::Lazy::new(|| {
            crate::SledRetriever::init(test_assets().join("db/assets"), palette_path()).unwrap()
        });
        &*RETRIEVER
    }
}
#[cfg(test)]
use test_stuff::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn load_frm_from_zip_and_convert_to_png() {
        let retriever = test_retriever();
        //"art/tiles/FOM1000.FRM"
        let image = retriever.get_png("art/tiles/fom1000.frm").unwrap();
        std::fs::write(test_assets().join("output/FOM1000.png"), image.data).unwrap();
    }

    fn save_frame<'a>(frame: &'a frm::Frame<'a>, palette: &[(u8, u8, u8)], path: impl AsRef<Path>) {
        let image = image::GrayImage::from_raw(
            frame.width as u32,
            frame.height as u32,
            frame.data.to_owned(),
        )
        .unwrap();
        let colored = image.expand_palette(palette, None);
        colored.save(path).unwrap();
    }

    #[test]
    fn colored_tile() {
        let file = std::fs::read(palette_path()).unwrap();
        let (_, palette) = palette::palette_verbose(&file).unwrap();

        let file = std::fs::read(test_assets().join("EDG1001.FRM")).unwrap();
        let (_, frm) = frm::frm_verbose(&file).unwrap();

        save_frame(
            &frm.directions[0].frames[0],
            &palette.colors_multiply(1),
            test_assets().join("output/EDG1001_1.png"),
        );
        save_frame(
            &frm.directions[0].frames[0],
            &palette.colors_multiply(2),
            test_assets().join("output/EDG1001_2.png"),
        );
        save_frame(
            &frm.directions[0].frames[0],
            &palette.colors_multiply(3),
            test_assets().join("output/EDG1001_3.png"),
        );
        save_frame(
            &frm.directions[0].frames[0],
            &palette.colors_multiply(4),
            test_assets().join("output/EDG1001_4.png"),
        );
    }

    #[test]
    fn colored_animation() {
        let file = std::fs::read(palette_path()).unwrap();
        let (_, palette) = palette::palette_verbose(&file).unwrap();
        let palette4 = palette.colors_multiply(4);

        let file = std::fs::read(test_assets().join("HMWARRAA.FRM")).unwrap();
        let (_, frm) = frm::frm_verbose(&file).unwrap();

        for (dir_index, dir) in frm.directions.iter().enumerate() {
            for (frame_index, frame) in dir.frames.iter().enumerate() {
                save_frame(
                    &frame,
                    &palette4,
                    format!(
                        "{}/output/HMWARRAA_{}_{}.png",
                        TEST_ASSETS_FOLDER, dir_index, frame_index
                    ),
                );
            }
        }
    }

    #[test]
    fn print_frm_animation_info() {
        let retriever = test_retriever();
        let bytes = retriever.file_by_path("art/scenery/gizsign.frm").unwrap();
        let (rest, frm) = frm::frm_verbose(&bytes).unwrap();
        println!("{:?}", frm);
    }
}
