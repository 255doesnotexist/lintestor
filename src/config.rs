use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub distros: Vec<String>,
    pub packages: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DistroConfig {
    pub testing_type: String, // 'locally' or 'remote' or 'qemu-based-remote'
    #[serde(rename = "startup_script")]
    #[serde(default, skip_serializing_if = "is_not_qemu_based_remote")]
    pub startup_script: String, 

    #[serde(rename = "stop_script")]
    #[serde(default, skip_serializing_if = "is_not_qemu_based_remote")]
    pub stop_script: String,

    #[serde(rename = "connection")]
    pub connection: ConnectionConfig,
    pub skip_packages: Option<Vec<String>>,
}

impl DistroConfig {
    fn is_not_qemu_based_remote(&self) -> bool {
        self.testing_type != "qemu-based-remote"
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionConfig {
    pub method: String,
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

impl DistroConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: DistroConfig = toml::de::from_str(&contents)?;
        Ok(config)
    }
}
