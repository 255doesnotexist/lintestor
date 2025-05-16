//! Represents the configuration for a connection.
///
/// This struct is used to store the connection details such as the IP address, port, username, and password.
/// It is used in the context of a larger application to define how the application connects to a remote server or service.
///
/// # Fields
///
/// - `ip`: Astring representing the IP address.
/// - `port`: An unsigned 16-bit integer representing the port number.
/// - `username`: A string representing the username.
/// - `password`: An optional string representing the password.
/// - `private_key_path`: An optional string representing the path to the private key file.
/// - `jump_hosts`: An optional vector of strings representing the jump hosts to use.
/// - `max_retries`: A integer representing the maximum number of connection retry attempts.
/// - `timeout`: A integer representing the connection timeout in seconds.
///
use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionConfig {
    pub ip: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub public_key_path: Option<String>,
    pub jump_hosts: Option<Vec<String>>,
    pub max_retries: u8,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
}
impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            ip: "localhost".to_string(),
            port: 22,
            username: "root".to_string(),
            password: None,
            private_key_path: None,
            public_key_path: None,
            jump_hosts: None,
            max_retries: 3,
            timeout: Duration::from_secs(15),
        }
    }
}
