use std::fs::File;
use std::io::prelude::*;
use crate::utils::Report;
use serde_json;

pub fn generate_markdown_report(distros: &[&str], packages: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("reports.json")?;
    let reports: Vec<Report> = serde_json::from_reader(file)?;

    let mut report_matrix: Vec<Vec<Option<&Report>>> = vec![vec![None; distros.len()]; packages.len()];

    for report in &reports {
        if let (Some(pkg_idx), Some(distro_idx)) = (
            packages.iter().position(|&pkg| pkg == report.package_name),
            distros.iter().position(|&distro| distro == report.distro),
        ) {
            report_matrix[pkg_idx][distro_idx] = Some(report);
        }
    }

    let mut markdown = String::new();
    markdown.push_str("# 软件包测试结果矩阵\n\n");
    markdown.push_str("| 软件包 | 种类 | ");
    for distro in distros {
        markdown.push_str(&format!("{} | ", distro));
    }
    markdown.pop();
    markdown.push_str("\n|:------|:-----| ");
    for _ in distros {
        markdown.push_str(":-------| ");
    }
    markdown.pop();
    markdown.push_str("\n");

    for (pkg_idx, &package) in packages.iter().enumerate() {
        let package_type = reports.iter().find(|r| r.package_name == package).map_or("", |r| r.package_type.as_str());
        markdown.push_str(&format!("| {} | {} ", package, package_type));
        
        for distro_idx in 0..distros.len() {
            if let Some(report) = report_matrix[pkg_idx][distro_idx] {
                markdown.push_str(&format!("| {} {}-{} ", 
                    if report.all_tests_passed { "✅" } else { "⚠️" }, report.package_name, report.package_version));
            } else {
                markdown.push_str("| ❓ ");
            }
        }
        markdown.push_str("|\n");
    }

    let mut file = File::create("summary.md")?;
    file.write_all(markdown.as_bytes())?;
    Ok(())
}