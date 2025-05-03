//! Entry point of whole application
mod aggregator;
mod config;
mod markdown_report;
mod test_runner;
mod testenv_manager;
mod testscript_manager;
mod utils;
use crate::config::target_config::TargetConfig;
use crate::test_runner::{local::LocalTestRunner, remote::RemoteTestRunner, TestRunner};
use crate::utils::Report;
use clap::{Arg, ArgAction, ArgMatches, Command};
use env_logger::Env;
use log::{debug, error, info, warn};
use std::{env, fs::File, path::Path};
use test_runner::boardtest::BoardtestRunner;

#[macro_use]
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

    if test {
        info!("Running tests");
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
                .help("Run tests (for all distributions by default)"),
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
                .help("Specify distributions to test")
                .action(ArgAction::Set)
                .num_args(1),
        )
        .arg(
            Arg::new("unit")
                .short('p')
                .long("unit")
                .help("Specify units to test")
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
        let purely_remote = target_config.testing_type != "qemu-based-remote";
        let testenv_manager = crate::testenv_manager::TestEnvManager::new(&target_config, dir);

        info!(
            "Connection method: {}",
            if let Some(connection) = &target_config.connection {
                &connection.method
            } else {
                "None (Locally)"
            }
        );

        let qemu_needed = !run_locally && !purely_remote;

        if qemu_needed {
            if let Err(e) = testenv_manager.start() {
                error!(
                    "Failed to initialize test environment for {}: {}",
                    target, e
                );
                continue;
            }
        }

        let units_of_target = utils::get_units(target, dir).unwrap_or_default();
        for unit in units
            .iter()
            .filter(|p| units_of_target.iter().any(|pkg| p == &pkg))
        {
            let mut skipped_scripts = Vec::new();

            let unit_directory = target_directory.join(unit);
            if !unit_directory.exists() {
                warn!(
                    "Package testing directory '{}' not found, skipping",
                    unit_directory.display()
                );
                continue;
            }
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

                                        skipped_scripts.push(result.test_name);
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

            if let Some(skip_units) = &target_config.skip_units {
                if skip_units.iter().any(|pkg| pkg == unit) {
                    info!("Skipping test for {}/{}", target, unit);
                    continue;
                }
            }

            info!(
                "Running test for {}/{}, {}.",
                target,
                unit,
                if run_locally {
                    "locally"
                } else if purely_remote {
                    "remotely"
                } else if via_boardtest {
                    "via Boardtest Server"
                } else {
                    "with QEMU"
                }
            );

            // TODO: refactor to matching-case and runner_manager
            let test_runner: Box<dyn TestRunner> = if run_locally {
                Box::new(LocalTestRunner::new(target, unit))
            } else if via_boardtest {
                if let Some(ref boardtest_config) = target_config.boardtest {
                    Box::new(BoardtestRunner::new(boardtest_config))
                } else {
                    error!("No boardtest config found for {}", target);
                    continue;
                }
            } else {
                // assert!(target_config.connection.method == "ssh");

                let _connection_config = match &target_config.connection {
                    Some(c) => c,
                    None => {
                        error!("No connection config found for {}", target);
                        continue;
                    }
                };
                let ip = _connection_config.ip.as_deref().unwrap_or("localhost");
                let port = _connection_config.port.unwrap_or(2222);
                let username = _connection_config.username.as_deref().unwrap_or("root");
                let password = _connection_config.password.as_deref();
                let private_key_path = _connection_config.private_key_path.as_deref();
                debug!("Connecting to environment with credentials: IP={}, Port={}, Username={}, Password={}",ip,port,username,password.unwrap_or("None"));
                Box::new(RemoteTestRunner::new(
                    ip.to_string(),
                    port,
                    username.to_string(),
                    password.map(|p| p.to_string()),
                    private_key_path.map(|p| p.to_string()),
                ))
            };

            match test_runner.run_test(target, unit, skipped_scripts, dir) {
                Ok(_) => info!("Test passed for {}/{}", target, unit),
                Err(e) => {
                    error!("Test failed for {}/{}: {}", target, unit, e); // error or warn?
                    if *allow_interactive_prompts {
                        use dialoguer::Confirm;
                        let resume = Confirm::new()
                            .with_prompt(format!("An previous test was failed for {}/{}. Do you want to continue the test?", target, unit))
                            .default(true)
                            .interact()
                            .unwrap();
                        if !resume {
                            info!("Skipping the test for {}/{}", target, unit);
                            break;
                        }
                    }
                }
            }
        }

        if !run_locally {
            if let Err(e) = testenv_manager.stop() {
                error!("Failed to stop environment for {}: {}", target, e);
            }
        }
    }
}
