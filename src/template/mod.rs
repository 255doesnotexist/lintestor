//! 处理Markdown测试模板的模块。
//! 
//! 这个模块负责解析、验证和管理Markdown格式的测试模板，
//! 这些模板定义了针对特定单元在特定目标上的测试步骤和预期结果。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context, bail};
use regex::Regex;

mod parser;
mod executor;
mod reporter;
mod discovery;
mod template_dependency_manager;

use log::{warn, debug};
pub use parser::parse_template;
pub use executor::{TemplateExecutor, ExecutionResult, ExecutorOptions};
pub use reporter::Reporter;
pub use discovery::{discover_templates, filter_templates, TemplateFilter};
pub use template_dependency_manager::TemplateDependencyManager;

/// 外部模板引用
#[derive(Debug, Clone)]
pub struct TemplateReference {
    /// 引用的模板路径（相对于tests目录）
    pub template_path: String,
    /// 命名空间（用于变量引用）
    pub namespace: String,
}

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
    /// 引用的外部模板列表
    pub references: Vec<TemplateReference>,
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

/// 模板上下文，用于存储执行过程中的变量和结果
#[derive(Debug, Clone)]
pub struct TemplateContext {
    /// 当前模板
    pub template: TestTemplate,
    /// 变量表
    pub variables: HashMap<String, String>,
    /// 特殊变量表（不可被覆盖）
    pub special_vars: HashMap<String, String>,
    /// 外部变量表（按命名空间分组）
    pub external_vars: HashMap<String, HashMap<String, String>>,
    /// 执行结果
    pub results: HashMap<String, StepResult>,
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
    /// 创建新的模板上下文
    pub fn new(template: TestTemplate) -> Self {
        let now = chrono::Local::now();
        
        let mut special_vars = HashMap::new();
        // 添加执行时间特殊变量
        special_vars.insert("execution_date".to_string(), now.format("%Y-%m-%d").to_string());
        special_vars.insert("execution_time".to_string(), now.format("%H:%M:%S").to_string());
        special_vars.insert("execution_datetime".to_string(), now.format("%Y-%m-%d %H:%M:%S").to_string());
        
        Self {
            template,
            variables: HashMap::new(),
            special_vars,
            external_vars: HashMap::new(),
            results: HashMap::new(),
        }
    }
    
    /// 获取步骤执行状态
    pub fn get_step_status(&self, step_id: &str) -> StepStatus {
        // 检查是否包含命名空间引用（如 namespace::step_id）
        if step_id.contains("::") {
            // 不做任何特殊处理，直接按完整ID查找
            match self.results.get(step_id) {
                Some(result) => result.status.clone(),
                None => StepStatus::NotRun,
            }
        } else {
            // 原始逻辑，按本地ID查找
            match self.results.get(step_id) {
                Some(result) => result.status.clone(),
                None => StepStatus::NotRun,
            }
        }
    }
    
    /// 设置外部变量（从外部引用模板中加载）
    pub fn set_external_variables(&mut self, namespace: &str, variables: HashMap<String, String>) {
        // 直接将变量添加到主变量表中，使用命名空间作为前缀
        for (key, value) in variables.clone() {
            let namespaced_key = format!("{}::{}", namespace, key);
            self.variables.insert(namespaced_key, value);
        }
        
        // 同时保留原有的命名空间分组存储，便于完整性和调试
        self.external_vars.insert(namespace.to_string(), variables);
    }
    
    /// 替换文本中的变量引用
    ///
    /// 支持多种变量引用格式:
    /// - ${variable_name} - 标准变量引用
    /// - ${namespace::variable_name} - 带命名空间的变量引用
    /// - {{ variable_name }} - 模板风格的变量引用
    /// - {{ namespace.variable_name }} - 带命名空间的模板风格变量引用
    pub fn replace_variables(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // 匹配所有标准变量引用 ${variable} 或 ${namespace::variable}
        let var_pattern = r"\$\{([a-zA-Z0-9_.:]+)(?:\|([^}]+))?\}";
        let re = Regex::new(var_pattern).unwrap();
        
        // 匹配模板风格变量引用 {{ variable }} 或 {{ namespace.variable }}
        let template_pattern = r"\{\{\s*([a-zA-Z0-9_.]+)\s*\}\}";
        let template_re = Regex::new(template_pattern).unwrap();
        
        // 使用循环而不是单次替换，以处理嵌套变量
        let mut prev_result = String::new();
        let mut iteration = 0;
        let max_iterations = 10; // 防止无限循环
        
        while prev_result != result && iteration < max_iterations {
            prev_result = result.clone();
            iteration += 1;
            
            // 处理标准变量引用 ${variable} 或 ${namespace::variable}
            result = re.replace_all(&prev_result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                let default_value = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                
                self.get_variable_value(var_name, default_value)
            }).to_string();
            
            // 处理模板风格变量引用 {{ variable }} 或 {{ namespace.variable }}
            result = template_re.replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                
                // 将 namespace.variable 格式转换为 namespace::variable
                let normalized_name = var_name.replace('.', "::");
                
                self.get_variable_value(&normalized_name, "")
            }).to_string();
        }
        
        result
    }
    
    /// 获取变量值，支持命名空间
    fn get_variable_value(&self, var_name: &str, default_value: &str) -> String {
        debug!("尝试获取变量值: {}", var_name);
        
        // 首先检查特殊变量
        if let Some(value) = self.special_vars.get(var_name) {
            debug!("找到特殊变量: {} = {}", var_name, value);
            return value.clone();
        }
        
        // 然后检查普通变量（包括命名空间变量）
        if let Some(value) = self.variables.get(var_name) {
            debug!("找到普通变量: {} = {}", var_name, value);
            return value.clone();
        }
        
        // 如果包含::分隔符，可能是命名空间变量引用
        if var_name.contains("::") {
            let parts: Vec<&str> = var_name.splitn(2, "::").collect();
            if parts.len() == 2 {
                let namespace = parts[0];
                let local_var = parts[1];
                
                debug!("尝试查找命名空间变量: {}::{}", namespace, local_var);
                
                if let Some(ns_vars) = self.external_vars.get(namespace) {
                    if let Some(value) = ns_vars.get(local_var) {
                        debug!("找到命名空间变量: {}::{} = {}", namespace, local_var, value);
                        return value.clone();
                    }
                }
            }
        }
        
        // 尝试将点表示法转换为双冒号，再次查找
        if var_name.contains('.') {
            let normalized_name = var_name.replace('.', "::");
            debug!("转换点表示法尝试查找: {} -> {}", var_name, normalized_name);
            
            if let Some(value) = self.variables.get(&normalized_name) {
                debug!("找到通过点表示法转换的变量: {} = {}", normalized_name, value);
                return value.clone();
            }
            
            // 如果转换后包含::分隔符，尝试通过命名空间查找
            if normalized_name.contains("::") {
                let parts: Vec<&str> = normalized_name.splitn(2, "::").collect();
                if parts.len() == 2 {
                    let namespace = parts[0];
                    let local_var = parts[1];
                    
                    if let Some(ns_vars) = self.external_vars.get(namespace) {
                        if let Some(value) = ns_vars.get(local_var) {
                            debug!("找到通过点表示法转换的命名空间变量: {}::{} = {}", namespace, local_var, value);
                            return value.clone();
                        }
                    }
                }
            }
        }
        
        // 使用默认值或保留原始引用
        if default_value.is_empty() {
            debug!("变量 {} 未找到，保留原始引用", var_name);
            format!("${{{}}}", var_name) // 保留原始引用
        } else {
            debug!("变量 {} 未找到，使用默认值: {}", var_name, default_value);
            default_value.to_string()
        }
    }
}