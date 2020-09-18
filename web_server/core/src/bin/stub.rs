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

    let (mrhandy, join_handle) = if let Some(discord) = &config.discord {
        let (a, b) = mrhandy::start(&discord.bot.token, discord.main_guild_id);
        (Some(a), Some(b))
    } else {
        (None, None)
    };

    let state = web::AppState::new(config, mrhandy, db, fo_data, items);
    web::run(state);
    //db.flush().expect("Can't flush sled database");
    join_handle.map(|handle| handle.join().unwrap());
    Ok(())
}
