use std::path::{Path, PathBuf};

pub fn working_dir() -> Option<PathBuf> {
    let env = std::env::var_os("TNF_WEB_PATH");
    if let Some(path) = env.and_then(|env| Path::new(&env).canonicalize().ok()) {
        Some(path)
    } else if let Ok(path) =
        std::fs::read_to_string("./tnf_web.path").and_then(|env| Path::new(&env).canonicalize())
    {
        Some(path)
    } else {
        None
    }
}

pub fn setup_working_dir() -> std::io::Result<()> {
    if let Some(working_dir) = working_dir() {
        std::env::set_current_dir(working_dir)?;
    }
    Ok(())
}
