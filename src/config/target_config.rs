//! Represents the configuration for each target.

use crate::config::connection_config::ConnectionConfig;
use crate::config::serial_config::SerialConfig;
use crate::utils;
/// This struct is used to deserialize the configuration from a file using the `utils::read_toml_from_file` method.
/// It contains the following fields:
/// - `testing_type`: A string representing the type of testing ('locally' or 'remote' or 'qemu-based-remote').
/// - `connection`: An instance of `ConnectionConfig` struct representing the connection configuration.
/// - `boardtest`: An instance of `BoardtestConfig` struct representing the boardtest configuration.
/// - `skip_units`: An optional vector of strings representing the units to be skipped.
/// - `serial`: An instance of `SerialConfig` struct representing the serial connection configuration (only required when testing_type is 'serial').
use serde::Deserialize;

#[allow(dead_code)]
fn is_not_qemu_based_remote(value: &String) -> bool {
    // keep this function as it is, just for serde plz
    value != "qemu-based-remote"
}

#[allow(dead_code)]
fn is_not_remote(value: &String) -> bool {
    // keep this function as it is, just for serde plz
    value != "remote" && value != "qemu-based-remote"
}

#[allow(dead_code)]
fn is_not_boardtest(value: &String) -> bool {
    // keep this function as it is, just for serde plz
    value != "boardtest"
}

#[allow(dead_code)]
fn is_not_serial(value: &String) -> bool {
    // keep this function as it is, just for serde plz
    value != "serial"
}

#[derive(Debug, Deserialize, Clone)]
pub struct TargetConfig {
    pub testing_type: String, // 'locally' or 'remote' or 'qemu-based-remote' or 'boardtest' or 'serial'

    #[serde(rename = "connection")]
    #[serde(default, skip_serializing_if = "is_not_remote")]
    pub connection: Option<ConnectionConfig>,

    #[serde(rename = "serial")]
    #[serde(default, skip_serializing_if = "is_not_serial")]
    pub serial: Option<SerialConfig>,
}

impl TargetConfig {
    /// 获取测试类型
    #[allow(dead_code)]
    pub fn get_testing_type(&self) -> &str {
        &self.testing_type
    }
    
    /// 获取连接配置
    #[allow(dead_code)]
    pub fn get_connection(&self) -> Option<&ConnectionConfig> {
        self.connection.as_ref()
    }

    /// 从文件中读取
    pub fn from_file(file_path: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        use std::path::PathBuf;
        let path = PathBuf::from(file_path);
        let config: Self = utils::read_toml_from_file(&path)?;
        Ok(config)
    }
}