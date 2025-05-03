//! 处理Markdown测试模板的模块。
//! 
//! 这个模块负责解析、验证和管理Markdown格式的测试模板，
//! 这些模板定义了针对特定单元在特定目标上的测试步骤和预期结果。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context, bail};

mod parser;
mod executor;
mod reporter;
mod discovery;

pub use parser::parse_template;
pub use executor::{TemplateExecutor, ExecutionResult, ExecutorOptions};
pub use reporter::Reporter;
pub use discovery::{discover_templates, filter_templates, TemplateFilter};

/// Markdown测试模板元数据（YAML前置数据）
#[derive(Debug, Clone)]
pub struct TemplateMetadata {
    /// 测试标题
    pub title: String,
    /// 目标配置文件路径（相对于工作区根目录）
    pub target_config: PathBuf,
    /// 测试单元名称
    pub unit_name: String,
    /// 获取单元版本的命令（可选）
    pub unit_version_command: Option<String>,
    /// 测试标签列表
    pub tags: Vec<String>,
    /// 其他自定义元数据
    pub custom: HashMap<String, String>,
}

/// 测试断言类型
#[derive(Debug, Clone)]
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

/// 数据提取定义
#[derive(Debug, Clone)]
pub struct DataExtraction {
    /// 变量名
    pub variable: String,
    /// 用于提取数据的正则表达式
    pub regex: String,
}

/// 测试步骤
#[derive(Debug, Clone)]
pub struct TestStep {
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
    Blocked,
    /// 尚未执行
    NotRun,
}

/// 步骤执行结果
#[derive(Debug, Clone)]
pub struct StepResult {
    /// 步骤ID
    pub id: String,
    /// 执行状态
    pub status: StepStatus,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: i32,
    /// 提取的变量
    pub extracted_vars: HashMap<String, String>,
}

/// Markdown测试模板
#[derive(Debug, Clone)]
pub struct TestTemplate {
    /// 模板元数据
    pub metadata: TemplateMetadata,
    /// 测试步骤
    pub steps: Vec<TestStep>,
    /// 模板文件路径
    pub file_path: PathBuf,
    /// 原始模板内容
    pub raw_content: String,
}

/// 模板执行上下文
#[derive(Debug, Clone)]
pub struct TemplateContext {
    /// 模板
    pub template: TestTemplate,
    /// 步骤结果
    pub results: HashMap<String, StepResult>,
    /// 提取的变量
    pub variables: HashMap<String, String>,
    /// 特殊变量
    pub special_vars: HashMap<String, String>,
}

impl TestTemplate {
    /// 从模板文件路径创建测试模板
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("无法读取模板文件: {}", path.display()))?;
        
        let mut template = parse_template(&content)
            .with_context(|| format!("解析模板失败: {}", path.display()))?;
        
        template.file_path = path.to_path_buf();
        
        Ok(template)
    }
    
    /// 验证模板的完整性和正确性
    pub fn validate(&self) -> Result<()> {
        // 检查是否有循环依赖
        // 检查是否引用了不存在的步骤ID
        // 其他验证逻辑...
        Ok(())
    }
}

impl TemplateContext {
    /// 创建新的模板执行上下文
    pub fn new(template: TestTemplate) -> Self {
        let mut special_vars = HashMap::new();
        // 添加执行日期
        let now = chrono::Local::now();
        special_vars.insert("execution_date".to_string(), now.format("%Y-%m-%d %H:%M:%S").to_string());
        
        Self {
            template,
            results: HashMap::new(),
            variables: HashMap::new(),
            special_vars,
        }
    }
    
    /// 获取变量值（包括提取的变量和特殊变量）
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name).or_else(|| self.special_vars.get(name))
    }
    
    /// 替换字符串中的变量引用
    pub fn replace_variables(&self, text: &str) -> String {
        // 简单的实现，实际代码中可能需要更健壮的解析逻辑
        let mut result = text.to_string();
        
        // 匹配 {{ variable_name }} 格式
        for (name, value) in self.variables.iter() {
            let pattern = format!("{{{{ {} }}}}", name);
            result = result.replace(&pattern, value);
        }
        
        for (name, value) in self.special_vars.iter() {
            let pattern = format!("{{{{ {} }}}}", name);
            result = result.replace(&pattern, value);
        }
        
        result
    }
    
    /// 获取步骤状态
    pub fn get_step_status(&self, step_id: &str) -> StepStatus {
        match self.results.get(step_id) {
            Some(result) => result.status.clone(),
            None => StepStatus::NotRun,
        }
    }
}