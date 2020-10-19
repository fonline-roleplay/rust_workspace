use once_cell::sync::OnceCell;
use std::path::PathBuf;

struct Config {
    name: &'static str,
    reports_dir: &'static str,
    starting_dir: PathBuf,
}

impl Config {
    fn report(&self) -> std::io::Result<(PathBuf, String)> {
        let mut path = self.starting_dir.join(self.reports_dir);
        std::fs::create_dir_all(&path)?;
        let timedate = time::OffsetDateTime::now_local().format("%-Y_%m_%d_%H_%M_%S");
        let filename = format!("{}_{}.txt", timedate, self.name);
        let local = format!("{}/{}", self.reports_dir, filename);
        path.push(filename);
        Ok((path, local))
    }
}

static CONFIG: OnceCell<Config> = OnceCell::new();

pub fn setup(reports_dir: &'static str, name: &'static str) {
    std::panic::set_hook(Box::new(panic_handler));
    let config = Config {
        name,
        reports_dir,
        starting_dir: std::env::current_dir().expect("Starting directory"),
    };
    if let Err(_) = CONFIG.set(config) {
        panic!("Called achtung::setup() second time.");
    }
}

fn full_message(exe: &str, info: &std::panic::PanicInfo) -> String {
    format!(
        "Current executable: {:?},\n\
        Panic info:\n  {}\n\
        Backtrace:\n{:#?}",
        exe,
        info,
        backtrace::Backtrace::new()
    )
}

pub fn panic_handler(info: &std::panic::PanicInfo) {
    let exe = std::env::current_exe().ok();
    let exe = exe
        .as_ref()
        .and_then(|exe| exe.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("?");
    let local = CONFIG.get().and_then(|config| {
        let (path, local) = config.report().ok()?;
        let full_message = full_message(exe, info);
        println!(
            "\n\
            =================================\n\
            ===========>> PANIC! <<==========\n\
            =================================\n\
            \n{}",
            &full_message
        );
        std::fs::write(path, full_message)
            .ok()
            .map(|_| format!("Подробное сообщение в файле:\n{:?}\n", local))
    });

    let message = format!(
        "Произошла критическая ошибка в {:?}, сообщите об этом разработчикам!\n\
        {}\
        Краткое сообщение:\n{}",
        exe,
        local.unwrap_or(String::new()),
        info,
    );
    let _ = msgbox::create("Шеф! Все пропало!", &message, msgbox::IconType::Error);
    std::process::exit(1);
}
