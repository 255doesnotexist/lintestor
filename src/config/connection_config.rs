/// Represents the configuration for a connection.
///
/// This struct is used to store the connection details such as the method, IP address, port, username, and password.
/// It is used in the context of a larger application to define how the application connects to a remote server or service.
///
/// # Fields
///
/// - `method`: A string representing the connection method.
/// - `ip`: An optional string representing the IP address.
/// - `port`: An optional unsigned 16-bit integer representing the port number.
/// - `username`: An optional string representing the username.
/// - `password`: An optional string representing the password.
///
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionConfig {
    pub method: String,
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}
