//! Entry point of whole application
mod config;
mod connection;
mod pool;
mod template;
mod utils;

use crate::config::cli_args::CliArgs;
use crate::config::target_config::TargetConfig;
use crate::template::{discover_templates, filter_templates, TemplateFilter};
use crate::template::{StepStatus, TestTemplate};
use env_logger::Env;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::error::Error;
use std::{
    env,
    path::{Path, PathBuf},
};
use template::{BatchExecutor, BatchOptions, ExecutionResult, ExecutorOptions};

/// The main function of the application.
fn main() -> Result<(), Box<dyn Error>> {
    // 解析命令行参数
    let cli_args = parse_args();

    // 设置日志级别
    env_logger::Builder::from_env(Env::default().default_filter_or(cli_args.get_log_level()))
        .init();

    // 获取工作目录
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let test_dir = cli_args
        .test_dir
        .as_ref()
        .map(|dir| cwd.join(dir))
        .unwrap_or(match cli_args.template.as_ref() {
            Some(template_file) => {
                // 如果指定了单个模板文件，则使用该文件所在目录作为工作目录
                template_file.parent().unwrap_or(&cwd).to_path_buf()
            }
            None => cwd.clone(), // 如果没有指定模板文件，则使用当前工作目录
        });
    debug!("Working directory: {}", test_dir.display());

    // 检查是否有指定单个模板文件
    if let Some(template_file) = cli_args.template.as_ref() {
        // 应用环境类型设置--
        run_single_template_test(template_file, &cli_args, &test_dir)?;
    } else {
        // 创建模板过滤器，应用单元和标签过滤
        run_template_tests(&cli_args, &test_dir)?
    }
    Ok(())
}

/// 解析命令行参数
/// 返回解析后的`CliArgs`对象
fn parse_args() -> CliArgs {
    CliArgs::parse_args()
}

/// 运行单个测试模板文件
///
/// # Arguments
///
/// * `template_file` - 模板文件路径
/// * `cli_args` - 命令行参数
/// * `test_dir` - 工作目录
fn run_single_template_test(
    template_file: &Path,
    cli_args: &CliArgs,
    test_dir: &Path,
) -> Result<String, Box<dyn Error>> {
    info!("Processing single template: {}", template_file.display());

    // 解析模板
    let template = match template::TestTemplate::from_file(template_file) {
        Ok(t) => t,
        Err(e) => {
            return Err(format!("Failed to load template from file: {e}").into());
        }
    };

    let target_config_path = template.metadata.target_config.get_path();

    // 如果是仅解析模式，则只验证模板格式并显示信息
    if cli_args.parse_only {
        info!("  Title: {}", template.metadata.title);
        info!("  Unit: {}", template.metadata.unit_name);
        info!("  Target config: {}", target_config_path.display());
        info!("  Total steps: {}", template.steps.len());
        return Ok("Template parsed successfully".to_string());
    }

    // 加载目标配置
    info!("Loading target config: {}", target_config_path.display());

    let mut target_config: TargetConfig = match TargetConfig::from_file(
        target_config_path
            .to_str()
            .expect("Target path does not exist"),
    ) {
        Ok(config) => config,
        Err(e) => {
            return Err(format!("Failed to load target config: {e}").into());
        }
    };

    // 如果有环境类型覆盖，则更新目标配置
    let environment_type = cli_args.get_environment_type();
    if let Some(env_type) = &environment_type {
        info!("Overriding environment type to: {env_type}");
        target_config.testing_type = env_type.clone();
    }

    // 创建变量管理器
    let variable_manager = template::VariableManager::new();

    // 创建连接池
    let connection_pool = pool::ConnectionManagerPool::new();

    // 准备执行选项，优先级顺序: CLI参数 > target_config.executor > 默认值
    let default_options = ExecutorOptions::default();
    let executor_options = ExecutorOptions {
        command_timeout: cli_args
            .timeout
            .or(target_config.executor.command_timeout.map(|d| d.as_secs()))
            .unwrap_or(default_options.command_timeout),
        retry_count: cli_args
            .retry
            .or(target_config.executor.retry_count)
            .unwrap_or(default_options.retry_count),
        retry_interval: cli_args
            .retry_interval
            .or(target_config.executor.retry_interval)
            .unwrap_or(default_options.retry_interval),
        maintain_session: cli_args
            .maintain_session
            .or(target_config.executor.maintain_session)
            .unwrap_or(default_options.maintain_session),
        continue_on_error: cli_args
            .continue_on_error
            .or(target_config.executor.continue_on_error)
            .unwrap_or(default_options.continue_on_error),
    };

    // 定义报告目录
    let report_dir = cli_args
        .reports_dir
        .as_ref()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| test_dir.join("reports"));

    // 准备批量选项
    let batch_options = BatchOptions {
        test_directory: Some(test_dir.to_path_buf()),
        report_directory: Some(report_dir.clone()),
        executor_options: executor_options.clone(),
        keep_template_directory_structure: cli_args.keep_template_directory_structure,
    };

    // 创建批量执行器
    let mut batch_executor =
        BatchExecutor::new(variable_manager, connection_pool, Some(batch_options));

    // 添加模板到执行器
    batch_executor.add_template(template.into())?;

    // 执行模板测试
    match batch_executor.execute_all() {
        Ok(results) => {
            info!("Template execution completed");

            // 检查测试结果
            if !results.is_empty() {
                // 获取第一个结果，使用迭代器而不是索引
                if let Some(result) = results.first() {
                    Ok(format!(
                        "Test completed with status: {:?}",
                        result.overall_status
                    ))
                } else {
                    Err("Failed to fetch first execution result".to_string().into())
                }
            } else {
                Err("Execution result is empty".to_string().into())
            }
        }
        Err(e) => Err(format!("Failed to execute template: {e}").into()),
    }
}

/// Run tests using Markdown templates, with batching for shared configurations.
///
/// # Arguments
///
/// * `cli_args` - Command line arguments.
/// * `test_dir` - Working directory containing templates and target configs.
fn run_template_tests(cli_args: &CliArgs, test_dir: &Path) -> Result<(), Box<dyn Error>> {
    info!("Discovering Markdown test templates...");

    // Guess the template directories in "tests" and "templates" subdirectories
    // Hardcoded LOL...
    let template_dirs = vec![
        test_dir.to_path_buf(),
        test_dir.join("tests"),
        test_dir.join("templates"),
    ];
    let mut all_template_paths = Vec::new();
    for dir in &template_dirs {
        if let Ok(mut paths) = discover_templates(dir, true) {
            paths.retain(|path| !all_template_paths.contains(path));
            all_template_paths.extend(paths);
        }
    }
    info!(
        "Found {} template files (deduplicated)",
        all_template_paths.len()
    );
    if all_template_paths.is_empty() {
        warn!("No template files found in the specified directories.");
        return Ok(());
    }

    let (unit_filter, tag_filter, target_metadata_filter) = cli_args.get_filters();
    let filter = TemplateFilter {
        target: target_metadata_filter.map(|t| t.to_string()),
        unit: unit_filter.map(|u| u.to_string()),
        tags: tag_filter.map_or_else(Vec::new, |t| vec![t.to_string()]),
    };

    let loaded_templates: Vec<TestTemplate> = match filter_templates(all_template_paths, &filter) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to load or filter templates: {e}");
            return Err(e.into()); // Propagate error
        }
    };

    if loaded_templates.is_empty() {
        warn!("No templates found matching the criteria after filtering.");
        return Ok(());
    }
    info!(
        "Successfully loaded and filtered {} templates.",
        loaded_templates.len()
    );

    if cli_args.parse_only {
        info!("Parse-only mode. Displaying template information:");
        for template in &loaded_templates {
            info!("  Title: {}", template.metadata.title);
            info!("  Unit: {}", template.metadata.unit_name);
            info!(
                "  Target config: {}",
                template.metadata.target_config.get_path().display()
            );
            info!("  Total steps: {}", template.steps.len());
        }
        return Ok(());
    }

    let environment_type_override = cli_args.get_environment_type();
    let mut grouped_templates: HashMap<(PathBuf, Option<String>), Vec<TestTemplate>> =
        HashMap::new();

    let templates_for_display = loaded_templates.clone();

    for template in loaded_templates {
        let target_config_file_path = test_dir.join(template.metadata.target_config.get_path());
        let group_key = (target_config_file_path, environment_type_override.clone());
        grouped_templates
            .entry(group_key)
            .or_default()
            .push(template);
    }

    info!(
        "Templates grouped into {} batches based on target configuration and environment override.",
        grouped_templates.len()
    );

    let report_dir = cli_args
        .reports_dir
        .as_ref()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| test_dir.join("reports"));

    if !report_dir.exists() {
        std::fs::create_dir_all(&report_dir).map_err(|e| {
            format!(
                "Failed to create report directory '{}': {}",
                report_dir.display(),
                e
            )
        })?;
    }

    let mut all_results: Vec<(PathBuf, ExecutionResult)> = Vec::new();
    for template in &templates_for_display {
        let metadata = &template.metadata;
        info!("Template: {}", metadata.title);
        info!("  Unit: {}", metadata.unit_name);
        info!(
            "  Target config: {}",
            metadata.target_config.get_path().display()
        );
        info!("  Total steps: {}", template.steps.len());
    }

    for ((target_config_path, group_env_override), templates_in_group) in grouped_templates {
        info!(
            "Processing batch for target_config: {}, env_override: {:?}",
            target_config_path.display(),
            group_env_override
        );

        if !target_config_path.exists() {
            let msg = format!(
                "Target configuration file not found: {}. Skipping {} templates in this group.",
                target_config_path.display(),
                templates_in_group.len()
            );
            error!("{msg}");
            warn!(
                "Skipping batch due to missing target configuration: {}",
                target_config_path.display()
            );
            continue;
        }

        let mut target_config: TargetConfig = match utils::read_toml_from_file(&target_config_path)
        {
            Ok(config) => config,
            Err(e) => {
                let msg = format!(
                    "Failed to load target config '{}': {}. Skipping {} templates in this group.",
                    target_config_path.display(),
                    e,
                    templates_in_group.len()
                );
                error!("{msg}");
                warn!(
                    "Skipping batch due to target configuration load failure: {}",
                    target_config_path.display()
                );
                continue;
            }
        };

        if let Some(env_type) = &group_env_override {
            info!(
                "Overriding environment type to: {} for target config {}",
                env_type,
                target_config_path.display()
            );
            target_config.testing_type = env_type.clone();
        }

        // 执行器选项，优先级顺序: CLI参数 > target_config.executor > 默认值
        let default_options = ExecutorOptions::default();
        let executor_options = ExecutorOptions {
            command_timeout: cli_args
                .timeout
                .or(target_config.executor.command_timeout.map(|d| d.as_secs()))
                .unwrap_or(default_options.command_timeout),
            retry_count: cli_args
                .retry
                .or(target_config.executor.retry_count)
                .unwrap_or(default_options.retry_count),
            retry_interval: cli_args
                .retry_interval
                .or(target_config.executor.retry_interval)
                .unwrap_or(default_options.retry_interval),
            maintain_session: cli_args
                .maintain_session
                .or(target_config.executor.maintain_session)
                .unwrap_or(default_options.maintain_session),
            continue_on_error: cli_args
                .continue_on_error
                .or(target_config.executor.continue_on_error)
                .unwrap_or(default_options.continue_on_error),
        };

        // 批量执行选项
        let batch_options = BatchOptions {
            test_directory: Some(test_dir.to_path_buf()),
            report_directory: Some(report_dir.clone()),
            executor_options: executor_options.clone(),
            keep_template_directory_structure: cli_args.keep_template_directory_structure,
        };

        let variable_manager = template::VariableManager::new();
        // 创建连接池
        let connection_pool = pool::ConnectionManagerPool::new();
        let batch_execution_results = {
            let mut batch_executor =
                BatchExecutor::new(variable_manager, connection_pool, Some(batch_options));

            info!(
                "Adding {} templates individually to batch for target_config '{}'",
                templates_in_group.len(),
                target_config_path.display()
            );
            for template in templates_in_group {
                let metadata = &template.metadata;
                debug!("Added template '{}' to batch executor", metadata.title);
                batch_executor.add_template(template.into())?;
            }

            info!(
                "Executing batch for target_config '{}'...",
                target_config_path.display()
            );
            match batch_executor.execute_all() {
                Ok(results_vec) => {
                    info!(
                        "Batch execution completed for target_config '{}'. {} results.",
                        target_config_path.display(),
                        results_vec.len()
                    );
                    results_vec
                }
                Err(e) => {
                    let msg = format!(
                        "Failed to execute template batch for target_config '{}': {}",
                        target_config_path.display(),
                        e
                    );
                    error!("{msg}");
                    warn!(
                        "Continuing after batch execution failure for: {}",
                        target_config_path.display()
                    );
                    Vec::new()
                }
            }
        };

        let converted_results: Vec<(PathBuf, ExecutionResult)> = batch_execution_results
            .into_iter()
            .map(|exec_result| (exec_result.template.file_path.clone(), exec_result))
            .collect();
        all_results.extend(converted_results);
    }

    let mut success_count = 0;
    let mut fail_count = 0;
    for (_path, result) in &all_results {
        match result.overall_status {
            StepStatus::Pass => success_count += 1,
            StepStatus::Fail => fail_count += 1,
            _ => {}
        }
    }

    info!(
        "Overall test summary: {} successful, {} failed out of {} executed templates/results.",
        success_count,
        fail_count,
        all_results.len()
    );

    if fail_count > 0 {
        return Err(format!("{fail_count} tests failed and continue_on_error is false.").into());
    }

    Ok(())
}
