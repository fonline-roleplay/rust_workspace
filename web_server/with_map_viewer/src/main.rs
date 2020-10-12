use std::path::{Path, PathBuf};
use web_server_core::*;

fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        //.filter(Some("actix_web"), LevelFilter::Debug)
        .init();
    if Path::new("./working.path").exists() {
        let path = std::fs::read_to_string("./working.path").unwrap();
        std::env::set_current_dir(path);
    }
    let config = config::setup().expect("config.toml file");
    //println!("{:?}", config);

    let items = fo_proto_format::build_btree(&config.paths.proto_items);

    let fo_data = fo_data::FoData::init(&config.paths.game_client, &config.paths.palette)
        .expect("FoData loading");
    println!(
        "FoData loaded, archives: {}, files: {}",
        fo_data.count_archives(),
        fo_data.count_files()
    );

    let mut db_path = PathBuf::new();
    db_path.push("db");
    db_path.push("sled");
    let db = sled::open(db_path).expect("Can't open sled database");

    //let (mrhandy, join_handle) =
    //    mrhandy::start(&config.discord.bot.token, config.discord.main_guild_id)

    let (mrhandy, join_handle) = if let Some(discord) = &config.discord {
        let (a, b) = mrhandy::start(&discord.bot.token, discord.main_guild_id);
        (Some(a), Some(b))
    } else {
        (None, None)
    };

    let state = web::AppState::new(config, mrhandy, db, fo_data.into_retriever(), items);
    web::run(state);
    //db.flush().expect("Can't flush sled database");
    //join_handle.join().unwrap();
    join_handle.map(|handle| handle.join().unwrap());
    Ok(())
}
