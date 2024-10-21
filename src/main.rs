//! Entry point of whole application
mod aggregator;
mod config;
mod markdown_report;
mod test_runner;
mod testenv_manager;
mod testscript_manager;
mod utils;
use crate::config::distro_config::DistroConfig;
use crate::test_runner::{local::LocalTestRunner, remote::RemoteTestRunner, TestRunner};
use crate::utils::Report;
use clap::{Arg, ArgAction, ArgMatches, Command};
use env_logger::Env;
use log::{debug, error, info, warn};
use std::{env, fs::File, path::Path};

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
    let cwd = env::current_dir().unwrap_or(".".into()); // is "." viable?
    let working_dir = matches
        .get_one::<String>("directory")
        .map(|s| cwd.join(s))
        .unwrap_or(cwd);
    debug!("Working directory: {}", working_dir.display());

    let discovered_distros = utils::get_distros(&working_dir).unwrap_or_default();
    let distros: Vec<&str> = matches
        .get_one::<String>("distro")
        .map(|s| s.as_str().split(',').collect::<Vec<&str>>())
        .unwrap_or(discovered_distros.iter().map(|s| s.as_str()).collect());
    debug!("Distros: {:?}", distros);
    let discovered_packages = utils::get_all_packages(&distros, &working_dir).unwrap_or_default();
    let packages: Vec<&str> = matches
        .get_one::<String>("package")
        .map(|s| s.as_str().split(',').collect::<Vec<&str>>())
        .unwrap_or(discovered_packages.iter().map(|s| s.as_str()).collect());
    debug!("Packages: {:?}", packages);

    if test {
        info!("Running tests");
        run_tests(&distros, &packages, skip_successful, &working_dir);
    }

    if aggr {
        info!("Aggregating reports");
        if let Err(e) = aggregator::aggregate_reports(&distros, &packages, &working_dir) {
            error!("Failed to aggregate reports: {}", e);
        }
    }

    if summ {
        info!("Generating summary report");
        if let Err(e) = markdown_report::generate_markdown_report(&distros, &packages, &working_dir)
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
            Arg::new("distro")
                .short('d')
                .long("distro")
                .help("Specify distributions to test")
                .action(ArgAction::Set)
                .num_args(1),
        )
        .arg(
            Arg::new("package")
                .short('p')
                .long("package")
                .help("Specify packages to test")
                .action(ArgAction::Set)
                .num_args(1),
        )
        .arg(
            Arg::new("skip-successful")
                .long("skip-successful")
                .action(ArgAction::SetTrue)
                .help("Skip previous successful tests (instead of overwriting their results)"),
        )
        .get_matches()
}

/// Run tests (for all distributions by default)
/// # Arguments
/// - `distros`: Array of distribution names.
/// - `packages`: Array of package names.
/// - `skip_successful`: Skip previous successful tests (instead of overwriting their results).
///
/// # Returns
/// Returns `Ok(())` if successful, otherwise returns an error.
///
fn run_tests(distros: &[&str], packages: &[&str], skip_successful: bool, dir: &Path) {
    for distro in distros {
        let distro_directory = dir.join(distro);
        if !distro_directory.exists() {
            warn!(
                "Distro directory '{}' not found, skipping",
                distro_directory.display()
            );
            continue;
        }
        let distro_config_path = distro_directory.join("config.toml");
        let distro_config: DistroConfig = match utils::read_toml_from_file(&distro_config_path) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to load config for {}: {}", distro, e);
                continue;
            }
        };

        let run_locally = distro_config.testing_type == "locally";
        let purely_remote = distro_config.testing_type != "qemu-based-remote";
        let testenv_manager = crate::testenv_manager::TestEnvManager::new(&distro_config);

        info!(
            "Connection method: {}",
            if let Some(connection) = &distro_config.connection {
                &connection.method
            } else {
                "None"
            }
        );

        let qemu_needed = !run_locally && !purely_remote;

        if qemu_needed {
            if let Err(e) = testenv_manager.start() {
                error!(
                    "Failed to initialize test environment for {}: {}",
                    distro, e
                );
                continue;
            }
        }

        for package in packages {
            let mut skipped_scripts = Vec::new();

            let package_directory = distro_directory.join(package);
            if !package_directory.exists() {
                warn!(
                    "Package testing directory '{}' not found, skipping",
                    package_directory.display()
                );
                continue;
            }
            if skip_successful {
                let report_path = package_directory.join("report.json");
                if let Ok(file) = File::open(&report_path) {
                    let report: Result<Report, serde_json::Error> = serde_json::from_reader(file);
                    match report {
                        Ok(r) => {
                            if r.all_tests_passed {
                                info!("Skipping previous successful test {}/{}", distro, package);
                                continue;
                            } else {
                                for result in r.test_results {
                                    if result.passed {
                                        info!(
                                            "Skipping previous successful test {}/{}: {}",
                                            distro, package, result.test_name
                                        );

                                        skipped_scripts.push(result.test_name);
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            warn!(
                                "Failed to parse test report for {}/{}, test will run anyway",
                                distro, package
                            )
                        }
                    }
                } else {
                    warn!(
                        "Failed to open test report for {}/{}, test will run anyway",
                        distro, package
                    );
                }
            }

            if let Some(skip_packages) = &distro_config.skip_packages {
                if skip_packages.contains(&package.to_string()) {
                    info!("Skipping test for {}/{}", distro, package);
                    continue;
                }
            }

            info!(
                "Running test for {}/{}, {}.",
                distro,
                package,
                if run_locally {
                    "locally"
                } else if purely_remote {
                    "remotely"
                } else {
                    "with QEMU"
                }
            );

            let test_runner: Box<dyn TestRunner> = if run_locally {
                Box::new(LocalTestRunner::new(distro, package))
            } else {
                // assert!(distro_config.connection.method == "ssh");

                let _connection_config = match &distro_config.connection {
                    Some(c) => c,
                    None => {
                        error!("No connection config found for {}", distro);
                        continue;
                    }
                };
                let ip = _connection_config.ip.as_deref().unwrap_or("localhost");
                let port = _connection_config.port.unwrap_or(2222);
                let username = _connection_config.username.as_deref().unwrap_or("root");
                let password = _connection_config.password.as_deref();
                debug!("Connecting to environment with credentials: IP={}, Port={}, Username={}, Password={}",ip,port,username,password.unwrap_or("None"));
                Box::new(RemoteTestRunner::new(
                    ip.to_string(),
                    port,
                    username.to_string(),
                    password.map(|p| p.to_string()),
                ))
            };

            match test_runner.run_test(distro, package, skipped_scripts, dir) {
                Ok(_) => info!("Test passed for {}/{}", distro, package),
                Err(e) => error!("Test failed for {}/{}: {}", distro, package, e), // error or warn?
            }
        }

        if !run_locally {
            if let Err(e) = testenv_manager.stop() {
                error!("Failed to stop environment for {}: {}", distro, e);
            }
        }
    }
}
