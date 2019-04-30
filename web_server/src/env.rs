use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub fn dir<P1: AsRef<OsStr>, P2: AsRef<Path>>(env: P1, file: P2) -> Option<PathBuf> {
    let env = std::env::var_os(env);
    if let Some(path) = env.and_then(|env| Path::new(&env).canonicalize().ok()) {
        Some(path)
    } else if let Ok(path) =
        std::fs::read_to_string(file).and_then(|env| Path::new(&env).canonicalize())
    {
        Some(path)
    } else {
        None
    }
}

pub fn expect_clients() -> PathBuf {
    dir("TNF_CLIENTS_PATH", "./tnf_clients.path").expect("Don't know where are clients' savefiles")
}

pub fn setup_working_dir() -> std::io::Result<()> {
    if let Some(working_dir) = dir("TNF_WEB_PATH", "./tnf_web.path") {
        std::env::set_current_dir(working_dir)?;
    }
    Ok(())
}
