mod scheduler;
mod aggregator;
mod markdown_report;
mod utils;
mod config;

fn main() {
    let config = match config::Config::from_file("config.toml") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

    let distros: Vec<&str> = config.distros.iter().map(|s| &**s).collect();
    println!("Distros: {:?}", distros);
    let packages: Vec<&str> = config.packages.iter().map(|s| &**s).collect();
    println!("Packages: {:?}", packages);

    for distro in &distros {
        for package in &packages {
            match scheduler::run_test("localhost", 2222, "root", Some("root"), distro, package) {
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
