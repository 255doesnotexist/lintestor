mod aggregator;
mod config;
mod markdown_report;
mod test_runner;
mod testenv_manager;
mod testscript_manager;
mod utils;

use crate::config::{distro_config::DistroConfig, root_config::Config};
use crate::test_runner::{local::LocalTestRunner, remote::RemoteTestRunner, TestRunner};
use clap::{Arg, ArgAction, ArgMatches, Command};
use log::{debug, error, info, warn};
use std::{env, fs::File, path::Path};
use utils::Report;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init();
    let matches = parse_args();

    let test = matches.get_flag("test");
    let aggr = matches.get_flag("aggr");
    let summ = matches.get_flag("summ");
    let skip_successful = matches.get_flag("skip-successful");
    let config_file = matches
        .get_one::<String>("config")
        .map(|s| s.as_str())
        .unwrap_or("config.toml");
    let base_config: Config = match utils::read_toml_from_file(config_file) {
        Ok(base_config) => base_config,
        Err(e) => {
            error!("Failed to load config from {}: {}", config_file, e);
            return;
        }
    };

    let distros: Vec<&str> = matches
        .get_one::<String>("distro")
        .map(|s| s.as_str().split(',').collect::<Vec<&str>>())
        .unwrap_or(base_config.distros.iter().map(|s| &**s).collect());
    debug!("Distros: {:?}", distros);
    let packages: Vec<&str> = matches
        .get_one::<String>("package")
        .map(|s| s.as_str().split(',').collect::<Vec<&str>>())
        .unwrap_or(base_config.packages.iter().map(|s| &**s).collect());
    debug!("Packages: {:?}", packages);

    if test {
        info!("Running tests");
        run_tests(&distros, &packages, skip_successful);
    }

    if aggr {
        info!("Aggregating reports");
        if let Err(e) = aggregator::aggregate_reports(&distros, &packages) {
            error!("Failed to aggregate reports: {}", e);
        }
    }

    if summ {
        info!("Generating summary report");
        if let Err(e) = markdown_report::generate_markdown_report(&distros, &packages) {
            error!("Failed to generate markdown report: {}", e);
        }
    }
}

fn parse_args() -> ArgMatches {
    Command::new(NAME)
        .version(VERSION)
        .author(AUTHORS)
        .about(DESCRIPTION)
        .arg(
            Arg::new("test")
                .long("test")
                .action(ArgAction::SetTrue)
                .help("Run tests for all distributions"),
        )
        .arg(
            Arg::new("aggr")
                .long("aggr")
                .action(ArgAction::SetTrue)
                .help("Aggregate multiple report.json files into a single reports.json"),
        )
        .arg(
            Arg::new("summ")
                .long("summ")
                .action(ArgAction::SetTrue)
                .help("Generate a summary report"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("Config file name")
                .help("Specify a different base configuration file"),
        )
        .arg(
            Arg::new("distro")
                .long("distro")
                .help("Specify distros to test")
                .action(ArgAction::Set)
                .num_args(1),
        )
        .arg(
            Arg::new("package")
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

fn run_tests(distros: &[&str], packages: &[&str], skip_successful: bool) {
    for distro in distros {
        if !Path::new(distro).exists() {
            warn!("Distro directory '{}' not found, skipping", distro);
            continue;
        }
        let distro_config_path = format!("{}/config.toml", distro);
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
            if run_locally {
                "local"
            } else {
                &distro_config.connection.method
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
            if skip_successful {
                let report_path = format!("{}/{}/report.json", distro, package);
                if let Ok(file) = File::open(&report_path) {
                    let report: Result<Report, serde_json::Error> = serde_json::from_reader(file);
                    // TODO: only select failed *test scripts* in a package
                    match report {
                        Ok(r) => {
                            if r.all_tests_passed {
                                info!("Skipping previous successful test {}/{}", distro, package);
                                continue;
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
            if !Path::new(distro).exists() {
                warn!("Package directory '{}' not found, skipping", package);
                continue;
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

            if !Path::new(format!("{}/{}", distro, package).as_str()).exists() {
                error!(
                    "Package testing directory '{}/{}' does not exist, skipping",
                    distro, package
                );
                continue;
            }

            let test_runner: Box<dyn TestRunner> = if run_locally {
                Box::new(LocalTestRunner::new(distro, package))
            } else {
                // assert!(distro_config.connection.method == "ssh");

                let ip = distro_config
                    .connection
                    .ip
                    .as_deref()
                    .unwrap_or("localhost");
                let port = distro_config.connection.port.unwrap_or(2222);
                let username = distro_config
                    .connection
                    .username
                    .as_deref()
                    .unwrap_or("root");
                let password = distro_config.connection.password.as_deref();
                debug!("Connecting to environment with credentials: IP={}, Port={}, Username={}, Password={}",ip,port,username,password.unwrap_or("None"));
                Box::new(RemoteTestRunner::new(
                    ip.to_string(),
                    port,
                    username.to_string(),
                    password.map(|p| p.to_string()),
                ))
            };

            match test_runner.run_test(distro, package) {
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
