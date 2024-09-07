use serde::Deserialize;
use std::fs;
use crate::config::connection_config::ConnectionConfig;

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
    pub fn is_not_qemu_based_remote(&self) -> bool {
        self.testing_type != "qemu-based-remote"
    }
    
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: DistroConfig = toml::de::from_str(&contents)?;
        Ok(config)
    }
}
