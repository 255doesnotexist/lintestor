//! 处理Markdown测试模板的模块。
//!
//! 这个模块负责解析、验证和管理Markdown格式的测试模板，
//! 这些模板定义了针对特定单元在特定目标上的测试步骤和预期结果。

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod batch_executor;
mod dependency;
mod discovery;
pub mod executor; // Changed to public
mod parser;
mod reporter;
mod variable;

// Re-export types from step.rs
pub mod step;
pub use step::{ExecutionStep, GlobalStepId};

use crate::config::target_config::TargetConfig;
use crate::utils;
// Import ContentBlock from parser, and the new parsing function
pub use batch_executor::BatchExecutor;
pub use discovery::{discover_templates, filter_templates, TemplateFilter};
pub use executor::{ExecutionResult, ExecutorOptions};
pub use parser::ContentBlock;
pub use variable::VariableManager; // Added StepDependencyManager

/// Options for controlling batch execution
#[derive(Debug, Clone, Default)]
pub struct BatchOptions {
    /// Directory where reports should be saved.
    pub report_directory: Option<PathBuf>,
    /// Default command timeout in seconds for steps in the batch.
    /// Can be overridden by individual step timeouts.
    pub command_timeout_seconds: Option<u64>,
    /// Whether to continue executing other steps in a template if one step fails.
    pub continue_on_error: bool,
}

/// 外部模板引用
#[derive(Debug, Clone)]
pub struct TemplateReference {
    /// 引用的模板路径（相对于tests目录）
    /// template_path 这里在实际的 metadata 中对应 "template" 键
    pub template_path: String,
    /// 命名空间（用于变量引用）
    /// namespace 这里在实际的 metadata 中对应 "as" 键
    pub namespace: String,
}

/// Markdown测试模板元数据（YAML前置数据）
#[derive(Debug, Clone)]
pub struct TemplateMetadata {
    /// 测试标题
    pub title: String,
    /// 目标配置文件路径（相对于工作区根目录）
    pub target_config: TargetConfig,
    /// 测试单元名称
    pub unit_name: String,
    /// 单元版本字符串
    pub unit_version: String,
    /// 测试标签列表
    pub tags: Vec<String>,
    /// 引用的外部模板列表
    pub references: Vec<TemplateReference>,
    /// 其他自定义元数据
    pub custom: HashMap<String, String>,
}

/// 测试步骤 (原 TestStep，现为 ParsedTestStep，代表解析出的原始步骤信息)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ParsedTestStep {
    /// 步骤ID
    pub id: String,
    /// 步骤描述
    pub description: Option<String>,
    /// 执行命令
    pub command: Option<String>,
    /// 依赖的步骤ID列表
    pub depends_on: Vec<String>,
    /// 断言列表
    pub assertions: Vec<AssertionType>,
    /// 数据提取列表
    pub extractions: Vec<DataExtraction>,
    /// 是否为可执行步骤
    pub executable: bool,
    /// 引用的命令ID（用于输出块）
    pub ref_command: Option<String>,
    /// 步骤的原始Markdown内容
    pub raw_content: String,
    /// Whether the step is active and should be run (parsed from attributes)
    pub active: Option<bool>,
    /// Timeout for the step in milliseconds (parsed from attributes)
    pub timeout_ms: Option<u64>,
}

/// 测试断言类型
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum AssertionType {
    /// 检查命令退出码
    ExitCode(i32),
    /// 检查标准输出是否包含特定文本
    StdoutContains(String),
    /// 检查标准输出是否不包含特定文本
    StdoutNotContains(String),
    /// 检查标准输出是否匹配正则表达式
    StdoutMatches(String),
    /// 检查标准错误是否包含特定文本
    StderrContains(String),
    /// 检查标准错误是否不包含特定文本
    StderrNotContains(String),
    /// 检查标准错误是否匹配正则表达式
    StderrMatches(String),
}

/// 数据提取定义：说是数据提取，其实是用来从输出中提取这个变量的正则表达式
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DataExtraction {
    /// 变量名
    pub variable: String,
    /// 用于提取数据的正则表达式
    pub regex: String,
}

/// 测试步骤执行状态
#[derive(Debug, Clone, PartialEq)]
pub enum StepStatus {
    /// 通过
    Pass,
    /// 失败
    Fail,
    /// 由于依赖失败而跳过
    Skipped,
    /// 由于依赖尚未执行而阻塞
    #[allow(dead_code)]
    Blocked,
    /// 尚未执行
    #[allow(dead_code)]
    NotRun,
} // Blocked 和 NotRun 是为将来可能的异步 or 并行执行保留的状态，当前实现还没用上

impl StepStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            StepStatus::Pass => "Pass",
            StepStatus::Fail => "Fail",
            StepStatus::Skipped => "Skipped",
            StepStatus::Blocked => "Blocked",
            StepStatus::NotRun => "NotRun",
        }
    }
}

// 这个 StepStatus 的实现是为了方便在报告中输出状态字符串，会把状态关联到 template_id::step_id::status.execution/assertion 的格式
// 变量名允许带点（如 status.execution），查找时整体作为变量名处理，不做特殊分割

/// 步骤执行结果
#[derive(Debug, Clone)]
pub struct StepResult {
    /// 步骤ID (全局唯一)
    pub id: GlobalStepId,
    /// 步骤描述
    pub description: Option<String>,
    /// 执行状态
    pub status: StepStatus,
    /// 标准输出
    pub stdout: Option<String>,
    /// 标准错误
    pub stderr: Option<String>,
    /// 退出码
    pub exit_code: Option<i32>,
    /// 执行耗时 (毫秒)
    pub duration_ms: Option<u128>,
    /// 断言错误信息
    pub assertion_error: Option<String>,
    /// 提取的变量
    pub extracted_vars: HashMap<String, String>,
}

/// Markdown测试模板
#[derive(Debug, Clone)]
pub struct TestTemplate {
    /// 模板元数据
    pub metadata: TemplateMetadata,
    /// 测试步骤 (Now Vec<ExecutionStep> instead of Vec<TestStep>)
    pub steps: Vec<ExecutionStep>,
    /// 模板文件路径
    pub file_path: PathBuf,
    /// 原始模板内容
    pub raw_content: String, // Keep for now, might be useful for debugging or other purposes
    /// 结构化的内容块，用于报告生成
    pub content_blocks: Vec<ContentBlock>,
}

impl TestTemplate {
    /// 获取模板ID（文件名，转换为纯字母和下划线格式）
    pub fn get_template_id(&self) -> String {
        
        utils::get_template_id_from_path(&self.file_path)
    }

    /// 从模板文件路径创建测试模板
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("无法读取模板文件: {}", path.display()))?;

        // Use the new parser function that returns content_blocks as well
        let (metadata, steps, content_blocks) =
            parser::parse_template_into_content_blocks_and_steps(&content, path)
                .with_context(|| format!("解析模板失败: {}", path.display()))?;

        Ok(TestTemplate {
            metadata,
            steps,
            file_path: path.to_path_buf(),
            raw_content: content.to_string(),
            content_blocks, // Store the parsed content blocks
        })
    }
}
