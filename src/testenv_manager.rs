use crate::config::{ConnectionConfig, DistroConfig};
use std::io::Error;
use std::process::Command;

pub struct TestEnvManager {
    startup_script: String,
    stop_script: String,
    connection: ConnectionConfig,
}

impl TestEnvManager {
    pub fn new(config: &DistroConfig) -> Self {
        TestEnvManager {
            startup_script: config.startup_script.clone(),
            stop_script: config.stop_script.clone(),
            connection: config.connection.clone(),
        }
    }

    pub fn start(&self) -> Result<(), Error> {
        Command::new("bash")
            .arg(&self.startup_script)
            .env_remove("USER")
            .env_remove("PASSWORD")
            .env_remove("ADDRESS")
            .env_remove("PORT")
            .env(
                "USER",
                self.connection.username.as_deref().unwrap_or("root"),
            )
            .env(
                "PASSWORD",
                self.connection.password.as_deref().unwrap_or(""),
            )
            .env(
                "ADDRESS",
                self.connection.ip.as_deref().unwrap_or("localhost"),
            )
            .env("PORT", self.connection.port.unwrap_or(2222).to_string())
            .spawn()?
            .wait()?;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), Error> {
        Command::new("bash")
            .arg(&self.stop_script)
            .spawn()?
            .wait()?;
        Ok(())
    }
}
