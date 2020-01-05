use log::LevelFilter;
use std::path::PathBuf;
use tnf_web_server::*;

fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        //.filter(Some("actix_web"), LevelFilter::Debug)
        .init();

    let config = config::setup().expect("config.toml file");
    //println!("{:?}", config);

    let items = Default::default();
    let fo_data = fo_data::FoData::stub();

    let mut db_path = PathBuf::new();
    db_path.push("db");
    db_path.push("sled");
    let db = sled::open(db_path).expect("Can't open sled database");

    let (mrhandy, join_handle) =
        mrhandy::start(&config.discord.bot.token, config.discord.main_guild_id);

    let state = web::AppState::new(config, mrhandy, db, fo_data, items);
    web::run(state);
    //db.flush().expect("Can't flush sled database");
    join_handle.join().unwrap();
    Ok(())
}
