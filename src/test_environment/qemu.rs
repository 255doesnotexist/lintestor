//! QEMU 虚拟机测试环境的实现

use crate::test_environment::remote::RemoteEnvironment;
use crate::test_environment::TestEnvironment;
use log::{debug, error, info};
use std::error::Error;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use std::thread;

/// QEMU 虚拟机测试环境的实现
pub struct QemuEnvironment {
    // 基础远程环境
    remote_env: RemoteEnvironment,
    // 启动和停止脚本路径
    startup_script: String,
    stop_script: String,
    // 工作目录
    working_dir: std::path::PathBuf,
    // 连接尝试次数
    max_attempts: u32,
    // 连接尝试间隔（秒）
    retry_interval: u64,
}

impl QemuEnvironment {
    /// 创建一个新的 QEMU 环境
    ///
    /// # 参数
    /// - `remote_ip`: QEMU 虚拟机的 IP 地址
    /// - `port`: SSH 端口
    /// - `username`: 用户名
    /// - `password`: 可选密码
    /// - `private_key_path`: 可选私钥路径
    /// - `startup_script`: 启动 QEMU 虚拟机的脚本路径
    /// - `stop_script`: 停止 QEMU 虚拟机的脚本路径
    /// - `working_dir`: 工作目录路径
    pub fn new(
        remote_ip: String,
        port: u16,
        username: String,
        password: Option<String>,
        private_key_path: Option<String>,
        startup_script: String,
        stop_script: String,
        working_dir: &Path,
    ) -> Self {
        QemuEnvironment {
            remote_env: RemoteEnvironment::new(remote_ip, port, username, password, private_key_path),
            startup_script,
            stop_script,
            working_dir: working_dir.to_path_buf(),
            max_attempts: 10, // 默认重试10次
            retry_interval: 5, // 默认每次间隔5秒
        }
    }

    /// 执行 QEMU 启动脚本
    fn start_qemu(&self) -> Result<(), Box<dyn Error>> {
        if self.startup_script.is_empty() {
            debug!("未指定 QEMU 启动脚本，跳过启动步骤");
            return Ok(());
        }

        info!("启动 QEMU 虚拟机: {}", self.startup_script);
        let startup_script_path = self.working_dir.join(&self.startup_script);
        
        if !startup_script_path.exists() {
            return Err(format!("启动脚本不存在: {}", startup_script_path.display()).into());
        }

        debug!("执行启动脚本: {}", startup_script_path.display());
        
        let output = Command::new("bash")
            .arg(startup_script_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("启动 QEMU 失败: {stderr}");
            return Err(format!("启动 QEMU 失败: {stderr}").into());
        }

        info!("QEMU 虚拟机启动成功，等待 SSH 可用");
        Ok(())
    }

    /// 停止 QEMU 虚拟机
    fn stop_qemu(&self) -> Result<(), Box<dyn Error>> {
        if self.stop_script.is_empty() {
            debug!("未指定 QEMU 停止脚本，跳过停止步骤");
            return Ok(());
        }

        info!("停止 QEMU 虚拟机: {}", self.stop_script);
        let stop_script_path = self.working_dir.join(&self.stop_script);
        
        if !stop_script_path.exists() {
            return Err(format!("停止脚本不存在: {}", stop_script_path.display()).into());
        }

        debug!("执行停止脚本: {}", stop_script_path.display());
        
        let output = Command::new("bash")
            .arg(stop_script_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("停止 QEMU 失败: {stderr}");
            return Err(format!("停止 QEMU 失败: {stderr}").into());
        }

        info!("QEMU 虚拟机停止成功");
        Ok(())
    }
    
    /// 等待 SSH 连接可用
    fn wait_for_ssh(&mut self) -> Result<(), Box<dyn Error>> {
        let start_time = Instant::now();
        let timeout = Duration::from_secs(60); // 最长等待1分钟
        
        info!("等待 QEMU 虚拟机的 SSH 服务启动...");
        
        let mut attempt = 0;
        while start_time.elapsed() < timeout {
            attempt += 1;
            debug!("尝试 SSH 连接 (尝试 {attempt})");
            
            // 尝试建立连接
            match self.remote_env.setup() {
                Ok(_) => {
                    info!("SSH 连接成功 (尝试 {attempt})");
                    return Ok(());
                }
                Err(e) => {
                    debug!("SSH 连接尝试失败: {e}, 等待重试");
                    // 在重试前，确保先断开任何可能的部分连接
                    let _ = self.remote_env.teardown();
                }
            }
            
            // 等待指定的间隔后重试
            thread::sleep(Duration::from_secs(self.retry_interval));
        }
        
        Err(format!(
            "等待 SSH 连接超时，{attempt} 次尝试后失败"
        ).into())
    }

    // 设置重试参数
    pub fn with_retry_params(mut self, max_attempts: u32, retry_interval: u64) -> Self {
        self.max_attempts = max_attempts;
        self.retry_interval = retry_interval;
        self
    }
}

impl TestEnvironment for QemuEnvironment {
    fn setup(&mut self) -> Result<(), Box<dyn Error>> {
        debug!("QemuEnvironment setup called.");
        
        // 步骤 1: 启动 QEMU 虚拟机
        self.start_qemu()?;
        
        // 步骤 2: 等待 SSH 连接就绪
        self.wait_for_ssh()?;
        
        Ok(())
    }

    fn teardown(&mut self) -> Result<(), Box<dyn Error>> {
        debug!("QemuEnvironment teardown called.");
        
        // 先断开 SSH 连接
        self.remote_env.teardown()?;
        
        // 然后停止 QEMU 虚拟机
        self.stop_qemu()?;
        
        Ok(())
    }

    // 委托所有方法到 remote_env
    fn run_command(&self, command: &str) -> Result<crate::utils::CommandOutput, Box<dyn Error>> {
        self.remote_env.run_command(command)
    }

    fn upload_file(&self, local_path: &Path, remote_path: &str, mode: i32) -> Result<(), Box<dyn Error>> {
        self.remote_env.upload_file(local_path, remote_path, mode)
    }

    fn download_file(&self, remote_path: &str, local_path: &Path) -> Result<(), Box<dyn Error>> {
        self.remote_env.download_file(remote_path, local_path)
    }

    fn read_remote_file(&self, remote_path: &str) -> Result<String, Box<dyn Error>> {
        self.remote_env.read_remote_file(remote_path)
    }

    fn mkdir(&self, remote_path: &str) -> Result<(), Box<dyn Error>> {
        self.remote_env.mkdir(remote_path)
    }

    fn rm_rf(&self, remote_path: &str) -> Result<(), Box<dyn Error>> {
        self.remote_env.rm_rf(remote_path)
    }

    fn get_os_info(&self) -> Result<(String, String), Box<dyn Error>> {
        self.remote_env.get_os_info()
    }
}

// 为 QemuEnvironment 实现 Drop 以确保调用 teardown
impl Drop for QemuEnvironment {
    fn drop(&mut self) {
        if let Err(e) = self.teardown() {
            error!("QemuEnvironment teardown 过程中出错: {e}");
        }
    }
}