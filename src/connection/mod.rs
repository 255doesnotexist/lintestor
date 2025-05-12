//! 连接管理模块
//!
//! 该模块提供了不同类型连接（SSH、本地、QEMU等）的统一接口

use std::time::Duration;
use anyhow::{Result, bail};
use crate::config::target_config::TargetConfig;

/// 命令执行结果
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: i32,
}

/// 连接管理器特质
pub trait ConnectionManager {
    /// 执行命令并返回结果
    fn execute_command(&mut self, command: &str, timeout: Option<Duration>) -> Result<CommandOutput>;
    
    /// 关闭连接
    fn close(&mut self) -> Result<()>;

    /// 设置连接
    /// 
    /// 默认实现什么也不做并返回Ok，使实现该trait的类型可以选择是否重写此方法
    #[allow(unused)]
    fn setup(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// 清理连接
    fn teardown(&mut self) -> Result<()>;
}

/// 连接工厂，用于创建适合指定配置的连接管理器
pub struct ConnectionFactory;

impl ConnectionFactory {
    /// 根据目标配置创建适当类型的连接管理器
    pub fn create_manager(config: &TargetConfig) -> Result<Box<dyn ConnectionManager>> {
        match config.testing_type.as_str() {
            "remote" | "ssh" => {
                // 创建SSH连接
                let connection = match &config.connection {
                    Some(conn) => conn,
                    None => bail!("No connection configuration provided for remote/SSH"),
                };
                
                Ok(Box::new(SSHConnectionManager::new(connection)?))
            },
            "local" | "locally" => {
                // 创建本地执行环境
                Ok(Box::new(LocalConnectionManager::new()))
            },
            "qemu" | "qemu-based-remote" => {
                // 创建QEMU连接（实际上也是SSH）
                let connection = match &config.connection {
                    Some(conn) => conn,
                    None => bail!("No connection configuration provided for QEMU"),
                };
                
                Ok(Box::new(SSHConnectionManager::new(connection)?))
            },
            "boardtest" => {
                // 这里应该实现BoardTest连接类型
                bail!("Boardtest connection type not yet implemented for template system")
            },
            _ => {
                bail!("Unknown testing type: {}", config.testing_type)
            }
        }
    }
}

// 实现本地连接管理器
mod local;
pub use local::LocalConnectionManager;

// 实现SSH连接管理器
mod ssh;
pub use ssh::SSHConnectionManager;