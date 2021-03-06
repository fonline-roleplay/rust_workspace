use std::path::{Path, PathBuf};
use web_server_core::*;

fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .filter(Some("actix_web"), log::LevelFilter::Info)
        .filter(Some("actix_server"), log::LevelFilter::Info)
        //.filter(Some("serenity"), log::LevelFilter::Info)
        .init();
    if Path::new("./working.path").exists() {
        let path = std::fs::read_to_string("./working.path").unwrap();
        std::env::set_current_dir(path).unwrap();
    }
    let config = config::setup().expect("config.toml file");
    //println!("{:?}", config);

    let mut db_path = PathBuf::new();
    db_path.push("db");
    db_path.push("sled");
    let db = sled::open(db_path).expect("Can't open sled database");

    let state = web::AppState::new(config, db);
    web::run(state);
    //db.flush().expect("Can't flush sled database");
    Ok(())
}
