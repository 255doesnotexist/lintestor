//! QEMU 测试运行器的实现

use crate::aggregator::generate_report;
use crate::test_environment::qemu::QemuEnvironment;
use crate::test_environment::TestEnvironment;
use crate::test_executor::TestExecutor;
use crate::test_runner::TestRunner;
use std::error::Error;
use std::path::Path;

/// QEMU 测试运行器，使用 QEMU 虚拟机环境执行测试
pub struct QemuTestRunner {
    environment: QemuEnvironment,
}

impl QemuTestRunner {
    /// 创建一个新的 QEMU 测试运行器
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
        QemuTestRunner {
            environment: QemuEnvironment::new(
                remote_ip,
                port,
                username,
                password,
                private_key_path,
                startup_script,
                stop_script,
                working_dir,
            ),
        }
    }
    
    /// 设置 QEMU 环境的重试参数
    pub fn with_retry_params(mut self, max_attempts: u32, retry_interval: u64) -> Self {
        self.environment = self.environment.with_retry_params(max_attempts, retry_interval);
        self
    }
}

impl TestRunner for QemuTestRunner {
    fn environment(&self) -> &dyn TestEnvironment {
        &self.environment
    }

    /// 在 QEMU 虚拟机环境中运行测试
    ///
    /// # 参数
    ///
    /// * `target` - 分发版本/目标的名称
    /// * `unit` - 测试单元的名称
    /// * `skip_templates` - 要跳过的脚本名称列表
    /// * `dir` - 包含测试文件的基础目录
    ///
    /// # 错误
    ///
    /// 如果测试执行失败则返回错误
    fn run_test(
        &mut self,
        target: &str,
        unit: &str,
        skip_templates: Vec<String>,
        dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        // 创建测试执行器
        let mut executor = TestExecutor::new(&mut self.environment);
        
        // 执行测试并获取报告
        let report = executor.execute_remote_test(target, unit, skip_templates, dir)?;
        let all_passed = report.all_tests_passed;
        
        // 生成报告文件
        let report_path = dir.join(format!("{}/{}/report.json", target, unit));
        generate_report(&report_path, report)?;
        
        // 检查是否所有测试都通过
        if !all_passed {
            return Err(format!("{}/{} 的测试未全部通过", target, unit).into());
        }
        
        Ok(())
    }
}