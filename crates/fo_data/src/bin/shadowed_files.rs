use fo_data::{crawler::shadowed_files, datafiles::parse_datafile};

fn main() {
    let path = std::path::Path::new("../../../CL4RP")
        .canonicalize()
        .unwrap();
    let archives = parse_datafile(&path).expect("Parse datafiles");
    let files = shadowed_files(&archives).expect("Find shadowed files");
    let mut total_size = 0;
    for (name, size, old, new) in files {
        if old == new {
            continue;
        }
        println!(
            "File {:?} from {:?} replaced in {:?}",
            name,
            old.strip_prefix(&path).expect("strip prefix"),
            new.strip_prefix(&path).expect("strip prefix"),
        );
        total_size += size;
    }
    println!("Total shadowed size: {}", total_size);
}
