//! 测试报告生成器
//!
//! 这个模块负责根据测试模板和执行结果生成Markdown格式的测试报告
//!
//! 当BatchExecutor执行完成测试模板后，会调用Reporter生成最终的测试报告
//! Reporter主要做两件事:
//! 1. 替换模板中的变量为已执行结果中的变量
//! 2. 将命令执行结果插入到对应的输出块中

use anyhow::{Context, Result};
use log::{debug, info};
use regex::Regex;
use std::fs;
use std::path::PathBuf; // Path is not used directly, PathBuf is.
use std::sync::Arc;

use crate::template::executor::ExecutionResult;
use crate::template::variable::VariableManager;
use crate::template::{ContentBlock, StepStatus, TestTemplate};
use crate::utils;

/// 报告生成器
/// 负责将执行结果转换为Markdown格式的测试报告
pub struct Reporter {
    /// 测试模板文件所在的目录，用于解析相对路径等
    template_base_dir: PathBuf,
    /// 报告最终输出的目录
    report_output_dir: PathBuf,
}

impl Reporter {
    /// 创建新的报告生成器
    ///
    /// # 参数
    /// * `template_base_dir`: 当前处理的测试模板文件所在的目录。
    ///   用于解析模板中可能存在的相对路径引用，或作为报告中相对路径的基础。
    /// * `report_output_dir`: 所有生成的报告文件最终应存放的目录。
    ///   如果为 `None`，可能会使用 `template_base_dir` 下的 "reports" 子目录或其他默认逻辑。
    pub fn new(template_base_dir: PathBuf, report_output_dir: Option<PathBuf>) -> Self {
        let final_report_output_dir =
            report_output_dir.unwrap_or_else(|| template_base_dir.join("reports"));
        Self {
            template_base_dir,
            report_output_dir: final_report_output_dir,
        }
    }

    /// 生成单个测试模板的测试报告
    ///
    /// # 参数
    /// * `template`: 要为其生成报告的测试模板的 Arc 引用。
    /// * `result`: 该模板的执行结果 (`ExecutionResult`)。
    /// * `var_manager`: 一个对全局 `VariableManager` 的引用，用于在报告内容中替换变量。
    ///
    /// # 返回
    /// * `Result<PathBuf>`: 如果成功，返回生成的报告文件的绝对路径。
    pub fn generate_report(
        &self,
        template: &Arc<TestTemplate>,
        result: &ExecutionResult,
        var_manager: &VariableManager,
    ) -> Result<PathBuf> {
        debug!("开始为模板生成测试报告: {}", result.template_id());
        debug!("模板标题: {}", result.template_title());
        debug!(
            "模板文件所在目录 (基准目录): {}",
            self.template_base_dir.display()
        );
        debug!("报告计划输出目录: {}", self.report_output_dir.display());

        // 确保报告输出目录存在
        fs::create_dir_all(&self.report_output_dir).with_context(|| {
            format!("无法创建报告输出目录: {}", self.report_output_dir.display())
        })?;

        // 确定报告文件名 (可以基于模板ID和目标名称)
        let report_filename = format!(
            "{}_{}.report.md",
            result
                .template_id()
                .replace(['/', '\\', ':', ' '], "_")
                .to_lowercase(),
            result
                .target_name
                .replace(['/', '\\', ':', ' '], "_")
                .to_lowercase()
        );

        // 构建报告文件的完整路径
        let report_path = self.report_output_dir.join(&report_filename);

        // 生成报告的Markdown内容
        let report_content = self.generate_report_content(template, result, var_manager)?;

        // 将生成的Markdown内容写入报告文件
        fs::write(&report_path, &report_content)
            .with_context(|| format!("无法写入报告文件: {}", report_path.display()))?;

        info!("已成功生成测试报告: {}", report_path.display());
        Ok(report_path)
    }

    /// 生成报告的Markdown内容核心逻辑
    ///
    /// # 参数
    /// * `template`: 测试模板的 Arc 引用。
    /// * `result`: 模板的执行结果。
    /// * `var_manager`: 全局 `VariableManager` 的引用，用于变量替换。
    ///
    /// # 返回
    /// * `Result<String>`: 生成的Markdown报告内容字符串。
    fn generate_report_content(
        &self,
        template: &Arc<TestTemplate>,
        result: &ExecutionResult,
        var_manager: &VariableManager,
    ) -> Result<String> {
        let mut report_parts = Vec::new();
        let template_id = template.get_template_id();

        // Process metadata first if it exists as the first block
        if let Some(ContentBlock::Metadata(yaml_content)) = template.content_blocks.first() {
            let mut processed_yaml =
                var_manager.replace_variables(yaml_content, Some(&template_id), None);
            processed_yaml = format!("---\n{}\n---\n", processed_yaml.trim());
            report_parts.push(processed_yaml);
        }

        for content_block in &template.content_blocks {
            match content_block {
                ContentBlock::Metadata(_) => { /* Already handled or ignore if not first */ }
                ContentBlock::Text(text_content) => {
                    let mut processed_text = text_content.clone();
                    // Global and step-specific variable replacement (ensure correct step_id context if needed)
                    processed_text =
                        var_manager.replace_variables(&processed_text, Some(&template_id), None); // Broad pass
                                                                                                  // More specific passes if text can contain step-scoped variables:
                    let mut sorted_step_ids: Vec<_> = result.step_results.keys().cloned().collect();
                    sorted_step_ids.sort();
                    for step_id_key in &sorted_step_ids {
                        let local_step_id_for_var_lookup =
                            step_id_key.split("::").last().unwrap_or(step_id_key);
                        processed_text = var_manager.replace_variables(
                            &processed_text,
                            Some(&template_id),
                            Some(local_step_id_for_var_lookup),
                        );
                    }
                    processed_text.push('\n');
                    report_parts.push(processed_text);
                }
                ContentBlock::HeadingBlock {
                    id,
                    level,
                    text,
                    attributes,
                } => {
                    // 判断visible属性，默认true
                    let visible = attributes
                        .get("visible")
                        .map(|v| v != "false")
                        .unwrap_or(true);
                    if visible {
                        // 变量替换，支持全局和step级变量
                        let mut processed_text =
                            var_manager.replace_variables(text, Some(&template_id), Some(id));
                        processed_text = var_manager.replace_variables(
                            &processed_text,
                            Some(&template_id),
                            None,
                        );
                        let heading_line = format!(
                            "{} {}\n",
                            "#".repeat(*level as usize),
                            processed_text.trim()
                        );
                        report_parts.push(heading_line);
                    }
                }
                ContentBlock::CodeBlock {
                    id,
                    lang,
                    code,
                    attributes,
                } => {
                    // 判断visible属性
                    let visible = attributes
                        .get("visible")
                        .map(|v| v != "false")
                        .unwrap_or(true);
                    if visible {
                        // 变量替换
                        let processed_code =
                            var_manager.replace_variables(code, Some(&template_id), Some(id));
                        // 只输出lang和code内容，不输出任何属性
                        let code_block_str = format!("```{lang}\n{processed_code}\n```");
                        report_parts.push(self.clean_markdown_markup(&code_block_str)?);
                    }
                }
                ContentBlock::OutputBlock { step_id, stream } => {
                    // The step_id here is the *local* ID referenced in the template (e.g., {ref="local_step_id"})
                    // We need to find the corresponding StepResult using the global ID.
                    // 因为我们这边直接用 HashMap 通过 id 得到 results 了，不是原本那种 map 然后 id == id 的方式了
                    // 所以 Result 那边的 id 就成 dead_code 了需要 allow 一下
                    let global_step_id_to_find =
                        utils::get_result_id(template_id.as_str(), step_id);
                    if let Some(step_result) = result.step_results.get(&global_step_id_to_find) {
                        let mut output_block_content =
                            format!("```output {{ref=\"{step_id}\"}}\n");
                        
                        match stream.as_str() {
                            "stdout" => {
                                let stdout_content = step_result.stdout.trim_end_matches('\n');
                                output_block_content.push_str(stdout_content);
                                if !stdout_content.is_empty() {
                                    output_block_content.push('\n');
                                }
                            }
                            "stderr" => {
                                let stderr_content = step_result.stderr.trim_end_matches('\n');
                                output_block_content.push_str(stderr_content);
                                if !stderr_content.is_empty() {
                                    output_block_content.push('\n');
                                }
                            }
                            "both" => {
                                let stdout_content = step_result.stdout.trim_end_matches('\n');
                                let stderr_content = step_result.stderr.trim_end_matches('\n');
                                if !stdout_content.is_empty() {
                                    output_block_content.push_str("[stdout]\n");
                                    output_block_content.push_str(stdout_content);
                                    output_block_content.push('\n');
                                }
                                if !stderr_content.is_empty() {
                                    output_block_content.push_str("[stderr]\n");
                                    output_block_content.push_str(stderr_content);
                                    output_block_content.push('\n');
                                }
                            }
                            _ => {
                                output_block_content.push_str("[Invalid stream specified]\n");
                            }
                        }

                        output_block_content.push_str("```\n");
                        report_parts.push(output_block_content);
                    } else {
                        // Fallback if step result not found (should ideally not happen if template is valid)
                        report_parts.push(format!(
                            "```output {{ref=\"{step_id}\"}}\n[Output for step '{step_id}' not found]\n```\n"
                        ));
                    }
                }
                ContentBlock::SummaryTablePlaceholder => {
                    let summary_table =
                        self.generate_summary_table_string(result, var_manager, &template_id)?;
                    report_parts.push(summary_table);
                }
            }
        }
        let mut final_content = report_parts.join("\n");
        let yaml_front_matter_re = Regex::new(r"(?s)^---\s*\n(.*?)\n---\s*\n")?;
        if let Some(captures) = yaml_front_matter_re.captures(&final_content) {
            let yaml_part_end = captures.get(0).unwrap().end();
            if final_content.len() > yaml_part_end
                && !final_content[yaml_part_end..].starts_with('\n')
                && !final_content[yaml_part_end..].starts_with("\n\n") {
                    final_content.insert(yaml_part_end, '\n');
                }
        }

        final_content = self.clean_markdown_markup(&final_content)?;

        Ok(final_content)
    }

    /// Generates the Markdown string for a step summary table.
    fn generate_summary_table_string(
        &self,
        result: &ExecutionResult,
        var_manager: &VariableManager,
        template_id: &str,
    ) -> Result<String> {
        let mut table = String::new();
        table.push_str("| 步骤ID | 描述 | 状态 | 退出码 | 输出摘要 | 错误信息 |\n");
        table.push_str("|--------|------|------|--------|----------|----------|\n");

        let mut sorted_step_global_ids: Vec<_> = result.step_results.keys().cloned().collect();
        sorted_step_global_ids.sort();

        for global_step_id in sorted_step_global_ids {
            if let Some(step_result) = result.step_results.get(&global_step_id) {
                let display_step_id = global_step_id.split("::").last().unwrap_or(&global_step_id);

                let status_icon = match step_result.status {
                    StepStatus::Pass => "✅ Pass",
                    StepStatus::Fail => "❌ Fail",
                    StepStatus::Skipped => "⚠️ Skipped",
                    StepStatus::Blocked => "❓ Blocked",
                    StepStatus::NotRun => "❓ Not Run",
                };

                let original_description = step_result.description.as_deref().unwrap_or("-");
                let processed_description = var_manager.replace_variables(
                    original_description,
                    Some(template_id),
                    Some(display_step_id),
                );

                let stdout_summary = Self::summarize_output(&step_result.stdout, 50);
                let stderr_summary = Self::summarize_output(&step_result.stderr, 30);

                table.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} |\n",
                    display_step_id.replace("|", "\\\\|"),
                    processed_description
                        .replace("|", "\\\\|")
                        .replace("\n", "<br>"),
                    status_icon,
                    step_result.exit_code,
                    stdout_summary.replace("|", "\\\\|").replace("\n", "<br>"),
                    stderr_summary.replace("|", "\\\\|").replace("\n", "<br>")
                ));
            }
        }
        table.push('\n');
        Ok(table)
    }

    /// 清理最终报告内容中不应出现的Markdown特殊属性标记
    fn clean_markdown_markup(&self, content: &str) -> Result<String> {
        // 定义要从属性块内部移除的特定属性的正则表达式
        // 这些模式匹配 "key=value" 对，并包含尾随空格以帮助清理
        // TODO: 如何匹配某个 attr，和之前解析的时候用的表达式应该类似，未来要考虑复用
        let attribute_rules_for_inner_cleaning = vec![
            Regex::new(r#"id=(?:\"[^\"]+\"|'[^']+')\s*"#)?,
            Regex::new(r#"exec=(?:true|false)\s*"#)?,
            Regex::new(r#"description=(?:\"[^\"]+\"|'[^']+')\s*"#)?,
            Regex::new(r#"assert\.[a-zA-Z0-9_]+=(?:\"[^\"]*\"|'[^']*'|[^}\s]+)\s*"#)?,
            Regex::new(r#"extract\.[a-zA-Z0-9_]+=/.*?/[dimsx]*\s*"#)?,
            Regex::new(
                r#"depends_on=\[(?:\"[^\"]*\"|'[^']*')(?:\s*,\s*(?:\"[^\"]*\"|'[^']*'))*\]\s*"#,
            )?,
            Regex::new(r#"generate_summary=(?:true|false)\s*"#)?,
        ];

        // 匹配整个 {...} 块并捕获其内部内容
        let attr_block_regex = Regex::new(r#"\{([^{}]+)\}"#)?;
        let space_collapse_regex = Regex::new(r"\s\s+")?;

        let mut result = attr_block_regex
            .replace_all(content, |caps: &regex::Captures| {
                let mut inner_attrs = caps[1].to_string();
                for pattern in &attribute_rules_for_inner_cleaning {
                    inner_attrs = pattern.replace_all(&inner_attrs, "").to_string();
                }

                inner_attrs = inner_attrs.trim().to_string();
                if inner_attrs.is_empty() {
                    "".to_string() // 如果所有属性都被移除，则移除花括号
                } else {
                    // 清理内部可能留下的多余空格
                    inner_attrs = space_collapse_regex
                        .replace_all(&inner_attrs, " ")
                        .to_string();
                    format!("{{{inner_attrs}}}") // 重建花括号和清理后的属性
                }
            })
            .into_owned();

        // 通用空格和换行符清理
        result = Regex::new(r"[^\S\r\n]{2,}")?
            .replace_all(&result, " ")
            .to_string();
        result = Regex::new(r"[^\S\r\n]+\n")?
            .replace_all(&result, "\n")
            .to_string();
        result = Regex::new(r"\n{3,}")?
            .replace_all(&result, "\n\n")
            .to_string();

        Ok(result.trim().to_string() + "\n")
    }

    /// 辅助函数：对输出字符串进行摘要
    fn summarize_output(output: &str, max_len: usize) -> String {
        let trimmed = output.trim();
        if trimmed.is_empty() {
            "-".to_string()
        } else {
            let first_line = trimmed.lines().next().unwrap_or("").trim();
            if first_line.len() > max_len {
                format!("{}...", &first_line[..max_len])
            } else {
                first_line.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::target_config::TargetConfig;
    use crate::template::executor::{ExecutionResult, StepResult};
    use crate::template::{ContentBlock, ExecutionStep, TemplateMetadata, TemplateReference};
    use anyhow::Result;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    

    fn create_dummy_template(
        id: &str,
        raw_content: &str,
        content_blocks: Vec<ContentBlock>,
        steps: Vec<ExecutionStep>,
    ) -> Arc<TestTemplate> {
        // 这里可以根据需要构造 TemplateMetadata、steps 等
        // 但通常测试用例只关心 content_blocks

        // 写入 /tmp/dummy_target.toml，内容为本地测试
        let dummy_target_path = "/tmp/dummy_target.toml";
        let dummy_target_content = r#"
    name = "本地测试"
    testing_type = "local"
    description = "本地测试"
    "#;
        let _ = std::fs::write(dummy_target_path, dummy_target_content);

        Arc::new(TestTemplate {
            file_path: PathBuf::from(format!("/test/{}.test.md", id)),
            raw_content: raw_content.to_string(),
            content_blocks,
            metadata: TemplateMetadata {
                title: format!("{} Title", id),
                target_config: TargetConfig::from_file(dummy_target_path)
                    .expect("Failed to load dummy target config"),
                unit_name: format!("{}_unit", id),
                unit_version: "0.0.1".to_string(),
                tags: Vec::new(),
                references: Vec::<TemplateReference>::new(),
                custom: HashMap::new(),
            },
            steps,
        })
    }

    #[allow(dead_code)]
    fn create_dummy_execution_result(
        template_arc: Arc<TestTemplate>,
        target_name: &str,
        unit_name: &str,
        status: StepStatus,
    ) -> ExecutionResult {
        let mut step_results = HashMap::new();
        let template_id = template_arc.get_template_id();

        let global_step1_id = format!("{}::step1", template_id);
        let global_step2_id = format!("{}::step2_output_ref", template_id);

        step_results.insert(
            global_step1_id.clone(),
            StepResult {
                id: global_step1_id.clone(),
                description: Some("First step".to_string()),
                status: StepStatus::Pass,
                stdout: "Step 1 output".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: Some(100),
                assertion_error: None,
            },
        );
        step_results.insert(
            global_step2_id.clone(),
            StepResult {
                id: global_step2_id.clone(),
                description: Some("Second step with output".to_string()),
                status: StepStatus::Pass,
                stdout: "This is the output of step2_output_ref.".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: Some(120),
                assertion_error: None,
            },
        );

        ExecutionResult {
            template: template_arc,
            unit_name: unit_name.to_string(),
            target_name: target_name.to_string(),
            overall_status: status,
            step_results,
            variables: HashMap::new(),
            report_path: None,
        }
    }

    #[test]
    fn test_generate_report_with_variable_substitution_and_output_blocks(
    ) -> Result<(), Box<dyn Error>> {
        // 创建临时目录用于测试
        let temp_dir = tempfile::tempdir()?;
        let template_base_dir = temp_dir.path().join("templates");
        let report_output_dir = temp_dir.path().join("reports");
        fs::create_dir_all(&template_base_dir)?;
        fs::create_dir_all(&report_output_dir)?;

        // 定义模板ID和内容
        let template_id = "test_template";
        let template_content = r#"---
title: Test Report
unit_name: Test Unit
---

# {{ metadata.title }}

This is a test report for {{ metadata.unit_name }} targeting {{ metadata.target_name }}.

```output {ref="code1"}
Placeholder for code1 output.
```

```bash {id="code1"}
echo "Hello, {{ execution_time }}"
```
"#;

        // 创建测试模板
        let template = create_dummy_template(
            template_id,
            template_content,
            vec![
            ContentBlock::Metadata("title: Test Report\nunit_name: Test Unit\ntarget_name: Test Target".to_string()),
            ContentBlock::HeadingBlock {
                id: "heading1".to_string(),
                level: 1,
                text: "{{ metadata.title }}".to_string(),
                attributes: Default::default(),
            },
            ContentBlock::Text("This is a test report for {{ metadata.unit_name }} targeting {{ metadata.target_name }}.".to_string()),
            ContentBlock::OutputBlock { step_id: "code1".to_string(), stream: "stdout".to_string() },
            ContentBlock::CodeBlock {
                id: "code1".to_string(),
                lang: "bash".to_string(),
                code: "echo \"Hello, {{ execution_time }}\"".to_string(),
                attributes: {
                let mut m = std::collections::HashMap::new();
                m.insert("id".to_string(), "code1".to_string());
                m
                },
            },
            ],
            vec![],
        );

        // 创建执行结果
        let execution_result = ExecutionResult {
            template: Arc::clone(&template),
            unit_name: "default".to_string(), // 无关，这个最终会从 target_config 中获取
            target_name: "Test Target".to_string(),
            overall_status: StepStatus::Pass,
            step_results: HashMap::from([(
                "test_template::step1".to_string(),
                StepResult {
                    id: "test_template::step1".to_string(),
                    description: Some("Step 1".to_string()),
                    status: StepStatus::Pass,
                    stdout: "Step 1 output".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                    duration_ms: Some(100),
                    assertion_error: None,
                },
            )]),
            variables: HashMap::new(),
            report_path: None,
        };

        // 创建变量管理器并设置变量
        let mut var_manager = VariableManager::new();
        var_manager.register_template(&template, Some(template.get_template_id().as_str()))?;
        var_manager.set_variable("GLOBAL", "GLOBAL", "var.global_var", "World")?;

        // 创建Reporter实例
        let reporter = Reporter::new(template_base_dir.clone(), Some(report_output_dir.clone()));

        // 生成报告
        let report_path = reporter.generate_report(&template, &execution_result, &var_manager)?;
        assert!(report_path.exists());

        // 验证报告内容
        let report_content = fs::read_to_string(report_path)?;
        debug!("报告内容:\n{}", report_content);
        assert!(report_content.contains("Test Report"));
        assert!(report_content.contains("This is a test report for test_template_unit targeting 本地测试."));
        assert!(report_content.contains("Hello"));

        Ok(())
    }

    #[test]
    fn test_clean_markdown_markup_removes_attributes() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let reporter = Reporter::new(temp_dir.path().to_path_buf(), None);

        let input7 = "Start {id=\"id1\"} then {exec=true} finally {description=\"desc\"} end";
        assert_eq!(
            reporter.clean_markdown_markup(input7)?,
            "Start then finally end\n"
        );

        let input8 =
            "\n\n  leading space and   multiple spaces {id=\"id8\"} \n\n\n trailing line\n  ";
        let expected8 = "leading space and multiple spaces\n\n trailing line\n";
        assert_eq!(reporter.clean_markdown_markup(input8)?, expected8);

        Ok(())
    }
}
