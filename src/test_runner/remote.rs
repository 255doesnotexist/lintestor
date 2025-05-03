//! 远程 SSH 测试运行器的实现

use crate::aggregator::generate_report;
use crate::test_environment::remote::RemoteEnvironment;
use crate::test_executor::TestExecutor;
use crate::test_runner::TestRunner;
use crate::test_environment::TestEnvironment;
use std::error::Error;
use std::path::Path;

/// 远程 SSH 测试运行器，使用远程 SSH 环境执行测试
pub struct RemoteTestRunner {
    environment: RemoteEnvironment,
}

impl RemoteTestRunner {
    /// 创建一个新的远程测试运行器
    pub fn new(
        remote_ip: String,
        port: u16,
        username: String,
        password: Option<String>,
        private_key_path: Option<String>,
    ) -> Self {
        RemoteTestRunner {
            environment: RemoteEnvironment::new(remote_ip, port, username, password, private_key_path),
        }
    }
}

impl TestRunner for RemoteTestRunner {
    fn environment(&self) -> &dyn TestEnvironment {
        &self.environment
    }

    /// 在远程 SSH 环境中运行测试
    ///
    /// # 参数
    ///
    /// * `target` - 分发版本/目标的名称
    /// * `unit` - 测试单元的名称
    /// * `skip_scripts` - 要跳过的脚本名称列表
    /// * `dir` - 包含测试文件的基础目录
    ///
    /// # 错误
    ///
    /// 如果测试执行失败则返回错误
    fn run_test(
        &mut self,
        target: &str,
        unit: &str,
        skip_scripts: Vec<String>,
        dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        // 创建测试执行器
        let mut executor = TestExecutor::new(&mut self.environment);
        
        // 执行测试并获取报告
        let report = executor.execute_remote_test(target, unit, skip_scripts, dir)?;
        let all_passed = report.all_tests_passed;

        // 生成报告文件
        let report_path = dir.join(format!("{}/{}/report.json", target, unit));
        generate_report(&report_path, report)?;
        
        // 检查是否所有测试都通过
        if !all_passed{
            return Err(format!("{}/{} 的测试未全部通过", target, unit).into());
        }
        
        Ok(())
    }
}
