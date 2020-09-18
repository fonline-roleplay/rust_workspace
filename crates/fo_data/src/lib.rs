//mod converter;
pub mod crawler;
pub mod datafiles;
mod frm;
mod palette;
mod retriever;

use once_cell::sync::OnceCell;
use std::borrow::Cow;
use std::collections::BTreeMap;
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
    cache: OnceCell<bytes::Bytes>,
    compressed_size: u64,
}

pub struct FoData {
    archives: Vec<std::path::PathBuf>,
    files: PathMap<String, FileInfo>,
    palette: Vec<(u8, u8, u8)>,
}

pub struct FileData {
    pub file_type: FileType,
    pub data: bytes::Bytes,
    pub offset_x: i16,
    pub offset_y: i16,
}

#[derive(Debug)]
pub enum FileType {
    Png,
    Frm,
    Gif,
    Unsupported(String),
    Unknown,
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
    pub fn get_image(&self, path: &str) -> Result<FileData, GetImageError> {
        type Error = GetImageError;
        let file_type = retriever::recognize_type(path);
        let retriever = move || retriever::retrieve_file(self, path).map_err(Error::Retrieve);
        Ok(match file_type {
            FileType::Png => {
                let data = retriever()?;
                let mut slice = &data[..];
                let decoder =
                    image::png::PngDecoder::new(slice).map_err(GetImageError::PngDecode)?;
                use image::ImageDecoder;
                let (width, height) = decoder.dimensions();

                FileData {
                    data: data.clone(),
                    file_type,
                    offset_x: width as i16 / -2,
                    offset_y: height as i16 * -1,
                }
            }
            FileType::Frm => {
                let data = retriever()?;
                let frm = frm::frm(&data).map_err(Error::FrmParse)?;
                let frame_number = 0;
                let direction0 = frm.directions.get(0).ok_or(Error::EmptyFrm)?;
                let offsets = direction0.frames.iter().skip(1).take(frame_number);
                let offset_x: i16 = offsets.clone().map(|frame| frame.offset_x).sum();
                let offset_y: i16 = offsets.map(|frame| frame.offset_y).sum();
                let frame = direction0.frames.get(frame_number).ok_or(Error::EmptyFrm)?;
                let size = (frame.width as usize * frame.height as usize + 512).next_power_of_two();
                let image = image::GrayImage::from_raw(
                    frame.width as u32,
                    frame.height as u32,
                    frame.data.to_owned(),
                )
                .ok_or(Error::ImageFromRaw)?;
                let image =
                    image::DynamicImage::ImageRgba8(image.expand_palette(&self.palette, Some(0)));
                let mut data = Vec::with_capacity(size);

                image
                    .write_to(&mut data, image::ImageFormat::Png)
                    .map_err(Error::ImageWrite)?;
                FileData {
                    data: data.into(),
                    file_type: FileType::Png,
                    offset_x: direction0.shift_x + offset_x - frame.width as i16 / 2,
                    offset_y: direction0.shift_y + offset_y - frame.height as i16,
                }
            }
            _ => return Err(Error::FileType(file_type)),
        })
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
