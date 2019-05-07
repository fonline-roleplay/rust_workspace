use tnf_web_server::*;

fn main() -> std::io::Result<()> {
    let clients = env::expect_clients();
    println!("Clients dir: {:?}", clients);
    env::setup_working_dir()?;
    println!("Working dir: {:?}", std::env::current_dir());
    webserver::run(clients);
    Ok(())
}
