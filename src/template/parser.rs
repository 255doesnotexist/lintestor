//! Markdown测试模板解析器
//!
//! 这个模块负责解析Markdown格式的测试模板内容，识别其中的元数据、可执行代码块和特殊属性。

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use anyhow::{Result, Context, bail, anyhow};
use regex::Regex;
use log::{debug, info, warn, error};

// Import the new ExecutionStep related types
use crate::template::step::{ExecutionStep, GlobalStepId, StepType};
// Import ParsedTestStep directly, ContentBlock is defined in this file
use crate::template::{
    TemplateMetadata, ParsedTestStep, AssertionType, DataExtraction, TemplateReference
};
use crate::utils;


/// 表示模板文件内容的不同结构化块。
/// 解析器 (Parser) 会将原始模板字符串转换为 `Vec<ContentBlock>`。
#[derive(Debug, Clone, PartialEq)]
pub enum ContentBlock {
    /// YAML 前置元数据块。
    /// 存储的是 `---` 分隔符内部的原始 YAML 字符串。
    Metadata(String),

    /// 通用 Markdown 文本块。
    /// 这可以包含任何 Markdown内容，包括原始的步骤定义文本（如果它们不被特殊处理为其他类型的块）。
    Text(String),

    /// 代表一个旨在报告中显示的代码块。
    /// 其可见性可能由其属性控制。
    DisplayableCodeBlock {
        /// 代码块的原始内容，包括 ```lang {attrs}...``` 和代码本身。
        original_content: String,
        /// 此代码块对应的步骤的本地ID (如果有)。
        /// 用于查找 ExecutionStep 以获取属性 (例如可见性)。
        local_step_id: Option<String>,
    },

    /// 代表一个步骤输出的占位符。
    /// 例如 ` ```output {ref="step_id"} ... ``` `。
    OutputBlock {
        step_id: String,
    },

    /// 一个标记，指示在此处应插入自动生成的步骤摘要表。
    SummaryTablePlaceholder,
}

/// 解析Markdown测试模板内容，返回元数据、执行步骤列表和内容块列表
pub fn parse_template_into_content_blocks_and_steps(
    content: &str, 
    file_path: &Path
) -> Result<(TemplateMetadata, Vec<ExecutionStep>, Vec<ContentBlock>)> {
    info!("开始解析测试模板 (结构化内容和步骤): {}", file_path.display());
    
    let mut content_blocks = Vec::new();

    let (yaml_front_matter, markdown_content) = extract_front_matter(content)?;
    debug!("YAML前置数据长度: {} 字节", yaml_front_matter.len());
    debug!("Markdown内容长度: {} 字节", markdown_content.len());

    content_blocks.push(ContentBlock::Metadata(yaml_front_matter.clone()));
    
    let metadata = parse_metadata(&yaml_front_matter)?;
    info!("模板元数据解析完成: title=\"{}\", unit=\"{}\"", metadata.title, metadata.unit_name);
    
    let template_id = utils::get_template_id_from_path(file_path);
    debug!("生成的模板 ID: {}", template_id);

    // 同时解析步骤和内容块
    let (execution_steps, md_content_blocks) = parse_markdown_to_steps_and_content_blocks(&markdown_content, &template_id, &metadata)?;
    content_blocks.extend(md_content_blocks);
    
    info!("已解析 {} 个执行步骤和 {} 个内容块", execution_steps.len(), content_blocks.len());
    
    for step in &execution_steps {
        debug!("ExecutionStep: id={}, type={:?}, local_id={}, template_id={}, deps={:?}", 
            step.id, step.step_type, step.local_id, step.template_id, step.dependencies);
        if let Some(parsed_step) = &step.original_parsed_step {
            debug!("  Original Parsed Step: id={}, exec={}, assertions={}, extractions={}", 
                parsed_step.id, parsed_step.executable, parsed_step.assertions.len(), parsed_step.extractions.len());
        }
    }
    
    Ok((metadata, execution_steps, content_blocks))
}

/// 从Markdown内容中解析出 ExecutionSteps 和 ContentBlocks
fn parse_markdown_to_steps_and_content_blocks(
    markdown: &str, 
    template_id: &str, 
    metadata: &TemplateMetadata
) -> Result<(Vec<ExecutionStep>, Vec<ContentBlock>)> {
    debug!("开始将Markdown内容解析为 ExecutionSteps 和 ContentBlocks (template_id: {})", template_id);
    let mut execution_steps = Vec::new();
    let mut content_blocks = Vec::new();
    
    let heading_re = Regex::new(r#"(?m)^(#+)\s+(.*?)(?:\s+(\{.*\})|\s*)$"#)?;
    let code_block_re = Regex::new(r"(?ms)```(\w*)\s*(\{([^}]*)\})?\n(.*?)```")?;
    let output_block_re = Regex::new(r#"(?ms)```output\s+\{ref=(?:"([^"]+)"|'([^']+)')\}\n(?:.*?)```"#)?;
    let summary_table_re = Regex::new(r#"(?im)^\s*<!--\s*LINTESOR_SUMMARY_TABLE\s*-->\s*$"#)?;

    let mut current_heading_stack: Vec<(GlobalStepId, u8)> = Vec::new();
    let mut local_id_counter = 0;

    let mut last_match_end = 0;

    let combined_re_str = format!(
        "(?P<heading>{})|(?P<output_block>{})|(?P<summary_table>{})|(?P<code_block>{})",
        heading_re.as_str(),
        output_block_re.as_str(),
        summary_table_re.as_str(),
        code_block_re.as_str()
    );
    let combined_re = Regex::new(&combined_re_str)?;

    for captures in combined_re.captures_iter(markdown) {
        let match_start = captures.get(0).unwrap().start();
        let match_end = captures.get(0).unwrap().end();

        if match_start > last_match_end {
            let text_segment = &markdown[last_match_end..match_start];
            if !text_segment.trim().is_empty() {
                content_blocks.push(ContentBlock::Text(text_segment.to_string()));
            }
        }

        if let Some(heading_match) = captures.name("heading") {
            let line = heading_match.as_str();
            content_blocks.push(ContentBlock::Text(line.to_string()));

            if let Some(caps) = heading_re.captures(line) {
                let level = caps.get(1).map_or(0, |m| m.as_str().len() as u8);
                let text = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();
                let attributes_str = caps.get(3).map_or("", |m| m.as_str());
                
                let attributes = parse_inline_attributes(attributes_str);
                let local_id = attributes.get("id").cloned().unwrap_or_else(|| {
                    local_id_counter += 1;
                    format!("heading_{}", local_id_counter)
                });
                let global_id = format!("{}::{}", template_id, local_id);

                while let Some((_, last_level)) = current_heading_stack.last() {
                    if *last_level >= level {
                        current_heading_stack.pop();
                    } else {
                        break;
                    }
                }

                let mut dependencies = HashSet::new();
                if let Some(parent_heading_id) = current_heading_stack.last() {
                    dependencies.insert(parent_heading_id.0.clone());
                }
                if let Some(deps_str) = attributes.get("depends_on") {
                    parse_depends_on_str(deps_str, &mut dependencies, template_id, &metadata.references);
                }

                execution_steps.push(ExecutionStep {
                    id: global_id.clone(),
                    template_id: template_id.to_string(),
                    local_id,
                    step_type: StepType::Heading { level, text, attributes: attributes.clone() },
                    dependencies,
                    original_parsed_step: None,
                });
                current_heading_stack.push((global_id, level));
            }
        } else if let Some(output_match) = captures.name("output_block") {
            if let Some(caps) = output_block_re.captures(output_match.as_str()) {
                let ref_id_attr = caps.get(1).or_else(|| caps.get(2)).map_or("", |m| m.as_str()).to_string();
                content_blocks.push(ContentBlock::OutputBlock { step_id: ref_id_attr.clone() });
                
                let local_id = format!("{}-outputplaceholder", ref_id_attr);
                let global_id = format!("{}::{}", template_id, local_id);
                let mut dependencies = HashSet::new();
                let ref_global_id = resolve_dependency_ref(&ref_id_attr, template_id, &metadata.references);
                dependencies.insert(ref_global_id.clone());

                if let Some(parent_heading_id) = current_heading_stack.last() {
                    dependencies.insert(parent_heading_id.0.clone());
                }

                let parsed_step_info = ParsedTestStep {
                    id: local_id.clone(),
                    description: Some(format!("Placeholder for output of step {}", ref_id_attr)),
                    command: None, 
                    depends_on: vec![ref_id_attr.to_string()],
                    assertions: Vec::new(),
                    extractions: Vec::new(),
                    executable: false, // Not directly executable
                    ref_command: Some(ref_id_attr.to_string()),
                    raw_content: output_match.as_str().to_string(),
                    active: Some(true), // Output blocks are typically always active if the ref step runs
                    timeout_ms: None,
                };

                execution_steps.push(ExecutionStep {
                    id: global_id,
                    template_id: template_id.to_string(),
                    local_id,
                    step_type: StepType::OutputPlaceholder,
                    dependencies,
                    original_parsed_step: Some(parsed_step_info),
                });
            }
        } else if captures.name("summary_table").is_some() {
            content_blocks.push(ContentBlock::SummaryTablePlaceholder);
        } else if let Some(code_match) = captures.name("code_block") {
            // This is a general code block, which is an ExecutionStep 
            // and potentially a DisplayableCodeBlock for the report.
            let block_content = code_match.as_str();
            
            // Parse attributes to get local_id for DisplayableCodeBlock and ExecutionStep
            let preliminary_caps = code_block_re.captures(block_content); // Re-capture to get groups
            let lang_for_check = preliminary_caps.as_ref().and_then(|c| c.get(1)).map_or("", |m| m.as_str());
            let attrs_str_for_check = preliminary_caps.as_ref().and_then(|c| c.get(3)).map_or("", |m| m.as_str());

            // Avoid double-processing if it's an output block that was missed by the more specific output_block_re
            // (though output_block_re should be preferred and capture it first)
            if lang_for_check == "output" && attrs_str_for_check.contains("ref=") {
                // This should have been caught by the OutputBlock regex. 
                // If it reaches here, it implies a parsing logic subtlety or an edge case.
                // For safety, we can push it as Text, or log a warning.
                // However, the combined regex order should prevent this.
                // If it does happen, pushing as Text is safer than creating a confusing DisplayableCodeBlock.
                // For now, assume combined_re handles order correctly and this branch is less likely for true output blocks.
                // Let's proceed to treat it as a potential DisplayableCodeBlock / ExecutionStep.
            }

            // Now parse it as an ExecutionStep::CodeBlock and get its local_id
            if let Some(caps) = code_block_re.captures(block_content) { // Re-capture for full parsing
                let lang = caps.get(1).map_or("", |m| m.as_str()).to_string();
                let attributes_str = caps.get(3).map_or("", |m| m.as_str());
                let command = caps.get(4).map_or("", |m| m.as_str()).trim().to_string();
                
                let attributes = parse_inline_attributes(attributes_str);
                let local_id = attributes.get("id").cloned().unwrap_or_else(|| {
                    local_id_counter += 1;
                    format!("codeblock_{}", local_id_counter)
                });

                // Add as DisplayableCodeBlock for the report structure
                content_blocks.push(ContentBlock::DisplayableCodeBlock {
                    original_content: block_content.to_string(),
                    local_step_id: Some(local_id.clone()),
                });

                // Create the ExecutionStep as before
                let global_id = format!("{}::{}", template_id, local_id);
                let mut dependencies = HashSet::new();
                if let Some(parent_heading_id) = current_heading_stack.last() {
                    dependencies.insert(parent_heading_id.0.clone());
                }
                if let Some(deps_str) = attributes.get("depends_on") {
                    parse_depends_on_str(deps_str, &mut dependencies, template_id, &metadata.references);
                }
                
                let parsed_step_info = ParsedTestStep {
                    id: local_id.clone(),
                    description: attributes.get("description").cloned(),
                    command: Some(command.clone()),
                    depends_on: dependencies.iter().map(|gsid| gsid.split("::").last().unwrap_or("").to_string()).collect(),
                    assertions: parse_assertions_from_attributes(&attributes),
                    extractions: parse_extractions_from_attributes(&attributes),
                    executable: attributes.get("exec").and_then(|v_str| v_str.parse::<bool>().ok()).unwrap_or(true),
                    ref_command: None,
                    raw_content: block_content.to_string(), // raw_content in ParsedTestStep is the code itself, not the full block with ```
                    active: attributes.get("active").and_then(|v_str| v_str.parse::<bool>().ok()),
                    timeout_ms: attributes.get("timeout_ms").and_then(|v_str| v_str.parse::<u64>().ok()),
                };

                execution_steps.push(ExecutionStep {
                    id: global_id,
                    template_id: template_id.to_string(),
                    local_id,
                    step_type: StepType::CodeBlock { lang, command, attributes: attributes.clone() },
                    dependencies,
                    original_parsed_step: Some(parsed_step_info),
                });
            }
        }
        last_match_end = match_end;
    }

    if last_match_end < markdown.len() {
        let remaining_text = &markdown[last_match_end..];
        if !remaining_text.trim().is_empty() {
            content_blocks.push(ContentBlock::Text(remaining_text.to_string()));
        }
    }
    
    debug!("完成 ExecutionSteps ({}) 和 ContentBlocks ({}) 解析", execution_steps.len(), content_blocks.len());
    Ok((execution_steps, content_blocks))
}

/// 从Markdown内容中提取YAML前置数据
fn extract_front_matter(content: &str) -> Result<(String, &str)> {
    debug!("从模板内容中提取YAML前置数据");
    let re = Regex::new(r"(?s)^---\s*\n(.*?)\n---\s*\n(.*)$")?;
    
    match re.captures(content) {
        Some(caps) => {
            let yaml = caps.get(1).unwrap().as_str();
            let markdown = caps.get(2).unwrap().as_str();
            debug!("成功提取YAML前置数据");
            Ok((yaml.to_string(), markdown))
        },
        None => {
            error!("未找到YAML前置数据");
            bail!("未找到YAML前置数据，格式应为 '---\\n<yaml>\\n---\\n<markdown>'")
        }
    }
}

/// 解析YAML元数据
fn parse_metadata(yaml: &str) -> Result<TemplateMetadata> {
    debug!("解析YAML元数据");
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml)
        .with_context(|| "无法解析YAML前置数据")?;
    
    debug!("YAML解析成功，开始提取字段");
    
    let title = yaml_value["title"].as_str()
        .ok_or_else(|| anyhow!("元数据缺少'title'字段"))?
        .to_string();
    debug!("提取title: {}", title);
    
    let target_config_str = yaml_value["target_config"].as_str()
        .ok_or_else(|| anyhow!("元数据缺少'target_config'字段"))?;
    debug!("提取target_config: {}", target_config_str);
    
    let target_config = PathBuf::from(target_config_str);
    
    let unit_name = yaml_value["unit_name"].as_str()
        .ok_or_else(|| anyhow!("元数据缺少'unit_name'字段"))?
        .to_string();
    debug!("提取unit_name: {}", unit_name);
    
    let unit_version_command = yaml_value["unit_version_command"]
        .as_str()
        .map(|s| s.to_string());
    if let Some(ref cmd) = unit_version_command {
        debug!("提取unit_version_command: {}", cmd);
    }
    
    let tags = match yaml_value["tags"] {
        serde_yaml::Value::Sequence(ref seq) => {
            let tags: Vec<_> = seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            debug!("提取tags: {:?}", tags);
            tags
        },
        _ => Vec::new(),
    };
    
    let references = match yaml_value["references"] {
        serde_yaml::Value::Sequence(ref seq) => {
            let mut refs = Vec::new();
            for ref_value in seq {
                if let serde_yaml::Value::Mapping(ref mapping) = ref_value {
                    // 为了在模板里看起来舒服，我们实际上的对应是按下面这样的
                    // template -> template_path
                    // as -> namespace
                    let template_path = mapping.get(&serde_yaml::Value::String("template".to_string()))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .ok_or_else(|| anyhow!("references中的项缺少'template(template_path)'字段"))?;
                    
                    let namespace = mapping.get(&serde_yaml::Value::String("as".to_string()))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .ok_or_else(|| anyhow!("references中的项缺少'as(namespace)'字段"))?;
                    
                    debug!("提取模板引用: template_path={}, namespace={}", template_path, namespace);
                    refs.push(TemplateReference {
                        template_path,
                        namespace,
                    });
                }
            }
            refs
        },
        _ => Vec::new(),
    };
    
    if !references.is_empty() {
        debug!("共提取到 {} 个外部模板引用", references.len());
    }
    
    let mut custom = HashMap::new();
    if let serde_yaml::Value::Mapping(mapping) = &yaml_value {
        for (key, value) in mapping {
            if let Some(key_str) = key.as_str() {
                if ["title", "target_config", "unit_name", "unit_version_command", "tags", "references"]
                    .contains(&key_str) {
                    continue;
                }
                
                if let Some(value_str) = value.as_str() {
                    debug!("提取自定义字段: {} = {}", key_str, value_str);
                    custom.insert(key_str.to_string(), value_str.to_string());
                }
            }
        }
    }
    
    Ok(TemplateMetadata {
        title,
        target_config,
        unit_name,
        unit_version_command,
        tags,
        references,
        custom,
    })
}

/// Helper to parse inline attributes like {id="foo" exec="true"}
fn parse_inline_attributes(attr_str: &str) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    let attr_re = Regex::new(r#"\s*(\w+)\s*=\s*"([^"]*)""#).unwrap();
    for cap in attr_re.captures_iter(attr_str) {
        attributes.insert(cap[1].to_string(), cap[2].to_string());
    }
    attributes
}

/// Helper to parse "depends_on" string and populate dependencies set
fn parse_depends_on_str(deps_str: &str, dependencies: &mut HashSet<GlobalStepId>, current_template_id: &str, references: &[TemplateReference]) {
    let deps_list_str = deps_str.trim_matches(|c| c == '[' || c == ']');
    for dep_item_str in deps_list_str.split(',') {
        let trimmed_dep = dep_item_str.trim().trim_matches(|c| c == '\'' || c == '\"');
        if !trimmed_dep.is_empty() {
            dependencies.insert(resolve_dependency_ref(trimmed_dep, current_template_id, references));
        }
    }
}

/// Helper to resolve a dependency reference (e.g., "step_id" or "namespace::step_id") to a GlobalStepId
fn resolve_dependency_ref(dep_ref: &str, current_template_id: &str, references: &[TemplateReference]) -> GlobalStepId {
    if dep_ref.contains("::") {
        let parts: Vec<&str> = dep_ref.splitn(2, "::").collect();
        if parts.len() == 2 {
            let namespace_or_template_id = parts[0];
            let local_step_id = parts[1];

            for reference in references {
                if reference.namespace == namespace_or_template_id {
                    let referenced_template_file_name = Path::new(&reference.template_path)
                        .file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or(namespace_or_template_id);
                    return format!("{}::{}", referenced_template_file_name, local_step_id);
                }
            }
            return dep_ref.to_string();
        }
    }
    format!("{}::{}", current_template_id, dep_ref)
}

/// Helper to parse assertions from a HashMap of attributes
fn parse_assertions_from_attributes(attributes: &HashMap<String, String>) -> Vec<AssertionType> {
    let mut assertions = Vec::new();
    for (key, value) in attributes {
        if key.starts_with("assert.") {
            let assertion_key = key.trim_start_matches("assert.");
            match assertion_key {
                "exit_code" => if let Ok(code) = value.parse::<i32>() {
                    assertions.push(AssertionType::ExitCode(code));
                },
                "stdout_contains" => assertions.push(AssertionType::StdoutContains(value.clone())),
                "stdout_not_contains" => assertions.push(AssertionType::StdoutNotContains(value.clone())),
                "stdout_matches" => assertions.push(AssertionType::StdoutMatches(value.clone())),
                "stderr_contains" => assertions.push(AssertionType::StderrContains(value.clone())),
                "stderr_not_contains" => assertions.push(AssertionType::StderrNotContains(value.clone())),
                "stderr_matches" => assertions.push(AssertionType::StderrMatches(value.clone())),
                _ => warn!("未知断言类型: {}", key),
            }
        }
    }
    assertions
}

/// Helper to parse extractions from a HashMap of attributes
fn parse_extractions_from_attributes(attributes: &HashMap<String, String>) -> Vec<DataExtraction> {
    let mut extractions = Vec::new();
    for (key, value) in attributes {
        if key.starts_with("extract.") {
            let var_name = key.trim_start_matches("extract.").to_string();
            let regex_str = if value.starts_with('/') && value.ends_with('/') && value.len() > 1 {
                value[1..value.len()-1].to_string()
            } else {
                value.clone()
            };
            extractions.push(DataExtraction { variable: var_name, regex: regex_str });
        }
    }
    extractions
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_front_matter() {
        let content = r#"---
title: "Test Template"
target_config: "targets/my_target/config.toml"
unit_name: "MyUnit"
---

# Test Content"#;
        
        let (yaml, markdown) = extract_front_matter(content).unwrap();
        assert!(yaml.contains("title"));
        assert!(markdown.contains("# Test Content"));
    }
    
    #[test]
    fn test_parse_metadata() {
        let yaml = r#"
title: "Test Template"
target_config: "targets/my_target/config.toml"
unit_name: "MyUnit"
unit_version_command: "myunit --version"
tags:
  - core
  - feature-abc
references:
  - template: "external_template_1.md"
    as: "namespace1"
  - template: "external_template_2.md"
    as: "namespace2"
custom_field: "custom value"
"#;
        
        let metadata = parse_metadata(yaml).unwrap();
        assert_eq!(metadata.title, "Test Template");
        assert_eq!(metadata.target_config, PathBuf::from("targets/my_target/config.toml"));
        assert_eq!(metadata.unit_name, "MyUnit");
        assert_eq!(metadata.unit_version_command, Some("myunit --version".to_string()));
        assert_eq!(metadata.tags, vec!["core".to_string(), "feature-abc".to_string()]);
        assert_eq!(metadata.references.len(), 2);
        assert_eq!(metadata.references[0].template_path, "external_template_1.md");
        assert_eq!(metadata.references[0].namespace, "namespace1");
        assert_eq!(metadata.references[1].template_path, "external_template_2.md");
        assert_eq!(metadata.references[1].namespace, "namespace2");
        assert_eq!(metadata.custom.get("custom_field"), Some(&"custom value".to_string()));
    }
}