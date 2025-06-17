//! Markdown测试模板解析器
//!
//! 这个模块负责解析Markdown格式的测试模板内容，识别其中的元数据、可执行代码块和特殊属性。

use anyhow::{anyhow, bail, Context, Result};
use log::{debug, error, info, warn};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::config::target_config::TargetConfig;
// Import the new ExecutionStep related types
use crate::template::step::{ExecutionStep, GlobalStepId, StepType};
// Import ParsedTestStep directly, ContentBlock is defined in this file
use crate::template::{
    AssertionType, DataExtraction, ParsedTestStep, TemplateMetadata, TemplateReference,
};
use crate::utils;

/// 表示模板文件内容的不同结构化块。
/// 解析器 (Parser) 会将原始模板字符串转换为 `Vec<ContentBlock>`。
#[derive(Debug, Clone, PartialEq)]
pub enum ContentBlock {
    /// YAML 前置元数据块。
    /// 存储的是 `---` 分隔符内部的原始 YAML 字符串。
    Metadata(String),

    /// 结构化的标题块（Heading），包含id、级别、文本和属性。
    HeadingBlock {
        /// 步骤唯一id（local id）
        id: String,
        /// 标题级别 (1-6)
        level: u8,
        /// 标题文本内容
        text: String,
        /// 标题的原始属性（如 {id="..."}）
        attributes: HashMap<String, String>,
    },

    /// 结构化的代码块，包含id、语言、代码内容和属性。
    CodeBlock {
        /// 步骤唯一id（local id）
        id: String,
        /// 代码块的语言标识 (如 "bash", "python")
        lang: String,
        /// 代码内容
        code: String,
        /// 代码块的原始属性（如 {id="...", depends_on="..."}）
        attributes: HashMap<String, String>,
    },

    /// 代表一个步骤输出的占位符。
    /// 例如 ` ```output {ref="step_id"} ... ``` `。
    OutputBlock { step_id: String, stream: String },

    /// 通用 Markdown 文本块。
    /// 这可以包含任何 Markdown内容，包括原始的步骤定义文本（如果它们不被特殊处理为其他类型的块）。
    Text(String),

    /// 一个标记，指示在此处应插入自动生成的步骤摘要表。
    SummaryTablePlaceholder,
}

/// 解析Markdown测试模板内容，返回元数据、执行步骤列表和内容块列表
pub fn parse_template_into_content_blocks_and_steps(
    content: &str,
    file_path: &Path,
) -> Result<(TemplateMetadata, Vec<ExecutionStep>, Vec<ContentBlock>)> {
    info!(
        "Starting to parse test template (structured content and steps): {}", // 开始解析测试模板 (结构化内容和步骤): {}
        file_path.display()
    );

    let mut content_blocks = Vec::new();

    let (yaml_front_matter, markdown_content) = extract_front_matter(content)?;
    debug!("YAML front matter length: {} bytes", yaml_front_matter.len()); // YAML前置数据长度: {} 字节
    debug!("Markdown content length: {} bytes", markdown_content.len()); // Markdown内容长度: {} 字节

    content_blocks.push(ContentBlock::Metadata(yaml_front_matter.clone()));

    let metadata = parse_metadata(&yaml_front_matter)?;
    info!(
        "Template metadata parsing completed: title=\"{}\", unit=\"{}\"", // 模板元数据解析完成: title=\"{}\", unit=\"{}\"
        metadata.title, metadata.unit_name
    );

    let template_id = utils::get_template_id_from_path(file_path);
    debug!("Generated template ID: {template_id}"); // 生成的模板 ID: {template_id}

    // 同时解析步骤和内容块
    let (execution_steps, md_content_blocks) =
        parse_markdown_to_steps_and_content_blocks(markdown_content, &template_id, &metadata)?;
    content_blocks.extend(md_content_blocks);

    info!(
        "Parsed {} execution steps and {} content blocks", // 已解析 {} 个执行步骤和 {} 个内容块
        execution_steps.len(),
        content_blocks.len()
    );

    for step in &execution_steps {
        debug!(
            "ExecutionStep: id={}, type={:?}, local_id={}, template_id={}, deps={:?}",
            step.id, step.step_type, step.local_id, step.template_id, step.dependencies
        );
        if let Some(parsed_step) = &step.original_parsed_step {
            debug!(
                "  Original Parsed Step: id={}, exec={}, assertions={}, extractions={}",
                parsed_step.id,
                parsed_step.executable,
                parsed_step.assertions.len(),
                parsed_step.extractions.len()
            );
        }
    }

    Ok((metadata, execution_steps, content_blocks))
}

/// 从Markdown内容中解析出 ExecutionSteps 和 ContentBlocks
fn parse_markdown_to_steps_and_content_blocks(
    markdown: &str,
    template_id: &str,
    metadata: &TemplateMetadata,
) -> Result<(Vec<ExecutionStep>, Vec<ContentBlock>)> {
    debug!("Starting to parse Markdown content into ExecutionSteps and ContentBlocks (template_id: {template_id})"); // 开始将Markdown内容解析为 ExecutionSteps 和 ContentBlocks (template_id: {template_id})
    let mut execution_steps: Vec<ExecutionStep> = Vec::new();
    let mut content_blocks = Vec::new();
    let mut all_local_ids: HashSet<String> = HashSet::new();
    let all_depends_refs: Vec<(String, String)> = Vec::new(); // (当前step global_id, depends_on的原始id)
    let heading_re = Regex::new(r"(?m)^(#+)\s+(.*?)(?:\s+\{([^}]*)\}\s*|\s*)$")?;
    let code_block_re = Regex::new(r"(?ms)```(bash)\s*(\{([^}]*)\})?\n(.*?)```")?;
    let output_block_re = match Regex::new(r#"(?ms)^```output\s*\{([^\r\n}]*)\}.*?^```\s*$"#) {
        Ok(re) => re,
        Err(e) => {
            error!("Failed to compile regex: {e}"); // 正则表达式编译失败: {e}
            return Err(anyhow!("Failed to compile regex: {}", e)); // 正则表达式编译失败: {}
        }
    };
    let summary_table_re = Regex::new(r#"(?im)^\s*<!--\s*LINTESOR_SUMMARY_TABLE\s*-->\s*$"#)?;

    let mut current_heading_stack: Vec<(GlobalStepId, u8, Vec<GlobalStepId>)> = Vec::new(); // (id, level, children)
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
            // 生成结构化 HeadingBlock
            if let Some(caps) = heading_re.captures(line) {
                let level = caps.get(1).map_or(0, |m| m.as_str().len() as u8);
                let text = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();
                let attributes_str = caps.get(3).map_or("", |m| m.as_str()).trim();
                let attributes = parse_inline_attributes(attributes_str);
                let local_id = attributes.get("id").cloned().unwrap_or_else(|| {
                    local_id_counter += 1;
                    format!("heading_{local_id_counter}")
                });
                all_local_ids.insert(local_id.clone());
                let global_id = format!("{template_id}::{local_id}");
                // 处理 heading stack，弹出比当前 level 大的 heading，并把子节点挂到 dependencies
                // 这里 current_heading_stack 是一个栈，保存了所有未闭合的 heading 及其 level 和子节点列表
                // 每遇到一个更高或同级 heading，就把栈顶 heading 弹出，并把它收集到的所有直接子节点（children）
                // 全部加入到该 heading 的 dependencies 字段，实现“父依赖所有直接子节点”
                while let Some((_, last_level, _)) = current_heading_stack.last_mut() {
                    if *last_level >= level {
                        // 弹出并更新 dependencies
                        let (parent_id, _, children) = current_heading_stack.pop().unwrap();
                        // 这里 parent_id 是 heading 的全局 id，children 是它的所有直接子节点 id
                        if let Some(parent_step) =
                            execution_steps.iter_mut().find(|s| s.id == parent_id)
                        {
                            for child in children {
                                // 将所有直接子节点加入 heading 的依赖
                                parent_step.dependencies.insert(child);
                            }
                        }
                    } else {
                        break;
                    }
                }
                // 只处理 heading 自己的 depends_on，结构性依赖全部通过 children 机制实现
                let mut dependencies = HashSet::new();
                if let Some(deps_str) = attributes.get("depends_on") {
                    // 插入依赖集
                    parse_depends_on_str(
                        deps_str,
                        &mut dependencies,
                        template_id,
                        &metadata.references,
                    );
                }
                content_blocks.push(ContentBlock::HeadingBlock {
                    id: local_id.clone(),
                    level,
                    text: text.clone(),
                    attributes: attributes.clone(),
                });
                execution_steps.push(ExecutionStep {
                    id: global_id.clone(),
                    template_id: template_id.to_string(),
                    local_id,
                    step_type: StepType::Heading {
                        level,
                        text,
                        attributes: attributes.clone(),
                    },
                    dependencies,
                    original_parsed_step: None,
                });
                current_heading_stack.push((global_id, level, Vec::new())); // 新 heading 入栈，准备收集子节点
            }
        } else if let Some(output_match) = captures.name("output_block") {
            if let Some(caps) = output_block_re.captures(output_match.as_str()) {
                debug!("Found output_block: {}", output_match.as_str()); // 发现 output_block: {}
                let attributes_str = caps.get(1).map_or("", |m| m.as_str());
                let attributes = parse_inline_attributes(attributes_str);
                let ref_id_attr = attributes
                    .get("ref")
                    .ok_or_else(|| anyhow!("output_block missing ref attribute"))?; // output_block 缺少 ref 属性
                let stream = match attributes.get("stream") {
                    Some(stream) => stream.to_string(),
                    _ => "stdout".to_string(),
                };
                content_blocks.push(ContentBlock::OutputBlock {
                    step_id: ref_id_attr.clone(),
                    stream,
                });
                let local_id = format!("{ref_id_attr}-outputplaceholder");
                all_local_ids.insert(local_id.clone());
                let global_id = format!("{template_id}::{local_id}");
                let mut dependencies = HashSet::new();
                let ref_global_id =
                    resolve_dependency_ref(ref_id_attr, template_id, &metadata.references);
                dependencies.insert(ref_global_id.clone());
                let parsed_step_info = ParsedTestStep {
                    id: local_id.clone(),
                    description: Some(format!("Placeholder for output of step {ref_id_attr}")),
                    command: None,
                    depends_on: vec![ref_id_attr.to_string()],
                    assertions: Vec::new(),
                    extractions: Vec::new(),
                    executable: false, // Not directly executable
                    ref_command: Some(ref_id_attr.to_string()),
                    raw_content: output_match.as_str().to_string(),
                    active: Some(true),
                    timeout_ms: None,
                };
                execution_steps.push(ExecutionStep {
                    id: global_id.clone(),
                    template_id: template_id.to_string(),
                    local_id,
                    step_type: StepType::OutputPlaceholder,
                    dependencies,
                    original_parsed_step: Some(parsed_step_info),
                });
                // 只让父 heading 的 children 收集这个 output block
                // 这样 heading 只依赖于自己直接的 output/code/heading 子节点
                if let Some((_, _, children)) = current_heading_stack.last_mut() {
                    children.push(global_id.clone());
                }
            }
        } else if captures.name("summary_table").is_some() {
            content_blocks.push(ContentBlock::SummaryTablePlaceholder);
        } else if let Some(code_match) = captures.name("code_block") {
            let block_content = code_match.as_str();
            let preliminary_caps = code_block_re.captures(block_content);
            let lang_for_check = preliminary_caps
                .as_ref()
                .and_then(|c| c.get(1))
                .map_or("", |m| m.as_str());
            let attrs_str_for_check = preliminary_caps
                .as_ref()
                .and_then(|c: &regex::Captures<'_>| c.get(2))
                .map_or("", |m| m.as_str());
            if lang_for_check == "output" && attrs_str_for_check.contains("ref=") {
                // skip, handled by output_block
            }
            if let Some(caps) = code_block_re.captures(block_content) {
                let lang = caps.get(1).map_or("", |m| m.as_str()).to_string();
                let attributes_str = caps.get(3).map_or("", |m| m.as_str());
                let command = caps.get(4).map_or("", |m| m.as_str()).trim().to_string();
                let attributes = parse_inline_attributes(attributes_str);
                let local_id = attributes.get("id").cloned().unwrap_or_else(|| {
                    local_id_counter += 1;
                    format!("codeblock_{local_id_counter}")
                });
                all_local_ids.insert(local_id.clone());
                content_blocks.push(ContentBlock::CodeBlock {
                    id: local_id.clone(),
                    lang: lang.clone(),
                    code: command.clone(),
                    attributes: attributes.clone(),
                });
                let global_id = format!("{template_id}::{local_id}");
                let mut dependencies = HashSet::new();
                if let Some(deps_str) = attributes.get("depends_on") {
                    // 插入依赖集
                    parse_depends_on_str(
                        deps_str,
                        &mut dependencies,
                        template_id,
                        &metadata.references,
                    );
                }
                let parsed_step_info = ParsedTestStep {
                    id: local_id.clone(),
                    description: attributes.get("description").cloned(),
                    command: Some(command.clone()),
                    depends_on: dependencies
                        .iter()
                        .map(|gsid| gsid.split("::").last().unwrap_or("").to_string())
                        .collect(),
                    assertions: parse_assertions_from_attributes(&attributes),
                    extractions: parse_extractions_from_attributes(&attributes),
                    executable: attributes
                        .get("exec")
                        .and_then(|v_str| v_str.parse::<bool>().ok())
                        .unwrap_or(true),
                    ref_command: None,
                    raw_content: block_content.to_string(),
                    active: attributes
                        .get("active")
                        .and_then(|v_str| v_str.parse::<bool>().ok()),
                    timeout_ms: attributes
                        .get("timeout_ms")
                        .and_then(|v_str| v_str.parse::<u64>().ok()),
                };
                execution_steps.push(ExecutionStep {
                    id: global_id.clone(),
                    template_id: template_id.to_string(),
                    local_id,
                    step_type: StepType::CodeBlock {
                        lang,
                        command,
                        attributes: attributes.clone(),
                    },
                    dependencies,
                    original_parsed_step: Some(parsed_step_info),
                });
                // 只让父 heading 的 children 收集这个 codeblock
                // 这样 heading 只依赖于自己直接的 codeblock/output/heading 子节点
                if let Some((_, _, children)) = current_heading_stack.last_mut() {
                    children.push(global_id.clone());
                }
            }
        }
        last_match_end = match_end;
    }
    // 处理所有未闭合 heading 的 children
    while let Some((parent_id, _, children)) = current_heading_stack.pop() {
        if let Some(parent_step) = execution_steps.iter_mut().find(|s| s.id == parent_id) {
            for child in children {
                parent_step.dependencies.insert(child);
            }
        }
    }
    if last_match_end < markdown.len() {
        let remaining_text = &markdown[last_match_end..];
        if !remaining_text.trim().is_empty() {
            content_blocks.push(ContentBlock::Text(remaining_text.to_string()));
        }
    }
    // 检查所有 depends_on 的 id 是否都存在
    for (from_id, dep_id) in &all_depends_refs {
        if !all_local_ids.contains(dep_id) {
            panic!("depends_on references non-existent step id: {dep_id} (from step {from_id})"); // depends_on 中引用了不存在的步骤 id: {dep_id} (from step {from_id})
        }
    }
    debug!(
        "Completed ExecutionSteps ({}) and ContentBlocks ({}) parsing", // 完成 ExecutionSteps ({}) 和 ContentBlocks ({}) 解析
        execution_steps.len(),
        content_blocks.len()
    );
    Ok((execution_steps, content_blocks))
}

/// 从Markdown内容中提取YAML前置数据
fn extract_front_matter(content: &str) -> Result<(String, &str)> {
    debug!("Extracting YAML front matter from template content"); // 从模板内容中提取YAML前置数据
    let re = Regex::new(r"(?s)^---\s*\n(.*?)\n---\s*\n(.*)$")?;

    match re.captures(content) {
        Some(caps) => {
            let yaml = caps.get(1).unwrap().as_str();
            let markdown = caps.get(2).unwrap().as_str();
            debug!("Successfully extracted YAML front matter"); // 成功提取YAML前置数据
            Ok((yaml.to_string(), markdown))
        }
        None => {
            error!("YAML front matter not found"); // 未找到YAML前置数据
            bail!("YAML front matter not found, format should be '---\\n<yaml>\\n---\\n<markdown>'") // 未找到YAML前置数据，格式应为 '---\\n<yaml>\\n---\\n<markdown>'
        }
    }
}

/// 解析YAML元数据
fn parse_metadata(yaml: &str) -> Result<TemplateMetadata> {
    debug!("Parsing YAML metadata"); // 解析YAML元数据
    let yaml_value: serde_yaml::Value =
        serde_yaml::from_str(yaml).with_context(|| "Unable to parse YAML front matter")?; // 无法解析YAML前置数据

    debug!("YAML parsing successful, starting field extraction"); // YAML解析成功，开始提取字段

    let title = yaml_value["title"]
        .as_str()
        .ok_or_else(|| anyhow!("Metadata missing 'title' field"))?  // 元数据缺少'title'字段
        .to_string();
    debug!("Extracted title: {title}"); // 提取title: {title}

    let target_config_str = yaml_value["target_config"]
        .as_str()
        .ok_or_else(|| anyhow!("Metadata missing 'target_config' field"))?; // 元数据缺少'target_config'字段
    debug!("Extracted target_config: {target_config_str}"); // 提取target_config: {target_config_str}

    let target_config = TargetConfig::from_file(target_config_str)
        .unwrap_or_else(|_| panic!("Unable to load target_config file: {target_config_str}")); // 无法加载 target_config 文件: {target_config_str}

    let unit_name = yaml_value["unit_name"]
        .as_str()
        .ok_or_else(|| anyhow!("Metadata missing 'unit_name' field"))? // 元数据缺少'unit_name'字段
        .to_string();
    debug!("Extracted unit_name: {unit_name}"); // 提取unit_name: {unit_name}

    let unit_version = yaml_value["unit_version"]
        .as_str()
        .ok_or_else(|| anyhow!("Metadata missing 'unit_version' field"))? // 元数据缺少'unit_version'字段
        .to_string();
    debug!("Extracted unit_version: {unit_version}"); // 提取unit_version: {unit_version}

    let tags = match yaml_value["tags"] {
        serde_yaml::Value::Sequence(ref seq) => {
            let tags: Vec<_> = seq
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            debug!("Extracted tags: {tags:?}"); // 提取tags: {tags:?}
            tags
        }
        _ => Vec::new(),
    };

    let references = match yaml_value["references"] {
        serde_yaml::Value::Sequence(ref seq) => {
            let mut refs = Vec::new();
            for ref_value in seq {
                if let serde_yaml::Value::Mapping(mapping) = ref_value {
                    // 为了在模板里看起来舒服，我们实际上的对应是按下面这样的
                    // template -> template_path
                    // as -> namespace
                    let template_path = mapping
                        .get(serde_yaml::Value::String("template".to_string()))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .ok_or_else(|| {
                            anyhow!("Item in references missing 'template(template_path)' field") // references中的项缺少'template(template_path)'字段
                        })?;

                    let namespace = mapping
                        .get(serde_yaml::Value::String("as".to_string()))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .ok_or_else(|| anyhow!("Item in references missing 'as(namespace)' field"))?; // references中的项缺少'as(namespace)'字段

                    debug!("Extracted template reference: template_path={template_path}, namespace={namespace}"); // 提取模板引用: template_path={template_path}, namespace={namespace}
                    refs.push(TemplateReference {
                        template_path,
                        namespace,
                    });
                }
            }
            refs
        }
        _ => Vec::new(),
    };

    if !references.is_empty() {
        debug!("Extracted {} external template references in total", references.len()); // 共提取到 {} 个外部模板引用
    }

    let mut custom = HashMap::new();
    if let serde_yaml::Value::Mapping(mapping) = &yaml_value {
        for (key, value) in mapping {
            if let Some(key_str) = key.as_str() {
                if [
                    "title",
                    "target_config",
                    "unit_name",
                    "unit_version",
                    "tags",
                    "references",
                ]
                .contains(&key_str)
                {
                    continue;
                }

                if let Some(value_str) = value.as_str() {
                    debug!("Extracted custom field: {key_str} = {value_str}"); // 提取自定义字段: {key_str} = {value_str}
                    custom.insert(key_str.to_string(), value_str.to_string());
                }
            }
        }
    }

    Ok(TemplateMetadata {
        title,
        target_config,
        unit_name,
        unit_version,
        tags,
        references,
        custom,
    })
}

/// 提取 depends_on 字符串中的单个依赖 id（去除 namespace，仅返回本地 id）
/// 意思是暂时不考虑跨 namespace 的依赖
fn extract_dep_id_from_dep_str(dep_str: &str) -> &str {
    // 依赖 id 的格式是 "namespace::local_id" 或者 "local_id"
    let dep_str = dep_str.trim().trim_matches('"').trim_matches('\'');

    if dep_str.contains("::") {
        dep_str.split("::").last().unwrap_or("")
    } else {
        dep_str
    }
}

/// Helper to parse inline attributes like id="foo" exec="true" assert.exit.code=0 extract.lintestor=/Lintestor/
/// 这个函数解析类似于 id="foo" exec="true" assert.exit.code=0 extract.lintestor=/Lintestor/ 的内联属性
/// 实现方式是使用有限状态机来解析键值对，内部状态似乎没什么复用的可能性所以 State 就不对外暴露了
/// 注意我们在状态里没考虑 { 和 } 所以不许传入整个带 {} 的 attr_str
fn parse_inline_attributes(input: &str) -> HashMap<String, String> {
    debug!("Parsing inline attributes: {input}"); // 解析内联属性: {input}

    let mut result = HashMap::new();
    let mut chars = input.chars().peekable();
    let mut key = String::new();
    let mut value = String::new();

    if input.is_empty() {
        // Early return if input is empty, because of state machine expect {} mustly
        return result;
    }

    enum State {
        Start,
        Key,
        ExpectEq,
        ValueStart,
        StringValue,
        StringEscape,
        RegexValue,
        RegexEscape,
        RegexFlags,
        ListValue { bracket_level: usize },
        Bareword,
        // Done, // Not strictly needed if loop handles EOF
    }

    let mut state = State::Start;

    loop {
        match state {
            State::Start => {
                if let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() {
                        chars.next();
                    } else {
                        key.clear();
                        value.clear();
                        state = State::Key;
                    }
                } else {
                    // EOF reached, exit the loop
                    debug!("Reached EOF in Start state, exiting loop");
                    break;
                }
            }
            State::Key => {
                if let Some(&ch) = chars.peek() {
                    if ch == '=' {
                        state = State::ExpectEq;
                    } else if ch.is_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
                        key.push(ch);
                        chars.next();
                    } else if ch.is_whitespace() {
                        chars.next();
                        state = State::ExpectEq;
                    } else {
                        if key.is_empty() {
                            panic!("Unexpected character '{ch}' at start of key or empty key before '='");
                        }
                        panic!("Unexpected character '{ch}' after key '{key}', expected '=' or whitespace or '}}'");
                    }
                } else {
                    if !key.is_empty() {
                        panic!("Unexpected EOF after key: {key}");
                    }
                    // EOF after attributes have started, but before '}'
                    panic!("Unexpected EOF, expected attributes or '}}'");
                }
            }
            State::ExpectEq => {
                if let Some(&ch) = chars.peek() {
                    if ch == '=' {
                        chars.next();
                        state = State::ValueStart;
                    } else if ch.is_whitespace() {
                        chars.next();
                    } else {
                        panic!("Expected '=' after key '{key}', found '{ch}'");
                    }
                } else {
                    panic!("Unexpected EOF after key '{key}', expected '='");
                }
            }
            State::ValueStart => {
                if let Some(&ch) = chars.peek() {
                    match ch {
                        '"' => {
                            chars.next();
                            value.clear();
                            state = State::StringValue;
                        }
                        '/' => {
                            chars.next();
                            value.clear();
                            value.push('/');
                            state = State::RegexValue;
                        }
                        '[' => {
                            chars.next();
                            value.clear();
                            state = State::ListValue { bracket_level: 1 };
                        }
                        c if c.is_whitespace() => {
                            chars.next();
                        }
                        _ => {
                            value.clear();
                            state = State::Bareword;
                        }
                    }
                } else {
                    panic!("Unexpected EOF for key '{key}', expected a value.");
                }
            }
            State::StringValue => {
                if let Some(&ch) = chars.peek() {
                    match ch {
                        '\\' => {
                            chars.next();
                            state = State::StringEscape;
                        }
                        '"' => {
                            chars.next();
                            result.insert(key.clone(), value.clone());
                            state = State::Start; // Go back to start to look for more attributes or '}'
                        }
                        _ => {
                            value.push(ch);
                            chars.next();
                        }
                    }
                } else {
                    panic!("Unexpected EOF in string value for key '{key}'");
                }
            }
            State::StringEscape => {
                if let Some(&ch) = chars.peek() {
                    value.push(ch);
                    chars.next();
                    state = State::StringValue;
                } else {
                    panic!("Unexpected EOF after string escape for key '{key}'");
                }
            }
            State::RegexValue => {
                if let Some(&ch) = chars.peek() {
                    match ch {
                        '\\' => {
                            chars.next();
                            state = State::RegexEscape;
                        }
                        '/' => {
                            chars.next();
                            value.push('/');
                            state = State::RegexFlags;
                        }
                        _ => {
                            value.push(ch);
                            chars.next();
                        }
                    }
                } else {
                    panic!("Unexpected EOF in regex value for key '{key}'");
                }
            }
            State::RegexEscape => {
                if let Some(&ch) = chars.peek() {
                    value.push('\\');
                    value.push(ch);
                    chars.next();
                    state = State::RegexValue;
                } else {
                    panic!("Unexpected EOF after regex escape for key '{key}'");
                }
            }
            State::RegexFlags => {
                let mut flags_str = String::new();
                while let Some(&ch_flag) = chars.peek() {
                    if ch_flag.is_alphabetic() {
                        flags_str.push(ch_flag);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if !flags_str.is_empty() {
                    value.push_str(&flags_str);
                }
                result.insert(key.clone(), value.clone());
                state = State::Start; // Go back to start
            }
            State::ListValue { bracket_level } => {
                if let Some(&ch) = chars.peek() {
                    match ch {
                        '[' => {
                            let new_level = bracket_level + 1;
                            value.push(ch); // [
                            chars.next();
                            state = State::ListValue {
                                bracket_level: new_level,
                            };
                        }
                        ']' => {
                            let new_level = bracket_level - 1;
                            value.push(ch); // ]
                            chars.next();
                            if new_level == 0 {
                                result.insert(key.clone(), value.trim().to_string());
                                state = State::Start;
                            } else {
                                state = State::ListValue {
                                    bracket_level: new_level,
                                };
                            }
                        }
                        ',' => {
                            // 逗号分隔，直接加入
                            value.push(ch);
                            chars.next();
                        }
                        '"' => {
                            // 进入字符串子状态，允许list里有string
                            chars.next();
                            value.push('"');
                            // 读取直到下一个未转义的引号
                            while let Some(&c) = chars.peek() {
                                value.push(c);
                                chars.next();
                                if c == '"' {
                                    // 检查是否为转义
                                    let mut backslash_count = 0;
                                    for bc in value.chars().rev().skip(1) {
                                        if bc == '\\' {
                                            backslash_count += 1;
                                        } else {
                                            break;
                                        }
                                    }
                                    if backslash_count % 2 == 0 {
                                        break;
                                    }
                                }
                            }
                        }
                        c if c.is_whitespace() => {
                            // 跳过空白
                            chars.next();
                        }
                        _ => {
                            // bareword
                            value.push(ch);
                            chars.next();
                        }
                    }
                } else {
                    panic!("Unexpected EOF in list value for key '{key}': missing closing ']'");
                }
            }
            State::Bareword => {
                if let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() {
                        chars.next();
                        result.insert(key.clone(), value.clone());
                        state = State::Start; // Go back to start
                    } else {
                        value.push(ch);
                        chars.next();
                    }
                } else {
                    // EOF, this is the end of the input and also the end of the bareword
                    result.insert(key.clone(), value.clone());
                    // Since input ended, we expect it to be implicitly closed
                    break;
                }
            }
        }
    }

    debug!("Inline attribute parsing completed: {input} -> {result:?}"); // 解析内联属性完成: {input} -> {result:?}

    if chars.peek().is_some() {
        // If we reach here and there are still characters left, it means we didn't close the attributes properly
        panic!("Unclosed attribute definition, possibly missing '}}'"); // 未闭合的属性定义，可能缺少 '}}'
    }

    result
}

/// Helper to parse "depends_on" string and populate dependencies set
fn parse_depends_on_str(
    deps_str: &str,
    dependencies: &mut HashSet<GlobalStepId>,
    current_template_id: &str,
    references: &[TemplateReference],
) {
    let deps_list_str = deps_str.trim_matches(|c| c == '[' || c == ']');
    for dep_item_str in deps_list_str.split(',') {
        let trimmed_dep = extract_dep_id_from_dep_str(dep_item_str);
        if !trimmed_dep.is_empty() {
            // 收集依赖之把这个 block 显式在 depends_on 声明的的所有依赖都放到 dependencies 里
            dependencies.insert(resolve_dependency_ref(
                trimmed_dep,
                current_template_id,
                references,
            ));
        }
    }
}

/// Helper to resolve a dependency reference (e.g., "step_id" or "namespace::step_id") to a GlobalStepId
fn resolve_dependency_ref(
    dep_ref: &str,
    current_template_id: &str,
    references: &[TemplateReference],
) -> GlobalStepId {
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
                    return format!("{referenced_template_file_name}::{local_step_id}");
                }
            }
            return dep_ref.to_string();
        }
    }
    format!("{current_template_id}::{dep_ref}")
}

/// Helper to parse assertions from a HashMap of attributes
fn parse_assertions_from_attributes(attributes: &HashMap<String, String>) -> Vec<AssertionType> {
    let mut assertions = Vec::new();
    for (key, value) in attributes {
        if key.starts_with("assert.") {
            let assertion_key = key.trim_start_matches("assert.");
            match assertion_key {
                "exit_code" => {
                    if let Ok(code) = value.parse::<i32>() {
                        assertions.push(AssertionType::ExitCode(code));
                    }
                }
                "stdout_contains" => assertions.push(AssertionType::StdoutContains(value.clone())),
                "stdout_not_contains" => {
                    assertions.push(AssertionType::StdoutNotContains(value.clone()))
                }
                "stdout_matches" => assertions.push(AssertionType::StdoutMatches(value.clone())),
                "stderr_contains" => assertions.push(AssertionType::StderrContains(value.clone())),
                "stderr_not_contains" => {
                    assertions.push(AssertionType::StderrNotContains(value.clone()))
                }
                "stderr_matches" => assertions.push(AssertionType::StderrMatches(value.clone())),
                _ => warn!("Unknown assertion type: {key}"), // 未知断言类型: {key}
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
                value[1..value.len() - 1].to_string()
            } else {
                value.clone()
            };
            extractions.push(DataExtraction {
                variable: var_name,
                regex: regex_str,
            });
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
target_config: "tests/test_files/local_target.toml"
unit_name: "MyUnit"
unit_version: "1.0.0"
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
        assert_eq!(metadata.target_config.get_name(), "Local Test");
        assert_eq!(metadata.unit_name, "MyUnit");
        assert_eq!(metadata.unit_version, "1.0.0");
        assert_eq!(
            metadata.tags,
            vec!["core".to_string(), "feature-abc".to_string()]
        );
        assert_eq!(metadata.references.len(), 2);
        assert_eq!(
            metadata.references[0].template_path,
            "external_template_1.md"
        );
        assert_eq!(metadata.references[0].namespace, "namespace1");
        assert_eq!(
            metadata.references[1].template_path,
            "external_template_2.md"
        );
        assert_eq!(metadata.references[1].namespace, "namespace2");
        assert_eq!(
            metadata.custom.get("custom_field"),
            Some(&"custom value".to_string())
        );
    }
}
