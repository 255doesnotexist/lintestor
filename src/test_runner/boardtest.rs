//! Boardtest 测试运行器的实现

use crate::aggregator::generate_report;
use crate::config::boardtest_config::BoardtestConfig;
use crate::test_environment::boardtest::BoardtestEnvironment;
use crate::test_environment::TestEnvironment;
use crate::test_runner::TestRunner;
use crate::test_script_manager::TestScriptManager;
use crate::utils::{PackageMetadata, Report, TestResult, REMOTE_TMP_DIR};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use log::{debug, info, warn};
use std::error::Error;
use std::path::Path;
use std::process::Command;

/// Boardtest 测试运行器，使用 Boardtest API 执行测试
pub struct BoardtestRunner {
    environment: BoardtestEnvironment,
}

impl BoardtestRunner {
    /// 创建一个新的 Boardtest 测试运行器
    pub fn new(config: &BoardtestConfig) -> Self {
        BoardtestRunner {
            environment: BoardtestEnvironment::new(config.clone()),
        }
    }
}

impl TestRunner for BoardtestRunner {
    fn environment(&self) -> &dyn TestEnvironment {
        &self.environment
    }

    /// 在 Boardtest 环境中运行测试
    ///
    /// # 参数
    ///
    /// * `target` - 分发版本/目标名称
    /// * `unit` - 测试单元名称
    /// * `skip_templates` - 要跳过的脚本列表
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
        info!("开始在 Boardtest 上为 {}/{} 执行测试", target, unit);
        
        // 注意：在 Boardtest 环境中，skip_templates 实际上无法直接使用
        // 因为所有测试脚本都打包到一个命令中执行
        if !skip_templates.is_empty() {
            warn!(
                "应跳过的脚本: {:?}，但这个功能在 Boardtest 环境中尚未实现。",
                skip_templates
            );
        }

        // 设置环境
        self.environment.setup()?;

        // 压缩本地测试目录
        let local_dir = dir.join(format!("{}/{}", target, unit));
        let mut tar_buffer = Vec::new();

        info!("压缩本地测试目录: {}", local_dir.display());
        let tar_output = Command::new("tar")
            .arg("czf")
            .arg("-")
            .arg("-C")
            .arg(&local_dir)
            .arg(".")
            .output()?;

        if !tar_output.status.success() {
            return Err(format!(
                "tar 命令失败: {}",
                String::from_utf8_lossy(&tar_output.stderr)
            )
            .into());
        }

        tar_buffer = tar_output.stdout;

        // 转换为 base64
        let base64_content = BASE64.encode(&tar_buffer);
        
        // 创建测试脚本
        let test_template = format!(
            "cd {} && \
             mkdir -p test && \
             echo '{}' | base64 -d | tar xzf - -C test && \
             cd test && \
             bash -c './test.sh'",
            REMOTE_TMP_DIR, base64_content
        );

        // 执行测试命令
        info!("在 Boardtest 上执行测试脚本");
        let result = self.environment.execute_command(&test_template)?;
        let test_passed = result.exit_status == 0;

        // 创建测试结果
        let test_results = vec![TestResult {
            test_name: format!("{}/{}", target, unit),
            output: result.output,
            passed: test_passed,
        }];

        // 构建报告
        // 注意：在 Boardtest 环境中，我们无法获取详细的操作系统信息，
        // 所以这些字段将保持为空
        let report = Report {
            target: target.to_string(),
            os_version: String::new(),
            kernel_version: String::new(),
            unit_name: unit.to_string(),
            unit_metadata: PackageMetadata {
                unit_pretty_name: unit.to_string(),
                ..Default::default()
            },
            test_results,
            all_tests_passed: test_passed,
        };

        // 生成报告文件
        let report_path = local_dir.join("report.json");
        generate_report(&report_path, report)?;
        
        // 清理环境
        self.environment.teardown()?;

        if !test_passed {
            return Err(format!("测试失败: {}/{}", target, unit).into());
        }

        Ok(())
    }
}
