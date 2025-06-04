//! SSH连接管理器
//!
//! 该模块实现了基于SSH的远程命令执行管理器
//!
//! ## 跳板机支持
//!
//! 本模块支持通过一个或多个跳板机（jump hosts）建立SSH连接。跳板机的配置和凭据遵循以下规则：
//!
//! 1. 跳板机的凭据（如用户名、密码、私钥等）从用户的SSH配置文件（~/.ssh/config）和系统配置文件
//!    (/etc/ssh/ssh_config）中读取，而不是从本程序的配置文件中设置。
//!
//! 2. 跳板机可以使用别名，这些别名需要在SSH配置文件中定义。
//!
//! 3. 最终目标主机的凭据可以在配置文件中指定，也可以从SSH配置文件中读取。
//!
//! ## 示例
//!
//! 以下是一个使用跳板机的连接配置示例：
//!
//! ```toml
//! [connection]
//! method = "ssh"
//! ip = "target-server"
//! port = 22
//! username = "user"
//! jump_hosts = ["jumphost1", "user2@jumphost2:2222"]
//! ```
//!
//! 在此示例中，连接会首先通过jumphost1，然后通过jumphost2，最后到达target-server。
//! 跳板机的认证将使用本地SSH客户端的配置。

use anyhow::{bail, Context, Result};
use log::{debug, error, warn};
use ssh2::{Channel, Session};
use std::fs::File;
use std::io::Read;
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::config::connection_config::ConnectionConfig;
use crate::connection::{CommandOutput, ConnectionManager};

use crate::template::ExecutorOptions;

/// SSH连接管理器
pub struct SSHConnectionManager {
    /// SSH会话
    session: Session,
    /// 连接状态
    connected: bool,
    /// 保持会话状态
    maintain_session: bool,
    /// 环境变量
    env_vars: Vec<(String, String)>,
    #[allow(dead_code)]
    /// 执行器选项
    executor_options: ExecutorOptions,
}

impl SSHConnectionManager {
    /// 创建新的SSH连接管理器
    pub fn new(connection_config: &ConnectionConfig, _executor_options: &ExecutorOptions) -> Result<Self> {
        let host = &connection_config.ip;
        let port = connection_config.port;
        let username = &connection_config.username;
        let password = connection_config.password.as_deref();
        let private_key_path = connection_config.private_key_path.as_deref();
        let public_key_path = connection_config.public_key_path.as_deref();
        let jump_hosts = &connection_config.jump_hosts;

        debug!("创建SSH连接: {username}@{host}:{port}");

        // 从executor_options获取重试和超时设置
        let max_retries = _executor_options.retry_count;
        let connect_timeout_secs = _executor_options.command_timeout;

        // 处理跳板机连接
        if let Some(jumps) = jump_hosts {
            if !jumps.is_empty() {
                debug!("通过{}个跳板机创建SSH连接", jumps.len());

                // 使用系统SSH客户端建立带跳板机的连接并创建本地端口转发
                let local_port = Self::setup_ssh_proxy_with_jumphosts(
                    host,
                    port,
                    username,
                    jumps,
                    max_retries as usize,
                    connect_timeout_secs,
                )?;

                // 连接到本地转发端口
                debug!("连接到本地转发端口: localhost:{local_port}");
                let tcp = Self::connect_with_retry(
                    || TcpStream::connect(format!("localhost:{local_port}")),
                    max_retries as usize,
                    connect_timeout_secs,
                    &format!("无法连接到本地转发端口 localhost:{local_port}"),
                )?;

                // 创建SSH会话
                let mut session = Session::new().with_context(|| "无法创建SSH会话")?;
                session.set_tcp_stream(tcp);
                session.handshake().with_context(|| "SSH握手失败")?;

                // 身份验证
                Self::authenticate_session(
                    &mut session,
                    username,
                    password,
                    private_key_path,
                    public_key_path,
                )?;

                return Ok(Self {
                    session,
                    connected: true,
                    maintain_session: true,
                    env_vars: Vec::new(),
                    executor_options: _executor_options.clone(),
                });
            }
        }

        // 没有跳板机时的普通连接方式，使用重试机制
        let tcp = Self::connect_with_retry(
            || TcpStream::connect(format!("{host}:{port}")),
            max_retries as usize,
            connect_timeout_secs,
            &format!("无法连接到 {host}:{port}"),
        )?;

        // 创建SSH会话
        let mut session = Session::new().with_context(|| "无法创建SSH会话")?;
        session.set_tcp_stream(tcp);
        session.handshake().with_context(|| "SSH握手失败")?;

        // 身份验证
        Self::authenticate_session(
            &mut session,
            username,
            password,
            private_key_path,
            public_key_path,
        )?;

        Ok(Self {
            session,
            connected: true,
            maintain_session: true,
            env_vars: Vec::new(),
            executor_options: _executor_options.clone(),
        })
    }

    /// 设置SSH代理跳转并返回本地转发端口
    fn setup_ssh_proxy_with_jumphosts(
        final_host: &str,
        final_port: u16,
        final_username: &str,
        jump_hosts: &[String],
        max_retries: usize,
        connect_timeout_secs: u64,
    ) -> Result<u16> {
        use std::io::ErrorKind;
        use std::net::SocketAddr;
        use std::net::TcpListener;

        // 重试循环
        for retry in 0..max_retries {
            if retry > 0 {
                debug!("SSH转发连接重试 #{retry}");
                std::thread::sleep(Duration::from_secs(1)); // 重试前短暂等待
            }

            // 获取一个可用的本地端口用于端口转发
            let listener = match TcpListener::bind("127.0.0.1:0") {
                Ok(l) => l,
                Err(e) => {
                    if e.kind() == ErrorKind::AddrInUse {
                        continue; // 重试
                    } else {
                        return Err(anyhow::anyhow!("无法绑定本地端口: {}", e));
                    }
                }
            };

            let local_port = match listener.local_addr() {
                Ok(addr) => addr.port(),
                Err(_) => {
                    continue; // 重试
                }
            };

            // 构建ssh命令和参数
            let mut cmd = Command::new("ssh");

            // 添加通用SSH选项
            cmd.arg("-N") // 不执行远程命令
                .arg("-o")
                .arg("StrictHostKeyChecking=accept-new")
                .arg("-o")
                .arg("BatchMode=yes") // 不询问密码（使用SSH配置和密钥）
                .arg("-o")
                .arg("ServerAliveInterval=30")
                .arg("-o")
                .arg(format!("ConnectTimeout={connect_timeout_secs}"))
                .arg("-v"); // 详细输出，便于调试

            // 添加本地端口转发
            cmd.arg("-L")
                .arg(format!("{local_port}:{final_host}:{final_port}"));

            // 添加跳板机配置
            if !jump_hosts.is_empty() {
                let proxy_command = jump_hosts.join(",");
                cmd.arg("-J").arg(proxy_command);
            }

            // 目标主机
            cmd.arg(format!("{final_username}@{final_host}"));

            // 在后台启动ssh
            debug!("执行SSH转发命令: {cmd:?}");

            let mut child = match cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
                Ok(c) => c,
                Err(_) => {
                    continue; // 重试
                }
            };

            // 释放监听器，让SSH进程可以绑定该端口
            drop(listener);

            // 等待端口转发建立并可用
            let start_time = Instant::now();
            let timeout = Duration::from_secs(connect_timeout_secs);
            let mut connected = false;
            let local_addr = match format!("127.0.0.1:{local_port}").parse::<SocketAddr>() {
                Ok(addr) => addr,
                Err(_) => {
                    let _ = child.kill();
                    continue; // 重试
                }
            };

            // 尝试连接直到成功或超时
            while start_time.elapsed() < timeout {
                // 检查SSH进程是否仍在运行
                match child.try_wait() {
                    Ok(Some(_)) => {
                        break; // 跳出内循环，尝试重试
                    }
                    Err(_) => {
                        let _ = child.kill();
                        break; // 跳出内循环，尝试重试
                    }
                    _ => {} // 进程仍在运行
                }

                // 尝试连接到转发端口
                match TcpStream::connect_timeout(&local_addr, Duration::from_millis(500)) {
                    Ok(_) => {
                        // 连接成功，端口转发已建立
                        connected = true;
                        break;
                    }
                    Err(e) if e.kind() == ErrorKind::ConnectionRefused => {
                        // 连接被拒绝，端口转发可能尚未建立，稍后重试
                        std::thread::sleep(Duration::from_millis(200));
                    }
                    Err(_) => {
                        // 记录错误但继续尝试
                        std::thread::sleep(Duration::from_millis(200));
                    }
                }
            }

            // 检查是否成功连接
            if connected {
                debug!("SSH转发已成功建立，本地端口: {local_port}");

                // 如果一切正常，保持SSH进程在后台运行
                std::mem::forget(child); // 防止进程被终止

                return Ok(local_port);
            } else {
                // 连接失败，终止SSH进程并重试
                let _ = child.kill();
            }
        }

        // 所有重试都失败了
        Err(anyhow::anyhow!("无法建立SSH跳板机连接，达到最大重试次数"))
    }

    /// 使用密钥文件进行认证
    fn authenticate_with_key(
        session: &mut Session,
        username: &str,
        private_key_path: &str,
    ) -> Result<()> {
        let mut prikey_file = File::open(private_key_path)
            .with_context(|| format!("无法打开密钥文件: {private_key_path}"))?;

        let mut prikey_contents = Vec::new();
        prikey_file
            .read_to_end(&mut prikey_contents)
            .with_context(|| format!("无法读取密钥文件: {private_key_path}"))?;

        if !prikey_contents.starts_with(b"-----BEGIN RSA PRIVATE KEY-----")
            && !prikey_contents.starts_with(b"-----BEGIN OPENSSH PRIVATE KEY-----")
        {
            debug!("密钥文件不是PEM格式");
        }

        let res = session.userauth_pubkey_memory(
            username,
            None,
            &String::from_utf8_lossy(&prikey_contents),
            None,
        );

        match res {
            Ok(_) => {
                debug!("公钥认证成功");
            }
            Err(e) => {
                debug!("公钥认证失败: {e}");
            }
        }

        Ok(())
    }

    /// 身份验证
    fn authenticate_session(
        session: &mut Session,
        username: &str,
        password: Option<&str>,
        private_key_path: Option<&str>,
        #[allow(unused)] public_key_path: Option<&str>,
    ) -> Result<()> {
        if let Some(private_key) = private_key_path {
            Self::authenticate_with_key(session, username, private_key)
                .with_context(|| format!("密钥认证失败: {private_key}"))?;
        } else if let Some(pass) = password {
            debug!("使用密码进行认证");
            session
                .userauth_password(username, pass)
                .with_context(|| "密码认证失败")?;
        } else {
            debug!("尝试无密码认证");
            session
                .userauth_agent(username)
                .with_context(|| "SSH代理认证失败")?;
        }

        if !session.authenticated() {
            bail!("SSH认证失败");
        }

        Ok(())
    }

    /// 带重试的连接方法
    fn connect_with_retry<F>(
        connect_fn: F,
        max_retries: usize,
        timeout_secs: u64,
        error_message: &str,
    ) -> Result<TcpStream>
    where
        F: Fn() -> std::io::Result<TcpStream>,
    {
        let start_time = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        let mut retry = 0;
        loop {
            match connect_fn() {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    debug!("连接失败: {e}");
                    if start_time.elapsed() > timeout {
                        return Err(anyhow::anyhow!(error_message.to_string()));
                    }
                }
            }
            retry += 1;
            if retry > max_retries {
                return Err(anyhow::anyhow!(error_message.to_string()));
            }
            debug!("连接重试 #{retry}");
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

impl ConnectionManager for SSHConnectionManager {
    /// 设置SSH连接
    fn setup(&mut self) -> Result<()> {
        // 如果连接已关闭，尝试重新连接（这里简化处理，实际可能需要存储重连信息）
        if !self.connected {
            debug!("SSH连接已关闭，需要重新连接");
            return Err(anyhow::anyhow!("SSH连接已关闭，需要重新连接"));
        }

        debug!("SSH连接设置完成");
        Ok(())
    }

    /// 执行远程命令
    fn execute_command(
        &mut self,
        command: &str,
        timeout: Option<Duration>,
    ) -> Result<CommandOutput> {
        if !self.connected {
            bail!("SSH连接已关闭");
        }

        debug!("执行SSH命令: {command}");

        // 创建命令
        let mut actual_command = String::new();

        // 添加环境变量（如果启用了会话状态保持）
        if self.maintain_session {
            for (name, value) in &self.env_vars {
                actual_command.push_str(&format!("export {name}=\"{value}\"; "));
            }
        }

        // 添加实际命令
        actual_command.push_str(command);

        // 打开通道
        let mut channel = self
            .session
            .channel_session()
            .with_context(|| "无法打开SSH会话通道")?;

        // 执行命令
        channel
            .exec(&actual_command)
            .with_context(|| format!("无法执行远程命令: {actual_command}"))?;

        // 关闭标准输入
        channel.send_eof().with_context(|| "无法关闭标准输入")?;

        // 读取输出（带超时）
        let (stdout, stderr) = read_channel_with_timeout(&mut channel, timeout)?;

        // 获取退出码
        let exit_code = channel.exit_status().with_context(|| "无法获取退出码")?;

        // 关闭通道
        channel.wait_close().with_context(|| "等待通道关闭失败")?;

        debug!("SSH命令执行完成: exit_code={exit_code}");

        // 解析命令的环境变量设置（如果启用了会话状态保持）
        if self.maintain_session {
            parse_environment_vars(command, &mut self.env_vars);
        }

        Ok(CommandOutput {
            stdout,
            stderr,
            exit_code,
        })
    }

    /// 清理SSH连接
    fn destroy(&mut self) -> Result<()> {
        // 在这里可以执行一些清理操作，比如发送特定命令
        // 但通常我们只需要关闭连接
        self.close()
    }

    /// 关闭SSH连接
    fn close(&mut self) -> Result<()> {
        if self.connected {
            self.session
                .disconnect(None, "正常关闭", None)
                .with_context(|| "关闭SSH连接失败")?;
            self.connected = false;
        }
        Ok(())
    }
}

impl Drop for SSHConnectionManager {
    fn drop(&mut self) {
        if self.connected {
            if let Err(e) = self.session.disconnect(None, "连接被丢弃", None) {
                error!("关闭SSH连接失败: {e}");
            }
        }
    }
}

/// 解析命令中的环境变量设置（支持export VAR=value语法）
fn parse_environment_vars(command: &str, env_vars: &mut Vec<(String, String)>) {
    // 修复正则表达式模式，避免使用可能导致问题的转义序列
    let patterns = [
        r"export\s+([A-Za-z_][A-Za-z0-9_]*)=([^;]+)",
        r"([A-Za-z_][A-Za-z0-9_]*)=([^;]+)\s+",
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.captures_iter(command) {
                if cap.len() >= 3 {
                    let var_name = cap[1].trim().to_string();
                    let var_value = cap[2]
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();

                    // 更新或添加环境变量
                    if let Some(existing) = env_vars.iter_mut().find(|(name, _)| name == &var_name)
                    {
                        existing.1 = var_value;
                    } else {
                        env_vars.push((var_name, var_value));
                    }
                }
            }
        }
    }
}

/// 读取通道输出（带超时）
fn read_channel_with_timeout(
    channel: &mut Channel,
    timeout: Option<Duration>,
) -> Result<(String, String)> {
    let timeout_duration = timeout.unwrap_or(Duration::from_secs(60)); // 默认60秒超时
    let start_time = Instant::now();

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut buffer = [0; 4096];
    let mut stderr_buffer = [0; 4096];

    // 循环读取直到通道关闭或超时
    while !channel.eof() {
        // 检查超时
        if start_time.elapsed() > timeout_duration {
            warn!("SSH命令执行超时");
            break;
        }

        // 读取标准输出
        match channel.read(&mut buffer) {
            Ok(n) => {
                if n > 0 {
                    stdout.extend_from_slice(&buffer[..n]);
                }
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    return Err(anyhow::Error::from(e).context("读取标准输出失败"));
                }
            }
        }

        // 读取标准错误
        match channel.stderr().read(&mut stderr_buffer) {
            Ok(n) => {
                if n > 0 {
                    stderr.extend_from_slice(&stderr_buffer[..n]);
                }
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    return Err(anyhow::Error::from(e).context("读取标准错误失败"));
                }
            }
        }

        // 等待一小段时间
        std::thread::sleep(Duration::from_millis(50));
    }

    // 转换为字符串
    let stdout_str = String::from_utf8_lossy(&stdout).into_owned();
    let stderr_str = String::from_utf8_lossy(&stderr).into_owned();

    Ok((stdout_str, stderr_str))
}
