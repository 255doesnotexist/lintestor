//! Represents the configuration for a connection.
///
/// This struct is used to store the connection details such as the IP address, port, username, and password.
/// It is used in the context of a larger application to define how the application connects to a remote server or service.
///
/// # Fields
///
/// - `ip`: An optional string representing the IP address.
/// - `port`: An optional unsigned 16-bit integer representing the port number.
/// - `username`: An optional string representing the username.
/// - `password`: An optional string representing the password.
/// - `private_key_path`: An optional string representing the path to the private key file.
/// - `jump_hosts`: An optional vector of strings representing the jump hosts to use.
/// - `max_retries`: An optional integer representing the maximum number of connection retry attempts.
/// - `connect_timeout_secs`: An optional integer representing the connection timeout in seconds.
///
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionConfig {
    ip: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    private_key_path: Option<String>,
    public_key_path: Option<String>,
    jump_hosts: Option<Vec<String>>,
    max_retries: Option<u8>,
    connect_timeout_secs: Option<u64>,
}

impl ConnectionConfig {
    /// 获取IP地址，如果未配置则返回"localhost"
    pub fn get_ip(&self) -> &str {
        self.ip.as_deref().unwrap_or("localhost")
    }
    
    /// 获取端口号，如果未配置则返回默认值22
    pub fn get_port(&self) -> u16 {
        self.port.unwrap_or(22)
    }
    
    /// 获取用户名，如果未配置则返回默认值"root"
    pub fn get_username(&self) -> &str {
        self.username.as_deref().unwrap_or("root")
    }
    
    /// 获取密码，如果未配置则返回None
    pub fn get_password(&self) -> Option<&str> {
        self.password.as_deref()
    }
    
    /// 获取私钥路径，如果未配置则返回None
    pub fn get_private_key_path(&self) -> Option<&str> {
        self.private_key_path.as_deref()
    }

    /// 获取公钥路径，如果未配置则返回None
    pub fn get_public_key_path(&self) -> Option<&str> {
        self.public_key_path.as_deref()
    }
    
    /// 获取跳板机列表，如果未配置则返回None
    pub fn get_jump_hosts(&self) -> Option<&Vec<String>> {
        self.jump_hosts.as_ref()
    }
    
    /// 获取最大重试次数，如果未配置则返回默认值3
    pub fn get_max_retries(&self) -> u8 {
        self.max_retries.unwrap_or(3)
    }
    
    /// 获取连接超时时间（秒），如果未配置则返回默认值15秒
    pub fn get_connect_timeout_secs(&self) -> u64 {
        self.connect_timeout_secs.unwrap_or(15)
    }
}
