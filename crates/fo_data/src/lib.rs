//mod converter;
pub mod crawler;
pub mod datafiles;
mod frm;
mod palette;
mod retriever;

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
pub type PathMap<K, V> = BTreeMap<K, V>;

#[derive(Debug)]
pub enum FileLocation {
    Archive(u16),
    Local,
}
impl Default for FileLocation {
    fn default() -> Self {
        FileLocation::Local
    }
}

#[derive(Debug, Default)]
pub struct FileInfo {
    location: FileLocation,
    original_path: String,
    compressed_size: u64,
}

pub struct FoData {
    archives: Vec<std::path::PathBuf>,
    files: PathMap<String, FileInfo>,
    //cache: HashMap<(String, OutputType), FileData>,
    palette: Vec<(u8, u8, u8)>,
}

pub struct FileData {
    pub data_type: DataType,
    pub data: bytes::Bytes,
    pub dimensions: (u32, u32),
    pub offset: (i16, i16),
}

#[derive(Debug)]
pub enum FileType {
    Png,
    Frm,
    Gif,
    Rgba,
    Unsupported(String),
    Unknown,
}

#[derive(Debug, Hash)]
pub enum DataType {
    Png,
    Rgba,
}

#[derive(Debug)]
pub enum GetImageError {
    FileType(FileType),
    Retrieve(retriever::Error),
    FrmParse(nom_prelude::ErrorKind),
    EmptyFrm,
    ImageFromRaw,
    ImageWrite(image::ImageError),
    PngDecode(image::ImageError),
}

#[derive(Debug)]
pub enum DataInitError {
    LoadPalette(palette::Error),
    ParseDatafile(datafiles::Error),
    GatherPaths(crawler::Error),
}

#[derive(Debug, Clone)]
pub struct RawImage {
    pub image: image::RgbaImage,
    offset_x: i16,
    offset_y: i16,
}
impl RawImage {
    pub fn offsets(&self) -> (i16, i16) {
        (self.offset_x, self.offset_y)
    }
}

impl RawImage {
    fn to_png(self) -> Result<FileData, image::ImageError> {
        let dimensions = self.image.dimensions();
        let size = (dimensions.0 as usize * dimensions.1 as usize * 4 + 512).next_power_of_two();
        let image = image::DynamicImage::ImageRgba8(self.image);
        let mut data = Vec::with_capacity(size);

        image.write_to(&mut data, image::ImageFormat::Png)?;
        Ok(FileData {
            data: data.into(),
            data_type: DataType::Png,
            dimensions,
            offset: (self.offset_x, self.offset_y),
        })
    }
}

impl FoData {
    pub fn stub() -> Self {
        FoData {
            archives: Default::default(),
            files: Default::default(),
            palette: Default::default(),
        }
    }
    pub fn init<P: AsRef<Path>, P2: AsRef<Path>>(
        client_root: P,
        palette_path: P2,
    ) -> Result<Self, DataInitError> {
        type Error = DataInitError;
        let palette = palette::load_palette(palette_path).map_err(Error::LoadPalette)?;
        let palette = palette.colors_multiply(4);
        let archives = datafiles::parse_datafile(client_root).map_err(Error::ParseDatafile)?;
        let files = crawler::gather_paths(&archives).map_err(Error::GatherPaths)?;
        Ok(FoData {
            archives,
            files,
            palette,
        })
    }

    fn get_raw(&self, path: &str) -> Result<RawImage, GetImageError> {
        let file_type = retriever::recognize_type(path);
        let retriever =
            move || retriever::retrieve_file(self, path).map_err(GetImageError::Retrieve);

        Ok(match file_type {
            FileType::Png => {
                let data = retriever()?;
                let mut slice = &data[..];

                let dynamic = image::load_from_memory_with_format(slice, image::ImageFormat::Png)
                    .map_err(GetImageError::PngDecode)?;
                let mut image = dynamic.into_rgba();
                let (width, height) = image.dimensions();

                image.pixels_mut().for_each(|pixel| {
                    if pixel.0 == [0, 0, 255, 255] {
                        pixel.0 = [0, 0, 0, 0];
                    }
                });

                RawImage {
                    image,
                    offset_x: width as i16 / -2,
                    offset_y: height as i16 * -1,
                }
            }
            FileType::Frm => {
                let data = retriever()?;
                let frm = frm::frm(&data).map_err(GetImageError::FrmParse)?;
                let frame_number = 0;
                let direction0 = frm.directions.get(0).ok_or(GetImageError::EmptyFrm)?;
                let offsets = direction0.frames.iter().skip(1).take(frame_number);
                let offset_x: i16 = offsets.clone().map(|frame| frame.offset_x).sum();
                let offset_y: i16 = offsets.map(|frame| frame.offset_y).sum();
                let frame = direction0
                    .frames
                    .get(frame_number)
                    .ok_or(GetImageError::EmptyFrm)?;

                let image = image::GrayImage::from_raw(
                    frame.width as u32,
                    frame.height as u32,
                    frame.data.to_owned(),
                )
                .ok_or(GetImageError::ImageFromRaw)?;
                let image = image.expand_palette(&self.palette, Some(0));
                RawImage {
                    image,
                    offset_x: direction0.shift_x + offset_x - frame.width as i16 / 2,
                    offset_y: direction0.shift_y + offset_y - frame.height as i16,
                }
            }
            _ => return Err(GetImageError::FileType(file_type)),
        })
    }

    pub fn get_png(&self, path: &str) -> Result<FileData, GetImageError> {
        let raw = self.get_raw(path)?;
        raw.to_png().map_err(GetImageError::ImageWrite)
    }
    pub fn get_rgba(&self, path: &str) -> Result<RawImage, GetImageError> {
        self.get_raw(path)
    }
    pub fn count_archives(&self) -> usize {
        self.archives.len()
    }
    pub fn count_files(&self) -> usize {
        self.files.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn load_frm_from_zip_and_convert_to_png() {
        let fo_data = FoData::init("../../../CL4RP", "../../../test_assets/COLOR.PAL").unwrap();
        let image = fo_data.get_image("art/tiles/FOM1000.FRM").unwrap();
        std::fs::write("../../../test_assets/output/FOM1000.png", image.data).unwrap();
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
        let file = std::fs::read("../../../test_assets/COLOR.PAL").unwrap();
        let (_, palette) = palette::palette_verbose(&file).unwrap();

        let file = std::fs::read("../../../test_assets/EDG1001.FRM").unwrap();
        let (_, frm) = frm::frm_verbose(&file).unwrap();

        save_frame(
            &frm.directions[0].frames[0],
            &palette.colors_multiply(1),
            "../../../test_assets/output/EDG1001_1.png",
        );
        save_frame(
            &frm.directions[0].frames[0],
            &palette.colors_multiply(2),
            "../../../test_assets/output/EDG1001_2.png",
        );
        save_frame(
            &frm.directions[0].frames[0],
            &palette.colors_multiply(3),
            "../../../test_assets/output/EDG1001_3.png",
        );
        save_frame(
            &frm.directions[0].frames[0],
            &palette.colors_multiply(4),
            "../../../test_assets/output/EDG1001_4.png",
        );
    }

    #[test]
    fn colored_animation() {
        let file = std::fs::read("../../../test_assets/COLOR.PAL").unwrap();
        let (_, palette) = palette::palette_verbose(&file).unwrap();
        let palette4 = palette.colors_multiply(4);

        let file = std::fs::read("../../../test_assets/HMWARRAA.FRM").unwrap();
        let (_, frm) = frm::frm_verbose(&file).unwrap();

        for (dir_index, dir) in frm.directions.iter().enumerate() {
            for (frame_index, frame) in dir.frames.iter().enumerate() {
                save_frame(
                    &frame,
                    &palette4,
                    format!(
                        "../../test_assets/output/HMWARRAA_{}_{}.png",
                        dir_index, frame_index
                    ),
                );
            }
        }
    }

    #[test]
    fn print_frm_animation_info() {
        let fo_data = FoData::init("../../../CL4RP", "../../../test_assets/COLOR.PAL").unwrap();
        let bytes = retriever::retrieve_file(&fo_data, "art/scenery/gizsign.frm").unwrap();
        let (rest, frm) = frm::frm_verbose(&bytes).unwrap();
        println!("{:?}", frm);
    }
}
