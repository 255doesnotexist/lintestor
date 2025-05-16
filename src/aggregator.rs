//! Aggregates multiple test reports into a single report.
use anyhow::Result;
use log::{info, warn};
use std::{fs::File, io::prelude::*, path::Path};

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
pub fn aggregate_reports_from_dir(
    reports_dir: Option<&Path>,
    output_path: Option<&Path>,
) -> Result<()> {
    // 使用默认值或提供的参数
    let reports_dir = reports_dir.unwrap_or_else(|| Path::new("reports"));
    let output_path = output_path.unwrap_or_else(|| Path::new("reports.json"));

    info!(
        "Aggregating reports from directory: {}",
        reports_dir.display()
    );
    info!("Output will be written to: {}", output_path.display());

    // 确保报告目录存在
    if !reports_dir.exists() || !reports_dir.is_dir() {
        warn!(
            "Report directory does not exist or is not a directory: {}",
            reports_dir.display()
        );
        return Ok(());
    }

    // 存储所有测试报告数据
    let mut all_reports = Vec::new();

    // 遍历目录中的所有文件
    for entry in std::fs::read_dir(reports_dir)? {
        let entry = entry?;
        let path = entry.path();

        // 只处理.report.md文件
        if path.is_file()
            && path.extension().map_or(false, |ext| ext == "md")
            && path.to_string_lossy().contains(".report.")
        {
            info!("Processing report file: {}", path.display());

            // 读取报告文件内容
            let content = std::fs::read_to_string(&path)?;

            // 从Markdown报告中提取元数据
            let report_data = extract_report_metadata(&content, &path)?;

            all_reports.push(report_data);
        }
    }

    // 如果没有找到报告，发出警告并返回
    if all_reports.is_empty() {
        warn!(
            "No report files found in directory: {}",
            reports_dir.display()
        );
        return Ok(());
    }

    info!("Found {} report files", all_reports.len());

    // 将所有报告数据序列化为JSON
    let json_data = serde_json::json!({
        "reports": all_reports,
        "aggregation_date": chrono::Local::now().to_rfc3339(),
        "total_reports": all_reports.len(),
    });

    // 写入聚合结果到输出文件
    let mut file = File::create(output_path)?;
    let json_str = serde_json::to_string_pretty(&json_data)?;
    file.write_all(json_str.as_bytes())?;

    info!(
        "Successfully aggregated {} reports to {}",
        all_reports.len(),
        output_path.display()
    );

    Ok(())
}

/// 从Markdown报告中提取元数据
fn extract_report_metadata(content: &str, file_path: &Path) -> Result<serde_json::Value> {
    // 提取基本信息
    let template_id = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .replace(".report", "");

    // 尝试从内容中提取YAML前置数据
    let mut title = String::from("Untitled Test");
    let mut unit_name = String::from("unknown");
    let mut target_name = String::from("unknown");
    let mut overall_status = "unknown";

    // 正则表达式提取标题
    if let Some(title_match) = regex::Regex::new(r"#\s+(.+?)[\r\n]")
        .ok()
        .and_then(|re| re.captures(content))
    {
        title = title_match
            .get(1)
            .map_or("Untitled Test", |m| m.as_str())
            .to_string();
    }

    // 提取单元名称
    if let Some(unit_match) = regex::Regex::new(r#"unit_name:\s*"?(.*?)"?[\r\n]"#)
        .ok()
        .and_then(|re| re.captures(content))
    {
        unit_name = unit_match
            .get(1)
            .map_or("unknown", |m| m.as_str())
            .trim()
            .to_string();
    }

    // 提取目标名称
    if let Some(target_match) = regex::Regex::new(r#"target_config:\s*"?(.*?)"?[\r\n]"#)
        .ok()
        .and_then(|re| re.captures(content))
    {
        let target_path = target_match.get(1).map_or("", |m| m.as_str());
        // 从路径中提取目标名称
        target_name = std::path::Path::new(target_path)
            .components()
            .filter_map(|comp| match comp {
                std::path::Component::Normal(s) => Some(s.to_string_lossy().to_string()),
                _ => None,
            })
            .find(|s| s == "targets")
            .and_then(|_| {
                std::path::Path::new(target_path)
                    .components()
                    .filter_map(|comp| match comp {
                        std::path::Component::Normal(s) => Some(s.to_string_lossy().to_string()),
                        _ => None,
                    })
                    .nth(1)
            })
            .unwrap_or_else(|| "unknown".to_string());
    }

    // 确定测试状态
    if content.contains("状态: ✅") || content.contains("status: ✅") {
        overall_status = "pass";
    } else if content.contains("状态: ❌") || content.contains("status: ❌") {
        overall_status = "fail";
    } else if content.contains("状态: ⚠️") || content.contains("status: ⚠️") {
        overall_status = "partial";
    }

    // 构建报告JSON对象
    let report_data = serde_json::json!({
        "template_id": template_id,
        "template_title": title,
        "unit_name": unit_name,
        "target_name": target_name,
        "overall_status": overall_status,
        "execution_date": chrono::Local::now().to_rfc3339(),
        "report_path": file_path.to_string_lossy().to_string()
    });

    Ok(report_data)
}
