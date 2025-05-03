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
use crate::template::{TemplateFilter, discover_templates, filter_templates, TemplateExecutor, Reporter};
use crate::connection::ConnectionFactory;
use crate::test_runner::{local::LocalTestRunner, remote::RemoteTestRunner, qemu::QemuTestRunner, TestRunner};
use crate::utils::Report;
use clap::{Arg, ArgAction, ArgMatches, Command};
use env_logger::Env;
use log::{debug, error, info, warn};
use std::{env, fs::File, path::Path};
use test_runner::boardtest::BoardtestRunner;

extern crate anyhow;

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
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let matches = parse_args();

    let test = matches.get_flag("test");
    let test_template = matches.get_flag("test-template");
    let aggr = matches.get_flag("aggr");
    let summ = matches.get_flag("summ");
    let skip_successful = matches.get_flag("skip-successful");
    let allow_interactive_prompts = matches.get_flag("interactive");
    let cwd = env::current_dir().unwrap_or(".".into()); // is "." viable?
    let working_dir = matches
        .get_one::<String>("directory")
        .map(|s| cwd.join(s))
        .unwrap_or(cwd);
    debug!("Working directory: {}", working_dir.display());

    let discovered_targets = utils::get_targets(&working_dir).unwrap_or_default();
    let targets: Vec<&str> = matches
        .get_one::<String>("target")
        .map(|s| s.as_str().split(',').collect::<Vec<&str>>())
        .unwrap_or(discovered_targets.iter().map(|s| s.as_str()).collect());
    info!("targets: {:?}", targets);
    let discovered_units = utils::get_all_units(&targets, &working_dir).unwrap_or_default();
    let units: Vec<&str> = matches
        .get_one::<String>("unit")
        .map(|s| s.as_str().split(',').collect::<Vec<&str>>())
        .unwrap_or(discovered_units.iter().map(|s| s.as_str()).collect());
    info!("Packages: {:?}", units);
    
    if test_template {
        info!("Running tests using Markdown templates");
        
        // 获取标签过滤条件
        let tags: Vec<String> = matches
            .get_one::<String>("tag")
            .map(|s| s.split(',').map(|tag| tag.trim().to_string()).collect())
            .unwrap_or_default();
            
        run_template_tests(
            targets.iter().map(|&s| s.to_string()).collect(),
            units.iter().map(|&s| s.to_string()).collect(),
            tags,
            &working_dir,
        );
    } else if test {
        info!("Running tests with legacy .sh scripts");
        run_tests(
            &targets,
            &units,
            skip_successful,
            &working_dir,
            &allow_interactive_prompts,
        );
    }

    if aggr {
        info!("Aggregating reports");
        if let Err(e) = aggregator::aggregate_reports(&targets, &units, &working_dir) {
            error!("Failed to aggregate reports: {}", e);
        }
    }

    if summ {
        info!("Generating summary report");
        if let Err(e) = markdown_report::generate_markdown_report(&targets, &units, &working_dir)
        {
            error!("Failed to generate markdown report: {}", e);
        }
    }
}

/// Parses command line arguments.
/// Returns the parsed `ArgMatches` object.
fn parse_args() -> ArgMatches {
    Command::new(NAME)
        .version(VERSION)
        .author(AUTHORS)
        .about(DESCRIPTION)
        .arg(
            Arg::new("test")
                .short('t')
                .long("test")
                .action(ArgAction::SetTrue)
                .conflicts_with("test-template")
                .help("Run tests using legacy .sh scripts (for all distributions by default)"),
        )
        .arg(
            Arg::new("test-template")
                .long("test-template")
                .action(ArgAction::SetTrue)
                .conflicts_with("test")
                .help("Run tests using Markdown templates (.test.md files)"),
        )
        .arg(
            Arg::new("aggr")
                .short('a')
                .long("aggr")
                .action(ArgAction::SetTrue)
                .help("Aggregate multiple report.json files into a single reports.json"),
        )
        .arg(
            Arg::new("summ")
                .short('s')
                .long("summ")
                .action(ArgAction::SetTrue)
                .help("Generate a summary report"),
        )
        .arg(
            Arg::new("directory")
                .short('D')
                .long("directory")
                .value_name("working_directory")
                .help("Specify working directory with preconfigured test files"),
        )
        .arg(
            Arg::new("target")
                .short('d')
                .long("target")
                .help("Specify targets to test (comma separated)")
                .action(ArgAction::Set)
                .num_args(1),
        )
        .arg(
            Arg::new("unit")
                .short('u')
                .long("unit")
                .help("Specify units to test (comma separated)")
                .action(ArgAction::Set)
                .num_args(1),
        )
        .arg(
            Arg::new("tag")
                .long("tag")
                .help("Filter templates by tags (comma separated)")
                .action(ArgAction::Set)
                .num_args(1),
        )
        .arg(
            Arg::new("skip-successful")
                .long("skip-successful")
                .action(ArgAction::SetTrue)
                .help("Skip previous successful tests (instead of overwriting their results)"),
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .action(ArgAction::SetTrue)
                .help("Run lintestor in interactive mode. Possibly require user input which may pause the test."),
        )
        .get_matches()
}

/// Run tests using Markdown templates
/// 
/// # Arguments
/// 
/// * `targets` - Target names to filter templates by
/// * `units` - Unit names to filter templates by
/// * `tags` - Tags to filter templates by
/// * `working_dir` - Working directory containing templates and target configs
fn run_template_tests(
    targets: Vec<String>,
    units: Vec<String>,
    tags: Vec<String>,
    working_dir: &Path,
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
