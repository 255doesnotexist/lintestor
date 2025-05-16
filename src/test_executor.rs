//! 提供统一的测试执行逻辑，适用于不同的测试环境

use crate::test_script_manager::TestScriptManager; // 保持对旧版的兼容性（存疑）
                                                   // 疑似并没有保持成功，先留这坨东西在这里吧
use crate::test_environment::TestEnvironment;
use crate::utils::{PackageMetadata, Report, TestResult, REMOTE_TMP_DIR};
use anyhow::Result;
use log::{debug, info};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

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

        let template_manager = TestScriptManager::new(target, unit, skip_templates, dir)?;
        let (os_version, kernel_version) = self.environment.get_os_info()?;
        let mut all_tests_passed = true;
        let mut test_results = Vec::new();

        let prerequisite_path = dir.join(format!("{}/prerequisite.sh", target));
        let local_unit_dir = dir.join(format!("{}/{}", target, unit));

        // 确保临时目录存在
        self.environment.mkdir(REMOTE_TMP_DIR)?;

        for template_path in template_manager.get_test_scripts() {
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

        let unit_metadata = if let Some(metadata_template) = template_manager.get_metadata_script()
        {
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
            )
            .into());
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

        let _local_tar_guard = LocalCleanupGuard {
            path: local_tar_path.clone(),
        };

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
        self.environment
            .upload_file(&local_tar_path, &remote_tar_path, 0o644)?;

        if prerequisite_template_local_path.exists() {
            info!("上传前提条件脚本到 {}", remote_prerequisite_path);
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
            return Err(format!("解压远程存档失败: {}", extract_output.output).into());
        }

        // 解压后清理远程存档文件
        self.environment.rm_rf(&remote_tar_path)?;

        // --- 执行测试 ---
        info!("在远程目录中执行测试: {}", remote_unit_dir);
        let template_manager =
            TestScriptManager::new(target, unit, skip_templates, &local_unit_dir)?; // 使用本地目录来发现脚本
        let mut all_tests_passed = true;
        let mut test_results = Vec::new();

        for template_name in template_manager.get_test_script_names() {
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
        let unit_metadata = if let Some(metadata_template_name) =
            template_manager.get_metadata_script_name()
        {
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
