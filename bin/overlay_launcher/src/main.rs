//#![windows_subsystem = "windows"]

fn main() {
    let file_out = std::fs::File::create("FOnlineOverlay.log").expect("overlay log file");
    let file_err = file_out.try_clone().expect("overlay err log file");
    let mut child = std::process::Command::new("FOnlineOverlay.exe")
        .args(std::env::args().skip(1))
        .env("RUST_BACKTRACE", "1")
        .stdout(file_out)
        .stderr(file_err)
        .spawn()
        .expect("Can't spawn");
    child.wait().expect("Error waiting");
}
