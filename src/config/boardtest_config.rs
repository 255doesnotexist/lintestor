use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BoardtestConfig {
    pub token: String,
    pub board_config: String,  // Path to board config TOML file
    pub serial: String,        // Serial number for SD Mux device
    #[serde(default)]
    pub mi_sdk_enabled: bool,  // Optional: Enable Mi SDK controller
    #[serde(default = "default_api_url")]
    pub api_url: String,       // API server URL
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,     // Test timeout in seconds
}

fn default_api_url() -> String {
    "http://localhost:8000".to_string()
}

fn default_timeout() -> u64 {
    300 // 5 minutes in seconds
}