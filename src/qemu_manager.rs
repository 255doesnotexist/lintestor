use std::process::Command;
use std::io::Error;
use crate::config::DistroConfig;

pub struct QemuManager {
    startup_script: String,
    stop_script: String,
}

impl QemuManager {
    pub fn new(config: &DistroConfig) -> Self {
        QemuManager {
            startup_script: config.startup_script.clone(),
            stop_script: config.stop_script.clone(),
        }
    }

    pub fn start(&self) -> Result<(), Error> {
        Command::new("bash")
            .arg(&self.startup_script)
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
