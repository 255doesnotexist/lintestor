use crate::utils::{PackageMetadata, Report};
use log::info;
use std::fs::File;
use std::io::prelude::*;

/// Generates a markdown report summarizing the test results for various packages across different distributions.
/// Warning: hard coded for specific report markdown file XD
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
/// Returns an error if file opening, reading, or writing fails.
pub fn generate_markdown_report(
    distros: &[&str],
    packages: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("reports.json")?;
    let reports: Vec<Report> = serde_json::from_reader(file)?;

    let mut report_matrix: Vec<Vec<Option<&Report>>> =
        vec![vec![None; distros.len()]; packages.len()];

    for report in &reports {
        if let (Some(pkg_idx), Some(distro_idx)) = (
            packages.iter().position(|&pkg| pkg == report.package_name),
            distros.iter().position(|&distro| distro == report.distro),
        ) {
            report_matrix[pkg_idx][distro_idx] = Some(report);
        }
    }

    let mut markdown = String::new();
    markdown.push_str("# 软件包测试结果矩阵 Software package test results\n\n");
    markdown.push_str("> 图标说明 Legend: ✅ = 通过 Passed; ⚠️ = 部分或全部测试不通过 Not all tests passed; ❓ = 未知 Unknown\n\n");
    markdown.push_str("| 软件包 Package | 种类 Type | "); // TODO: add field for description
    for distro in distros {
        markdown.push_str(&format!(
            "{}: 测试环境信息 Env. info | {}: 测试结果 Results | ",
            distro, distro
        ));
    }
    markdown.pop();
    markdown.push_str("\n|:------|:------| ");
    for _ in distros {
        markdown.push_str(":-------| ");
        markdown.push_str(":-------| ");
    }
    markdown.pop();
    markdown.push('\n');

    for (pkg_idx, &package) in packages.iter().enumerate() {
        let package_metadata = reports.iter().find(|r| r.package_name == package).map_or(
            PackageMetadata {
                ..Default::default()
            },
            |r| r.package_metadata.clone(),
        ); // is clone really needed...?

        markdown.push_str(&format!(
            "| {} | {} ",
            package_metadata.package_pretty_name, package_metadata.package_type
        ));

        for distro_idx in 0..distros.len() {
            if let Some(report) = report_matrix[pkg_idx][distro_idx] {
                let mut env_info = report.os_version.clone();
                env_info.pop();
                markdown.push_str(&format!(
                    "| {} | {} {}{} ",
                    env_info,
                    if report.all_tests_passed {
                        "✅"
                    } else {
                        "⚠️"
                    },
                    if !package_metadata.package_version.is_empty() {
                        format!("{}=", report.package_name)
                    } else {
                        String::from("")
                    },
                    package_metadata.package_version
                ));
            } else {
                markdown.push_str("| ❓ ");
            }
        }
        markdown.push_str("|\n");
    }
    let file_path = "summary.md";
    let mut file = File::create(file_path)?;
    file.write_all(markdown.as_bytes())?;
    info!("Markdown report generated at {}", file_path);
    Ok(())
}
