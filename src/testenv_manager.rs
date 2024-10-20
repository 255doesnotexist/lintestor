//! Manages the test environment for a distribution.
use crate::config::{connection_config::ConnectionConfig, distro_config::DistroConfig};
use std::io::Error;
use std::process::Command;

/// Manages the test environment for a distribution.
///
/// This struct is responsible for starting and stopping the test environment
/// using provided scripts and connection configuration.
pub struct TestEnvManager {
    startup_script: String,
    stop_script: String,
    connection: Option<ConnectionConfig>,
}

impl TestEnvManager {
    /// Creates a new `TestEnvManager` instance.
    ///
    /// # Arguments
    ///
    /// * `config` - A reference to a `DistroConfig` containing the necessary configuration.
    ///
    /// # Returns
    ///
    /// A new `TestEnvManager` instance initialized with the provided configuration.
    pub fn new(config: &DistroConfig) -> Self {
        TestEnvManager {
            startup_script: config.startup_script.clone(),
            stop_script: config.stop_script.clone(),
            connection: config.connection.clone(),
        }
    }

    /// Runs a bash script with environment variables set from the connection configuration.
    ///
    /// # Arguments
    ///
    /// * `script` - The path to the bash script to be executed.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` if the script runs successfully, or an `Error` if it fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the script fails to execute or returns a non-zero exit status.
    fn run_script(&self, script: &String) -> Result<(), Error> {
        let connection_unwrapped = self.connection.clone().unwrap();

        Command::new("bash")
            .arg(script)
            .env_remove("USER")
            .env_remove("PASSWORD")
            .env_remove("ADDRESS")
            .env_remove("PORT")
            .env(
                "USER",
                connection_unwrapped.username.as_deref().unwrap_or("root"),
            )
            .env(
                "PASSWORD",
                connection_unwrapped.password.as_deref().unwrap_or(""),
            )
            .env(
                "ADDRESS",
                connection_unwrapped.ip.as_deref().unwrap_or("localhost"),
            )
            .env(
                "PORT",
                connection_unwrapped.port.unwrap_or(2222).to_string(),
            )
            .spawn()?
            .wait()?;
        Ok(())
    }

    /// Starts the test environment by running the startup script.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` if the startup script runs successfully, or an `Error` if it fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the startup script fails to execute or returns a non-zero exit status.
    pub fn start(&self) -> Result<(), Error> {
        self.run_script(&self.startup_script)
    }

    /// Stops the test environment by running the stop script.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` if the stop script runs successfully, or an `Error` if it fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the stop script fails to execute or returns a non-zero exit status.
    pub fn stop(&self) -> Result<(), Error> {
        self.run_script(&self.stop_script)
    }
}
