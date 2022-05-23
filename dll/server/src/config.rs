use arc_swap::ArcSwap;
use derivative::Derivative;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{ops::Deref, sync::Arc, net::SocketAddr};

mod defaults {
    pub const MAP_UTILITY_START: u16 = 92;
    pub const NPC_FAST_FROM: u16 = 2200;
    pub const NPC_FAST_TO: u16 = u16::max_value();

    pub const DEFAULT_BONUS: u32 = 10;
    pub const DEFAULT_PERCEPTION: u32 = 5;

    pub const VIEW_BONUS: u32 = 10;
    pub const VIEW_PERCEPTION: u32 = 5;

    pub const HEAR_BONUS: u32 = 5;
    pub const HEAR_BONUS_NPC: u32 = 25;
    pub const HEAR_PERCEPTION: u32 = 2;

    pub const DIR_RATE_DEFAULT: [f32; 4] = [1.0; 4];
    //                                    0    1    2    3
    pub const DIR_RATE_VIEW: [f32; 4] = [1.0, 0.8, 0.5, 0.4];
    pub const DIR_RATE_HEAR: [f32; 4] = [0.8, 1.0, 0.8, 0.8];

    pub const MOVING_DEFAULT: f32 = 1.0;
    pub const MOVING_SELF_RUN: f32 = 0.8;
    pub const MOVING_TARGET_RUN: f32 = 3.0;

    pub const WALL_RATE_DEFAULT: [f32; 10] = [0.0; 10];
    pub const WALL_RATE_HEAR: [f32; 10] = [
        0.1, 0.1, 0.1, 0.1, // 1..=4
        0.3, 0.3, 0.3, 0.3, // 5..=8
        0.4, 0.4, // 9..=10
    ];
}
use defaults::*;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ServerConfig {
    pub check_look: CheckLook,
    #[serde(default)]
    pub bridge: Bridge,
}

#[derive(Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
pub struct CheckLook {
    pub npc_fast: NpcFast,
    #[derivative(Default(value = "vec![SenseRates::default_view(), SenseRates::default_hear()]"))]
    pub senses: Vec<SenseRates>,
    #[derivative(Default(value = "MAP_UTILITY_START"))]
    pub map_utility_start: u16,
}

#[derive(Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
pub struct SenseRates {
    pub player: CritterRates,
    pub npc: CritterRates,
    #[derivative(Default(value = "DIR_RATE_DEFAULT"))]
    pub dir_rate: [f32; 4],
    pub self_moving: MovingRates,
    pub target_moving: MovingRates,
    #[derivative(Default(value = "WALL_RATE_DEFAULT"))]
    pub wall_rate: [f32; 10],
}
impl SenseRates {
    fn default_view() -> Self {
        Self {
            dir_rate: DIR_RATE_VIEW,
            ..Default::default()
        }
    }
    fn default_hear() -> Self {
        Self {
            player: CritterRates::default_hear_player(),
            npc: CritterRates::default_hear_npc(),
            dir_rate: DIR_RATE_HEAR,
            self_moving: MovingRates::default_hear_self(),
            target_moving: MovingRates::default_hear_target(),
            wall_rate: WALL_RATE_HEAR,
        }
    }
}

/*
#[derivative(Default(value = "VIEW_BONUS"))]
    pub view_bonus: u32,
    #[derivative(Default(value = "VIEW_PERCEPTION"))]
    pub view_perception: u32,
    #[derivative(Default(value = "HEAR_BONUS"))]
    pub hear_bonus: u32,
    #[derivative(Default(value = "HEAR_PERCEPTION"))]
    pub hear_perception: u32,
    */

#[derive(Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
pub struct NpcFast {
    #[derivative(Default(value = "true"))]
    pub enable: bool,
    #[derivative(Default(value = "0"))]
    pub sense_index: usize,
    #[derivative(Default(value = "NPC_FAST_FROM"))]
    pub fast_from: u16,
    #[derivative(Default(value = "NPC_FAST_TO"))]
    pub fast_to: u16,
}

#[derive(Deserialize, Derivative)]
#[derivative(Default)]
pub struct CritterRates {
    #[derivative(Default(value = "DEFAULT_BONUS"))]
    pub basic_bonus: u32,
    #[derivative(Default(value = "DEFAULT_PERCEPTION"))]
    pub basic_perception_rate: u32,
}
impl CritterRates {
    const fn default_hear_npc() -> Self {
        Self {
            basic_bonus: HEAR_BONUS_NPC,
            basic_perception_rate: HEAR_PERCEPTION,
        }
    }
    const fn default_hear_player() -> Self {
        Self {
            basic_bonus: HEAR_BONUS,
            basic_perception_rate: HEAR_PERCEPTION,
        }
    }
}

#[derive(Deserialize, Derivative)]
#[derivative(Default)]
pub struct MovingRates {
    #[derivative(Default(value = "MOVING_DEFAULT"))]
    pub still: f32,
    #[derivative(Default(value = "MOVING_DEFAULT"))]
    pub walking: f32,
    #[derivative(Default(value = "MOVING_DEFAULT"))]
    pub running: f32,
}
impl MovingRates {
    fn default_hear_self() -> Self {
        MovingRates {
            running: MOVING_SELF_RUN,
            ..Default::default()
        }
    }
    fn default_hear_target() -> Self {
        MovingRates {
            running: MOVING_TARGET_RUN,
            ..Default::default()
        }
    }
}

#[derive(Deserialize)]
pub struct Bridge {
    pub addr: SocketAddr,
}
impl Bridge {
    fn defaul_addr() -> SocketAddr {
        "127.0.0.1:33852".parse().unwrap()
    }
}
impl Default for Bridge {
    fn default() -> Self {
        Self { addr: Self::defaul_addr() }
    }
}

static CONFIG: Lazy<ArcSwap<ServerConfig>> =
    Lazy::new(|| ArcSwap::new(Arc::new(load_config_or_default())));

pub fn config() -> impl Deref<Target = impl Deref<Target = ServerConfig>> {
    CONFIG.load()
}

#[no_mangle]
pub extern "C" fn reload_config() {
    let new_config = load_config_or_default();
    CONFIG.store(Arc::new(new_config));
}

#[derive(Debug)]
enum ConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
}

fn load_config_or_default() -> ServerConfig {
    let res = load_config();
    match res {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error loading server config: {:?}", err);
            Default::default()
        }
    }
}

fn load_config() -> Result<ServerConfig, ConfigError> {
    let str = std::fs::read_to_string("ServerConfig.toml").map_err(ConfigError::Io)?;
    let config: ServerConfig = toml::de::from_str(&str).map_err(ConfigError::Toml)?;
    Ok(config)
}
