//! 定义测试执行的基本单元：步骤 (Step)
//!
//! 一个测试模板被分解为一系列的步骤，这些步骤可以是文档中的标题，
//! 也可以是可执行的代码块。步骤之间存在依赖关系，执行器会按照
//! 依赖关系顺序执行这些步骤。

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

use crate::template::ParsedTestStep; // Import the original ParsedTestStep from template module // 从 template 模块导入原始的 ParsedTestStep

/// 全局步骤ID类型别名
///
/// 格式通常为: "TEMPLATE_ID::LOCAL_STEP_ID"
/// 例如: "my_template_file::section_1_code_block"
/// TEMPLATE_ID 可以是模板文件名（不含扩展名）或在元数据中定义的唯一ID。
/// LOCAL_STEP_ID 是步骤在模板内的唯一标识符，通常来自代码块的 `{id="..."}` 属性，
/// 或者为标题自动生成。
pub type GlobalStepId = String;

/// 定义一个执行步骤的具体类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] // Removed Hash
pub enum StepType {
    /// 表示一个Markdown文档中的标题 (例如 H1, H2 等)
    /// 标题本身不执行命令，但作为组织结构和依赖关系的一部分。
    Heading {
        /// 标题级别 (1-6)
        level: u8,
        /// 标题的文本内容
        text: String,
        /// 标题行原始属性 (例如 {id="...", description="..."})
        attributes: HashMap<String, String>,
    },
    /// 表示一个可执行的代码块
    CodeBlock {
        /// 代码块的语言标识 (例如 "bash", "python")
        lang: String,
        /// 要执行的命令或脚本内容
        command: String,
        /// 代码块的原始属性 (例如 {id="...", exec="true", depends_on="..."})
        attributes: HashMap<String, String>,
    },
    /// 表示一个步骤输出的占位符，其内容将在报告生成时填充。
    OutputPlaceholder,
}

/// 代表依赖图中的一个节点，即一个可执行或概念性的步骤
///
/// 这个结构体在解析阶段由 `parser.rs` 创建，并在 `dependency/mod.rs` 中用于构建依赖图。
/// `BatchExecutor` 则使用这个结构体（通过依赖管理器获取）来指导执行。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)] // Removed Hash
pub struct ExecutionStep {
    /// 全局唯一的步骤ID (e.g., "template_id::local_step_id")
    pub id: GlobalStepId,
    /// 此步骤所属的模板的唯一ID
    pub template_id: String,
    /// 步骤在其模板内的局部ID (例如，从 `{id="..."}` 获取，或为标题自动生成)
    pub local_id: String,
    /// 步骤的类型 (标题或代码块)
    pub step_type: StepType,
    /// 此步骤显式或隐式依赖的其他步骤的 `GlobalStepId` 集合。
    /// 这个字段在解析时初步填充（显式依赖），并在依赖分析阶段进一步完善（隐式依赖）。
    pub dependencies: HashSet<GlobalStepId>,
    /// 如果此步骤是代码块或输出块，则这里存储从Markdown解析出来的原始 `ParsedTestStep` 结构。
    /// `ParsedTestStep` 包含了如断言、变量提取规则、原始命令、是否可执行等详细信息。
    /// 对于标题类型的步骤，此字段为 `None`。
    pub original_parsed_step: Option<ParsedTestStep>,
    // 未来可以添加更多信息，如步骤在源文件中的行号等，用于错误报告。
    // pub line_number: Option<usize>,
}

impl ExecutionStep {
    /// 辅助函数，获取步骤的描述信息
    /// 对于代码块，尝试从其原始属性或 `ParsedTestStep` 中获取。
    /// 对于标题，描述就是其文本。
    pub fn description(&self) -> String {
        match &self.step_type {
            StepType::Heading { text, attributes, .. } => {
                attributes.get("description").cloned().unwrap_or_else(|| text.clone())
            }
            StepType::CodeBlock { attributes, .. } => {
                if let Some(desc) = attributes.get("description") {
                    return desc.clone();
                }
                if let Some(parsed_step) = &self.original_parsed_step {
                    if let Some(desc) = &parsed_step.description {
                        return desc.clone();
                    }
                }
                // 如果都没有，返回一个默认值或基于命令的摘要
                if let StepType::CodeBlock { command, .. } = &self.step_type {
                    let cmd_summary = command.lines().next().unwrap_or("").trim();
                    if cmd_summary.len() > 50 {
                        format!("{}...", &cmd_summary[..50])
                    } else if !cmd_summary.is_empty() {
                        cmd_summary.to_string()
                    } else {
                        format!("Code Block ({})", self.local_id)
                    }
                } else {
                     // Should not happen if step_type is CodeBlock
                    "Code Block".to_string()
                }
            }
            StepType::OutputPlaceholder => {
                if let Some(parsed_step) = &self.original_parsed_step {
                    if let Some(desc) = &parsed_step.description {
                        return desc.clone();
                    }
                }
                format!("Output Placeholder for ({})", self.local_id)
            }
        }
    }
}