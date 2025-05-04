//! Entry point of whole application
mod aggregator;
mod config;
mod connection;
mod markdown_report;
mod template;
mod test_runner;
mod test_environment;
mod test_executor;
mod test_template_manager;
mod utils;

use crate::config::target_config::TargetConfig;
use crate::config::cli_args::CliArgs;
use crate::template::{TemplateFilter, discover_templates, filter_templates, TemplateExecutor, Reporter};
use crate::connection::ConnectionFactory;
use crate::test_runner::{local::LocalTestRunner, remote::RemoteTestRunner, qemu::QemuTestRunner, TestRunner};
use crate::utils::Report;
use clap::{Arg, ArgAction, ArgMatches, Command};
use connection::ConnectionManager;
use env_logger::Env;
use log::{debug, error, info, warn};
use template::ExecutorOptions;
use test_executor::BatchExecutor;
use std::{env, error::Error, fs::File, path::{Path, PathBuf}};
use test_runner::boardtest::BoardtestRunner;

/// The version of the application.
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The name of the application.
const NAME: &str = env!("CARGO_PKG_NAME");
/// The authors of the application.
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
/// The description of the application.
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

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
    
    // 获取目标列表
    let discovered_targets = utils::get_targets(&working_dir).unwrap_or_default();
    
    // 如果指定了单元过滤，则解析单元名称列表
    let (unit_filter, tag_filter) = cli_args.get_filters();
    
    // 获取是否有环境类型设置
    let environment_type = cli_args.get_environment_type();
    
    // 执行测试、聚合或汇总操作
    if cli_args.should_test() {
        // 检查是否有指定单个模板文件
        if let Some(template_file) = &cli_args.template {
            info!("Running test for single template file: {}", template_file.display());
            // 实现单个模板文件的测试逻辑
            run_single_template_test(template_file, &environment_type, &working_dir, cli_args.is_parse_only());
        } else {
            // 正常的测试模板处理流程
            info!("Running tests using Markdown templates");
            
            // 提取单元和标签过滤条件
            let units = match unit_filter {
                Some(unit) => vec![unit.to_string()],
                None => utils::get_all_units(&discovered_targets.iter().map(|s| s.as_str()).collect::<Vec<&str>>(), &working_dir).unwrap_or_default()
            };
            
            let tags = tag_filter.map(|tag| vec![tag.to_string()]).unwrap_or_default();
            
            run_template_tests(
                discovered_targets,
                units,
                tags,
                &working_dir,
                cli_args.is_parse_only(),
                environment_type,
                cli_args.report_path.as_deref(),
            );
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

/// Run tests using Markdown templates
/// 
/// # Arguments
/// 
/// * `targets` - Target names to filter templates by
/// * `units` - Unit names to filter templates by
/// * `tags` - Tags to filter templates by
/// * `working_dir` - Working directory containing templates and target configs
/// * `parse_only` - If true, only parse templates without executing commands
/// * `environment_type` - Optional environment type override
/// * `report_path` - Optional custom report output path
fn run_template_tests(
    targets: Vec<String>,
    units: Vec<String>,
    tags: Vec<String>,
    working_dir: &Path,
    parse_only: bool,
    environment_type: Option<String>,
    report_path: Option<&Path>,
) {
    info!("Discovering Markdown test templates...");
    
    // 搜索工作目录下的测试模板
    let template_dirs = vec![
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
    
    // 根据参数过滤模板
    let filter = TemplateFilter {
        target: if targets.len() == 1 { Some(targets[0].clone()) } else { None },
        unit: if units.len() == 1 { Some(units[0].clone()) } else { None },
        tags,
    };
    
    let templates = match filter_templates(&all_template_paths, &filter) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to filter templates: {}", e);
            return;
        }
    };
    
    if templates.is_empty() {
        warn!("No templates found matching the criteria");
        return;
    }
    
    info!("Running {} filtered templates", templates.len());
    
    let mut results = Vec::new();
    let reporter = Reporter::new(working_dir.to_path_buf(), None);
    
    // 执行每个模板
    for template in templates {
        info!("Processing template: {}", template.file_path.display());
        
        // 加载目标配置
        let target_config_path = working_dir.join(&template.metadata.target_config);
        info!("Loading target config: {}", target_config_path.display());
        
        let target_config: TargetConfig = match utils::read_toml_from_file(&target_config_path) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to load target config: {}", e);
                continue;
            }
        };
        
        // 创建连接管理器
        let mut connection_manager = match ConnectionFactory::create_manager(&target_config) {
            Ok(manager) => manager,
            Err(e) => {
                error!("Failed to create connection manager: {}", e);
                continue;
            }
        };
        
        // 创建模板执行器
        let mut executor = TemplateExecutor::new(
            working_dir.to_path_buf(),
            connection_manager.as_mut(),
            None,
        );
        
        // 执行模板
        match executor.execute_template(template.clone(), target_config) {
            Ok(result) => {
                info!("Template execution completed with status: {:?}", result.overall_status);
                
                // 生成报告
                match reporter.generate_report(&template, &result) {
                    Ok(report_path) => {
                        let mut result_with_path = result;
                        result_with_path.report_path = Some(report_path);
                        results.push(result_with_path);
                    }
                    Err(e) => {
                        error!("Failed to generate report: {}", e);
                        results.push(result);
                    }
                }
            }
            Err(e) => {
                error!("Failed to execute template: {}", e);
            }
        }
    }
    
    // 生成总结报告
    if !results.is_empty() {
        info!("Generating summary report");
        match reporter.generate_summary_report(&results, None) {
            Ok(path) => info!("Summary report generated: {}", path.display()),
            Err(e) => error!("Failed to generate summary report: {}", e),
        }
    }
}

/// Run tests (for all distributions by default)
/// # Arguments
/// - `targets`: Array of distribution names.
/// - `units`: Array of unit names.
/// - `skip_successful`: Skip previous successful tests (instead of overwriting their results).
/// - `dir`: Working directory which contains the test folders and files, defaults to env::current_dir()
///
/// # Returns
/// Returns `Ok(())` if successful, otherwise returns an error.
///
fn run_tests(
    targets: &[&str],
    units: &[&str],
    skip_successful: bool,
    dir: &Path,
    allow_interactive_prompts: &bool,
) {
    for target in targets {
        let target_directory = dir.join(target);
        if !target_directory.exists() {
            warn!(
                "target directory '{}' not found, skipping",
                target_directory.display()
            );
            continue;
        }
        let target_config_path = target_directory.join("config.toml");
        let target_config: TargetConfig = match utils::read_toml_from_file(&target_config_path) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to load config for {}: {}", target, e);
                continue;
            }
        };

        let run_locally = target_config.testing_type == "locally";
        let via_boardtest = target_config.testing_type == "boardtest";
        let is_qemu = target_config.testing_type == "qemu-based-remote";

        info!(
            "Testing type: {}",
            &target_config.testing_type
        );

        // 基于测试环境类型选择适当的测试运行器
        let mut test_runner: Box<dyn TestRunner> = if run_locally {
            // 本地测试环境
            Box::new(LocalTestRunner::new())
        } else if via_boardtest {
            // Boardtest测试环境
            if let Some(ref boardtest_config) = target_config.boardtest {
                Box::new(BoardtestRunner::new(boardtest_config))
            } else {
                error!("No boardtest config found for {}", target);
                continue;
            }
        } else {
            // QEMU或远程SSH环境
            let connection_config = match &target_config.connection {
                Some(c) => c,
                None => {
                    error!("No connection config found for {}", target);
                    continue;
                }
            };
            
            let ip = connection_config.ip.as_deref().unwrap_or("localhost");
            let port = connection_config.port.unwrap_or(2222);
            let username = connection_config.username.as_deref().unwrap_or("root");
            let password = connection_config.password.as_deref();
            let private_key_path = connection_config.private_key_path.as_deref();
            
            if is_qemu {
                // QEMU虚拟机环境
                Box::new(QemuTestRunner::new(
                    ip.to_string(),
                    port,
                    username.to_string(),
                    password.map(|p| p.to_string()),
                    private_key_path.map(|p| p.to_string()),
                    target_config.startup_template.clone(),
                    target_config.stop_template.clone(),
                    dir,
                ))
            } else {
                // 普通远程SSH环境
                debug!("Connecting to remote environment: IP={}, Port={}, Username={}", ip, port, username);
                Box::new(RemoteTestRunner::new(
                    ip.to_string(),
                    port,
                    username.to_string(),
                    password.map(|p| p.to_string()),
                    private_key_path.map(|p| p.to_string()),
                ))
            }
        };

        // 获取目标下的所有测试单元
        let units_of_target = utils::get_units(target, dir).unwrap_or_default();
        for unit in units
            .iter()
            .filter(|p| units_of_target.iter().any(|pkg| p == &pkg))
        {
            let mut skipped_templates = Vec::new();

            let unit_directory = target_directory.join(unit);
            if !unit_directory.exists() {
                warn!(
                    "Package testing directory '{}' not found, skipping",
                    unit_directory.display()
                );
                continue;
            }
            
            // 处理跳过已成功的测试逻辑
            if skip_successful {
                let report_path = unit_directory.join("report.json");
                if let Ok(file) = File::open(&report_path) {
                    let report: Result<Report, serde_json::Error> = serde_json::from_reader(file);
                    match report {
                        Ok(r) => {
                            if r.all_tests_passed {
                                info!("Skipping previous successful test {}/{}", target, unit);
                                continue;
                            } else {
                                for result in r.test_results {
                                    if result.passed {
                                        info!(
                                            "Skipping previous successful test {}/{}: {}",
                                            target, unit, result.test_name
                                        );

                                        skipped_templates.push(result.test_name);
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            warn!(
                                "Failed to parse test report for {}/{}, test will run anyway",
                                target, unit
                            )
                        }
                    }
                } else {
                    warn!(
                        "Failed to open test report for {}/{}, test will run anyway",
                        target, unit
                    );
                }
            }

            // 检查是否应该跳过该单元
            if let Some(skip_units) = &target_config.skip_units {
                if skip_units.iter().any(|pkg| pkg == unit) {
                    info!("Skipping test for {}/{} as configured in config.toml", target, unit);
                    continue;
                }
            }

            // 输出测试信息
            info!(
                "Running test for {}/{}, using {} environment",
                target,
                unit,
                &target_config.testing_type
            );

            // 执行测试并处理结果
            match test_runner.run_test(target, unit, skipped_templates, dir) {
                Ok(_) => info!("Test passed for {}/{}", target, unit),
                Err(e) => {
                    error!("Test failed for {}/{}: {}", target, unit, e);
                    // 交互式提示是否继续
                    if *allow_interactive_prompts {
                        use dialoguer::Confirm;
                        let resume = Confirm::new()
                            .with_prompt(format!("Test failed for {}/{}. Do you want to continue testing?", target, unit))
                            .default(true)
                            .interact()
                            .unwrap();
                        if !resume {
                            info!("Stopping tests for {}", target);
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// 运行单个测试模板文件
/// 
/// # Arguments
/// 
/// * `template_file` - 模板文件路径
/// * `environment_type` - 可选的环境类型
/// * `working_dir` - 工作目录
/// * `parse_only` - 是否为仅解析模式
fn run_single_template_test(
    template_file: &Path,
    environment_type: &Option<String>,
    working_dir: &Path,
    parse_only: bool,
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
    if parse_only {
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
    if let Some(env_type) = environment_type {
        info!("Overriding environment type to: {}", env_type);
        target_config.testing_type = env_type.clone();
    }
    
    // 创建连接管理器
    let mut connection_manager = match ConnectionFactory::create_manager(&target_config) {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to create connection manager: {}", e);
            return;
        }
    };
    
    // 创建模板执行器
    let mut executor = TemplateExecutor::new(
        working_dir.to_path_buf(),
        connection_manager.as_mut(),
        None,
    );
    
    // 执行模板
    match executor.execute_template(template.clone(), target_config) {
        Ok(result) => {
            info!("Template execution completed with status: {:?}", result.overall_status);
            
            // 生成报告
            let reporter = Reporter::new(working_dir.to_path_buf(), None);
            match reporter.generate_report(&template, &result) {
                Ok(report_path) => {
                    info!("Report generated: {}", report_path.display());
                }
                Err(e) => {
                    error!("Failed to generate report: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to execute template: {}", e);
        }
    }
}

/// 使用批量执行器运行Markdown测试
/// 
/// # Arguments
/// 
/// * `args` - 命令行参数
fn run_markdown_tests(args: &CliArgs) -> Result<(), Box<dyn Error>> {
    // 获取工作目录
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let work_dir = args.test_dir
        .as_ref()
        .map(|dir| cwd.join(dir))
        .unwrap_or(cwd);
    info!("使用工作目录: {}", work_dir.display());

    // 查找所有测试模板文件
    let template_dirs = vec![
        work_dir.join("tests"),
        work_dir.join("templates"),
    ];
    
    let mut test_files = Vec::new();
    for dir in &template_dirs {
        if let Ok(paths) = discover_templates(dir, true) {
            test_files.extend(paths);
        }
    }
    info!("发现 {} 个测试模板文件", test_files.len());

    // 应用过滤器（如果指定了）并解析模板
    let filtered_templates = if args.tag.is_none() && args.unit.is_none() {
        // 没有过滤器，解析所有找到的文件
        let mut templates = Vec::new();
        for file_path in &test_files {
            match template::TestTemplate::from_file(file_path) {
                Ok(template) => templates.push(template),
                Err(e) => {
                    error!("加载模板 {} 失败: {}", file_path.display(), e);
                    if !args.continue_on_error {
                        // Convert anyhow::Error to a type implementing std::error::Error
                        let io_error = std::io::Error::new(std::io::ErrorKind::Other, e.to_string());
                        return Err(Box::new(io_error));
                    }
                }
            }
        }
        templates
    } else {
        // 获取过滤条件
        let (unit_filter, tag_filter) = args.get_filters();
        
        filter_templates(
            &test_files, 
            &TemplateFilter {
                tags: tag_filter.map(|t| vec![t.to_string()]).unwrap_or_default(),
                unit: unit_filter.map(|u| u.to_string()),
                target: None, // 不按目标过滤
            }
        )?
    };
    
    info!("过滤后剩余 {} 个测试模板文件", filtered_templates.len());
    
    if filtered_templates.is_empty() {
        warn!("没有符合条件的测试文件，退出");
        return Ok(());
    }
    
    // 准备目标配置
    let target_config_path = match &args.target {
        Some(path) => {
            // If path exists as specified (absolute or relative to CWD), use it.
            // Otherwise, assume it's relative to the work_dir.
            if path.exists() {
                path.clone()
            } else {
                work_dir.join(path)
            }
        }
        None => {
            // If --target is not provided, assume a default name in work_dir.
            // Adjust "config.toml" if the default name is different.
            work_dir.join("config.toml")
        }
    };
    info!("使用目标配置: {}", target_config_path.display());
    // Check if the resolved path actually exists before trying to load it.
    if !target_config_path.exists() {
        return Err(format!("Target configuration file not found: {}", target_config_path.display()).into());
    }
    let target_config = TargetConfig::from_file(&target_config_path)?;
    
    // 创建连接管理器
    let mut connection_manager = ConnectionFactory::create_manager(&target_config)?;
    
    // 设置连接
    connection_manager.setup()?;
    
    // 准备执行选项
    let options = ExecutorOptions {
        command_timeout: args.timeout,
        retry_count: args.retry,
        retry_interval: 5, // 默认5秒重试间隔
        maintain_session: true,
        continue_on_error: args.continue_on_error,
    };
    
    // Define report directory path
    let report_dir = work_dir.join("reports");

    // 将所有操作封装在一个代码块中，确保BatchExecutor在使用结束后被释放
    {
        // 创建批量执行器
        let mut batch_executor = BatchExecutor::new(
            work_dir.clone(),
            connection_manager.as_mut(),
            target_config.clone(),
            Some(options),
        );

        // 添加模板到执行器 (模板已在过滤/加载阶段解析)
        info!("添加测试模板到执行器...");
        batch_executor.add_templates(filtered_templates)?;

        // 执行测试
        info!("开始执行测试...");
        let results = batch_executor.execute_all()?; // 现在返回拥有的值，而不是引用
        info!("测试执行完成，共执行 {} 个模板", results.len());

        // 生成报告
        batch_executor.generate_reports(&report_dir)?;
        info!("报告已生成到目录: {}", report_dir.display());

        // 统计成功/失败数量
        let mut success_count = 0;
        let mut fail_count = 0;
        
        for (_path, result) in &results { // 加上引用操作符&，因为results现在是拥有的值
            match result.overall_status {
                template::StepStatus::Pass => success_count += 1,
                template::StepStatus::Fail => fail_count += 1,
                _ => {},
            }
        }
        
        info!("测试统计: {} 成功, {} 失败", success_count, fail_count);
        
        if fail_count > 0 && !args.continue_on_error {
            // 在此处先关闭连接，再返回错误
            connection_manager.teardown()?;
            return Err(format!("{} 个测试失败", fail_count).into());
        }
    } // batch_executor在这里超出作用域并被释放，使得connection_manager的可变借用结束
    
    // 此时batch_executor已经被释放，connection_manager不再被借用，可以安全调用
    connection_manager.teardown()?;
    
    Ok(())
}
