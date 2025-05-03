//! Aggregates multiple test reports into a single report.
use crate::utils::{get_units, Report};
use log::{error, info, warn};
use std::{
    fs::File,
    io::{prelude::*, BufWriter},
    path::{Path, PathBuf},
};
use anyhow::Result;

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

/// Aggregates reports from multiple distributions and units, and generates a consolidated report file.
///
/// # Parameters
///
/// - `targets`: Array of distribution names.
/// - `units`: Array of unit names.
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
    targets: &[&str],
    units: &[&str],
    dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut consolidated_report = vec![];

    for &target in targets {
        let units_of_target = get_units(target, dir).unwrap_or_default();

        for &unit in units
            .iter()
            .filter(|p| units_of_target.iter().any(|pkg| p == &pkg))
        {
            let report_path = dir.join(format!("{}/{}/report.json", target, unit));
            if let Ok(file) = File::open(&report_path) {
                info!("Aggregating {}", report_path.display());
                let mut report: Report = serde_json::from_reader(file)?;
                report.target = target.to_string();
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

/// 从目录中聚合报告
/// 
/// # Arguments
/// 
/// * `reports_dir` - 包含报告文件的目录路径（可选）
/// * `output_path` - 聚合结果输出路径（可选）
/// 
/// # Returns
/// 
/// 如果成功，返回 `Ok(())`
pub fn aggregate_reports_from_dir(reports_dir: Option<&Path>, output_path: Option<&Path>) -> Result<()> {
    // 使用默认值或提供的参数
    let reports_dir = reports_dir.unwrap_or_else(|| Path::new("reports"));
    let output_path = output_path.unwrap_or_else(|| Path::new("reports.json"));
    
    info!("Aggregating reports from directory: {}", reports_dir.display());
    info!("Output will be written to: {}", output_path.display());
    
    // TODO: 实现实际的聚合逻辑
    // 1. 扫描reports_dir中的所有.report.md文件
    // 2. 从每个文件中提取元数据和结果
    // 3. 合并为一个JSON结构
    // 4. 写入output_path
    
    // 占位实现，确保编译通过
    warn!("Report aggregation is not fully implemented yet");
    Ok(())
}
