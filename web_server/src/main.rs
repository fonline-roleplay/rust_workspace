mod critter_info;
mod critters_db;
mod env;
mod fix_encoding;
mod templates;
mod webserver;

fn main() -> std::io::Result<()> {
    let clients = env::expect_clients();
    println!("Clients dir: {:?}", clients);
    env::setup_working_dir()?;
    println!("Working dir: {:?}", std::env::current_dir());
    webserver::run(clients);
    Ok(())
}
