//! Entry point of whole application
mod aggregator;
mod config;
mod connection;
mod markdown_report;
mod template;
mod test_runner;
mod test_environment;
mod test_executor;
mod test_script_manager;
mod utils;

use crate::config::target_config::TargetConfig;
use crate::config::cli_args::CliArgs;
use crate::template::{TemplateFilter, discover_templates, filter_templates};
use env_logger::Env;
use log::{debug, error, info, warn};
use template::{BatchExecutor, BatchOptions, ExecutionResult, ExecutorOptions};
use std::error::Error;
use std::{env, path::{Path, PathBuf}};
use std::collections::HashMap;
use crate::template::{TestTemplate, StepStatus};

/// The main function of the application.
fn main() {
    // 解析命令行参数
    let cli_args = parse_args();
    
    // 设置日志级别
    env_logger::Builder::from_env(Env::default().default_filter_or(cli_args.get_log_level())).init();
    
    // 获取工作目录
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let working_dir = cli_args.test_dir
        .as_ref()
        .map(|dir| cwd.join(dir))
        .unwrap_or(cwd);
    debug!("Working directory: {}", working_dir.display());

    // 执行测试、聚合或汇总操作
    if cli_args.should_test() {
        // 检查是否有指定单个模板文件
        if let Some(template_file) = cli_args.template.as_ref() {
            // 应用环境类型设置
            run_single_template_test(template_file, &cli_args, &working_dir);
        } else {
            // 创建模板过滤器，应用单元和标签过滤
            if let Err(e) = run_template_tests(&cli_args, &working_dir) {
                error!("Failed to run template tests: {}", e);
            }
        }
    }

    if cli_args.should_aggregate() {
        info!("Aggregating reports");
        let reports_dir = cli_args.reports_dir.as_deref();
        let output_path = cli_args.output.as_deref();
        
        if let Err(e) = aggregator::aggregate_reports_from_dir(reports_dir, output_path) {
            error!("Failed to aggregate reports: {}", e);
        }
    }

    if cli_args.should_summarize() {
        info!("Generating summary report");
        let reports_json = cli_args.reports_json.as_deref();
        let summary_path = cli_args.summary_path.as_deref();
        
        if let Err(e) = markdown_report::generate_markdown_summary_from_json(reports_json, summary_path) {
            error!("Failed to generate markdown report: {}", e);
        }
    }
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
/// * `working_dir` - 工作目录
fn run_single_template_test(
    template_file: &Path,
    cli_args: &CliArgs,
    working_dir: &Path,
) {
    info!("Processing single template: {}", template_file.display());
    
    // 解析模板
    let template = match template::TestTemplate::from_file(template_file) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to load template from file: {}", e);
            return;
        }
    };
    
    // 如果是仅解析模式，则只验证模板格式并显示信息
    if cli_args.is_parse_only() {
        info!("Template parsed successfully:");
        info!("  Title: {}", template.metadata.title);
        info!("  Unit: {}", template.metadata.unit_name);
        info!("  Target config: {}", template.metadata.target_config.display());
        info!("  Total steps: {}", template.steps.len());
        return;
    }
    
    // 加载目标配置
    let target_config_path = working_dir.join(&template.metadata.target_config);
    info!("Loading target config: {}", target_config_path.display());
    
    let mut target_config: TargetConfig = match utils::read_toml_from_file(&target_config_path) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load target config: {}", e);
            return;
        }
    };
    
    // 如果有环境类型覆盖，则更新目标配置
    let environment_type = cli_args.get_environment_type();
    if let Some(env_type) = &environment_type {
        info!("Overriding environment type to: {}", env_type);
        target_config.testing_type = env_type.clone();
    }

    // 创建变量管理器
    let mut variable_manager = template::VariableManager::new();
    
    // 准备执行选项，从cli_args获取超时和重试设置
    #[allow(dead_code)]
    // TODO: 实现把这个传到 executor 里面，未来有机会的话
    let executor_options = ExecutorOptions {
        command_timeout: cli_args.timeout, // 从cli_args获取超时设置
        retry_count: cli_args.retry,            // 从cli_args获取重试次数
        retry_interval: 5,                      // 默认重试间隔5秒
        maintain_session: true,
        continue_on_error: cli_args.continue_on_error, // 从cli_args获取错误处理策略
    };

    // 定义报告目录
    let report_dir = cli_args.report_path
        .as_ref()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| working_dir.join("reports"));

    // 准备批量选项
    let batch_options = BatchOptions {
        report_directory: Some(report_dir.clone()),
        command_timeout_seconds: Some(executor_options.command_timeout),
        continue_on_error: executor_options.continue_on_error,
    };
    
    // 创建批量执行器
    let mut batch_executor = BatchExecutor::new(
        variable_manager,
        Some(batch_options),
    );
    
    // 添加模板到执行器
    batch_executor.add_template(template.into());
    
    // 执行模板测试
    match batch_executor.execute_all() {
        Ok(results) => {
            info!("Template execution completed");
            
            // 检查测试结果
            if !results.is_empty() {
                // 获取第一个结果，使用迭代器而不是索引
                if let Some(result) = results.first() {
                    info!("Test completed with status: {:?}", result.overall_status);
                }
            }
        }
        Err(e) => {
            error!("Failed to execute template: {}", e);
        }
    }
}

/// Run tests using Markdown templates, with batching for shared configurations.
///
/// # Arguments
///
/// * `cli_args` - Command line arguments.
/// * `working_dir` - Working directory containing templates and target configs.
fn run_template_tests(
    cli_args: &CliArgs,
    working_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    info!("Discovering Markdown test templates...");

    let template_dirs = vec![
        working_dir.to_path_buf(),
        working_dir.join("tests"),
        working_dir.join("templates"),
    ];
    let mut all_template_paths = Vec::new();
    for dir in &template_dirs {
        if let Ok(paths) = discover_templates(dir, true) {
            all_template_paths.extend(paths);
        }
    }
    info!("Found {} template files", all_template_paths.len());
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

    let loaded_templates: Vec<TestTemplate> = match filter_templates(&all_template_paths, &filter) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to load or filter templates: {}", e);
            return Err(e.into()); // Propagate error
        }
    };

    if loaded_templates.is_empty() {
        warn!("No templates found matching the criteria after filtering.");
        return Ok(());
    }
    info!("Successfully loaded and filtered {} templates.", loaded_templates.len());

    if cli_args.is_parse_only() {
        info!("Parse-only mode. Displaying template information:");
        for template in &loaded_templates {
            info!("  Title: {}", template.metadata.title);
            info!("  Unit: {}", template.metadata.unit_name);
            info!("  Target config: {}", template.metadata.target_config.display());
            info!("  Total steps: {}", template.steps.len());
        }
        return Ok(());
    }

    let environment_type_override = cli_args.get_environment_type();
    let mut grouped_templates: HashMap<(PathBuf, Option<String>), Vec<TestTemplate>> = HashMap::new();

    let templates_for_display = loaded_templates.clone();
    
    for template in loaded_templates {
        let target_config_file_path = working_dir.join(&template.metadata.target_config);
        let group_key = (target_config_file_path, environment_type_override.clone());
        grouped_templates.entry(group_key).or_default().push(template);
    }

    info!("Templates grouped into {} batches based on target configuration and environment override.", grouped_templates.len());

    let report_dir = cli_args.report_path
        .as_ref()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| working_dir.join("reports"));
    
    if !report_dir.exists() {
        std::fs::create_dir_all(&report_dir)
            .map_err(|e| format!("Failed to create report directory '{}': {}", report_dir.display(), e))?;
    }

    let mut all_results: Vec<(PathBuf, ExecutionResult)> = Vec::new();
    for template in &templates_for_display {
        let metadata = &template.metadata;
        info!("Template: {}", metadata.title);
        info!("  Unit: {}", metadata.unit_name);
        info!("  Target config: {}", metadata.target_config.display());
        info!("  Total steps: {}", template.steps.len());
    }

    for ((target_config_path, group_env_override), templates_in_group) in grouped_templates {
        info!("Processing batch for target_config: {}, env_override: {:?}", 
              target_config_path.display(), group_env_override);

        if !target_config_path.exists() {
            let msg = format!("Target configuration file not found: {}. Skipping {} templates in this group.", 
                              target_config_path.display(), templates_in_group.len());
            error!("{}", msg);
            if !cli_args.continue_on_error { return Err(msg.into()); }
            warn!("Skipping batch due to missing target configuration: {}", target_config_path.display());
            continue;
        }

        let mut target_config: TargetConfig = match utils::read_toml_from_file(&target_config_path) {
            Ok(config) => config,
            Err(e) => {
                let msg = format!("Failed to load target config '{}': {}. Skipping {} templates in this group.", 
                                  target_config_path.display(), e, templates_in_group.len());
                error!("{}", msg);
                if !cli_args.continue_on_error { return Err(msg.into()); }
                warn!("Skipping batch due to target configuration load failure: {}", target_config_path.display());
                continue;
            }
        };

        if let Some(env_type) = &group_env_override {
            info!("Overriding environment type to: {} for target config {}", env_type, target_config_path.display());
            target_config.testing_type = env_type.clone();
        }

        // 执行器选项
        #[allow(dead_code)]
        let executor_options = ExecutorOptions {
            command_timeout: cli_args.timeout,
            retry_count: cli_args.retry,
            retry_interval: 5,
            maintain_session: true,
            continue_on_error: cli_args.continue_on_error,
        };

        // 批量执行选项
        let batch_options = BatchOptions {
            report_directory: Some(report_dir.clone()),
            command_timeout_seconds: Some(executor_options.command_timeout),
            continue_on_error: executor_options.continue_on_error,
        };

        let mut variable_manager = template::VariableManager::new();

        let batch_execution_results = {
            let mut batch_executor = BatchExecutor::new(
                variable_manager,
                Some(batch_options),
            );

            info!("Adding {} templates individually to batch for target_config '{}'", templates_in_group.len(), target_config_path.display());
            for template in templates_in_group {
                let metadata = &template.metadata;
                debug!("Added template '{}' to batch executor", metadata.title);
                batch_executor.add_template(template.into());
            }

            info!("Executing batch for target_config '{}'...", target_config_path.display());
            match batch_executor.execute_all() {
                Ok(results_vec) => {
                    info!("Batch execution completed for target_config '{}'. {} results.", 
                          target_config_path.display(), results_vec.len());
                    results_vec
                }
                Err(e) => {
                    let msg = format!("Failed to execute template batch for target_config '{}': {}", target_config_path.display(), e);
                    error!("{}", msg);
                    if !cli_args.continue_on_error { return Err(msg.into()); }
                    warn!("Continuing after batch execution failure for: {}", target_config_path.display());
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

    info!("Overall test summary: {} successful, {} failed out of {} executed templates/results.", 
          success_count, fail_count, all_results.len());

    if fail_count > 0 && !cli_args.continue_on_error {
        return Err(format!("{} tests failed and continue_on_error is false.", fail_count).into());
    }

    Ok(())
}
