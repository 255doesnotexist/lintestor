//! Markdown测试模板解析器
//!
//! 这个模块负责解析Markdown格式的测试模板内容，识别其中的元数据、可执行代码块和特殊属性。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, Ordering};
use anyhow::{Result, Context, bail, anyhow};
use regex::Regex;
use log::{debug, info, warn, error};

use crate::template::{
    TestTemplate, TemplateMetadata, TestStep, AssertionType, DataExtraction
};

/// 解析Markdown测试模板内容
pub fn parse_template(content: &str) -> Result<TestTemplate> {
    info!("开始解析测试模板");
    
    // 分离YAML前置数据和Markdown内容
    let (yaml_front_matter, markdown_content) = extract_front_matter(content)?;
    debug!("YAML前置数据长度: {} 字节", yaml_front_matter.len());
    debug!("Markdown内容长度: {} 字节", markdown_content.len());
    
    // 解析元数据
    let metadata = parse_metadata(&yaml_front_matter)?;
    info!("模板元数据解析完成: title=\"{}\", unit=\"{}\"", metadata.title, metadata.unit_name);
    debug!("目标配置: {}", metadata.target_config.display());
    debug!("标签: {:?}", metadata.tags);
    
    // 解析Markdown内容中的步骤
    let steps = parse_steps(markdown_content)?;
    info!("已解析 {} 个测试步骤", steps.len());
    
    for (idx, step) in steps.iter().enumerate() {
        if step.executable {
            debug!("步骤 #{}: id={}, 可执行=true, 依赖={:?}", idx+1, step.id, step.depends_on);
            
            // 记录断言
            for assertion in &step.assertions {
                match assertion {
                    AssertionType::ExitCode(code) => {
                        debug!("  断言: exit_code={}", code);
                    },
                    AssertionType::StdoutContains(text) => {
                        debug!("  断言: stdout_contains=\"{}\"", text);
                    },
                    AssertionType::StdoutMatches(regex) => {
                        debug!("  断言: stdout_matches=/{}/", regex);
                    },
                    AssertionType::StderrContains(text) => {
                        debug!("  断言: stderr_contains=\"{}\"", text);
                    },
                    AssertionType::StderrMatches(regex) => {
                        debug!("  断言: stderr_matches=/{}/", regex);
                    },
                }
            }
            
            // 记录变量提取
            for extraction in &step.extractions {
                info!("  变量提取: {}=/{}/", extraction.variable, extraction.regex);
            }
        } else if let Some(ref_id) = &step.ref_command {
            debug!("步骤 #{}: id={}, 输出引用={}", idx+1, step.id, ref_id);
        } else {
            debug!("步骤 #{}: id={}, 非执行步骤", idx+1, step.id);
        }
    }
    
    Ok(TestTemplate {
        metadata,
        steps,
        file_path: PathBuf::new(), // 在TestTemplate::from_file中设置
        raw_content: content.to_string(),
    })
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
    
    // 提取必需字段
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
    
    // 提取可选字段
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
    
    // 收集其他自定义字段
    let mut custom = HashMap::new();
    if let serde_yaml::Value::Mapping(mapping) = &yaml_value {
        for (key, value) in mapping {
            // 跳过已处理的标准字段
            if let Some(key_str) = key.as_str() {
                if ["title", "target_config", "unit_name", "unit_version_command", "tags"]
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
        custom,
    })
}

/// 解析Markdown内容中的测试步骤
fn parse_steps(markdown: &str) -> Result<Vec<TestStep>> {
    debug!("开始解析Markdown内容中的测试步骤");
    let mut steps = Vec::new();
    
    // 匹配标题块
    let heading_re = Regex::new(r#"(?m)^(#+)\s+(.*?)(?:\s+[{]id="([^"]+)"(?:\s+depends_on=\["([^"]+)"(?:,\s*"([^"]+)")*\])?[}])?$"#)?;
    
    // 匹配代码块
    // 支持语法: ```bash {id="my-id" exec=true description="My description" assert.exit_code=0 assert.stdout_contains="text" extract.var_name=/regex/}
    let code_block_re = Regex::new(r"(?ms)```(\w+)\s+\{([^}]+)\}\n(.*?)```")?;
    
    // 匹配输出块
    // 支持语法: ```output {ref="cmd-id"}
    let output_block_re = Regex::new("(?ms)```output\\s+\\{ref=\"([^\"]+)\"\\}\\n(.*?)```")?;

    // 遍历找到的块
    let mut current_step_id = String::new();
    let mut current_step_content = String::new();
    
    debug!("解析Markdown内容为步骤块");
    
    for line in markdown.lines() {
        // 匹配标题
        if let Some(captures) = heading_re.captures(line) {
            // 保存之前的步骤（如果有）
            if !current_step_id.is_empty() && !current_step_content.is_empty() {
                // 解析之前收集的内容
                let step_content = current_step_content.trim();
                debug!("处理步骤内容: id={}, 内容长度={}", current_step_id, step_content.len());
                
                // 在内容中查找代码块和输出块
                parse_blocks(step_content, &current_step_id, &mut steps)?;
                
                current_step_content.clear();
            }
            
            // 开始新的步骤
            current_step_id = captures.get(3)
                .map_or_else(
                    || format!("step-{}", steps.len() + 1),
                    |m| m.as_str().to_string()
                );
            
            let heading_level = captures.get(1).unwrap().as_str().len();
            let heading_text = captures.get(2).unwrap().as_str();
            debug!("找到新步骤: level={}, title=\"{}\", id=\"{}\"", heading_level, heading_text, current_step_id);
            
            // 添加标题作为步骤内容的开始
            current_step_content = line.to_string() + "\n";
            continue;
        }
        
        // 添加到当前步骤的内容
        if !current_step_id.is_empty() {
            current_step_content.push_str(line);
            current_step_content.push('\n');
        }
    }
    
    // 处理最后一个步骤
    if !current_step_id.is_empty() && !current_step_content.is_empty() {
        // 解析之前收集的内容
        let step_content = current_step_content.trim();
        debug!("处理最后一个步骤内容: id={}, 内容长度={}", current_step_id, step_content.len());
        
        // 在内容中查找代码块和输出块
        parse_blocks(step_content, &current_step_id, &mut steps)?;
    }
    
    info!("共解析到 {} 个步骤", steps.len());
    Ok(steps)
}

/// 解析步骤内容中的代码块和输出块
fn parse_blocks(content: &str, step_id: &str, steps: &mut Vec<TestStep>) -> Result<()> {
    debug!("解析步骤 {} 中的代码块和输出块", step_id);
    
    // 识别代码块，支持不同的属性格式
    let code_block_re = Regex::new(r"(?ms)```(\w+)\s+\{([^}]+)\}\n(.*?)```")?;
    
    // 识别输出块，同时支持单引号和双引号的引用
    let output_block_re = Regex::new(r#"(?ms)```output\s+\{ref=(?:"([^"]+)"|'([^']+)')\}\n(.*?)```"#)?;
    
    // 解析代码块
    for cap in code_block_re.captures_iter(content) {
        let language = cap.get(1).unwrap().as_str();
        let attributes = cap.get(2).unwrap().as_str();
        let code = cap.get(3).unwrap().as_str();
        
        debug!("找到代码块: language={}, attributes='{}'", language, attributes);
        
        // 解析块属性
        let (id, description, executable, depends_on, assertions, extractions) = 
            parse_block_attributes(attributes, step_id)?;
        
        debug!("解析代码块属性: id={}, description=\"{}\", executable={}, 依赖数量={}, 断言数量={}, 提取数量={}",
                id, description, executable, depends_on.len(), assertions.len(), extractions.len());
        
        // 记录变量提取规则
        for extraction in &extractions {
            info!("代码块 {} 包含变量提取: {}=/{}/", id, extraction.variable, extraction.regex);
        }
        
        // 创建测试步骤
        let step = TestStep {
            id,
            description: Some(description),
            command: if language == "bash" || language == "sh" { Some(code.to_string()) } else { None },
            depends_on,
            assertions,
            extractions,
            executable,
            ref_command: None,
            raw_content: format!("```{} {{{}}}\n{}\n```", language, attributes, code),
        };
        
        steps.push(step);
    }
    
    // 解析输出块
    for cap in output_block_re.captures_iter(content) {
        // 获取引用ID（可能在第一个或第二个捕获组）
        let ref_id = cap.get(1).or_else(|| cap.get(2)).map_or("unknown", |m| m.as_str());
        let placeholder = cap.get(3).unwrap().as_str();
        
        debug!("找到输出引用块: ref_id={}, placeholder内容长度={}", ref_id, placeholder.len());
        
        // 创建引用步骤
        let step = TestStep {
            id: format!("{}-output", ref_id),
            description: None,
            command: None,
            depends_on: vec![ref_id.to_string()],
            assertions: Vec::new(),
            extractions: Vec::new(),
            executable: false,
            ref_command: Some(ref_id.to_string()),
            raw_content: format!("```output {{ref=\"{}\"}}\n{}\n```", ref_id, placeholder),
        };
        
        steps.push(step);
    }
    
    Ok(())
}

/// 解析代码块属性
fn parse_block_attributes(attributes: &str, parent_id: &str) -> Result<(String, String, bool, Vec<String>, Vec<AssertionType>, Vec<DataExtraction>)> {
    debug!("解析代码块属性: {}", attributes);
    
    let mut id = String::new();
    let mut description = String::new();
    let mut executable = false;
    let mut depends_on = Vec::new();
    let mut assertions = Vec::new();
    let mut extractions = Vec::new();
    
    // 分割属性
    for attr in attributes.split_whitespace() {
        debug!("处理属性: {}", attr);
        
        if attr.starts_with("id=") {
            // 提取ID属性，支持单引号或双引号
            id = attr.trim_start_matches("id=")
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string();
            debug!("  解析id=\"{}\"", id);
        } else if attr.starts_with("description=") {
            // 提取描述属性，支持单引号或双引号
            description = attr.trim_start_matches("description=")
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string();
            debug!("  解析description=\"{}\"", description);
        } else if attr == "exec=true" {
            executable = true;
            debug!("  设置executable=true");
        } else if attr.starts_with("depends_on=[") && attr.ends_with("]") {
            // 解析依赖列表，支持各种引号样式
            let deps_str = attr.trim_start_matches("depends_on=[").trim_end_matches("]");
            depends_on = deps_str.split(',')
                .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
                .collect();
            debug!("  解析depends_on={:?}", depends_on);
        } else if attr.starts_with("assert.") {
            // 解析断言
            if attr.starts_with("assert.exit_code=") {
                let code_str = attr.trim_start_matches("assert.exit_code=");
                if let Ok(code) = code_str.parse::<i32>() {
                    assertions.push(AssertionType::ExitCode(code));
                    debug!("  解析断言: exit_code={}", code);
                } else {
                    warn!("  无效的exit_code值: {}", code_str);
                }
            } else if attr.starts_with("assert.stdout_contains=") {
                let text = attr.trim_start_matches("assert.stdout_contains=").trim_matches('"').to_string();
                assertions.push(AssertionType::StdoutContains(text.clone()));
                debug!("  解析断言: stdout_contains=\"{}\"", text);
            } else if attr.starts_with("assert.stdout_matches=") {
                let regex = attr.trim_start_matches("assert.stdout_matches=").trim_matches('"').to_string();
                assertions.push(AssertionType::StdoutMatches(regex.clone()));
                debug!("  解析断言: stdout_matches=/{}/", regex);
            } else if attr.starts_with("assert.stderr_contains=") {
                let text = attr.trim_start_matches("assert.stderr_contains=").trim_matches('"').to_string();
                assertions.push(AssertionType::StderrContains(text.clone()));
                debug!("  解析断言: stderr_contains=\"{}\"", text);
            } else if attr.starts_with("assert.stderr_matches=") {
                let regex = attr.trim_start_matches("assert.stderr_matches=").trim_matches('"').to_string();
                assertions.push(AssertionType::StderrMatches(regex.clone()));
                debug!("  解析断言: stderr_matches=/{}/", regex);
            } else {
                warn!("  未知的断言类型: {}", attr);
            }
        } else if attr.starts_with("extract.") {
            // 解析数据提取
            let parts: Vec<&str> = attr.split('=').collect();
            if parts.len() == 2 {
                let var_name = parts[0].trim_start_matches("extract.").to_string();
                let regex = parts[1].trim_matches('/').to_string();
                extractions.push(DataExtraction {
                    variable: var_name.clone(),
                    regex: regex.clone(),
                });
                info!("  解析变量提取: {}=/{}/", var_name, regex);
            } else {
                warn!("  无效的extract语法: {}", attr);
            }
        }
    }
    
    // 如果没有指定ID，使用父ID加自动生成后缀
    if id.is_empty() {
        // 使用简单的计数器代替随机数
        static COUNTER: AtomicU16 = AtomicU16::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        id = format!("{}-block-{}", parent_id, counter);
        debug!("自动生成ID: {}", id);
    }
    
    Ok((id, description, executable, depends_on, assertions, extractions))
}

/// 从文本中提取变量值
fn extract_variable(text: &str, regex_str: &str) -> Result<String> {
    info!("尝试从文本中提取变量，正则表达式: {}", regex_str);
    
    let re = Regex::new(regex_str)
        .with_context(|| format!("无效的正则表达式: {}", regex_str))?;
    
    match re.captures(text) {
        Some(captures) => {
            if captures.len() > 1 {
                let value = captures.get(1).unwrap().as_str().to_string();
                info!("成功提取变量值: {}", value);
                Ok(value)
            } else {
                let value = captures.get(0).unwrap().as_str().to_string();
                info!("成功提取变量值(使用完整匹配): {}", value);
                Ok(value)
            }
        },
        None => {
            let preview = if text.len() > 50 { 
                format!("{}...", &text[..50]) 
            } else { 
                text.to_string() 
            };
            warn!("未能从文本中提取变量，文本预览: {}", preview);
            bail!("正则表达式没有匹配: {}", regex_str)
        }
    }
}

/// 解析命令中的环境变量设置（支持export VAR=value语法）
fn parse_environment_vars(command: &str, env_vars: &mut Vec<(String, String)>) {
    debug!("解析命令中的环境变量设置");
    let patterns = [
        r"export\s+([A-Za-z_][A-Za-z0-9_]*)=([^;]+)",
        r"([A-Za-z_][A-Za-z0-9_]*)=([^;]+)\s+",
    ];
    
    for pattern in &patterns {
        let re = Regex::new(pattern).unwrap();
        for cap in re.captures_iter(command) {
            let var_name = cap.get(1).unwrap().as_str().to_string();
            let value = cap.get(2).unwrap().as_str().to_string();
            debug!("提取环境变量: {}={}", var_name, value);
            env_vars.push((var_name, value));
        }
    }
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
custom_field: "custom value"
"#;
        
        let metadata = parse_metadata(yaml).unwrap();
        assert_eq!(metadata.title, "Test Template");
        assert_eq!(metadata.target_config, PathBuf::from("targets/my_target/config.toml"));
        assert_eq!(metadata.unit_name, "MyUnit");
        assert_eq!(metadata.unit_version_command, Some("myunit --version".to_string()));
        assert_eq!(metadata.tags, vec!["core".to_string(), "feature-abc".to_string()]);
        assert_eq!(metadata.custom.get("custom_field"), Some(&"custom value".to_string()));
    }
    
    // 更多测试...
}