//! 测试运行逻辑，使用不同的 TestEnvironment 执行测试。

use crate::test_environment::TestEnvironment;
use std::error::Error;
use std::path::Path;

pub mod boardtest;
pub mod local;
pub mod remote;
pub mod qemu;

/// 测试执行器 trait，负责使用特定的 TestEnvironment 协调测试执行。
pub trait TestRunner {
    /// 返回底层 TestEnvironment 的引用。
    fn environment(&self) -> &dyn TestEnvironment;

    /// 为特定目标和单元运行测试。
    ///
    /// # 参数
    ///
    /// * `target` - 分发版本/目标的名称。
    /// * `unit` - 测试单元的名称。
    /// * `skip_templates` - 要跳过的脚本名称列表。
    /// * `dir` - 包含测试文件的基础目录。
    ///
    /// # 错误
    ///
    /// 如果测试执行失败则返回错误。
    fn run_test(
        &mut self,
        target: &str,
        unit: &str,
        skip_templates: Vec<String>,
        dir: &Path,
    ) -> Result<(), Box<dyn Error>>;
}
