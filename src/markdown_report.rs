// File: markdown_report.rs
// Description: Markdown报告生成模块，负责生成Markdown格式的测试结果总结。

use std::fs::File;
use std::io::prelude::*;
use crate::utils::Report;
use toml;

pub fn generate_markdown_report() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("reports.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let reports: Vec<Report> = toml::from_str(&contents)?;

    let mut markdown = String::new();
    markdown.push_str("# 测试结果总结\n\n");
    for report in reports {
        markdown.push_str(&format!(
            "## {} ({})\n\n",
            report.package_version, report.os_version
        ));
        markdown.push_str("| 测试项 | 结果 |\n|---|---|\n");
        for result in &report.test_results {
            markdown.push_str(&format!(
                "| {} | {} |\n",
                result.test_name,
                if result.passed { "通过" } else { "失败" }
            ));
        }
        markdown.push_str("\n");
    }

    let mut file = File::create("summary.md")?;
    file.write_all(markdown.as_bytes())?;
    Ok(())
}
