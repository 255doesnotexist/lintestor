//! Represents the configuration for each distro.
use crate::config::connection_config::ConnectionConfig;
/// This struct is used to deserialize the configuration from a file using the `utils::read_toml_from_file` method.
/// It contains the following fields:
/// - `testing_type`: A string representing the type of testing ('locally' or 'remote' or 'qemu-based-remote').
/// - `startup_script`: A string representing the startup script.
/// - `stop_script`: A string representing the stop script.
/// - `connection`: An instance of `ConnectionConfig` struct representing the connection configuration.
/// - `skip_packages`: An optional vector of strings representing the packages to be skipped.
///
use serde::Deserialize;

impl DistroConfig {
    #[allow(dead_code)]
    pub fn is_not_qemu_based_remote(value: &String) -> bool { // keep this function as it is, just for serde plz
        value != "qemu-based-remote"
    }
}

#[derive(Debug, Deserialize)]
pub struct DistroConfig {
    pub enabled: bool,
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
