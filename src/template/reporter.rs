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
        
        // 打印所有收集到的特殊变量
        info!("处理特殊变量 - 共 {} 个", result.special_vars.len());
        for (name, value) in &result.special_vars {
            info!("特殊变量: {} = {}", name, value);
        }
        
        // 打印所有提取的变量
        info!("处理提取的变量 - 共 {} 个", result.variables.len());
        for (name, value) in &result.variables {
            info!("提取的变量: {} = {}", name, value);
        }
        
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
            let old_content = content.clone();
            content = content.replace(&pattern, value);
            
            // 检查是否发生了替换，并记录日志
            if old_content != content {
                info!("特殊变量替换成功: {} = {}", name, value);
            } else {
                warn!("特殊变量未找到匹配: {} = {}", name, value);
            }
        }
        
        // 2. 替换提取的变量 - 增强日志和替换逻辑
        for (name, value) in &result.variables {
            // 变量名前后可能有空格，使用更宽松的正则表达式
            let pattern_strict = format!("{{{{ {} }}}}", name); // 严格匹配，无空格
            
            // 修复：正确转义花括号，避免正则表达式错误
            let pattern_loose = format!(r"\{{\s*{}\s*\}}", name); // 宽松匹配，允许空格
            
            info!("尝试替换变量: {} = {}", name, value);
            info!("严格匹配模式: {}", pattern_strict);
            info!("宽松匹配模式: {}", pattern_loose);
            
            // 计算变量出现次数
            let occurrences = content.matches(&pattern_strict).count();
            info!("变量 {} 在内容中出现 {} 次 (严格匹配)", name, occurrences);
            
            // 使用正则表达式查找所有匹配
            let re_var = Regex::new(&pattern_loose)?;
            let matches = re_var.find_iter(&content).count();
            info!("变量 {} 在内容中找到 {} 个正则匹配", name, matches);
            
            // 首先尝试严格匹配替换
            let old_content = content.clone();
            content = content.replace(&pattern_strict, value);
            
            if old_content != content {
                info!("变量 {} 替换成功 (严格匹配)", name);
            } else {
                // 如果严格匹配失败，尝试正则替换
                info!("尝试使用正则表达式替换变量: {}", name);
                content = re_var.replace_all(&content, value).to_string();
                
                if old_content != content {
                    info!("变量 {} 替换成功 (正则匹配)", name);
                } else {
                    warn!("变量 {} 未能替换，在内容中未找到匹配", name);
                    
                    // 查找相似的变量模式
                    let var_pattern = Regex::new(r"\{\{\s*([a-zA-Z0-9_]+)\s*\}\}")?;
                    let mut found_vars = Vec::new();
                    for cap in var_pattern.captures_iter(&content) {
                        found_vars.push(cap[1].to_string());
                    }
                    
                    if !found_vars.is_empty() {
                        info!("在内容中发现其他变量占位符: {:?}", found_vars);
                    }
                }
            }
        }
        
        // 3. 替换状态变量
        // {{ status.step_id }} -> ✅ Pass, ❌ Fail, ⚠️ Skipped, ❓ Blocked
        let status_pattern = Regex::new(r"\{\{\s*status\.([a-zA-Z0-9_-]+)\s*\}\}")?;
        content = status_pattern.replace_all(&content, |caps: &regex::Captures| {
            let step_id = &caps[1];
            let status_value = match result.step_results.get(step_id) {
                Some(step_result) => match step_result.status {
                    StepStatus::Pass => "✅ Pass",
                    StepStatus::Fail => "❌ Fail",
                    StepStatus::Skipped => "⚠️ Skipped",
                    StepStatus::Blocked => "❓ Blocked",
                    StepStatus::NotRun => "❓ Not Run",
                },
                None => "❓ Unknown",
            };
            
            info!("替换状态变量: status.{} = {}", step_id, status_value);
            status_value.to_string()
        }).to_string();
        
        // 4. 替换命令输出
        // 支持双引号或单引号形式的引用
        let output_block_pattern = Regex::new(r#"(?ms)```output\s+\{ref=(?:"([^"]+)"|'([^']+)')\}\n.*?```"#)?;
        content = output_block_pattern.replace_all(&content, |caps: &regex::Captures| {
            // 获取引用ID（可能在第一个或第二个捕获组）
            let cmd_id = caps.get(1).or_else(|| caps.get(2)).map_or("unknown", |m| m.as_str());
            
            info!("替换命令输出块: ref={}", cmd_id);
            
            match result.step_results.get(cmd_id) {
                Some(step_result) => {
                    info!("找到命令结果: {} (输出长度: {} 字节)", cmd_id, step_result.stdout.len());
                    // 打印命令输出的内容预览，帮助诊断
                    // 修复：安全地处理UTF-8字符边界
                    let preview = if !step_result.stdout.is_empty() {
                        let char_count = step_result.stdout.chars().take(50).count();
                        let safe_index = step_result.stdout.char_indices()
                            .map(|(i, _)| i)
                            .nth(char_count)
                            .unwrap_or(step_result.stdout.len());
                            
                        let truncated = &step_result.stdout[..safe_index];
                        if safe_index < step_result.stdout.len() {
                            format!("{}...", truncated.replace('\n', "\\n"))
                        } else {
                            truncated.replace('\n', "\\n")
                        }
                    } else {
                        "<空输出>".to_string()
                    };
                    info!("输出内容预览: {}", preview);
                    
                    // 检查输出是否为空
                    if step_result.stdout.is_empty() {
                        // 检查stderr是否有内容
                        if !step_result.stderr.is_empty() {
                            warn!("命令 {} 的stdout为空，但stderr有内容: {}", cmd_id, step_result.stderr);
                        }
                        
                        // 检查退出码
                        info!("命令 {} 的退出码: {}", cmd_id, step_result.exit_code);
                    }
                    
                    // 检查命令是否有数据提取结果
                    if !step_result.extracted_vars.is_empty() {
                        info!("命令 {} 提取的变量: {:?}", cmd_id, step_result.extracted_vars);
                    }
                    
                    // 别改这三个 {} 因为这是原样字符串，你直接打 \n 在里面不是换行
                    format!(r#"```output {{ref="{}"}}{}{}{}```"#, cmd_id, "\n", &step_result.stdout, "\n")
                },
                None => {
                    warn!("未找到命令结果: {}", cmd_id);
                    // 显示所有可用的命令ID，帮助诊断
                    let available_ids: Vec<&String> = result.step_results.keys().collect();
                    warn!("可用的命令结果ID: {:?}", available_ids);
                    format!(r#"```output {{ref="{}"}}\n命令结果不可用\n```"#, cmd_id)
                }
            }
        }).to_string();
        
        // 5. 处理自动生成总结表 - 改进显示更多有用信息
        // 只在标记为generate_summary=true的节中生成摘要表
        let summary_block_pattern = Regex::new(r#"(?ms)^##\s+.*?\s+\{id=(?:"([^"]+)"|'([^']+)').*?generate_summary=true.*?\}\s*$"#)?;
        let mut processed_summary = false;  // 记录是否已生成摘要表
        
        content = summary_block_pattern.replace_all(&content, |caps: &regex::Captures| {
            let section_id = caps.get(1).or_else(|| caps.get(2)).map_or("unknown", |m| m.as_str());
            
            // 如果已经处理过摘要，跳过后续的摘要生成
            if processed_summary {
                warn!("检测到多个摘要标记(generate_summary=true)，忽略额外摘要: {}", section_id);
                return caps[0].to_string();  // 返回原始标题行，不生成表格
            }
            
            info!("生成测试结果摘要表: section_id={}", section_id);
            processed_summary = true;
            
            let mut summary = caps[0].to_string(); // 保留原始标题行
            summary.push_str("\n\n");  // 确保有足够的换行

            // 添加表头 - 更丰富的列信息
            summary.push_str("| 步骤ID | 描述 | 状态 | 退出码 | 输出摘要 | 错误信息 |\n");
            summary.push_str("|--------|------|------|--------|----------|----------|\n");

            // 收集所有有效的执行步骤（排除输出引用步骤）
            let mut valid_steps = Vec::new();
            
            for step in &template.steps {
                // 跳过输出引用步骤（以-output结尾的步骤ID通常是输出引用）
                if step.id.ends_with("-output") || step.ref_command.is_some() {
                    continue;
                }
                
                // 找到步骤结果
                if let Some(step_result) = result.step_results.get(&step.id) {
                    // 获取描述，如果没有则使用ID
                    let description = step.description.clone().unwrap_or_else(|| step.id.clone());
                    
                    // 获取状态
                    let status = match step_result.status {
                        StepStatus::Pass => "✅ Pass",
                        StepStatus::Fail => "❌ Fail",
                        StepStatus::Skipped => "⚠️ Skipped",
                        StepStatus::Blocked => "❓ Blocked",
                        StepStatus::NotRun => "❓ Not Run",
                    };
                    
                    // 获取输出和错误信息摘要
                    let stdout_summary = if !step_result.stdout.is_empty() {
                        // 获取第一行或前50个字符（以实际内容结构为准）
                        let first_line = step_result.stdout
                            .lines()
                            .next()
                            .unwrap_or("")
                            .trim();
                            
                        let char_count = first_line.chars().take(50).count();
                        let safe_index = first_line.char_indices()
                            .map(|(i, _)| i)
                            .nth(char_count)
                            .unwrap_or_else(|| first_line.len());
                            
                        if safe_index < first_line.len() {
                            format!("{}...", &first_line[..safe_index])
                        } else {
                            first_line.to_string()
                        }
                    } else {
                        "-".to_string()
                    };
                    
                    let stderr_summary = if !step_result.stderr.is_empty() {
                        // 获取第一行或前30个字符
                        let first_line = step_result.stderr
                            .lines()
                            .next()
                            .unwrap_or("")
                            .trim();
                            
                        let char_count = first_line.chars().take(30).count();
                        let safe_index = first_line.char_indices()
                            .map(|(i, _)| i)
                            .nth(char_count)
                            .unwrap_or_else(|| first_line.len());
                            
                        if safe_index < first_line.len() {
                            format!("{}...", &first_line[..safe_index])
                        } else {
                            first_line.to_string()
                        }
                    } else {
                        "-".to_string()
                    };
                    
                    // 准备退出码显示
                    let exit_code = format!("{}", step_result.exit_code);
                    
                    info!("添加摘要项: {} = {}", step.id, status);
                    valid_steps.push((
                        step.id.clone(),
                        description,
                        status.to_string(),
                        exit_code,
                        stdout_summary,
                        stderr_summary
                    ));
                }
            }
            
            // 如果没有找到有效步骤，添加一个提示
            if valid_steps.is_empty() {
                summary.push_str("| - | 未找到可执行步骤 | ❓ | - | - | - |\n");
            } else {
                // 添加步骤到表格
                for (id, description, status, exit_code, stdout, stderr) in valid_steps {
                    summary.push_str(&format!(
                        "| {} | {} | {} | {} | {} | {} |\n",
                        id, description, status, exit_code, stdout, stderr
                    ));
                }
            }

            summary.push_str("\n");
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
        
        // 检查是否仍有未替换的变量占位符
        let remaining_vars = Regex::new(r"\{\{\s*([a-zA-Z0-9_]+)\s*\}\}")?;
        let mut remaining_list = Vec::new();
        for cap in remaining_vars.captures_iter(&content) {
            let var_name = cap.get(1).unwrap().as_str();
            remaining_list.push(var_name.to_string());
        }
        
        if !remaining_list.is_empty() {
            warn!("报告中仍有未替换的变量: {:?}", remaining_list);
        } else {
            info!("所有变量都已成功替换");
        }
        
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