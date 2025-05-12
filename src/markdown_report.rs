//! Generates markdown reports summarizing test results for various units across different distributions.
use chrono::Local;
use log::info;
use std::{fs::File, io::prelude::*, path::Path};
use anyhow::Result;

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
    
    // 检查输入文件是否存在
    if !reports_json.exists() {
        return Err(anyhow::anyhow!("Reports JSON file not found: {}", reports_json.display()));
    }
    
    // 读取并解析聚合报告JSON文件
    let file = File::open(reports_json)?;
    let report_data: serde_json::Value = serde_json::from_reader(file)?;
    
    // 提取报告列表
    let reports = match report_data.get("reports") {
        Some(serde_json::Value::Array(reports)) => reports,
        _ => return Err(anyhow::anyhow!("Invalid report format: 'reports' array not found")),
    };
    
    // 提取聚合日期
    let aggregation_date = report_data.get("aggregation_date")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| "未知日期");
    
    // 统计通过、失败、部分通过的测试数量
    let mut pass_count = 0;
    let mut fail_count = 0;
    let mut partial_count = 0;
    
    // 收集单元和目标信息
    let mut units = std::collections::HashSet::new();
    let mut targets = std::collections::HashSet::new();
    
    // 用于构建报告表格的数据
    let mut table_rows = Vec::new();
    
    // 处理每个报告
    for report in reports {
        // 提取基本信息
        let unit_name = report.get("unit_name").and_then(|v| v.as_str()).unwrap_or("未知");
        let target_name = report.get("target_name").and_then(|v| v.as_str()).unwrap_or("未知");
        let status = report.get("overall_status").and_then(|v| v.as_str()).unwrap_or("unknown");
        let title = report.get("template_title").and_then(|v| v.as_str()).unwrap_or("未知测试");
        let report_path = report.get("report_path").and_then(|v| v.as_str()).unwrap_or("");
        
        // 收集单元和目标
        units.insert(unit_name.to_string());
        targets.insert(target_name.to_string());
        
        // 统计状态
        match status {
            "pass" => pass_count += 1,
            "fail" => fail_count += 1,
            "partial" => partial_count += 1,
            _ => {}
        }
        
        // 状态符号
        let status_icon = match status {
            "pass" => "✅",
            "fail" => "❌",
            "partial" => "⚠️",
            _ => "❓",
        };
        
        // 构建表格行
        let row = format!("| {} | {} | {} | {} | [查看报告]({})", 
            title, unit_name, target_name, status_icon, report_path);
        table_rows.push(row);
    }
    
    // 计算总数
    let total_count = reports.len();
    let pass_rate = if total_count > 0 {
        (pass_count as f64 / total_count as f64) * 100.0
    } else {
        0.0
    };
    
    // 构建摘要内容
    let mut summary_content = format!(r#"# 测试摘要报告

*生成时间: {}, 基于{}的聚合报告*

## 测试结果矩阵

| 单元 | 目标 | 状态 | 报告链接 |
|------|-----|------|---------|
"#, Local::now().format("%Y-%m-%d %H:%M:%S"), aggregation_date);

    // 添加表格内容
    for row in table_rows {
        summary_content.push_str(&format!("{}\n", row));
    }
    
    // 添加统计信息
    summary_content.push_str(&format!(r#"
## 摘要统计

- **总测试数**: {}
- **通过测试**: {} ({}%)
- **部分通过**: {}
- **失败测试**: {}
- **单元数量**: {}
- **目标数量**: {}

## 图例说明

- ✅ 通过: 所有测试步骤均成功
- ⚠️ 部分通过: 部分测试步骤成功
- ❌ 失败: 所有测试步骤失败
- ❓ 未知: 测试状态无法确定
"#, 
        total_count, 
        pass_count, 
        format!("{:.1}", pass_rate),
        partial_count,
        fail_count,
        units.len(),
        targets.len()
    ));
    
    // 写入摘要文件
    let mut file = File::create(summary_path)?;
    file.write_all(summary_content.as_bytes())?;
    
    info!("Summary report generated successfully at: {}", summary_path.display());
    Ok(())
}
