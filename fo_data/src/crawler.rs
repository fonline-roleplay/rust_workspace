use crate::{datafiles::parse_datafile, FileInfo, FileLocation, PathMap};
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {}

pub fn gather_paths(archives: &[PathBuf]) -> Result<PathMap<String, FileInfo>, Error> {
    assert!(archives.len() <= u16::max_value() as usize);

    let mut path_map = PathMap::new();

    /*use rayon::prelude::{
        IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelExtend,
        ParallelIterator,
    };*/
    path_map.extend(
        archives
            .iter()
            .enumerate()
            .flat_map(|(archive_index, path)| {
                println!("Crawling {:?}", path);
                let archive_file = std::fs::File::open(&path).unwrap();
                let buf_reader = BufReader::with_capacity(1024, archive_file);
                let mut archive = zip::ZipArchive::new(buf_reader).unwrap();
                let mut local_path_map = PathMap::new();
                for i in 0..archive.len() {
                    let mut entry = archive.by_index(i).unwrap();
                    let entry_name = entry.name();
                    local_path_map.insert(
                        nom_prelude::make_path_conventional(entry_name),
                        FileInfo {
                            location: FileLocation::Archive(archive_index as u16),
                            original_path: entry_name.to_owned(),
                            ..Default::default()
                        },
                    );
                }
                local_path_map
            }),
    );
    Ok(path_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gather_paths() {
        let archives = crate::datafiles::parse_datafile("../../CL4RP").unwrap();
        let res = gather_paths(&archives).unwrap();
        for (entry_name, info) in &res {
            match info.location {
                FileLocation::Local => {
                    println!("{:?} => local", entry_name);
                }
                FileLocation::Archive(index) => {
                    println!("{:?} => {:?}", entry_name, &archives[index as usize]);
                }
            }
        }
    }
}
