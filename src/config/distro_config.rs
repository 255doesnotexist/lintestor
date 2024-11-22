//! Represents the configuration for each distro.
use crate::config::connection_config::ConnectionConfig;
use crate::config::boardtest_config::BoardtestConfig;
/// This struct is used to deserialize the configuration from a file using the `utils::read_toml_from_file` method.
/// It contains the following fields:
/// - `testing_type`: A string representing the type of testing ('locally' or 'remote' or 'qemu-based-remote').
/// - `startup_script`: A string representing the startup script.
/// - `stop_script`: A string representing the stop script.
/// - `connection`: An instance of `ConnectionConfig` struct representing the connection configuration.
/// - `boardtest`: An instance of `BoardtestConfig` struct representing the boardtest configuration.
/// - `skip_packages`: An optional vector of strings representing the packages to be skipped.
///
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
    #[serde(default, skip_serializing_if = "is_not_remote")]
    pub connection: Option<ConnectionConfig>,

    #[serde(rename = "boardtest")]
    #[serde(default, skip_serializing_if = "is_not_boardtest")]
    pub boardtest: Option<BoardtestConfig>,

    pub skip_packages: Option<Vec<String>>,
}
