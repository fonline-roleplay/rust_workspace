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
                    if entry.is_dir() {
                        continue;
                    }
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

pub fn shadowed_files(archives: &[PathBuf]) -> Result<Vec<(String, u64, &Path, &Path)>, Error> {
    assert!(archives.len() <= u16::max_value() as usize);

    let mut path_map = PathMap::new();
    let mut shadowed = Vec::with_capacity(512);

    for (archive_index, path) in archives.iter().enumerate() {
        println!("Crawling {:?}", path);
        let archive_file = std::fs::File::open(&path).unwrap();
        let buf_reader = BufReader::with_capacity(1024, archive_file);
        let mut archive = zip::ZipArchive::new(buf_reader).unwrap();
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).unwrap();
            if entry.is_dir() {
                continue;
            }
            let entry_name = entry.name();
            let old = path_map.insert(
                nom_prelude::make_path_conventional(entry_name),
                FileInfo {
                    location: FileLocation::Archive(archive_index as u16),
                    original_path: entry_name.to_owned(),
                    compressed_size: entry.compressed_size(),
                    ..Default::default()
                },
            );
            if let Some(old) = old {
                if let FileLocation::Archive(old_index) = old.location {
                    shadowed.push((
                        old.original_path,
                        old.compressed_size,
                        archives[old_index as usize].as_path(),
                        archives[archive_index].as_path(),
                    ));
                }
            }
        }
    }
    Ok(shadowed)
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
