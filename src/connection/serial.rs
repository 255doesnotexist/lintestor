//! 串口连接管理器
//!
//! 该模块实现了通过串口执行Linux命令的连接管理器

use anyhow::{Context, Result};
use log::{debug};
use mio_serial::SerialPort;
use std::io::Read;
use std::thread;
use std::time::{Duration, Instant};

use crate::connection::{CommandOutput, ConnectionManager};
use crate::config::serial_config::SerialConfig;

use crate::template::ExecutorOptions;

/// 串口连接管理器
pub struct SerialConnectionManager {
    config: SerialConfig,
    executor_options: ExecutorOptions,
    port: Option<Box<dyn SerialPort + Send>>, // 线程安全
}

impl SerialConnectionManager {
    pub fn new(config: SerialConfig, executor_options: ExecutorOptions) -> Result<Self> {
        Ok(Self { 
            config,
            executor_options,
            port: None 
        })
    }

    /// 打开串口（使用mio-serial）
    fn open_port(&self) -> Result<Box<dyn SerialPort + Send>> {
        let mut builder = mio_serial::new(&self.config.port, self.config.baud_rate);
        builder = builder.timeout(Duration::from_secs(self.executor_options.command_timeout));
        let stream = builder.open_native().with_context(|| format!("Unable to open serial port: {}", self.config.port))?; // 无法打开串口: {}
        Ok(Box::new(stream))
    }

    /// 等待特定pattern出现
    fn wait_for_pattern(port: &mut dyn SerialPort, pattern: &str, timeout: Duration) -> Result<String> {
        let start = Instant::now();
        let mut buf = vec![0u8; 4096];
        let mut output = String::new();
        while start.elapsed() < timeout {
            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let s = String::from_utf8_lossy(&buf[..n]);
                    output.push_str(&s);
                    if output.contains(pattern) {
                        return Ok(output);
                    }
                }
                _ => thread::sleep(Duration::from_millis(50)),
            }
        }
        Err(anyhow::anyhow!("Waiting for pattern timeout: {pattern}")) // 等待pattern超时: {pattern}
    }

    /// 发送一行并刷新
    fn send_line(port: &mut dyn SerialPort, line: &str) -> Result<()> {
        port.write_all(line.as_bytes())?;
        port.write_all(b"\n")?;
        port.flush()?;
        Ok(())
    }
}

impl ConnectionManager for SerialConnectionManager {
    /// 建立串口连接并登录shell
    fn setup(&mut self) -> Result<()> {
        debug!("Serial setup: open serial port and wait for shell"); // 串口setup: 打开串口并等待shell
        let mut port: Box<dyn SerialPort + Send + 'static> = self.open_port()?;
        let timeout = Duration::from_secs(self.executor_options.command_timeout);
        // 登录流程
        if let Some(ref user_pat) = self.config.user_prompt {
            let _ = Self::wait_for_pattern(&mut *port, user_pat, timeout)?;
            if let Some(ref user) = self.config.username {
                Self::send_line(&mut *port, user)?;
            }
        }
        if let Some(ref pass_pat) = self.config.pass_prompt {
            let _ = Self::wait_for_pattern(&mut *port, pass_pat, timeout)?;
            if let Some(ref pass) = self.config.password {
                Self::send_line(&mut *port, pass)?;
            }
        }
        // 等待shell提示符
        let _ = Self::wait_for_pattern(&mut *port, &self.config.shell_prompt, timeout)?;
        self.port = Some(port);
        Ok(())
    }

    /// 关闭串口连接（可选logout）
    fn destroy(&mut self) -> Result<()> {
        debug!("Serial destroy: logout and close serial port"); // 串口destroy: logout并关闭串口
        if let Some(ref mut port) = self.port {
            let _ = Self::send_line(&mut **port, "logout");
        }
        self.port = None;
        Ok(())
    }

    /// 执行命令
    fn execute_command(&mut self, command: &str, timeout: Option<Duration>) -> Result<CommandOutput> {
        let timeout = timeout.unwrap_or(Duration::from_secs(self.executor_options.command_timeout));
        let shell_prompt = &self.config.shell_prompt;
        let port = self.port.as_mut().ok_or_else(|| anyhow::anyhow!("Serial port not connected, please setup first"))?; // 串口未连接，请先setup
        debug!("Serial executing command: {command}"); // 串口执行命令: {command}
        // 清空缓冲区
        let mut buf = [0u8; 4096];
        while let Ok(n) = port.read(&mut buf) {
            if n == 0 { break; }
        }
        // 发送命令
        Self::send_line(&mut **port, command)?;
        // 读取直到下一个shell提示符
        let output = Self::wait_for_pattern(&mut **port, shell_prompt, timeout)?;
        // 提取命令输出（去掉命令本身和提示符）
        let mut lines: Vec<&str> = output.lines().collect();
        if !lines.is_empty() && lines[0].trim() == command.trim() {
            lines.remove(0);
        }
        if !lines.is_empty() && lines.last().unwrap().contains(shell_prompt) {
            lines.pop();
        }
        let stdout = lines.join("\n");
        Ok(CommandOutput {
            stdout,
            stderr: String::new(), // 串口无法区分
            exit_code: 0, // 串口一般无法获取退出码
        })
    }

    fn close(&mut self) -> Result<()> {
        self.port = None;
        Ok(())
    }
}
