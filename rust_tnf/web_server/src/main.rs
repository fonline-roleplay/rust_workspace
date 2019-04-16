mod critter_info;
mod critters_db;
mod templates;
mod webserver;

fn main() -> std::io::Result<()> {
    let working_dir = std::env::var_os("TNF_PATH");
    let working_dir = working_dir
        .as_ref()
        .map(|dir| dir.as_os_str())
        .unwrap_or("../../web/".as_ref());
    let working_dir = std::path::Path::new(working_dir).canonicalize()?;
    println!("Current dir: {:?}", working_dir);
    std::env::set_current_dir(working_dir)?;
    webserver::run();
    Ok(())
}
