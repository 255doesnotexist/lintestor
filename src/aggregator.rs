use crate::utils::Report;
use serde_json;
use std::fs::File;
use std::io::prelude::*;

pub fn aggregate_reports(
    distros: &[&str],
    packages: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut consolidated_report = vec![];

    for &distro in distros {
        for &package in packages {
            let report_path = format!("{}/{}/report.json", distro, package);
            println!("Aggregating {}", report_path);
            if let Ok(file) = File::open(&report_path) {
                let mut report: Report = serde_json::from_reader(file)?;
                report.distro = distro.to_string();
                consolidated_report.push(report);
            }
        }
    }

    let consolidated_json = serde_json::to_string_pretty(&consolidated_report)?;
    let mut file = File::create("reports.json")?;
    file.write_all(consolidated_json.as_bytes())?;
    Ok(())
}
