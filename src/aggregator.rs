use crate::utils::Report;
use log::{error, info};
use std::fs::File;
use std::io::{prelude::*, BufWriter};

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
pub fn generate_report(
    file_path: String,
    report: Report,
) -> Result<(), Box<dyn std::error::Error>> {
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
) -> Result<(), Box<dyn std::error::Error>> {
    let mut consolidated_report = vec![];

    for &distro in distros {
        for &package in packages {
            let report_path = format!("{}/{}/report.json", distro, package);
            if let Ok(file) = File::open(&report_path) {
                info!("Aggregating {}", report_path);
                let mut report: Report = serde_json::from_reader(file)?;
                report.distro = distro.to_string();
                consolidated_report.push(report);
            } else {
                error!("Failed to open file {} for aggregation", report_path)
            }
        }
    }

    let consolidated_json = serde_json::to_string_pretty(&consolidated_report)?;
    let file_path = "reports.json";
    let mut file = File::create(file_path)?;
    file.write_all(consolidated_json.as_bytes())?;
    info!("Aggregated report generated at {}", file_path);
    Ok(())
}
