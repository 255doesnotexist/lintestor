/// Represents the root configuration for the application.
///
/// This struct is used to deserialize the configuration from a file using the `from_file` method.
/// It contains two fields:
/// - `distros`: A vector of strings representing the supported distributions.
/// - `packages`: A vector of strings representing the packages to be installed.
///
/// # Example
///
/// ```
/// use lintestor::config::root_config::Config;
///
/// let config = Config::from_file("/path/to/config.toml");
/// match config {
///     Ok(config) => {
///         // Use the config object
///         println!("{:?}", config);
///     }
///     Err(err) => {
///         // Handle the error
///         eprintln!("Failed to load config: {}", err);
///     }
/// }
/// ```
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]

pub struct Config {
    pub distros: Vec<String>,
    pub packages: Vec<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
