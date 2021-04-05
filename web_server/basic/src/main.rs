use std::path::{Path, PathBuf};
use web_server_core::*;

fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        //.filter(Some("actix_web"), LevelFilter::Debug)
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

    let (mrhandy, join_handle) = if let Some(discord) = &config.discord {
        let (a, b) = mrhandy::start(&discord.bot.token, discord.main_guild_id);
        (Some(a), Some(b))
    } else {
        (None, None)
    };

    let state = web::AppState::new(config, mrhandy, db);
    web::run(state);
    //db.flush().expect("Can't flush sled database");
    join_handle.map(|handle| handle.join().unwrap());
    Ok(())
}
