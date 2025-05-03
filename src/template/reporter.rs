//! 测试报告生成器
//!
//! 这个模块负责根据测试模板和执行结果生成Markdown格式的测试报告

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use regex::Regex;
use log::{info, warn, debug, error};

use crate::template::{
    TestTemplate, StepStatus, StepResult, TemplateContext
};
use crate::template::executor::ExecutionResult;

/// 报告生成器
pub struct Reporter {
    /// 工作目录
    work_dir: PathBuf,
    /// 输出目录
    output_dir: PathBuf,
}

impl Reporter {
    /// 创建新的报告生成器
    pub fn new(work_dir: PathBuf, output_dir: Option<PathBuf>) -> Self {
        let output_dir = output_dir.unwrap_or_else(|| work_dir.join("reports"));
        Self {
            work_dir,
            output_dir,
        }
    }
    
    /// 生成测试报告
    pub fn generate_report(&self, template: &TestTemplate, result: &ExecutionResult) -> Result<PathBuf> {
        // 确保输出目录存在
        fs::create_dir_all(&self.output_dir)
            .with_context(|| format!("无法创建输出目录: {}", self.output_dir.display()))?;
        
        // 确定报告文件名
        let report_filename = format!(
            "{}_{}.report.md",
            result.unit_name.replace(" ", "_").to_lowercase(),
            result.target_name.replace(" ", "_").to_lowercase()
        );
        
        // 构建报告文件路径
        let report_path = self.output_dir.join(&report_filename);
        
        // 生成报告内容
        let report_content = self.generate_report_content(template, result)?;
        
        // 写入报告文件
        fs::write(&report_path, &report_content)
            .with_context(|| format!("无法写入报告文件: {}", report_path.display()))?;
        
        info!("已生成测试报告: {}", report_path.display());
        
        Ok(report_path)
    }
    
    /// 生成报告内容
    fn generate_report_content(&self, template: &TestTemplate, result: &ExecutionResult) -> Result<String> {
        // 获取原始模板内容
        let mut content = template.raw_content.clone();
        
        // 确保YAML前置数据和正文之间有正确的换行
        let re = Regex::new(r"(?s)^---\s*\n(.*?)\n---\s*\n")?;
        if let Some(captures) = re.captures(&content) {
            let yaml_part = captures.get(0).unwrap().as_str();
            content = content.replacen(yaml_part, &format!("{}\n", yaml_part), 1);
        }
        
        // 替换正文中的变量
        
        // 0. 替换元数据变量（模板中的title, unit_name等）
        let pattern_title = "{{ title }}";
        content = content.replace(pattern_title, &template.metadata.title);
        
        let pattern_unit_name = "{{ unit_name }}";
        content = content.replace(pattern_unit_name, &template.metadata.unit_name);
        
        // 从target_config路径中提取目标名称
        if let Some(target_name) = template.metadata.target_config
            .components()
            .filter_map(|comp| match comp {
                std::path::Component::Normal(s) => Some(s.to_string_lossy().to_string()),
                _ => None,
            })
            .find(|s| s == "targets") 
            .and_then(|_| {
                template.metadata.target_config
                    .components()
                    .filter_map(|comp| match comp {
                        std::path::Component::Normal(s) => Some(s.to_string_lossy().to_string()),
                        _ => None,
                    })
                    .nth(1)
            }) 
        {
            let pattern_target = "{{ target_name }}";
            content = content.replace(pattern_target, &target_name);
        }
        
        // 处理自定义元数据
        for (key, value) in &template.metadata.custom {
            let pattern = format!("{{{{ {} }}}}", key);
            content = content.replace(&pattern, value);
        }
        
        // 1. 替换特殊变量
        for (name, value) in &result.special_vars {
            let pattern = format!("{{{{ {} }}}}", name);
            content = content.replace(&pattern, value);
        }
        
        // 2. 替换提取的变量 - 修复此部分
        for (name, value) in &result.variables {
            let pattern = format!("{{{{ {} }}}}", name);
            // 使用全局替换，确保所有出现的变量都被替换
            content = content.replace(&pattern, value);
        }
        
        // 3. 替换状态变量
        // {{ status.step_id }} -> ✅ Pass, ❌ Fail, ⚠️ Skipped, ❓ Blocked
        let status_pattern = Regex::new(r"\{\{\s*status\.([a-zA-Z0-9_-]+)\s*\}\}")?;
        content = status_pattern.replace_all(&content, |caps: &regex::Captures| {
            let step_id = &caps[1];
            match result.step_results.get(step_id) {
                Some(step_result) => match step_result.status {
                    StepStatus::Pass => "✅ Pass",
                    StepStatus::Fail => "❌ Fail",
                    StepStatus::Skipped => "⚠️ Skipped",
                    StepStatus::Blocked => "❓ Blocked",
                    StepStatus::NotRun => "❓ Not Run",
                },
                None => "❓ Unknown",
            }
            .to_string()
        }).to_string();
        
        // 4. 替换命令输出
        // 支持双引号或单引号形式的引用
        let output_block_pattern = Regex::new(r#"(?ms)```output\s+\{ref=(?:"([^"]+)"|'([^']+)')\}\n.*?```"#)?;
        content = output_block_pattern.replace_all(&content, |caps: &regex::Captures| {
            // 获取引用ID（可能在第一个或第二个捕获组）
            let cmd_id = caps.get(1).or_else(|| caps.get(2)).map_or("unknown", |m| m.as_str());
            
            match result.step_results.get(cmd_id) {
                Some(step_result) => {
                    format!(r#"```output {{ref="{}"}}\n{}\n```"#, cmd_id, step_result.stdout)
                },
                None => {
                    format!(r#"```output {{ref="{}"}}\n命令结果不可用\n```"#, cmd_id)
                }
            }
        }).to_string();
        
        // 5. 处理自动生成总结表
        // 支持双引号或单引号形式的ID
        let summary_block_pattern = Regex::new(r#"(?ms)^##\s+.*?\s+\{id=(?:"([^"]+)"|'([^']+)').*?generate_summary=true.*?\}\s*$"#)?;
        content = summary_block_pattern.replace_all(&content, |caps: &regex::Captures| {
            let section_id = caps.get(1).or_else(|| caps.get(2)).map_or("unknown", |m| m.as_str());
            let mut summary = caps[0].to_string(); // 保留原始标题行

            // 添加表头
            summary.push_str("\n\n| 步骤描述 | 状态 |\n");
            summary.push_str("|---------|------|\n");

            // 添加每个步骤的状态
            for (step_id, step_result) in &result.step_results {
                // 获取步骤描述
                let step = template.steps.iter()
                    .find(|s| &s.id == step_id);

                let description = step
                    .and_then(|s| s.description.clone())
                    .unwrap_or_else(|| step_id.clone());

                // 获取状态
                let status = match step_result.status {
                    StepStatus::Pass => "✅ Pass",
                    StepStatus::Fail => "❌ Fail",
                    StepStatus::Skipped => "⚠️ Skipped",
                    StepStatus::Blocked => "❓ Blocked",
                    StepStatus::NotRun => "❓ Not Run",
                };

                // 添加行
                summary.push_str(&format!("| {} | {} |\n", description, status));
            }

            summary
        }).to_string();
        
        // 6. 处理自动生成对比表格（未实现，可根据需要添加）
        
        // 7. 清理Markdown特殊标记
        // 清理 {id="xxx"} 标记
        let id_pattern = Regex::new(r#"\{id=(?:"[^"]+"|'[^']+')\}"#)?;
        content = id_pattern.replace_all(&content, "").to_string();

        // 清理 {exec=xxx} 标记
        let exec_pattern = Regex::new(r"\{exec=(?:true|false)\}")?;
        content = exec_pattern.replace_all(&content, "").to_string();
        
        // 清理 {description="xxx"} 标记
        let desc_pattern = Regex::new(r#"\{description=(?:"[^"]+"|'[^']+')\}"#)?;
        content = desc_pattern.replace_all(&content, "").to_string();
        
        // 清理 {assert.xxx=yyy} 标记
        let assert_pattern = Regex::new(r"\{assert\.[a-zA-Z_]+=[^\}]+\}")?;
        content = assert_pattern.replace_all(&content, "").to_string();
        
        // 清理 {extract.xxx=/yyy/} 标记
        let extract_pattern = Regex::new(r"\{extract\.[a-zA-Z_]+=/.*/\}")?;
        content = extract_pattern.replace_all(&content, "").to_string();
        
        // 清理 {depends_on=["xxx", "yyy"]} 标记
        let depends_pattern = Regex::new(r#"\{depends_on=\[(?:\"[^\"]*\"|'[^']*')(?:\s*,\s*(?:\"[^\"]*\"|'[^']*'))*\]\}"#)?;
        content = depends_pattern.replace_all(&content, "").to_string();
        
        // 清理所有其他花括号属性（捕获任何剩余的 {xxx=yyy} 格式）
        let misc_pattern = Regex::new(r"\{[a-zA-Z_][a-zA-Z0-9_]*=.*?\}")?;
        content = misc_pattern.replace_all(&content, "").to_string();
        
        // 清理连续的多余空格，但不清理换行符
        content = Regex::new(r"[^\S\r\n]{2,}")?.replace_all(&content, " ").to_string();
        
        // 清理行尾空格，但保留换行符
        content = Regex::new(r"[^\S\r\n]+\n")?.replace_all(&content, "\n").to_string();
        
        Ok(content)
    }
    
    /// 生成总结报告
    pub fn generate_summary_report(
        &self,
        results: &[ExecutionResult],
        output_path: Option<PathBuf>
    ) -> Result<PathBuf> {
        // 使用默认路径或指定路径
        let summary_path = output_path.unwrap_or_else(|| self.output_dir.join("summary.md"));
        
        // 生成总结内容
        let mut content = String::new();
        
        // 添加标题
        content.push_str("# 测试总结报告\n\n");
        content.push_str(&format!("生成时间: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        // 添加汇总统计
        let total = results.len();
        let passed = results.iter().filter(|r| r.overall_status == StepStatus::Pass).count();
        let failed = results.iter().filter(|r| r.overall_status == StepStatus::Fail).count();
        let skipped = results.iter().filter(|r| r.overall_status != StepStatus::Pass && r.overall_status != StepStatus::Fail).count();
        
        content.push_str("## 汇总统计\n\n");
        content.push_str(&format!("- 总计测试: {}\n", total));
        content.push_str(&format!("- 通过: {} ({}%)\n", passed, if total > 0 { passed * 100 / total } else { 0 }));
        content.push_str(&format!("- 失败: {} ({}%)\n", failed, if total > 0 { failed * 100 / total } else { 0 }));
        content.push_str(&format!("- 跳过: {} ({}%)\n", skipped, if total > 0 { skipped * 100 / total } else { 0 }));
        content.push_str("\n");
        
        // 获取所有目标和单元
        let mut targets = Vec::new();
        let mut units = Vec::new();
        
        for result in results {
            if !targets.contains(&result.target_name) {
                targets.push(result.target_name.clone());
            }
            if !units.contains(&result.unit_name) {
                units.push(result.unit_name.clone());
            }
        }
        
        targets.sort();
        units.sort();
        
        // 生成矩阵表
        content.push_str("## 测试矩阵\n\n");
        
        // 表头
        content.push_str("| 目标↓ / 单元→ |");
        for unit in &units {
            content.push_str(&format!(" {} |", unit));
        }
        content.push_str("\n");
        
        // 分隔行
        content.push_str("|--------------|");
        for _ in &units {
            content.push_str("------------|");
        }
        content.push_str("\n");
        
        // 表格内容
        for target in &targets {
            content.push_str(&format!("| {} |", target));
            
            for unit in &units {
                // 查找对应的结果
                let result = results.iter().find(|r| &r.target_name == target && &r.unit_name == unit);
                
                // 获取状态
                let status = match result {
                    Some(r) => match r.overall_status {
                        StepStatus::Pass => "✅",
                        StepStatus::Fail => "❌",
                        StepStatus::Skipped => "⚠️",
                        StepStatus::Blocked => "❓",
                        StepStatus::NotRun => "❓",
                    },
                    None => "🟢", // 未测试
                };
                
                // 如果有报告文件链接，添加链接
                if let Some(r) = result {
                    if let Some(ref path) = r.report_path {
                        // 计算相对路径
                        let rel_path = path.strip_prefix(&self.work_dir).unwrap_or(path);
                        content.push_str(&format!(" [{}]({}/) |", status, rel_path.display()));
                        continue;
                    }
                }
                
                // 无链接
                content.push_str(&format!(" {} |", status));
            }
            
            content.push_str("\n");
        }
        
        // 添加图例
        content.push_str("\n### 图例\n\n");
        content.push_str("- ✅ 通过\n");
        content.push_str("- ❌ 失败\n");
        content.push_str("- ⚠️ 跳过\n");
        content.push_str("- ❓ 阻塞/未运行\n");
        content.push_str("- 🟢 未测试\n");
        
        // 写入文件
        fs::write(&summary_path, &content)
            .with_context(|| format!("无法写入总结报告: {}", summary_path.display()))?;
        
        info!("已生成总结报告: {}", summary_path.display());
        
        Ok(summary_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 添加测试...
}