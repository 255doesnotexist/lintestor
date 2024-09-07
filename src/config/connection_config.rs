use serde::Deserialize;
use std::fs;

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionConfig {
    pub method: String,
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}