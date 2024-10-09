/// Represents the root configuration for the application.
///
/// This struct is used to deserialize the configuration from a file using the `utils::read_toml_from_file` method.
/// It contains two fields:
/// - `distros`: A vector of strings representing the supported distributions.
/// - `packages`: A vector of strings representing the packages to be installed.
///
/// # Example
///
/// ```
/// use lintestor::config::root_config::Config;
///
/// let config: Config = utils::read_toml_from_file("/path/to/config.toml");
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

#[derive(Debug, Deserialize)]

pub struct Config {
    pub distros: Vec<String>,
    pub packages: Vec<String>,
}
