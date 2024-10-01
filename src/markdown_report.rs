use crate::utils::{PackageMetadata, Report};
use log::info;
use std::collections::BTreeMap;
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
    markdown.push_str("> 图标说明 Legend: ✅ = 通过 Passed; ⚠️ = 部分测试不通过 Not all tests passed; ❌ = 全部测试不通过 All tests failed; ❓ = 未知 Unknown\n\n");
    markdown.push_str("| 软件包 Package | 种类 Type | "); // TODO: add field for description
    for distro in distros {
        markdown.push_str(&format!("[{}](#{}) | ", distro, distro));
    }
    markdown.pop();
    markdown.push_str("\n|:------|:------| ");
    for _ in distros {
        markdown.push_str(":-------| ");
        // markdown.push_str(":-------| ");
    }
    markdown.pop();
    markdown.push('\n');

    // map: distro -> (package, env_info)
    let mut distro_env_infos: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

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

        for (distro_idx, &_distro) in distros.iter().enumerate() {
            if let Some(report) = report_matrix[pkg_idx][distro_idx] {
                distro_env_infos
                    .entry(distros[distro_idx].to_string())
                    .or_default()
                    .push((packages[pkg_idx].to_string(), report.os_version.clone()));
                markdown.push_str(&format!(
                    "| {} {}{} ",
                    if report.all_tests_passed {
                        "✅"
                    } else if report.test_results.iter().any(|r| r.passed) {
                        "⚠️"
                    } else {
                        "❌"
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

    let mut appending_details = String::new();
    appending_details.push_str("# 测试环境信息 Environment info\n\n");
    for (distro, packages) in &distro_env_infos {
        appending_details.push_str(&format!("## <span id=\"{}\">{}</span>\n\n", distro, distro));

        for (package, env_info) in packages {
            let package_id = format!("{}_{}", distro, package); // 创建唯一的 id
            appending_details.push_str(&format!(
                "- <span id=\"{}\">**{}**: {}</span>\n\n",
                package_id, package, env_info
            ));
        }
    }

    markdown.push_str(&appending_details);

    let file_path = "summary.md";
    let mut file = File::create(file_path)?;
    file.write_all(markdown.as_bytes())?;
    info!("Markdown report generated at {}", file_path);
    Ok(())
}
