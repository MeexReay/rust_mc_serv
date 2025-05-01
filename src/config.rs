use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;


#[derive(Debug, DefaultFromSerde, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    /// Хост где забиндить сервер
    #[serde(default = "default_host")] pub host: String,

    /// Таймаут подключения в секундах
    #[serde(default = "default_timeout")] pub timeout: u64,
}

fn default_host() -> String { "127.0.0.1:25565".to_string() }
fn default_timeout() -> u64 { 5 }

impl ServerConfig {
    pub fn load_from_file(path: PathBuf) -> Option<ServerConfig> {
        if !fs::exists(&path).unwrap_or_default() {
            let table = ServerConfig::default();
            fs::create_dir_all(&path.parent()?).ok()?;
            fs::write(&path, toml::to_string_pretty(&table).ok()?).ok()?;
            return Some(table);
        }
        let content = fs::read_to_string(&path).ok()?;
        let table = toml::from_str::<ServerConfig>(&content).ok()?;
        Some(table)
    }
}