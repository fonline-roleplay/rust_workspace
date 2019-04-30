mod critter_info;
mod critters_db;
mod env;
mod fix_encoding;
mod templates;
mod webserver;

fn main() -> std::io::Result<()> {
    env::setup_working_dir()?;
    webserver::run();
    Ok(())
}
