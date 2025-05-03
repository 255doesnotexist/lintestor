//! Markdown测试模板解析器
//!
//! 这个模块负责解析Markdown格式的测试模板内容，识别其中的元数据、可执行代码块和特殊属性。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
                    AssertionType::StdoutNotContains(text) => {
                        debug!("  断言: stdout_not_contains=\"{}\"", text);
                    },
                    AssertionType::StdoutMatches(regex) => {
                        debug!("  断言: stdout_matches=/{}/", regex);
                    },
                    AssertionType::StderrContains(text) => {
                        debug!("  断言: stderr_contains=\"{}\"", text);
                    },
                    AssertionType::StderrNotContains(text) => {
                        debug!("  断言: stderr_not_contains=\"{}\"", text);
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
    
    // 匹配标题块 - 改进正则表达式，更好地处理ID和依赖属性
    let heading_re = Regex::new(r#"(?m)^(#+)\s+(.*?)(?:\s+\{id="([^"]+)"(?:\s+depends_on=(\[[^\]]+\]))?\s*\})?$"#)?;
    
    // 预扫描阶段：收集所有部分ID和代码块ID
    let mut section_ids = HashMap::new();
    let mut code_block_ids = HashMap::new();
    
    // 1. 首先扫描所有标题节ID
    for caps in heading_re.captures_iter(markdown) {
        if let Some(id_match) = caps.get(3) {
            let section_id = id_match.as_str().to_string();
            let heading_text = caps.get(2).unwrap().as_str().to_string();
            section_ids.insert(section_id.clone(), heading_text.clone());
            debug!("预扫描到标题ID: {}, 标题: {}", section_id, heading_text);
        }
    }
    
    // 2. 扫描所有代码块ID
    let code_block_re = Regex::new(r#"(?ms)```\w+\s+\{id="([^"]+)"[^}]*\}\n.*?```"#)?;
    for caps in code_block_re.captures_iter(markdown) {
        let block_id = caps.get(1).unwrap().as_str().to_string();
        code_block_ids.insert(block_id.clone(), true);
        debug!("预扫描到代码块ID: {}", block_id);
    }
    
    // 合并所有已知ID用于后续验证
    let mut all_known_ids = HashMap::new();
    for (id, title) in &section_ids {
        all_known_ids.insert(id.clone(), format!("标题: {}", title));
    }
    for (id, _) in &code_block_ids {
        all_known_ids.insert(id.clone(), "代码块".to_string());
    }
    
    info!("预扫描ID完成，找到 {} 个标题ID和代码块ID", all_known_ids.len());
    
    // 构建标题嵌套结构以处理依赖关系
    let mut heading_stack: Vec<(String, usize, Vec<String>)> = Vec::new(); // (id, level, depends_on)
    
    // 匹配代码块，扩展正则表达式支持依赖关系属性
    let code_block_re = Regex::new(r"(?ms)```(\w+)\s+\{([^}]+)\}\n(.*?)```")?;
    
    // 匹配输出块
    let output_block_re = Regex::new(r#"(?ms)```output\s+\{ref=(?:"([^"]+)"|'([^']+)')\}\n(.*?)```"#)?;

    // 遍历Markdown内容的每一行
    let mut lines = markdown.lines().peekable();
    let mut current_content = String::new();

    while let Some(line) = lines.next() {
        // 检查是否是标题行
        if let Some(captures) = heading_re.captures(line) {
            // 处理之前收集的内容（如果有）
            if !current_content.is_empty() {
                // 确保有父标题
                if !heading_stack.is_empty() {
                    let (parent_id, _, parent_deps) = heading_stack.last().unwrap();
                    debug!("处理标题 {} 下的内容，内容长度={}", parent_id, current_content.len());
                    
                    // 解析内容中的代码块
                    parse_blocks(&current_content, parent_id, parent_deps, &all_known_ids, &mut steps)?;
                }
                current_content.clear();
            }
            
            // 解析当前标题
            let level = captures.get(1).unwrap().as_str().len();
            let title = captures.get(2).unwrap().as_str();
            let section_id = captures.get(3).map_or_else(
                || format!("section-{}", steps.len() + 1),
                |m| m.as_str().to_string()
            );
            
            // 解析显式声明的依赖
            let mut depends_on = Vec::new();
            if let Some(deps_match) = captures.get(4) {
                let deps_str = deps_match.as_str();
                // 去掉方括号并解析依赖项
                let inner = deps_str.trim_start_matches('[').trim_end_matches(']');
                depends_on = inner.split(',')
                    .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\'')).filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
                
                debug!("标题 {} 显式依赖于: {:?}", section_id, depends_on);
            }
            
            // 处理标题级别和嵌套依赖
            // 移除比当前标题级别更低的标题
            while !heading_stack.is_empty() && heading_stack.last().unwrap().1 >= level {
                heading_stack.pop();
            }
            
            // 自动添加对父标题的依赖
            if !heading_stack.is_empty() {
                let parent_id = heading_stack.last().unwrap().0.clone();
                if !depends_on.contains(&parent_id) {
                    debug!("标题 {} 自动依赖于父标题 {}", section_id, parent_id);
                    depends_on.push(parent_id);
                }
            }
            
            // 验证依赖ID是否存在
            for dep_id in &depends_on {
                if !all_known_ids.contains_key(dep_id) {
                    warn!("标题 {} 依赖于未知ID: {}", section_id, dep_id);
                }
            }
            
            debug!("解析标题: level={}, title=\"{}\", id=\"{}\", 依赖={:?}", 
                level, title, section_id, depends_on);
            
            // 添加当前标题到栈中
            heading_stack.push((section_id.clone(), level, depends_on.clone()));
            
            // 创建标题步骤
            let step = TestStep {
                id: section_id,
                description: Some(title.to_string()),
                command: None,
                depends_on,
                assertions: Vec::new(),
                extractions: Vec::new(),
                executable: false,
                ref_command: None,
                raw_content: line.to_string(),
            };
            
            debug!("添加标题步骤: id={}", step.id);
            steps.push(step);
            
            // 将当前行添加到内容中
            current_content.push_str(line);
            current_content.push('\n');
            continue;
        }
        
        // 检查是否是代码块的开始
        if line.starts_with("```") {
            // 收集整个代码块
            let mut code_block = String::new();
            code_block.push_str(line);
            code_block.push('\n');
            
            // 继续读取直到代码块结束
            let mut in_code_block = true;
            while in_code_block {
                if let Some(next_line) = lines.next() {
                    code_block.push_str(next_line);
                    code_block.push('\n');
                    if next_line.starts_with("```") {
                        in_code_block = false;
                    }
                } else {
                    // 文件结束但代码块未关闭
                    warn!("代码块未正确关闭");
                    in_code_block = false;
                }
            }
            
            // 直接处理代码块
            if !heading_stack.is_empty() {
                let (parent_id, _, parent_deps) = heading_stack.last().unwrap();
                
                // 对代码块和输出块进行匹配
                if let Some(cap) = code_block_re.captures(&code_block) {
                    let language = cap.get(1).unwrap().as_str();
                    let attributes = cap.get(2).unwrap().as_str();
                    let code = cap.get(3).unwrap().as_str();
                    
                    debug!("找到代码块: language={}, attributes='{}'", language, attributes);
                    
                    // 解析属性
                    let (block_id, description, executable, mut depends_on, assertions, extractions) = 
                        parse_block_attributes(attributes, parent_id)?;
                    
                    // 合并从父标题继承的依赖关系
                    if depends_on.is_empty() && !parent_deps.is_empty() {
                        depends_on = parent_deps.clone();
                        debug!("代码块 {} 继承父标题 {} 的依赖: {:?}", block_id, parent_id, depends_on);
                    }
                    
                    // 创建测试步骤
                    let step = TestStep {
                        id: block_id,
                        description: Some(description),
                        command: if language == "bash" || language == "sh" { Some(code.to_string()) } else { None },
                        depends_on,
                        assertions,
                        extractions,
                        executable,
                        ref_command: None,
                        raw_content: format!("```{} {{{}}}\n{}\n```", language, attributes, code),
                    };
                    
                    debug!("添加代码块步骤: id={}", step.id);
                    steps.push(step);
                } else if let Some(cap) = output_block_re.captures(&code_block) {
                    // 处理输出引用块
                    let ref_id = cap.get(1).or_else(|| cap.get(2)).map_or("unknown", |m| m.as_str());
                    let placeholder = cap.get(3).unwrap().as_str();
                    
                    debug!("找到输出引用块: ref_id={}", ref_id);
                    
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
                    
                    debug!("添加输出引用步骤: id={}", step.id);
                    steps.push(step);
                }
            } else {
                // 如果没有父标题，将代码块添加到当前内容中
                current_content.push_str(&code_block);
            }
            continue;
        }
        
        // 普通行，添加到当前内容
        current_content.push_str(line);
        current_content.push('\n');
    }
    
    // 处理最后收集的内容
    if !current_content.is_empty() && !heading_stack.is_empty() {
        let (parent_id, _, parent_deps) = heading_stack.last().unwrap();
        debug!("处理最后的内容块: parent_id={}, 内容长度={}", parent_id, current_content.len());
        parse_blocks(&current_content, parent_id, parent_deps, &all_known_ids, &mut steps)?;
    }
    
    info!("共解析到 {} 个步骤", steps.len());
    Ok(steps)
}

/// 解析Markdown内容
pub fn parse_markdown(content: &str, file_path: &Path) -> Result<TestTemplate> {
    info!("开始解析Markdown内容");

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

    // 在返回前记录最终解析出的步骤 ID 列表
    let final_step_ids: Vec<String> = steps.iter().map(|s| s.id.clone()).collect();
    debug!("Parser finished for {}: Final parsed step IDs: {:?}", file_path.display(), final_step_ids);

    Ok(TestTemplate {
        metadata,
        steps,
        file_path: file_path.to_path_buf(),
        raw_content: content.to_string(),
    })
}

/// 解析步骤内容中的代码块和输出块
fn parse_blocks(content: &str, step_id: &str, header_depends_on: &Vec<String>, all_known_ids: &HashMap<String, String>, steps: &mut Vec<TestStep>) -> Result<()> {
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
        
        // 先进行属性解析，以便正确获取ID和其他属性
        let (block_id, description, executable, mut depends_on, assertions, extractions) = 
            parse_block_attributes(attributes, step_id)?;
        
        debug!("解析代码块属性结果: id={}", block_id);
        
        // 合并从标题继承的依赖关系
        // 如果代码块没有显式指定依赖，则继承标题的依赖
        if depends_on.is_empty() && !header_depends_on.is_empty() {
            debug!("代码块 {} 继承标题 {} 的依赖: {:?}", block_id, step_id, header_depends_on);
            depends_on = header_depends_on.clone();
        }
        
        debug!("解析代码块属性: id={}, description=\"{}\", executable={}, 依赖数量={}, 断言数量={}, 提取数量={}",
                block_id, description, executable, depends_on.len(), assertions.len(), extractions.len());
        
        // 验证依赖ID是否存在
        for dep_id in &depends_on {
            if !all_known_ids.contains_key(dep_id) {
                warn!("代码块 {} 依赖于未知ID: {}", block_id, dep_id);
            }
        }
        
        // 记录变量提取规则
        for extraction in &extractions {
            info!("代码块 {} 包含变量提取: {}=/{}/", block_id, extraction.variable, extraction.regex);
        }
        
        // 创建测试步骤
        let step = TestStep {
            id: block_id.clone(),
            description: Some(description),
            command: if language == "bash" || language == "sh" { Some(code.to_string()) } else { None },
            depends_on,
            assertions,
            extractions,
            executable,
            ref_command: None,
            raw_content: format!("```{} {{{}}}\n{}\n```", language, attributes, code),
        };
        
        debug!("添加代码块步骤: id={}", step.id);
        steps.push(step);
    }
    
    // 解析输出块
    for cap in output_block_re.captures_iter(content) {
        // 获取引用ID（可能在第一个或第二个捕获组）
        let ref_id = cap.get(1).or_else(|| cap.get(2)).map_or("unknown", |m| m.as_str());
        let placeholder = cap.get(3).unwrap().as_str();
        
        debug!("找到输出引用块: ref_id={}, placeholder内容长度={}", ref_id, placeholder.len());
        
        // 验证引用ID是否存在
        if !all_known_ids.contains_key(ref_id) {
            warn!("输出引用块依赖于未知ID: {}", ref_id);
        }
        
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
        
        debug!("添加输出引用步骤: id={}", step.id);
        steps.push(step);
    }
    
    Ok(())
}

/// 解析代码块属性
fn parse_block_attributes(attributes: &str, parent_id: &str) -> Result<(String, String, bool, Vec<String>, Vec<AssertionType>, Vec<DataExtraction>)> {
    debug!("解析代码块属性: {}", attributes);
    
    // 初始化返回值
    let mut id = String::new();
    let mut description = String::new();
    let mut executable = false;
    let mut depends_on = Vec::new();
    let mut assertions = Vec::new();
    let mut extractions = Vec::new();
    
    // 提取所有属性键值对
    let attributes_map = extract_attributes(attributes);
    
    // 记录找到的所有属性
    debug!("提取到 {} 个属性:", attributes_map.len());
    for (k, v) in &attributes_map {
        debug!("  {}=\"{}\"", k, v);
    }
    
    // 处理ID
    if let Some(value) = attributes_map.get("id") {
        id = value.clone();
        debug!("找到ID: {}", id);
    }
    
    // 处理描述
    if let Some(value) = attributes_map.get("description") {
        description = value.clone();
        debug!("找到描述: {}", description);
    }
    
    // 处理可执行标记
    if let Some(value) = attributes_map.get("exec") {
        executable = value == "true";
        debug!("找到可执行标记: {}", executable);
    }
    
    // 处理依赖关系
    if let Some(value) = attributes_map.get("depends_on") {
        let deps_str = value.trim().trim_matches(|c| c == '[' || c == ']');
        depends_on = deps_str.split(',')
            .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
            .filter(|s| !s.is_empty())
            .collect();
        debug!("找到依赖: {:?}", depends_on);
    }
    
    // 处理断言和提取 - 需要扫描所有以assert.和extract.开头的键
    for (key, value) in &attributes_map {
        if key.starts_with("assert.") {
            let assertion_type = key.trim_start_matches("assert.");
            let assertion = parse_assertion(assertion_type, value);
            if let Some(a) = assertion {
                debug!("找到断言: {:?}", a);
                assertions.push(a);
            }
        } else if key.starts_with("extract.") {
            let var_name = key.trim_start_matches("extract.");
            
            // 处理正则表达式格式，可能被/包裹
            let regex_str = if value.starts_with('/') && value.ends_with('/') {
                value[1..value.len()-1].to_string()
            } else {
                value.clone()
            };
            
            extractions.push(DataExtraction {
                variable: var_name.to_string(),
                regex: regex_str.clone(),
            });
            debug!("找到提取规则: {}=/{}/", var_name, regex_str);
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
    
    // 最终日志输出
    debug!("解析代码块属性完成: id={}, description=\"{}\", executable={}, 依赖数量={}, 断言数量={}, 提取数量={}",
            id, description, executable, depends_on.len(), assertions.len(), extractions.len());
    
    Ok((id, description, executable, depends_on, assertions, extractions))
}

/// 从属性字符串中提取所有键值对
fn extract_attributes(attributes: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    
    // 移除可能的大括号
    let attr_str = attributes.trim_start_matches('{').trim_end_matches('}');
    
    debug!("解析代码块属性: {}", attr_str);
    
    // 状态机变量
    enum ParseState {
        Key,        // 解析键
        Equal,      // 等号后
        Value,      // 解析没有引号的值
        QuoteValue, // 解析有引号的值
    }
    
    let mut state = ParseState::Key;
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut quote_char = ' '; // 当前引号类型 ' 或 "
    
    // 遍历字符
    let mut chars = attr_str.chars().peekable();
    while let Some(c) = chars.next() {
        match state {
            ParseState::Key => {
                if c == '=' {
                    // 遇到等号，切换到等号后状态
                    state = ParseState::Equal;
                } else if c.is_whitespace() {
                    // 键名后遇到空格
                    if !current_key.is_empty() {
                        // 视为无值的布尔属性
                        result.insert(current_key.trim().to_string(), "true".to_string());
                        current_key = String::new();
                    }
                } else {
                    // 继续收集键名
                    current_key.push(c);
                }
            },
            ParseState::Equal => {
                if c == '"' || c == '\'' {
                    // 等号后遇到引号，开始解析引号内的值
                    quote_char = c;
                    state = ParseState::QuoteValue;
                } else if c.is_whitespace() {
                    // 等号后的空格，忽略
                } else {
                    // 等号后开始收集非引号值
                    current_value.push(c);
                    state = ParseState::Value;
                }
            },
            ParseState::Value => {
                if c.is_whitespace() {
                    // 值后面遇到空格，表示值结束
                    result.insert(current_key.trim().to_string(), current_value.trim().to_string());
                    current_key = String::new();
                    current_value = String::new();
                    state = ParseState::Key;
                } else {
                    // 继续收集非引号值
                    current_value.push(c);
                }
            },
            ParseState::QuoteValue => {
                if c == quote_char {
                    // 遇到匹配的引号，引号值结束
                    result.insert(current_key.trim().to_string(), current_value.clone());
                    current_key = String::new();
                    current_value = String::new();
                    state = ParseState::Key;
                } else {
                    // 继续收集引号内的值
                    current_value.push(c);
                }
            }
        }
    }
    
    // 处理最后可能未完成的键值对
    if !current_key.is_empty() {
        if !current_value.is_empty() {
            result.insert(current_key.trim().to_string(), current_value.trim().to_string());
        } else {
            result.insert(current_key.trim().to_string(), "true".to_string());
        }
    }
    
    // 记录解析结果
    debug!("提取到 {} 个属性:", result.len());
    for (k, v) in &result {
        debug!("  {}=\"{}\"", k, v);
    }
    
    result
}

/// 解析断言
fn parse_assertion(assertion_type: &str, value: &str) -> Option<AssertionType> {
    match assertion_type {
        "exit_code" => {
            if let Ok(code) = value.parse::<i32>() {
                Some(AssertionType::ExitCode(code))
            } else {
                warn!("无效的exit_code值: {}", value);
                None
            }
        },
        "stdout_contains" => {
            Some(AssertionType::StdoutContains(value.to_string()))
        },
        "stdout_not_contains" => {
            Some(AssertionType::StdoutNotContains(value.to_string()))
        },
        "stdout_matches" => {
            Some(AssertionType::StdoutMatches(value.to_string()))
        },
        "stderr_contains" => {
            Some(AssertionType::StderrContains(value.to_string()))
        },
        "stderr_not_contains" => {
            Some(AssertionType::StderrNotContains(value.to_string()))
        },
        "stderr_matches" => {
            Some(AssertionType::StderrMatches(value.to_string()))
        },
        _ => {
            warn!("未知的断言类型: {}", assertion_type);
            None
        }
    }
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