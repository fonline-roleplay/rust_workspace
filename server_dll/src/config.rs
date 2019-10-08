use derivative::Derivative;
use once_cell::sync::OnceCell;
use serde::Deserialize;

const MAP_UTILITY_START: u16 = 92;
const DEFAULT_FAST_FROM: u16 = 2200;
const DEFAULT_FAST_TO: u16 = u16::max_value();

#[derive(Deserialize, Default)]
pub struct ServerConfig {
    pub check_look: CheckLook,
}

#[derive(Deserialize, Derivative)]
#[derivative(Default)]
pub struct CheckLook {
    pub npc: CheckLookNpc,
    #[derivative(Default(value = "MAP_UTILITY_START"))]
    pub map_utility_start: u16,
}

#[derive(Deserialize, Derivative)]
#[derivative(Default)]
pub struct CheckLookNpc {
    #[derivative(Default(value = "true"))]
    pub fast: bool,
    #[derivative(Default(value = "CheckLookNpc::default_fast_from()"))]
    #[serde(default = "CheckLookNpc::default_fast_from")]
    pub fast_from: u16,
    #[derivative(Default(value = "CheckLookNpc::default_fast_to()"))]
    #[serde(default = "CheckLookNpc::default_fast_to")]
    pub fast_to: u16,
}
impl CheckLookNpc {
    const fn default_fast_from() -> u16 {
        DEFAULT_FAST_FROM
    }
    const fn default_fast_to() -> u16 {
        DEFAULT_FAST_TO
    }
}

static CONFIG: OnceCell<ServerConfig> = OnceCell::new();

pub fn config() -> &'static ServerConfig {
    CONFIG.get_or_init(|| {
        let res = load_config();
        match res {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Error loading server config: {:?}", err);
                Default::default()
            }
        }
    })
}

#[derive(Debug)]
enum ConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
}

fn load_config() -> Result<ServerConfig, ConfigError> {
    let str = std::fs::read_to_string("ServerConfig.toml").map_err(ConfigError::Io)?;
    let config: ServerConfig = toml::de::from_str(&str).map_err(ConfigError::Toml)?;
    Ok(config)
}
