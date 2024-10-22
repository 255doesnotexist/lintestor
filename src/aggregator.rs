//! Aggregates multiple test reports into a single report.
use crate::utils::{get_packages, Report};
use log::{error, info};
use std::{
    fs::File,
    io::{prelude::*, BufWriter},
    path::Path,
};

/// Generates a report and writes it to the specified file path.
///
/// # Parameters
///
/// - `file_path`: The path to the report file.
/// - `report`: The report to be generated.
///
/// # Returns
///
/// Returns `Ok(())` if successful, otherwise returns an error.
///
/// # Errors
///
/// Returns an error if file creation or writing fails.
pub fn generate_report(file_path: &Path, report: Report) -> Result<(), Box<dyn std::error::Error>> {
    let report_file = File::create(file_path)?;
    let mut writer = BufWriter::new(report_file);
    serde_json::to_writer(&mut writer, &report)?;
    writer.flush()?;
    Ok(())
}

/// Aggregates reports from multiple distributions and packages, and generates a consolidated report file.
///
/// # Parameters
///
/// - `distros`: Array of distribution names.
/// - `packages`: Array of package names.
/// - `dir`: The path of the program's working directory.
///
/// # Returns
///
/// Returns `Ok(())` if successful, otherwise returns an error.
///
/// # Errors
///
/// Returns an error if file opening or reading fails.
pub fn aggregate_reports(
    distros: &[&str],
    packages: &[&str],
    dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut consolidated_report = vec![];

    for &distro in distros {
        let packages_of_distro = get_packages(distro, dir).unwrap_or_default();

        for &package in packages
            .iter()
            .filter(|p| packages_of_distro.contains(&String::from(**p)))
        {
            let report_path = dir.join(format!("{}/{}/report.json", distro, package));
            if let Ok(file) = File::open(&report_path) {
                info!("Aggregating {}", report_path.display());
                let mut report: Report = serde_json::from_reader(file)?;
                report.distro = distro.to_string();
                consolidated_report.push(report);
            } else {
                error!(
                    "Failed to open file {} for aggregation",
                    report_path.display()
                )
            }
        }
    }

    let consolidated_json = serde_json::to_string_pretty(&consolidated_report)?;
    let file_path = dir.join("reports.json");
    let mut file = File::create(&file_path)?;
    file.write_all(consolidated_json.as_bytes())?;
    info!("Aggregated report generated at {}", file_path.display());
    Ok(())
}
