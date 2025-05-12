//! 本地连接管理器
//!
//! 该模块实现了本地命令执行的连接管理器

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::io::Read;
use std::thread;
use anyhow::{Result, Context};
use log::{debug, warn};

use crate::connection::{ConnectionManager, CommandOutput};

/// 本地连接管理器
pub struct LocalConnectionManager;

impl LocalConnectionManager {
    /// 创建新的本地连接管理器
    pub fn new() -> Self {
        Self {}
    }
}

impl ConnectionManager for LocalConnectionManager {
    /// 执行本地命令
    fn execute_command(&mut self, command: &str, timeout: Option<Duration>) -> Result<CommandOutput> {
        debug!("执行本地命令: {}", command);
        
        // 创建命令进程
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("无法启动命令进程: {}", command))?;
        
        let start_time = Instant::now();
        let timeout_duration = timeout.unwrap_or(Duration::from_secs(60)); // 默认60秒超时
        
        // 检查是否超时
        let mut timed_out = false;
        while child.try_wait()?.is_none() {
            if start_time.elapsed() > timeout_duration {
                timed_out = true;
                warn!("命令执行超时: {}", command);
                child.kill()?;
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
        
        // 获取结果
        let mut stdout = String::new();
        let mut stderr = String::new();
        
        // 读取标准输出
        if let Some(mut stdout_pipe) = child.stdout.take() {
            stdout_pipe.read_to_string(&mut stdout)?;
        }
        
        // 读取标准错误
        if let Some(mut stderr_pipe) = child.stderr.take() {
            stderr_pipe.read_to_string(&mut stderr)?;
        }
        
        // 获取退出码
        let exit_code = if timed_out {
            -1 // 超时返回-1
        } else {
            match child.wait()?.code() {
                Some(code) => code,
                None => -1, // 被信号终止
            }
        };
        
        debug!("命令执行完成: exit_code={}", exit_code);
        
        Ok(CommandOutput {
            stdout,
            stderr,
            exit_code,
        })
    }
    
    /// 设置连接（本地连接无需设置）
    fn setup(&mut self) -> Result<()> {
        // 本地连接不需要额外设置
        Ok(())
    }
    
    /// 清理连接（本地连接无需清理）
    fn teardown(&mut self) -> Result<()> {
        // 本地连接不需要额外清理
        Ok(())
    }
    
    /// 关闭连接（本地连接无需关闭）
    fn close(&mut self) -> Result<()> {
        // 本地连接不需要显式关闭
        Ok(())
    }
}