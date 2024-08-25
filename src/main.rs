mod aggregator;
mod config;
mod markdown_report;
mod test_runner;
mod testenv_manager;
mod testscript_manager;
mod utils;

use crate::test_runner::{LocalTestRunner, RemoteTestRunner, TestRunner};
use clap::{Arg, ArgMatches, Command};
use std::fs::remove_file;
use std::path::Path;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() {
    let matches = parse_args();

    let test = matches.get_flag("test");
    let aggr = matches.get_flag("aggr");
    let summ = matches.get_flag("summ");
    let run_locally = matches.get_flag("locally");
    let cleanup = matches.get_flag("cleanup");
    let verbose = matches.get_flag("verbose");
    let config_file = matches
        .get_one::<String>("config")
        .map(|s| s.as_str())
        .unwrap_or("config.toml");
    let base_config = match config::Config::from_file(config_file) {
        Ok(base_config) => base_config,
        Err(e) => {
            eprintln!("Failed to load config from {}: {}", config_file, e);
            return;
        }
    };

    let distros: Vec<&str> = base_config.distros.iter().map(|s| &**s).collect();
    println!("Distros: {:?}", distros);
    let packages: Vec<&str> = base_config.packages.iter().map(|s| &**s).collect();
    println!("Packages: {:?}", packages);

    if test {
        println!("Running tests");
        run_tests(&distros, &packages, run_locally, cleanup, verbose);
    }

    if aggr {
        println!("Aggregating reports");
        if let Err(e) = aggregator::aggregate_reports(&distros, &packages) {
            eprintln!("Failed to aggregate reports: {}", e);
        }
    }

    if summ {
        println!("Generating summary report");
        if let Err(e) = markdown_report::generate_markdown_report(&distros, &packages) {
            eprintln!("Failed to generate markdown report: {}", e);
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
                .action(clap::ArgAction::SetTrue)
                .help("Run tests for all distributions"),
        )
        .arg(
            Arg::new("aggr")
                .long("aggr")
                .action(clap::ArgAction::SetTrue)
                .help("Aggregate multiple report.json files into a single reports.json"),
        )
        .arg(
            Arg::new("summ")
                .long("summ")
                .action(clap::ArgAction::SetTrue)
                .help("Generate a summary report"),
        )
        .arg(
            Arg::new("locally")
                .long("locally")
                .action(clap::ArgAction::SetTrue)
                .help("Run tests locally"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("Config file name")
                .help("Specify a different base configuration file"),
        )
        .arg(
            Arg::new("cleanup")
                .long("cleanup")
                .action(clap::ArgAction::SetTrue)
                .help("Clean up report.json files left by previous runs"),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Show all runtime output of test scripts in stdout"),
        )
        .get_matches()
}

fn run_tests(distros: &[&str], packages: &[&str], run_locally: bool, cleanup: bool, verbose: bool) {
    for distro in distros {
        if !Path::new(distro).exists() {
            eprintln!("Distro directory '{}' not found, skipping", distro);
            continue;
        }
        let distro_config_path = format!("{}/config.toml", distro);
        let distro_config = match config::DistroConfig::from_file(&distro_config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config for {}: {}", distro, e);
                continue;
            }
        };

        let testenv_manager = crate::testenv_manager::TestEnvManager::new(&distro_config);

        if !run_locally {
            if let Err(e) = testenv_manager.start() {
                eprintln!(
                    "Failed to initialize test environment for {}: {}",
                    distro, e
                );
                continue;
            }
        }

        for package in packages {
            if cleanup {
                let report_path = format!("{}/{}/report.json", distro, package);
                let report_file_path = Path::new(&report_path);
                if report_file_path.exists() {
                    if let Err(e) = remove_file(report_file_path) {
                        eprintln!(
                            "Failed to remove previous report file {}: {}",
                            report_path, e
                        );
                    } else {
                        println!("Removed previous report file {}", report_path);
                    }
                }
            }

            if let Some(skip_packages) = &distro_config.skip_packages {
                if skip_packages.contains(&package.to_string()) {
                    println!("Skipping test for {}/{}", distro, package);
                    continue;
                }
            }
            if !Path::new(distro).exists() {
                eprintln!("Package directory '{}' not found, skipping", package);
                continue;
            }

            println!("Running test for {}/{}, {}.", distro, package, if run_locally {"locally"} else {"with QEMU"});

            let test_runner: Box<dyn TestRunner> = if run_locally {
                Box::new(LocalTestRunner::new(distro, package, verbose))
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
                println!("Connecting to environment with credentials: IP={}, Port={}, Username={}, Password={}",ip,port,username,password.unwrap_or("None"));
                Box::new(RemoteTestRunner::new(
                    ip.to_string(),
                    port,
                    username.to_string(),
                    password.map(|p| p.to_string(),),
                    verbose,
                ))
            };

            match test_runner.run_test(&distro, &package) {
                Ok(_) => println!("Test passed for {}/{}", distro, package),
                Err(e) => println!("Test failed for {}/{}: {}", distro, package, e),
            }
        }

        if !run_locally {
            if let Err(e) = testenv_manager.stop() {
                eprintln!("Failed to stop environment for {}: {}", distro, e);
            }
        }
    }
}
