use fo_map_format::verbose_read_file;
use nom_prelude::nom_err_to_string;
use serde::Deserialize;
use std::collections::btree_map::BTreeMap;

#[derive(Deserialize, Debug)]
struct Patch {
    pid_name: String,
    pid_old: u16,
    pid_new: u16,
    type_new: u8,
}

fn read_patches() -> Result<BTreeMap<u16, Patch>, Box<dyn std::error::Error>> {
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_path("proto_patches.csv")?;
    let mut patches = BTreeMap::new();
    for result in rdr.deserialize() {
        let record: Patch = result?;
        if let Some(old) = patches.insert(record.pid_old, record) {
            panic!("Two patches with the same pid_old: {:?}", old.pid_old);
        }
    }
    Ok(patches)
}

fn main() {
    let patches = read_patches().expect("Read patches from file");

    for file in std::fs::read_dir("../../maps/")
        .unwrap()
        .filter_map(|r| r.ok())
    {
        let file = file.path();
        if !file.is_file() || file.extension() != Some("fomap".as_ref()) {
            continue;
        }
        println!("Parsing {:?}", file);

        let changes = verbose_read_file(&file, |text, res| {
            let (rest, map) = nom_err_to_string(text, res).expect("Can't parse map file");
            assert!(rest.is_empty());

            let text_bytes = text.as_bytes().as_ptr();
            let changes: Vec<_> = map
                .objects
                .0
                .iter()
                .rev()
                .filter_map(|obj| {
                    patches
                        .get(&obj.proto_id)
                        .filter(|patch| patch.type_new != obj.kind.map_object_type() as u8)
                        .map(|patch| {
                            let bytes = obj.ty_str.as_bytes();
                            let offset = u64::wrapping_sub(bytes.as_ptr() as _, text_bytes as _);
                            (offset, bytes.len(), patch.type_new)
                        })
                })
                .collect();
            changes
        })
        .expect("Can't read map file");

        if changes.is_empty() {
            continue;
        }
        println!("Writing {} changes to {:?}", changes.len(), file);

        //std::fs::copy(&file, file.with_extension("fomap.backup")).expect("Backup copy");

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .open(file)
            .expect("Open map file to write changes");

        for (offset, len, val) in changes {
            assert_eq!(len, 1);
            assert!(val <= 9);
            let buf = [b'0' as u8 + val];
            use std::io::{Seek, SeekFrom, Write};
            file.seek(SeekFrom::Start(offset)).expect("Seek file");
            file.write(&buf).expect("Write new type value to file");
        }
    }
}
