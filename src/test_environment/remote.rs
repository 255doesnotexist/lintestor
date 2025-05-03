//! 远程 SSH 测试环境的实现

use crate::test_environment::{CommandOutput, TestEnvironment};
use log::{debug, trace, warn};
use ssh2::{Session, Sftp};
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
};

/// 远程 SSH 测试环境的实现
pub struct RemoteEnvironment {
    remote_ip: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key_path: Option<String>,
    session: Option<Session>,
    sftp: Option<Sftp>,
}

impl RemoteEnvironment {
    pub fn new(
        remote_ip: String,
        port: u16,
        username: String,
        password: Option<String>,
        private_key_path: Option<String>,
    ) -> Self {
        RemoteEnvironment {
            remote_ip,
            port,
            username,
            password,
            private_key_path,
            session: None,
            sftp: None,
        }
    }

    // 获取连接的会话，如果需要则建立连接
    fn ensure_session(&mut self) -> Result<&Session, Box<dyn Error>> {
        if self.session.is_none() {
            self.connect_ssh()?;
        }
        self.session
            .as_ref()
            .ok_or_else(|| "SSH 会话不可用".into())
    }

    // 获取 SFTP 客户端，如果需要则建立
    fn ensure_sftp(&mut self) -> Result<&mut Sftp, Box<dyn Error>> {
        if self.sftp.is_none() {
            let session = self.ensure_session()?;
            let sftp = session.sftp()?;
            self.sftp = Some(sftp);
        }
        self.sftp
            .as_mut()
            .ok_or_else(|| "SFTP 客户端不可用".into())
    }

    // 建立 SSH 连接
    fn connect_ssh(&mut self) -> Result<(), Box<dyn Error>> {
        trace!("尝试 SSH 连接到 {}:{}", self.remote_ip, self.port);
        let tcp = TcpStream::connect((self.remote_ip.as_str(), self.port))?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        debug!("SSH 握手完成");

        // 身份验证逻辑
        let mut authenticated = false;

        // 1. 尝试公钥认证（代理）
        if !authenticated {
            match sess.userauth_agent(&self.username) {
                Ok(_) => {
                    debug!("SSH 代理认证成功");
                    authenticated = true;
                }
                Err(e) => {
                    // 不检查特定的错误代码，只处理通用错误
                    debug!("SSH 代理认证失败: {}", e);
                }
            }
        }

        // 2. 尝试公钥认证（文件）
        if !authenticated {
            if let Some(private_key_path_str) = &self.private_key_path {
                let private_key_path = if private_key_path_str.starts_with('~') {
                    dirs::home_dir()
                        .map(|home| home.join(private_key_path_str.trim_start_matches("~/")))
                        .unwrap_or_else(|| PathBuf::from(private_key_path_str))
                } else {
                    PathBuf::from(private_key_path_str)
                };

                if private_key_path.exists() {
                    match sess.userauth_pubkey_file(&self.username, None, &private_key_path, None) {
                        Ok(_) => {
                            debug!(
                                "SSH 公钥文件认证成功 ({})",
                                private_key_path.display()
                            );
                            authenticated = true;
                        }
                        Err(e) => {
                            debug!(
                                "SSH 公钥文件认证失败 ({}): {}",
                                private_key_path.display(), e
                            );
                        }
                    }
                } else {
                    debug!("私钥文件未找到: {}", private_key_path.display());
                }
            }
        }

        // 3. 尝试密码认证
        if !authenticated {
            if let Some(password) = &self.password {
                match sess.userauth_password(&self.username, password) {
                    Ok(_) => {
                        debug!("SSH 密码认证成功");
                        authenticated = true;
                    }
                    Err(e) => {
                        debug!("SSH 密码认证失败: {}", e);
                    }
                }
            }
        }

        if !authenticated {
            return Err("所有 SSH 认证方法都失败".into());
        }

        self.session = Some(sess);
        self.sftp = None; // 在新会话上重置 SFTP 客户端
        Ok(())
    }
}

impl TestEnvironment for RemoteEnvironment {
    fn setup(&mut self) -> Result<(), Box<dyn Error>> {
        debug!("RemoteEnvironment setup called.");
        self.ensure_session()?; // 在设置期间建立连接
        Ok(())
    }

    fn teardown(&mut self) -> Result<(), Box<dyn Error>> {
        debug!("RemoteEnvironment teardown called.");
        if let Some(mut sess) = self.session.take() {
            debug!("断开 SSH 会话连接。");
            // 如果存在则先关闭 SFTP
            if self.sftp.is_some() {
                self.sftp = None;
            }
            // 断开连接时忽略错误，因为我们无论如何都要清理
            let _ = sess.disconnect(None, "客户端断开连接", None);
        }
        Ok(())
    }

    fn run_command(&self, command: &str) -> Result<CommandOutput, Box<dyn Error>> {
        let session = self
            .session
            .as_ref()
            .ok_or("SSH 会话未建立。先调用 setup()。")?;

        trace!("运行远程命令: {}", command);
        let mut channel = session.channel_session()?;
        channel.exec(command)?;

        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();
        channel.stream(0).read_to_end(&mut stdout_buf)?; // 读取 stdout
        channel.stderr().read_to_end(&mut stderr_buf)?; // 读取 stderr

        channel.send_eof()?;
        channel.wait_close()?;
        let exit_status = channel.exit_status()?;

        let stdout_str = String::from_utf8_lossy(&stdout_buf).to_string();
        let stderr_str = String::from_utf8_lossy(&stderr_buf).to_string();

        // 记录输出
        if !stdout_str.is_empty() {
            debug!("stdout:\n{}", stdout_str);
        }
        if !stderr_str.is_empty() {
            debug!("stderr:\n{}", stderr_str);
        }
        debug!("Exit status: {}", exit_status);

        Ok(CommandOutput {
            command: command.to_string(),
            exit_status,
            output: format!("stdout:\n{}\nstderr:\n{}", stdout_str, stderr_str),
        })
    }

    fn upload_file(
        &self,
        local_path: &Path,
        remote_path: &str,
        mode: i32,
    ) -> Result<(), Box<dyn Error>> {
        let session = self
            .session
            .as_ref()
            .ok_or("SSH 会话未建立。先调用 setup()。")?;

        trace!(
            "上传 {:?} 到远程 {} 并设置模式 {:o}",
            local_path,
            remote_path,
            mode
        );
        let mut local_file = File::open(local_path)?;
        let file_size = local_file.metadata()?.len();

        // 使用 SCP 发送，如果不强烈需要 SFTP 或以不同方式管理
        let mut remote_file = session.scp_send(Path::new(remote_path), mode, file_size, None)?;

        // 读取本地文件并写入远程
        std::io::copy(&mut local_file, &mut remote_file)?;

        // 确保数据被刷新，文件在远程端关闭
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;

        Ok(())
    }

    fn download_file(&self, remote_path: &str, local_path: &Path) -> Result<(), Box<dyn Error>> {
        let session = self
            .session
            .as_ref()
            .ok_or("SSH 会话未建立。先调用 setup()。")?;

        trace!("下载远程 {} 到 {:?}", remote_path, local_path);
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let (mut remote_file, _stat) = session.scp_recv(Path::new(remote_path))?;
        let mut local_file = File::create(local_path)?;

        std::io::copy(&mut remote_file, &mut local_file)?;

        // 关闭远程通道
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;

        Ok(())
    }

    fn read_remote_file(&self, remote_path: &str) -> Result<String, Box<dyn Error>> {
        // 使用 run_command 运行 cat 命令来读取文件
        trace!("读取远程文件 {}", remote_path);
        // 使用带引号的路径以防止特殊字符
        let quoted_path = format!("\"{}\"", remote_path.replace("\"", "\\\""));
        let cat_command = format!("cat {}", quoted_path);
        let output = self.run_command(&cat_command)?;
        if output.exit_status == 0 {
            // 从输出中提取内容
            Ok(output.output
                .lines()
                .find(|line| line.starts_with("stdout:"))
                .map(|line| line.trim_start_matches("stdout:").trim())
                .unwrap_or("")
                .to_string())
        } else {
            Err(format!(
                "读取远程文件 '{}' 失败: {}",
                remote_path, output.output
            )
            .into())
        }
    }

    fn mkdir(&self, remote_path: &str) -> Result<(), Box<dyn Error>> {
        trace!("创建远程目录 {}", remote_path);
        // 使用带引号的路径以防止特殊字符
        let quoted_path = format!("\"{}\"", remote_path.replace("\"", "\\\""));
        let command = format!("mkdir -p {}", quoted_path);
        let output = self.run_command(&command)?;
        if output.exit_status == 0 {
            Ok(())
        } else {
            Err(format!(
                "创建远程目录 '{}' 失败: {}",
                remote_path, output.output
            )
            .into())
        }
    }

    fn rm_rf(&self, remote_path: &str) -> Result<(), Box<dyn Error>> {
        trace!("删除远程路径 {}", remote_path);
        // 使用带引号的路径以防止特殊字符
        let quoted_path = format!("\"{}\"", remote_path.replace("\"", "\\\""));
        let command = format!("rm -rf {}", quoted_path);
        let output = self.run_command(&command)?;
        if output.exit_status == 0 {
            Ok(())
        } else {
            Err(format!(
                "删除远程路径 '{}' 失败: {}",
                remote_path, output.output
            )
            .into())
        }
    }

    fn get_os_info(&self) -> Result<(String, String), Box<dyn Error>> {
        trace!("获取远程操作系统信息");
        let os_version = self.read_remote_file("/proc/version")?;
        let kernel_output = self.run_command("uname -r")?;
        let kernel_version = kernel_output.output
            .lines()
            .find(|line| line.starts_with("stdout:"))
            .map(|line| line.trim_start_matches("stdout:").trim())
            .unwrap_or("")
            .to_string();
            
        if kernel_version.is_empty() && kernel_output.exit_status != 0 {
            return Err(format!("获取内核版本失败: {}", kernel_output.output).into());
        }
        
        Ok((os_version, kernel_version))
    }
}

// 为 RemoteEnvironment 实现 Drop 以确保调用 teardown
impl Drop for RemoteEnvironment {
    fn drop(&mut self) {
        if let Err(e) = self.teardown() {
            warn!("RemoteEnvironment teardown 过程中出错: {}", e);
        }
    }
}