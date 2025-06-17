//! 执行器配置参数
use humantime_serde;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
pub struct ExecutorConfig {
    #[serde(with = "humantime_serde", default, skip_serializing_if = "Option::is_none")]
    pub command_timeout: Option<Duration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_interval: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maintain_session: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continue_on_error: Option<bool>, // 其实就是 !interactive
    // #[serde(default, skip_serializing_if = "Option::is_none", with = "humantime_serde")]
    // pub connection_timeout: Option<Duration>,
    // TODO: 目前连接超时直接用的命令超时时间
}

// fn default_connection_timeout() -> Duration {
//     Duration::from_secs(15)
// }

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            command_timeout: Some(Duration::from_secs(30)),
            retry_count: Some(3),
            retry_interval: Some(5),
            maintain_session: Some(true),
            continue_on_error: Some(false),
            // connection_timeout: Some(default_connection_timeout()),
        }
    }
}
