//! 本地测试环境的实现

use crate::test_environment::{CommandOutput, TestEnvironment};
use log::{debug, log_enabled, Level};
use std::error::Error;
use std::fs::{self, read_to_string};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

/// 本地测试环境的实现
pub struct LocalEnvironment {}

impl LocalEnvironment {
    pub fn new() -> Self {
        LocalEnvironment {}
    }
}

impl Default for LocalEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl TestEnvironment for LocalEnvironment {
    fn setup(&mut self) -> Result<(), Box<dyn Error>> {
        // 本地环境无需特殊设置
        debug!("LocalEnvironment setup called.");
        Ok(())
    }

    fn teardown(&mut self) -> Result<(), Box<dyn Error>> {
        // 本地环境无需特殊清理
        debug!("LocalEnvironment teardown called.");
        Ok(())
    }

    fn run_command(&self, command: &str) -> Result<CommandOutput, Box<dyn Error>> {
        debug!("Running local command: {}", command);
        let output = Command::new("bash")
            .arg("-c")
            .arg(command)
            // 无论日志级别如何都捕获 stdout/stderr 用于 CommandOutput
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

        // 如果启用了调试，则记录输出
        if log_enabled!(Level::Debug) {
            if !stdout_str.is_empty() {
                debug!("stdout:\n{}", stdout_str);
            }
            if !stderr_str.is_empty() {
                debug!("stderr:\n{}", stderr_str);
            }
            debug!("Exit status: {:?}", output.status.code());
        }

        Ok(CommandOutput {
            command: command.to_string(),
            exit_status: output.status.code().unwrap_or(1), // 如果被信号终止提供默认退出码
            output: format!("stdout:\n{}\nstderr:\n{}", stdout_str, stderr_str), // 合并 stdout/stderr 以兼容，后续考虑分开
        })
    }

    fn upload_file(
        &self,
        local_path: &Path,
        remote_path: &str,
        mode: i32,
    ) -> Result<(), Box<dyn Error>> {
        debug!(
            "Uploading local file {:?} to {:?} with mode {:o}",
            local_path, remote_path, mode
        );
        // 确保父目录存在
        if let Some(parent) = Path::new(remote_path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(local_path, remote_path)?;
        let metadata = fs::metadata(remote_path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(mode as u32); // 设置权限
        fs::set_permissions(remote_path, permissions)?;
        Ok(())
    }

    fn download_file(&self, remote_path: &str, local_path: &Path) -> Result<(), Box<dyn Error>> {
        debug!(
            "Downloading local file {:?} to {:?}",
            remote_path, local_path
        );
        // 确保父目录存在
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(remote_path, local_path)?;
        Ok(())
    }

    fn read_remote_file(&self, remote_path: &str) -> Result<String, Box<dyn Error>> {
        debug!("Reading local file {:?}", remote_path);
        Ok(read_to_string(remote_path)?)
    }

    fn mkdir(&self, remote_path: &str) -> Result<(), Box<dyn Error>> {
        debug!("Creating local directory {:?}", remote_path);
        fs::create_dir_all(remote_path)?;
        Ok(())
    }

    fn rm_rf(&self, remote_path: &str) -> Result<(), Box<dyn Error>> {
        debug!("Removing local path {:?}", remote_path);
        if Path::new(remote_path).is_dir() {
            fs::remove_dir_all(remote_path)?;
        } else if Path::new(remote_path).exists() {
            fs::remove_file(remote_path)?;
        }
        Ok(())
    }

    fn get_os_info(&self) -> Result<(String, String), Box<dyn Error>> {
        debug!("Getting local OS info");
        let os_version = read_to_string("/proc/version")?;
        let kernel_output = self.run_command("uname -r")?;

        // 从输出中提取内核版本
        let kernel_version = kernel_output
            .output
            .lines()
            .find(|line| line.starts_with("stdout:"))
            .map(|line| line.trim_start_matches("stdout:").trim())
            .unwrap_or("")
            .to_string();

        Ok((os_version, kernel_version))
    }
}
