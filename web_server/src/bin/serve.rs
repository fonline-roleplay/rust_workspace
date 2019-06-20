use tnf_web_server::*;
use log::LevelFilter;
use sled::Db;
use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        //.filter(Some("actix_web"), LevelFilter::Debug)
        .init();

    let clients = env::expect_clients();
    println!("Clients dir: {:?}", clients);
    env::setup_working_dir()?;
    println!("Working dir: {:?}", std::env::current_dir());

    let mut db_path = PathBuf::new();
    db_path.push("db");
    db_path.push("sled");
    let db = Db::start_default(db_path).expect("Can't open sled database");

    webserver::run(clients, db);
    //db.flush().expect("Can't flush sled database");
    Ok(())
}
