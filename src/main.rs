mod scheduler;
mod aggregator;
mod markdown_report;
mod utils;
mod config;

fn main() {
    let base_config = match config::Config::from_file("config.toml") {
        Ok(base_config) => base_config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

    let distros: Vec<&str> = base_config.distros.iter().map(|s| &**s).collect();
    println!("Distros: {:?}", distros);
    let packages: Vec<&str> = base_config.packages.iter().map(|s| &**s).collect();
    println!("Packages: {:?}", packages);

    for distro in &distros {
        let distro_config_path = format!("{}/config.toml", distro);
        let distro_config = match config::DistroConfig::from_file(&distro_config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config for {}: {}", distro, e);
                continue;
            }
        };

        for package in &packages {
            assert!(distro_config.connection.method == "ssh"); // Only SSH is supported now

            let ip = distro_config.connection.ip.as_deref().unwrap_or("localhost");
            let port = distro_config.connection.port.unwrap_or(2222);
            let username = distro_config.connection.username.as_deref().unwrap_or("root");
            let password = distro_config.connection.password.as_deref();
            
            println!("Connecting to QEMU with credentials: IP={}, Port={}, Username={}, Password={}", ip, port, username, password.unwrap_or("None"));
            println!("Running test for {}/{}", distro, package);

            match scheduler::run_test(ip, port, username, password, distro, package) {
                Ok(_) => println!("Test passed for {}/{}", distro, package),
                Err(e) => println!("Test failed for {}/{}: {}", distro, package, e),
            }
        }
    }

    if let Err(e) = aggregator::aggregate_reports(&distros, &packages) {
        eprintln!("Failed to aggregate reports: {}", e);
    }

    if let Err(e) = markdown_report::generate_markdown_report(&distros, &packages) {
        eprintln!("Failed to generate markdown report: {}", e);
    }
}
