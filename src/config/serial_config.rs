//! 串口连接配置
use humantime_serde;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub username: Option<String>,
    pub password: Option<String>,
    pub login_prompt: Option<String>, // 进入shell的pattern
    pub user_prompt: Option<String>,  // 输入用户名的pattern
    pub pass_prompt: Option<String>,  // 输入密码的pattern
    pub shell_prompt: String,         // shell提示符pattern
    #[serde(with = "humantime_serde")] // 支持toml/yaml友好时间格式
    pub timeout: Duration,
}
