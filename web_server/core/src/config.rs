use serde::Deserialize;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Cert {
    pub full_chain: PathBuf,
    pub key: PathBuf,
}
impl Cert {
    pub fn server_config(&self) -> Result<rustls::ServerConfig, ()> {
        let cert = std::fs::read(&self.full_chain).expect("Full-chain cert file");
        let key = std::fs::read(&self.key).expect("Private key file");

        let certs =
            rustls::internal::pemfile::certs(&mut cert.as_slice()).expect("Parsed full-chain cert");
        let key = rustls::internal::pemfile::pkcs8_private_keys(&mut key.as_slice())
            .ok()
            .and_then(|mut keys| {
                if keys.is_empty() {
                    None
                } else {
                    Some(keys.remove(0))
                }
            })
            .expect("Parsed private key");
        let mut tls_config = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        tls_config.set_single_cert(certs, key).unwrap();
        Ok(tls_config)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Url {
    pub domain: String,
    pub port: Option<u16>,
}
impl Url {
    pub fn url(&self, protocol: &str, url: &str) -> String {
        match self.port {
            Some(port) => format!("{}://{}:{}{}", protocol, self.domain, port, url),
            None => format!("{}://{}{}", protocol, self.domain, url),
        }
    }
    pub fn domain_port(&self) -> String {
        match self.port {
            Some(port) => format!("{}:{}", self.domain, port),
            None => self.domain.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Host {
    pub web_tls: Option<Cert>,
    pub web: Url,
    pub files: Url,
    #[serde(default)]
    pub overlay_use_files: bool,
}

impl Host {
    pub fn web_url(&self, url: &str) -> String {
        let protocol = if self.web_tls.is_some() {
            "https"
        } else {
            "http"
        };
        self.web.url(protocol, url)
    }
    pub fn web_port(&self) -> u16 {
        match (&self.web.port, &self.web_tls) {
            (&Some(port), _) => port,
            (None, Some(_)) => 443,
            (None, None) => 80,
        }
    }
    pub fn files_url(&self, url: &str) -> String {
        self.files.url("http", url)
    }
    pub fn files_port(&self) -> u16 {
        self.files.port.unwrap_or(80)
    }

    pub fn overlay_urls(&self) -> String {
        if self.overlay_use_files {
            format!("{}|{}\0", self.web_url(""), self.files_url(""))
        } else {
            self.web_url("\0")
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuth {
    pub client_id: String,
    pub secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Bot {
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Roles {
    pub admin: String,
    pub developer: String,
    pub gamemaster: String,
    pub player: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Discord {
    pub main_guild_id: u64,
    pub oauth2: OAuth,
    pub bot: Bot,
    pub roles: Roles,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum PrivatePaths {
    Vec(Vec<PathBuf>),
    #[serde(skip)]
    Map(BTreeMap<String, PathBuf>),
}
impl PrivatePaths {
    fn setup(&mut self) -> Result<(), ConfigError> {
        if let PrivatePaths::Vec(vec) = self {
            for path in &mut *vec {
                canon(path)?;
            }
            let mut privates = BTreeMap::new();

            for path in vec.drain(..) {
                let name = Some(path.as_path())
                    .filter(|path| path.is_dir())
                    .and_then(|path| path.file_name())
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| ConfigError::PrivateNotDir(path.clone()))?;
                let old = privates.insert(name.to_owned(), path);
                if let Some(old) = old {
                    return Err(ConfigError::PrivateSameName(old));
                }
            }
            *self = PrivatePaths::Map(privates);
        };
        Ok(())
    }
}
#[derive(Debug, Deserialize, Clone)]
pub struct Paths {
    pub save_clients: PathBuf, // "../../FO4RP/save/clients/"
    pub proto_items: PathBuf,  // "../../FO4RP/proto/items/items.lst"
    #[cfg(feature = "fo_map_format")]
    pub maps: PathBuf, // "../../FO4RP/maps/"
    pub working_dir: PathBuf,  // "../web"
    #[cfg(feature = "fo_data")]
    pub game_client: PathBuf, // "../../CL4RP"
    #[cfg(feature = "fo_data")]
    pub palette: PathBuf, // "../../FO4RP/proto/items/items.lst"
    pub private: PrivatePaths, // ["../../FO4RP/logs", "../../FO4RP/dumps", "../../FO4RP/save"]
}

impl Paths {
    pub fn privates(&self) -> &BTreeMap<String, PathBuf> {
        match &self.private {
            PrivatePaths::Map(map) => map,
            _ => panic!("PrivatePaths::setup wasn't successful"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Base64 {
    String(String),
    #[serde(skip)]
    Bytes([u8; 32]),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Session {
    cookie_key: Option<Base64>,
}
impl Session {
    fn setup_key(&mut self) -> Result<(), ConfigError> {
        if let Some(key) = &mut self.cookie_key {
            if let Base64::String(string) = &*key {
                let bytes = base64::decode(string).map_err(ConfigError::SessionKeyDecode)?;
                if bytes.len() != 32 {
                    return Err(ConfigError::SessionKeyLengthNot32(bytes.len()));
                }
                let mut buf = [0u8; 32];
                buf.copy_from_slice(&bytes);
                *key = Base64::Bytes(buf);
            }
        } else {
            let bytes: [u8; 32] = rand::random();
            let key = base64::encode(&bytes[..]);
            println!(
                "You need private key for cookie session, put this in the config: \"{}\"",
                key
            );
            return Err(ConfigError::NoSessionKey);
        }
        Ok(())
    }
    pub fn cookie_key(&self) -> &[u8; 32] {
        match self.cookie_key.as_ref().and_then(|base64| {
            if let Base64::Bytes(bytes) = base64 {
                Some(bytes)
            } else {
                None
            }
        }) {
            Some(key) => key,
            None => panic!("Session setup wasn't successful"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub host: Host,
    pub paths: Paths,
    pub discord: Option<Discord>,
    pub session: Session,
    #[serde(default)]
    pub bridge: Bridge,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    Canonicalize(PathBuf, std::io::Error),
    PrivateSameName(PathBuf),
    PrivateNotDir(PathBuf),
    NoSessionKey,
    SessionKeyDecode(base64::DecodeError),
    SessionKeyLengthNot32(usize),
}

fn canon(path: &mut PathBuf) -> Result<(), ConfigError> {
    *path = path
        .canonicalize()
        .map_err(|err| ConfigError::Canonicalize(path.clone(), err))?;
    Ok(())
}

pub fn setup() -> Result<Config, ConfigError> {
    if std::path::Path::new("./working.path").exists() {
        let path = std::fs::read_to_string("./working.path").unwrap();
        std::env::set_current_dir(path).unwrap();
    }

    let toml = std::fs::read_to_string("./config.toml").map_err(ConfigError::Io)?;
    let mut config: Config = toml::from_str(&toml).map_err(ConfigError::Toml)?;
    config.session.setup_key()?;

    let paths = &mut config.paths;
    canon(&mut paths.save_clients)?;
    #[cfg(feature = "fo_proto_format")]
    canon(&mut paths.proto_items)?;
    #[cfg(feature = "fo_map_format")]
    canon(&mut paths.maps)?;
    canon(&mut paths.working_dir)?;
    #[cfg(feature = "fo_data")]
    canon(&mut paths.game_client)?;
    #[cfg(feature = "fo_data")]
    canon(&mut paths.palette)?;
    paths.private.setup()?;

    std::env::set_current_dir(&paths.working_dir).map_err(ConfigError::Io)?;
    Ok(config)
}
