//! Generates markdown reports summarizing test results for various units across different distributions.
use crate::utils::{PackageMetadata, Report};
use chrono::{Local, Utc};
use log::{info, warn};
use std::{collections::BTreeMap, fs::File, io::prelude::*, path::{Path, PathBuf}};
use anyhow::Result;

/// Generates a markdown report summarizing the test results for various units across different distributions.
/// Warning: hard coded for specific report markdown file XD
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
/// Returns an error if file opening, reading, or writing fails.
pub fn generate_markdown_report(
    targets: &[&str],
    units: &[&str],
    dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let report_path = dir.join("reports.json");
    if let Ok(file) = File::open(&report_path) {
        let reports: Vec<Report> = serde_json::from_reader(file)?;

        let mut report_matrix: Vec<Vec<Option<&Report>>> =
            vec![vec![None; targets.len()]; units.len()];

        for report in &reports {
            if let (Some(pkg_idx), Some(target_idx)) = (
                units.iter().position(|&pkg| pkg == report.unit_name),
                targets.iter().position(|&target| target == report.target),
            ) {
                report_matrix[pkg_idx][target_idx] = Some(report);
            }
        }

        let mut report_matrix_by_target_name_and_unit_name: BTreeMap<
            String,
            BTreeMap<String, &Report>,
        > = BTreeMap::new();
        for report in &reports {
            if let (Some(pkg_idx), Some(target_idx)) = (
                units.iter().position(|&pkg| pkg == report.unit_name),
                targets.iter().position(|&target| target == report.target),
            ) {
                report_matrix[pkg_idx][target_idx] = Some(report);
                report_matrix_by_target_name_and_unit_name
                    .entry(report.target.clone())
                    .or_default()
                    .insert(report.unit_name.clone(), report);
            }
        }

        let mut markdown = String::new();
        markdown.push_str("# 软件包测试结果矩阵 Software unit test results\n\n");
        /// 测试时间的标准格式：YYYY-MM-DD HH:mm:ss
        const TEST_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

        let utc_time = Utc::now();
        markdown.push_str(&format!(
            "测试时间 Testing time: {} UTC\n\n",
            utc_time.format(TEST_TIME_FORMAT)
        ));

        markdown.push_str("> 图标说明 Legend: ✅ = 通过 Passed; ⚠️ = 部分测试不通过 Not all tests passed; ❌ = 全部测试不通过 All tests failed; ❓ = 未知 Unknown\n\n");
        markdown.push_str("| 软件包 Package | 种类 Type | "); // TODO: add field for detemplateion
        for target in targets {
            markdown.push_str(&format!("[{}](#{}) | ", target, target));
        }
        markdown.pop();
        markdown.push_str("\n|:------|:------| ");
        for _ in targets {
            markdown.push_str(":-------| ");
            // markdown.push_str(":-------| ");
        }
        markdown.pop();
        markdown.push('\n');

        // map: target -> (unit, env_info)
        let mut target_env_infos: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

        for (pkg_idx, &unit) in units.iter().enumerate() {
            let unit_metadata = reports.iter().find(|r| r.unit_name == unit).map_or(
                PackageMetadata {
                    ..Default::default()
                },
                |r| r.unit_metadata.clone(),
            ); // is clone really needed...?

            markdown.push_str(&format!(
                "| {} | {} ",
                unit_metadata.unit_pretty_name, unit_metadata.unit_type
            ));

            for (target_idx, &_target) in targets.iter().enumerate() {
                if let Some(report) = report_matrix[pkg_idx][target_idx] {
                    target_env_infos
                        .entry(targets[target_idx].to_string())
                        .or_default()
                        .push((units[pkg_idx].to_string(), report.os_version.clone()));
                    markdown.push_str(&format!(
                        "| {} [{}{}]({}) ",
                        if report.all_tests_passed {
                            "✅"
                        } else if report.test_results.iter().any(|r| r.passed) {
                            "⚠️"
                        } else {
                            "❌"
                        },
                        if !unit_metadata.unit_version.is_empty() {
                            format!("{}=", report.unit_name)
                        } else {
                            String::from("")
                        },
                        unit_metadata.unit_version,
                        format!("#{}_{}", targets[target_idx], units[pkg_idx])
                    ));
                } else {
                    markdown.push_str("| ❓ ");
                }
            }
            markdown.push_str("|\n");
        }

        let mut appending_details = String::new();
        appending_details.push_str("\n# 测试环境信息 Environment info\n\n");
        for (target, units) in &target_env_infos {
            appending_details
                .push_str(&format!("## <span id=\"{}\">{}</span>\n\n", target, target));

            for (unit, env_info) in units {
                let unit_id = format!("{}_{}", target, unit); // 创建唯一的 id
                appending_details.push_str(&format!(
                    "- <span id=\"{}\">**{}**: {}</span>\n\n",
                    unit_id, unit, env_info
                ));

                // check if all tests passed, or else append the test details
                if let Some(report) = report_matrix_by_target_name_and_unit_name
                    .get(target)
                    .and_then(|units_map| units_map.get(unit))
                {
                    if !report.all_tests_passed {
                        appending_details
                            .push_str(&format!("  - {} 未通过的测试 Unpassed tests\n\n", unit));
                        for test_result in &report.test_results {
                            appending_details.push_str(&format!(
                                "  - {}\n\n```shell\n{}\n```\n\n",
                                test_result.test_name, test_result.output
                            ));
                        }
                    }
                }
            }
        }

        markdown.push_str(&appending_details);

        let file_path = dir.join("summary.md");
        let mut file = File::create(&file_path)?;
        file.write_all(markdown.as_bytes())?;
        info!("Markdown report generated at {}", file_path.display());
        Ok(())
    } else {
        Err(format!(
            "Failed to open aggregated report file {}",
            report_path.display()
        )
        .into())
    }
}

/// 从聚合的reports.json文件生成Markdown格式的摘要报告
/// 
/// # Arguments
/// 
/// * `reports_json` - reports.json文件的路径（可选）
/// * `summary_path` - 摘要报告输出路径（可选）
/// 
/// # Returns
/// 
/// 如果成功，返回 `Ok(())`
pub fn generate_markdown_summary_from_json(reports_json: Option<&Path>, summary_path: Option<&Path>) -> Result<()> {
    // 使用默认值或提供的参数
    let reports_json = reports_json.unwrap_or_else(|| Path::new("reports.json"));
    let summary_path = summary_path.unwrap_or_else(|| Path::new("summary.md"));
    
    info!("Generating summary from: {}", reports_json.display());
    info!("Summary will be written to: {}", summary_path.display());
    
    // TODO: 实现实际的摘要生成逻辑
    // 1. 读取reports.json文件
    // 2. 分析每个测试的结果
    // 3. 生成Markdown格式的摘要表格
    // 4. 写入summary_path
    
    // 生成一个简单的模板摘要确保编译通过
    let now = Local::now();
    let date_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    
    let summary_content = format!(r#"# 测试摘要报告

*生成时间: {}*

## 测试结果矩阵

| 单元 | 目标 | 状态 | 报告链接 |
|------|-----|------|---------|
| *待实现* | *待实现* | *待实现* | *待实现* |

## 摘要统计

- **总测试数**: 0
- **通过测试**: 0
- **失败测试**: 0
- **通过率**: 0%

*注意: 此报告是自动生成的占位符，实际实现尚未完成。*
"#, date_str);

    // 写入摘要文件
    let mut file = File::create(summary_path)?;
    file.write_all(summary_content.as_bytes())?;
    
    warn!("Summary generation is not fully implemented yet");
    Ok(())
}
