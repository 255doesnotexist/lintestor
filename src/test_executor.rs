//! 提供统一的测试执行逻辑，适用于不同的测试环境

use crate::aggregator::generate_report;
use crate::test_environment::TestEnvironment;
use crate::test_template_manager::TestTemplateManager;
use crate::utils::{PackageMetadata, Report, TestResult, REMOTE_TMP_DIR};
use log::{debug, info};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashMap;
use anyhow::{Result, Context, bail};
use log::{warn, error};

use crate::config::target_config::TargetConfig;
use crate::connection::ConnectionManager;
use crate::template::{
    TestTemplate, TemplateExecutor, ExecutionResult, ExecutorOptions, TemplateDependencyManager
};

/// 通用测试执行器，实现与测试环境无关的测试逻辑
pub struct TestExecutor<'a> {
    environment: &'a mut dyn TestEnvironment,
}

impl<'a> TestExecutor<'a> {
    /// 创建一个新的测试执行器
    pub fn new(environment: &'a mut dyn TestEnvironment) -> Self {
        TestExecutor { environment }
    }

    /// 执行本地测试，适用于本地测试环境
    pub fn execute_local_test(
        &mut self,
        target: &str,
        unit: &str,
        skip_templates: Vec<String>,
        dir: &Path,
    ) -> Result<Report, Box<dyn Error>> {
        info!("在本地环境中执行测试：{}/{}", target, unit);
        // 设置环境
        self.environment.setup()?;

        let template_manager = TestTemplateManager::new(target, unit, skip_templates, dir)?;
        let (os_version, kernel_version) = self.environment.get_os_info()?;
        let mut all_tests_passed = true;
        let mut test_results = Vec::new();

        let prerequisite_path = dir.join(format!("{}/prerequisite.sh", target));
        let local_unit_dir = dir.join(format!("{}/{}", target, unit));

        // 确保临时目录存在
        self.environment.mkdir(REMOTE_TMP_DIR)?;

        for template_path in template_manager.get_test_templates() {
            // 构造要在单元目录上下文中运行的命令
            let template_name = PathBuf::from(template_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            
            let command = format!(
                "cd {} && {} source {}",
                local_unit_dir.display(),
                if prerequisite_path.exists() {
                    format!("source {} &&", prerequisite_path.display())
                } else {
                    String::from("")
                },
                template_path
            );

            let result = self.environment.run_command(&command)?;
            let test_passed = result.exit_status == 0;
            all_tests_passed &= test_passed;

            test_results.push(TestResult {
                test_name: template_name,
                output: result.output,
                passed: test_passed,
            });
        }

        let unit_metadata = if let Some(metadata_template) = template_manager.get_metadata_template() {
            // 构造元数据命令
            let metadata_command = format!(
                "source {} && echo $PACKAGE_VERSION && echo $PACKAGE_PRETTY_NAME && echo $PACKAGE_TYPE && echo $PACKAGE_DESCRIPTION",
                metadata_template
            );
            
            let metadata_output = self.environment.run_command(&metadata_command)?;
            
            // 从输出文本中解析元数据
            let mut version = String::new();
            let mut pretty_name = String::new();
            let mut unit_type = String::new();
            let mut detemplateion = String::new();
            
            let mut lines = metadata_output.output.lines();
            if let Some(stdout_line) = lines.find(|l| l.starts_with("stdout:")) {
                let mut data_lines = stdout_line.trim_start_matches("stdout:").trim().lines();
                version = data_lines.next().unwrap_or("").trim().to_string();
                pretty_name = data_lines.next().unwrap_or("").trim().to_string();
                unit_type = data_lines.next().unwrap_or("").trim().to_string();
                detemplateion = data_lines.next().unwrap_or("").trim().to_string();
            }

            if version.is_empty() && pretty_name.is_empty() {
                debug!("无法从输出解析元数据: {}", metadata_output.output);
                PackageMetadata {
                    unit_pretty_name: unit.to_string(),
                    ..Default::default()
                }
            } else {
                PackageMetadata {
                    unit_version: version,
                    unit_pretty_name: pretty_name,
                    unit_type,
                    unit_detemplateion: detemplateion,
                }
            }
        } else {
            PackageMetadata {
                unit_pretty_name: unit.to_string(),
                ..Default::default()
            }
        };

        let report = Report {
            target: target.to_string(),
            os_version,
            kernel_version,
            unit_name: unit.to_string(),
            unit_metadata,
            test_results,
            all_tests_passed,
        };

        // 调用环境的清理方法
        self.environment.teardown()?;

        Ok(report)
    }

    /// 执行远程测试，适用于SSH或其他远程环境
    pub fn execute_remote_test(
        &mut self,
        target: &str,
        unit: &str,
        skip_templates: Vec<String>,
        dir: &Path,
    ) -> Result<Report, Box<dyn Error>> {
        info!("在远程环境中执行测试：{}/{}", target, unit);
        // 设置环境
        self.environment.setup()?;

        // --- 准备本地文件 ---
        let local_unit_dir = dir.join(format!("{}/{}", target, unit));
        let tar_filename = format!("{}.tar.gz", unit);
        let local_tar_path = dir.join(&tar_filename);
        let prerequisite_template_local_path = dir.join(format!("{}/prerequisite.sh", target));

        // --- 压缩本地目录 ---
        info!("压缩本地目录: {}", local_unit_dir.display());
        let tar_output = Command::new("tar")
            .arg("czf")
            .arg(&local_tar_path)
            .arg("-C")
            .arg(&local_unit_dir)
            .arg(".")
            .output()?;
            
        if !tar_output.status.success() {
            return Err(format!(
                "压缩本地目录失败: {}",
                String::from_utf8_lossy(&tar_output.stderr)
            ).into());
        }

        // 使用RAII守卫在函数退出时清理本地tar文件
        struct LocalCleanupGuard {
            path: PathBuf,
        }

        impl Drop for LocalCleanupGuard {
            fn drop(&mut self) {
                if self.path.exists() {
                    if let Err(e) = std::fs::remove_file(&self.path) {
                        debug!("清理本地文件{}失败: {}", self.path.display(), e);
                    } else {
                        debug!("已清理本地文件: {}", self.path.display());
                    }
                }
            }
        }

        let _local_tar_guard = LocalCleanupGuard { path: local_tar_path.clone() };

        // --- 准备远程路径 ---
        let remote_unit_dir = format!("{}/{}/{}", REMOTE_TMP_DIR, target, unit);
        let remote_tar_path = format!("{}/{}", REMOTE_TMP_DIR, tar_filename);
        let remote_prerequisite_path = "/tmp/prerequisite.sh"; // 保持一致的远程路径

        // --- 远程设置 ---
        info!("设置远程目录: {}", REMOTE_TMP_DIR);
        self.environment.mkdir(REMOTE_TMP_DIR)?;
        info!("清理之前的远程单元目录: {}", remote_unit_dir);
        self.environment.rm_rf(&remote_unit_dir)?; // 上传前清理

        // --- 上传文件 ---
        info!("上传 {} 到 {}", tar_filename, remote_tar_path);
        self.environment.upload_file(&local_tar_path, &remote_tar_path, 0o644)?;

        if prerequisite_template_local_path.exists() {
            info!(
                "上传前提条件脚本到 {}",
                remote_prerequisite_path
            );
            self.environment.upload_file(
                &prerequisite_template_local_path,
                remote_prerequisite_path,
                0o755, // 设为可执行
            )?;
        }

        // --- 解压远程存档 ---
        info!("解压远程存档 {} 到 {}", remote_tar_path, remote_unit_dir);
        let extract_command = format!(
            "mkdir -p {} && tar xzf {} -C {} --overwrite",
            remote_unit_dir, remote_tar_path, remote_unit_dir
        );
        
        let extract_output = self.environment.run_command(&extract_command)?;
        if extract_output.exit_status != 0 {
            return Err(format!(
                "解压远程存档失败: {}",
                extract_output.output
            ).into());
        }
        
        // 解压后清理远程存档文件
        self.environment.rm_rf(&remote_tar_path)?;

        // --- 执行测试 ---
        info!("在远程目录中执行测试: {}", remote_unit_dir);
        let template_manager = TestTemplateManager::new(target, unit, skip_templates, &local_unit_dir)?; // 使用本地目录来发现脚本
        let mut all_tests_passed = true;
        let mut test_results = Vec::new();

        for template_name in template_manager.get_test_template_names() {
            // 构造命令在远程单元目录内运行
            let command = format!(
                "cd {} && {} source {}",
                remote_unit_dir,
                if prerequisite_template_local_path.exists() {
                    format!("source {} &&", remote_prerequisite_path)
                } else {
                    String::from("")
                },
                template_name // 脚本名称已经是相对路径
            );

            info!("执行远程测试脚本: {}", template_name);
            let result = self.environment.run_command(&command)?;
            let test_passed = result.exit_status == 0;
            all_tests_passed &= test_passed;

            test_results.push(TestResult {
                test_name: template_name,
                output: result.output,
                passed: test_passed,
            });
        }

        // --- 获取元数据 ---
        info!("从远程环境收集元数据");
        let (os_version, kernel_version) = self.environment.get_os_info()?;
        let unit_metadata = if let Some(metadata_template_name) = template_manager.get_metadata_template_name() {
            let metadata_command = format!(
                "cd {} && source {} && echo $PACKAGE_VERSION && echo $PACKAGE_PRETTY_NAME && echo $PACKAGE_TYPE && echo $PACKAGE_DESCRIPTION",
                remote_unit_dir, metadata_template_name
            );
            
            let metadata_output = self.environment.run_command(&metadata_command)?;
            
            // 解析元数据
            let mut version = String::new();
            let mut pretty_name = String::new();
            let mut unit_type = String::new();
            let mut detemplateion = String::new();
            
            let mut lines = metadata_output.output.lines();
            if let Some(stdout_line) = lines.find(|l| l.starts_with("stdout:")) {
                let mut data_lines = stdout_line.trim_start_matches("stdout:").trim().lines();
                version = data_lines.next().unwrap_or("").trim().to_string();
                pretty_name = data_lines.next().unwrap_or("").trim().to_string();
                unit_type = data_lines.next().unwrap_or("").trim().to_string();
                detemplateion = data_lines.next().unwrap_or("").trim().to_string();
            }

            if version.is_empty() && pretty_name.is_empty() && metadata_output.exit_status != 0 {
                debug!("从脚本获取元数据失败: {}", metadata_output.output);
                PackageMetadata {
                    unit_pretty_name: unit.to_string(),
                    ..Default::default()
                }
            } else {
                PackageMetadata {
                    unit_version: version,
                    unit_pretty_name: pretty_name,
                    unit_type,
                    unit_detemplateion: detemplateion,
                }
            }
        } else {
            debug!("未找到 {}/{} 的元数据脚本", target, unit);
            PackageMetadata {
                unit_pretty_name: unit.to_string(),
                ..Default::default()
            }
        };

        let report = Report {
            target: target.to_string(),
            os_version,
            kernel_version,
            unit_name: unit.to_string(),
            unit_metadata,
            test_results,
            all_tests_passed,
        };

        // 调用环境的清理方法
        self.environment.teardown()?;

        Ok(report)
    }
}

/// 批量测试执行器
/// 
/// 负责按照依赖关系顺序执行多个测试模板
pub struct BatchExecutor<'a> {
    /// 工作目录
    work_dir: PathBuf,
    /// 连接管理器
    connection_manager: &'a mut dyn ConnectionManager,
    /// 目标配置
    target_config: TargetConfig,
    /// 执行选项
    options: ExecutorOptions,
    /// 依赖管理器
    dependency_manager: TemplateDependencyManager,
    /// 执行结果
    results: HashMap<PathBuf, ExecutionResult>,
}

impl<'a> BatchExecutor<'a> {
    /// 创建新的批量测试执行器
    pub fn new(
        work_dir: PathBuf,
        connection_manager: &'a mut dyn ConnectionManager,
        target_config: TargetConfig,
        options: Option<ExecutorOptions>,
    ) -> Self {
        Self {
            work_dir: work_dir.clone(),
            connection_manager,
            target_config,
            options: options.unwrap_or_default(),
            dependency_manager: TemplateDependencyManager::new(work_dir),
            results: HashMap::new(),
        }
    }

    /// 添加测试模板
    pub fn add_template(&mut self, template: TestTemplate) -> Result<()> {
        self.dependency_manager.add_template(template)
    }

    /// 添加多个测试模板
    pub fn add_templates<I>(&mut self, templates: I) -> Result<()>
    where
        I: IntoIterator<Item = TestTemplate>,
    {
        self.dependency_manager.add_templates(templates)
    }

    /// 执行所有测试模板
    pub fn execute_all(&mut self) -> Result<HashMap<PathBuf, ExecutionResult>> {
        info!("开始执行批量测试");
        
        // 构建依赖图并确定执行顺序
        self.dependency_manager.build_dependency_graph()?;
        let execution_order = self.dependency_manager.get_execution_order();
        
        info!("按照依赖顺序执行 {} 个测试模板", execution_order.len());
        
        // 按照排序后的顺序执行模板
        for template_path in execution_order {
            // 获取模板
            let template = match self.dependency_manager.get_template(&template_path) {
                Some(t) => t.clone(),
                None => {
                    warn!("找不到模板：{}", template_path.display());
                    continue;
                }
            };
            
            info!("执行测试模板: {}", template_path.display());
            
            // 创建模板执行器
            let mut executor = TemplateExecutor::new(
                self.work_dir.clone(),
                self.connection_manager,
                Some(self.options.clone()),
            );
            
            // 执行模板
            match executor.execute_template(template.clone(), self.target_config.clone()) {
                Ok(result) => {
                    // 记录执行结果
                    debug!("测试模板执行完成: {}, 状态: {:?}", template_path.display(), result.overall_status);
                    self.results.insert(template_path.clone(), result);
                }
                Err(e) => {
                    error!("执行模板失败: {}, 错误: {}", template_path.display(), e);
                    // 如果不允许失败，中止执行
                    if !self.options.continue_on_error {
                        bail!("执行模板 {} 失败: {}", template_path.display(), e);
                    }
                }
            }
        }
        
        info!("批量测试执行完成，共完成 {} 个模板测试", self.results.len());
        
        // 返回结果的克隆，而不是引用
        Ok(self.results.clone())
    }

    /// 获取所有执行结果
    pub fn get_results(&self) -> &HashMap<PathBuf, ExecutionResult> {
        &self.results
    }

    /// 获取特定模板的执行结果
    pub fn get_result(&self, template_path: &Path) -> Option<&ExecutionResult> {
        self.results.get(template_path)
    }

    /// 生成所有模板的报告
    pub fn generate_reports(&self, report_dir: &Path) -> Result<()> {
        info!("生成测试报告");
        
        // 确保报告目录存在
        std::fs::create_dir_all(report_dir)?;
        
        let mut report_paths = HashMap::new();
        
        for (template_path, result) in &self.results {
            // 获取模板
            let template = match self.dependency_manager.get_template(&template_path) {
                Some(t) => t.clone(),
                None => {
                    warn!("找不到模板：{}", template_path.display());
                    continue;
                }
            };
            
            // 确定报告文件名
            let file_stem = template_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
                
            let target_name = &result.target_name;
            let report_name = format!("{}_{}.report.md", file_stem, target_name);
            let report_path = report_dir.join(report_name);
            
            // 生成报告
            let mut reporter = crate::template::Reporter::new(report_path.clone(), None);
            reporter.generate_report(&template, result)?;
            
            info!("生成报告: {}", report_path.display());
            report_paths.insert(template_path.clone(), report_path);
        }
        
        // 生成汇总报告
        let summary_path = report_dir.join("summary.md");
        self.generate_summary_report(&summary_path)?;
        
        Ok(())
    }

    /// 生成汇总报告
    fn generate_summary_report(&self, path: &Path) -> Result<()> {
        info!("生成汇总报告: {}", path.display());
        
        let mut content = String::new();
        content.push_str("# 测试结果汇总\n\n");
        content.push_str(&format!("执行时间: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        // 汇总表格
        content.push_str("| 测试模板 | 状态 | 单元名称 | 目标 |\n");
        content.push_str("|---------|------|----------|------|\n");
        
        // 按照执行顺序排序结果
        let mut sorted_results = Vec::new();
        for path in self.dependency_manager.get_execution_order() {
            if let Some(result) = self.results.get(path) {
                sorted_results.push((path, result));
            }
        }
        
        for (path, result) in sorted_results {
            let status_str = match result.overall_status {
                crate::template::StepStatus::Pass => "✅ 通过",
                crate::template::StepStatus::Fail => "❌ 失败",
                crate::template::StepStatus::Skipped => "⏭️ 跳过",
                _ => "❓ 未知",
            };
            
            let template_name = path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
                
            content.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                template_name, status_str, result.unit_name, result.target_name
            ));
        }
        
        // 依赖关系图
        content.push_str("\n## 测试依赖关系\n\n");
        content.push_str("```\n");
        
        for path in self.dependency_manager.get_execution_order() {
            let template_name = path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
                
            let dependencies = self.dependency_manager.get_dependencies(path);
            if dependencies.is_empty() {
                content.push_str(&format!("{} (无依赖)\n", template_name));
            } else {
                let dep_names: Vec<_> = dependencies.iter()
                    .filter_map(|p| p.file_name().and_then(|s| s.to_str()))
                    .collect();
                content.push_str(&format!("{} -> [{}]\n", template_name, dep_names.join(", ")));
            }
        }
        
        content.push_str("```\n");
        
        // 写入文件
        std::fs::write(path, content)?;
        
        Ok(())
    }
}