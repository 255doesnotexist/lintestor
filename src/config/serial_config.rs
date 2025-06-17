//! 串口连接配置
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub username: Option<String>,
    pub password: Option<String>,
    pub user_prompt: Option<String>, // 输入用户名的pattern
    pub pass_prompt: Option<String>, // 输入密码的pattern
    pub shell_prompt: String,        // shell提示符pattern
}
