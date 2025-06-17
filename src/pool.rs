use crate::config::target_config::TargetConfig;
use crate::connection::{ConnectionFactory, ConnectionManager};
use crate::template::ExecutorOptions;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// 连接管理池，复用ConnectionManager实例
pub struct ConnectionManagerPool {
    pool: HashMap<PathBuf, Box<dyn ConnectionManager>>,
}

impl ConnectionManagerPool {
    /// 创建一个新的连接池
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
        }
    }

    /// 获取或创建指定TargetConfig的ConnectionManager
    pub fn get_or_create(
        &mut self,
        config: &TargetConfig,
        allow_reuse: bool,
        executor_options: &ExecutorOptions,
    ) -> Result<&mut Box<dyn ConnectionManager>> {
        let key = config.get_path().clone();
        if !allow_reuse {
            // 不允许多次复用，直接创建新连接
            let manager = ConnectionFactory::create_manager(config, executor_options)?;
            self.pool.insert(key.clone(), manager);
        } else if !self.pool.contains_key(&key) {
            // 允许复用但池中没有，创建新连接
            let manager = ConnectionFactory::create_manager(config, executor_options)?;
            self.pool.insert(key.clone(), manager);
        }
        self.pool
            .get_mut(&key)
            .ok_or_else(|| anyhow::anyhow!("Failed to get or create ConnectionManager"))
    }

    #[allow(dead_code)]
    /// 移除指定TargetConfig的ConnectionManager
    pub fn remove(&mut self, config: &TargetConfig) {
        // 我猜在发现所有有关特定TargetConfig的连接都不再需要时会用到这个方法
        // 大概会需要做一下HashMap<TargetConfig, i64>的计数工作，现在先不写感觉模板量比较小不需要
        let key = config.get_path();
        self.pool.remove(key);
    }

    #[allow(dead_code)]
    /// 清空所有连接
    pub fn clear(&mut self) {
        // 虽然还没想到什么时候会用到这个，但是留一个接口总是好的
        self.pool.clear();
    }
}
