mod scheduler;
mod aggregator;
mod markdown_report;
mod utils;
mod config;
mod qemu_manager;
mod test_runner;

use clap::{Arg, ArgMatches, Command};
use crate::test_runner::{TestRunner, LocalTestRunner, RemoteTestRunner};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() {
    let matches = parse_args();

    let test = matches.get_flag("test");
    let aggr = matches.get_flag("aggr");
    let summ = matches.get_flag("summ");
    let config_file = matches.get_one::<String>("config").map(|s| s.as_str()).unwrap_or("config.toml");
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

    if test  {
        println!("Running tests");
        run_tests(&distros, &packages, &base_config);
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
        .arg(Arg::new("test")
            .long("test")
            .action(clap::ArgAction::SetTrue)
            .help("Run tests for all distributions"))
        .arg(Arg::new("aggr")
            .long("aggr")
            .action(clap::ArgAction::SetTrue)
            .help("Aggregate multiple report.json files into a single reports.json"))
        .arg(Arg::new("summ")
            .long("summ")
            .action(clap::ArgAction::SetTrue)
            .help("Generate a summary report"))
        .arg(Arg::new("locally")
            .long("locally")
            .action(clap::ArgAction::SetTrue)
            .help("Run tests locally"))
        .arg(Arg::new("config")
            .long("config")
            .value_name("Config file name")
            .help("Specify a different base configuration file"))
        .get_matches()
}

fn run_tests(distros: &[&str], packages: &[&str], base_config: &config::Config) {
    for distro in distros {
        let distro_config_path = format!("{}/config.toml", distro);
        let distro_config = match config::DistroConfig::from_file(&distro_config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config for {}: {}", distro, e);
                continue;
            }
        };

        let qemu_manager = crate::qemu_manager::QemuManager::new(&distro_config);

        if let Err(e) = qemu_manager.start() {
            eprintln!("Failed to start QEMU for {}: {}", distro, e);
            continue;
        }

        for package in packages {
            if let Some(skip_packages) = &distro_config.skip_packages {
                if skip_packages.contains(&package.to_string()) {
                    println!("Skipping test for {}/{}", distro, package);
                    continue;
                }
            }

            assert!(distro_config.connection.method == "ssh");

            let ip = distro_config.connection.ip.as_deref().unwrap_or("localhost");
            let port = distro_config.connection.port.unwrap_or(2222);
            let username = distro_config.connection.username.as_deref().unwrap_or("root");
            let password = distro_config.connection.password.as_deref();
            
            println!("Connecting to QEMU with credentials: IP={}, Port={}, Username={}, Password={}", ip, port, username, password.unwrap_or("None"));
            println!("Running test for {}/{}", distro, package);

            let test_runner: Box<dyn TestRunner> = if base_config.run_locally {
                Box::new(LocalTestRunner::new(distro, package))
            } else {
                Box::new(RemoteTestRunner::new(
                    ip.to_string(),
                    port,
                    username.to_string(),
                    password.map(|p| p.to_string()),
                ))
            };
    
            match test_runner.run_test(&distro, &package) {
                Ok(_) => println!("Test passed for {}/{}", distro, package),
                Err(e) => println!("Test failed for {}/{}: {}", distro, package, e),
            }
        }

        if let Err(e) = qemu_manager.stop() {
            eprintln!("Failed to stop QEMU for {}: {}", distro, e);
        }
    }
}