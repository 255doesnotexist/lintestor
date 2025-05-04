//! SSH连接管理器
//!
//! 该模块实现了基于SSH的远程命令执行管理器

use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::{Duration, Instant};
use anyhow::{Result, Context, bail};
use log::{debug, warn, error};
use ssh2::{Session, Channel};

use crate::connection::{ConnectionManager, CommandOutput};

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
}

impl SSHConnectionManager {
    /// 创建新的SSH连接管理器
    pub fn new(
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
        key_path: Option<&str>
    ) -> Result<Self> {
        debug!("创建SSH连接: {}@{}:{}", username, host, port);
        
        // 创建TCP连接
        let tcp = TcpStream::connect(format!("{}:{}", host, port))
            .with_context(|| format!("无法连接到 {}:{}", host, port))?;
        
        // 创建SSH会话
        let mut session = Session::new()
            .with_context(|| "无法创建SSH会话")?;
        session.set_tcp_stream(tcp);
        session.handshake()
            .with_context(|| "SSH握手失败")?;
        
        // 身份验证
        if let Some(key) = key_path {
            debug!("使用密钥文件进行认证: {}", key);
            SSHConnectionManager::authenticate_with_key(&mut session, username, key)
                .with_context(|| format!("密钥认证失败: {}", key))?;
        } else if let Some(pass) = password {
            debug!("使用密码进行认证");
            session.userauth_password(username, pass)
                .with_context(|| "密码认证失败")?;
        } else {
            debug!("尝试无密码认证");
            session.userauth_agent(username)
                .with_context(|| "SSH代理认证失败")?;
        }
        
        if !session.authenticated() {
            bail!("SSH认证失败");
        }
        
        Ok(Self {
            session,
            connected: true,
            maintain_session: true,
            env_vars: Vec::new(),
        })
    }
    
    /// 使用密钥文件进行认证
    fn authenticate_with_key(session: &mut Session, username: &str, key_path: &str) -> Result<()> {
        let mut key_file = File::open(key_path)
            .with_context(|| format!("无法打开密钥文件: {}", key_path))?;
        
        let mut key_contents = Vec::new();
        key_file.read_to_end(&mut key_contents)
            .with_context(|| format!("无法读取密钥文件: {}", key_path))?;
        
        session.userauth_pubkey_memory(username, None, &String::from_utf8_lossy(&key_contents), None)
            .with_context(|| "公钥认证失败")?;
        
        Ok(())
    }
    
    /// 设置是否保持会话状态（环境变量等）
    pub fn set_maintain_session(&mut self, maintain: bool) {
        self.maintain_session = maintain;
    }
    
    /// 添加环境变量
    pub fn add_env_var(&mut self, name: &str, value: &str) {
        self.env_vars.push((name.to_string(), value.to_string()));
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
    fn execute_command(&mut self, command: &str, timeout: Option<Duration>) -> Result<CommandOutput> {
        if !self.connected {
            bail!("SSH连接已关闭");
        }
        
        debug!("执行SSH命令: {}", command);
        
        // 创建命令
        let mut actual_command = String::new();
        
        // 添加环境变量（如果启用了会话状态保持）
        if self.maintain_session {
            for (name, value) in &self.env_vars {
                actual_command.push_str(&format!("export {}=\"{}\"; ", name, value));
            }
        }
        
        // 添加实际命令
        actual_command.push_str(command);
        
        // 打开通道
        let mut channel = self.session.channel_session()
            .with_context(|| "无法打开SSH会话通道")?;
        
        // 执行命令
        channel.exec(&actual_command)
            .with_context(|| format!("无法执行远程命令: {}", actual_command))?;
        
        // 关闭标准输入
        channel.send_eof()
            .with_context(|| "无法关闭标准输入")?;
        
        // 读取输出（带超时）
        let (stdout, stderr) = read_channel_with_timeout(&mut channel, timeout)?;
        
        // 获取退出码
        let exit_code = channel.exit_status()
            .with_context(|| "无法获取退出码")?;
        
        // 关闭通道
        channel.wait_close()
            .with_context(|| "等待通道关闭失败")?;
        
        debug!("SSH命令执行完成: exit_code={}", exit_code);
        
        // 解析命令的环境变量设置（如果启用了会话状态保持）
        if self.maintain_session {
            parse_environment_vars(command, &mut self.env_vars);
        }
        
        Ok(CommandOutput {
            stdout,
            stderr,
            exit_code: exit_code as i32,
        })
    }
    
    /// 清理SSH连接
    fn teardown(&mut self) -> Result<()> {
        // 在这里可以执行一些清理操作，比如发送特定命令
        // 但通常我们只需要关闭连接
        self.close()
    }

    /// 关闭SSH连接
    fn close(&mut self) -> Result<()> {
        if self.connected {
            self.session.disconnect(None, "正常关闭", None)
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
                error!("关闭SSH连接失败: {}", e);
            }
        }
    }
}

/// 从通道读取输出（带超时）
fn read_channel_with_timeout(channel: &mut Channel, timeout: Option<Duration>) -> Result<(String, String)> {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    
    let timeout_duration = timeout.unwrap_or(Duration::from_secs(60)); // 默认60秒超时
    let start_time = Instant::now();
    
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
            },
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
            },
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
                    let var_value = cap[2].trim().trim_matches('"').trim_matches('\'').to_string();
                    
                    // 更新或添加环境变量
                    if let Some(existing) = env_vars.iter_mut().find(|(name, _)| name == &var_name) {
                        existing.1 = var_value;
                    } else {
                        env_vars.push((var_name, var_value));
                    }
                }
            }
        }
    }
}