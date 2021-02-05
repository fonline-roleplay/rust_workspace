use structopt::StructOpt;
use wgpu::{BackendBit, PowerPreference};
use std::{str::FromStr,
    fmt::{
        Display, Formatter,
    },
    time::Duration,
};
use serde::{Deserialize};

#[derive(Debug)]
pub(crate) struct Config {
    args: Args,
    file: ConfigFile,
}

#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    wait: Option<bool>,
    hipow: Option<bool>,
    mode: Option<String>,
    reparent: Option<bool>,
    minfps: Option<u16>,
    maxfps: Option<u16>,
    backends: Option<Vec<Backend>>,
    url: Option<String>,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "FOnline Overlay", author = "qthree <qthree3@gmail.com>")]
pub(crate) struct Args {
    /// Wait for client window
    #[structopt(short)]
    wait: Option<bool>,

    // Prefer high power discrete gpu over integrated one
    #[structopt(long, short="H")]
    hipow: Option<bool>,

    // How overlay should keep windows on top of game client?
    // "reparent" - Transfer overlay windows ownership to game client. Can result in strange client behavior, like sticky mouse or keyboard buttons
    // "autosort" - If client or any of overlay windows is foreground then make one window topmost and sort all the other windows and client under topmost window. May be flickery.
    // "topmost" - Make all overlay windows topmost, Old overlay style. Can't place other windows over overlay windows, so have to hide overlay if client or overlay is not foreground.
    #[structopt(long, short="M")]
    mode: Option<String>,

    /// Sets the max sleep time between frames based on desired FPS, actual FPS can be a lot higher because of the reaction to events
    #[structopt(long)]
    minfps: Option<u16>,

    /// Sets the minimal sleep time between frames
    #[structopt(long)]
    maxfps: Option<u16>,

    /// Permited backends to use: Vulkan, Dx12, Dx11
    #[structopt(long, short="B")]
    backends: Option<Vec<Backend>>,

    /// Sets the client process id
    #[structopt(long)]
    pid: Option<u32>,

    /// Sets the web server address
    #[structopt(name = "URL")]
    url: Option<String>,
}
impl Config {
    pub(super) fn load() -> Self {
        let args = StructOpt::from_args();
        let file = match std::fs::read_to_string("OverlayConfig.toml") {
            Err(err) => {
                eprintln!("Overlay config not found: {:?}", err);
                Default::default()
            },
            Ok(string) => {
                match toml::from_str(&string) {
                    Err(err) => {
                        eprintln!("Can't parse overlay config: {:?}", err);
                        Default::default()
                    }
                    Ok(config) => {
                        config
                    }
                }
            }
        };
        Self{args, file}
    }
    pub(crate) fn backend_bits(&self) -> BackendBit {
        if let Some(backends) = self.args.backends.as_deref().or(self.file.backends.as_deref()) {
            let mut bits = BackendBit::empty();
            for backend in backends {
                bits |= backend.to_wgpu_backend().into()
            }
            bits   
        } else {
            BackendBit::PRIMARY
        }
    }
    pub(crate) fn power_preference(&self) -> PowerPreference {
        if self.args.hipow.or(self.file.hipow).unwrap_or(false) {
            PowerPreference::HighPerformance
        } else {
            PowerPreference::LowPower
        }   
    }
    pub(crate) fn wait(&self) -> bool {
        self.args.wait.or(self.file.wait).unwrap_or(false)
    }
    pub(crate) fn min_delay(&self) -> Duration {
        let max_fps = self.args.minfps.or(self.file.minfps).unwrap_or(60).max(12).min(1000);
        Duration::from_millis(1000 / max_fps as u64)
    }
    pub(crate) fn max_delay(&self) -> Duration {
        let min_fps = self.args.minfps.or(self.file.minfps).unwrap_or(12).max(12).min(1000);
        Duration::from_millis(1000 / min_fps as u64)
    }
    pub(crate) fn pid(&self) -> Option<u32> {
        self.args.pid
    }
    pub(crate) fn url(&self) -> Option<&str> {
        self.args.url.as_deref().or(self.file.url.as_deref())
    }
    pub(crate) fn mode(&self) -> OverlayMode {
        self.args.mode.as_deref().or(self.file.mode.as_deref()).map(|mode| {
            match mode {
                "reparent" => OverlayMode::Reparent,
                "autosort" => OverlayMode::AutoSort,
                "topmost" => OverlayMode::TopMost,
                _ => panic!("Unsupported mode: {}", mode),
            }
        }).unwrap_or(OverlayMode::AutoSort)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum OverlayMode {
    Reparent,
    AutoSort,
    TopMost,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub(crate) enum Backend {
    #[serde(alias = "vulkan", alias = "VULKAN", alias = "VK", alias = "Vk")]
    Vulkan = 1,
    #[serde(alias = "dx12", alias = "DX12")]
    Dx12 = 3,
    #[serde(alias = "dx11", alias = "DX11")]
    Dx11 = 4,
}

impl Backend {
    fn to_wgpu_backend(self) -> wgpu::Backend {
        match self {
            Backend::Vulkan => wgpu::Backend::Vulkan,
            Backend::Dx12 => wgpu::Backend::Dx12,
            Backend::Dx11 => wgpu::Backend::Dx11,
        }
    }
}

#[derive(Debug)]
pub(crate) struct UnsupportedBackend;
impl Display for UnsupportedBackend {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UnsupportedBackend")
    }
}

impl FromStr for Backend {
    type Err = UnsupportedBackend;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Vulkan" | "vulkan" | "VULKAN" | "VK" | "Vk" => Backend::Vulkan,
            "Dx12" | "dx12" | "DX12" => Backend::Dx12,
            "Dx11" | "dx11" | "DX11" => Backend::Dx11,
            _ => return Err(UnsupportedBackend),
        })
    }
}
