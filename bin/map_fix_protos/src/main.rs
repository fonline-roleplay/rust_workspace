use fo_map_format::{verbose_read_file, MapObjectType};
use fo_proto_format::ProtoItem;
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

fn _read_patches() -> Result<BTreeMap<u16, Patch>, Box<dyn std::error::Error>> {
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

fn items() -> BTreeMap<u16, ProtoItem> {
    fo_proto_format::build_btree("../../proto/items/items.lst")
}

fn item_type_to_map_type(item_type: u8) -> MapObjectType {
    match item_type {
        10 | 11 | 12 => MapObjectType::MAP_OBJECT_SCENERY, // ITEM_TYPE_GRID, ITEM_TYPE_GENERIC, ITEM_TYPE_WALL => MAP_OBJECT_SCENERY
        _ => MapObjectType::MAP_OBJECT_ITEM, // evything else => MAP_OBJECT_ITEM
    }
}

fn main() {
    let items = items();

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
                .filter(|obj| {
                    obj.kind.map_object_type() != MapObjectType::MAP_OBJECT_CRITTER
                })
                .filter_map(|obj| {
                    items
                        .get(&obj.proto_id)
                        .map(|proto| item_type_to_map_type(proto.Type))
                        .filter(|proto_map_type| *proto_map_type != obj.kind.map_object_type())
                        .map(|proto_map_type| {
                            let bytes = obj.ty_str.as_bytes();
                            let offset = u64::wrapping_sub(bytes.as_ptr() as _, text_bytes as _);
                            (offset, bytes.len(), proto_map_type as u8)
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
