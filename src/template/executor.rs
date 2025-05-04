//! 测试模板执行器
//!
//! 这个模块负责执行测试模板中定义的步骤，包括命令执行、依赖检查、断言评估和数据提取。

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::time::Duration;
use anyhow::{Result, Context, bail, anyhow};
use regex::Regex;
use log::{debug, warn};

use crate::template::{
    TestTemplate, TestStep, StepStatus, StepResult, TemplateContext, AssertionType, DataExtraction
};
use crate::config::target_config::TargetConfig;
use crate::connection::ConnectionManager;

/// 测试执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 模板ID
    pub template_id: String,
    /// 模板标题
    pub template_title: String,
    /// 测试单元名称
    pub unit_name: String,
    /// 目标名称
    pub target_name: String,
    /// 总体状态
    pub overall_status: StepStatus,
    /// 步骤结果
    pub step_results: HashMap<String, StepResult>,
    /// 变量
    pub variables: HashMap<String, String>,
    /// 特殊变量
    pub special_vars: HashMap<String, String>,
    /// 报告文件路径
    pub report_path: Option<PathBuf>,
}

/// 测试模板执行器
pub struct TemplateExecutor<'a> {
    /// 工作目录
    work_dir: PathBuf,
    /// 连接管理器
    connection_manager: &'a mut dyn ConnectionManager,
    /// 选项
    options: ExecutorOptions,
}

/// 执行器选项
#[derive(Debug, Clone)]
pub struct ExecutorOptions {
    /// 命令超时时间（秒）
    pub command_timeout: u64,
    /// 重试次数
    pub retry_count: u32,
    /// 重试间隔（秒）
    pub retry_interval: u64,
    /// 是否保持连接会话状态
    pub maintain_session: bool,
    /// 是否在出错时继续执行（尽可能多地执行）
    pub continue_on_error: bool,
}

impl Default for ExecutorOptions {
    fn default() -> Self {
        Self {
            command_timeout: 300, // 5 minutes
            retry_count: 1,
            retry_interval: 5,
            maintain_session: true,
            continue_on_error: false,
        }
    }
}

impl<'a> TemplateExecutor<'a> {
    /// 创建新的模板执行器
    pub fn new(
        work_dir: PathBuf, 
        connection_manager: &'a mut dyn ConnectionManager,
        options: Option<ExecutorOptions>
    ) -> Self {
        Self {
            work_dir,
            connection_manager,
            options: options.unwrap_or_default(),
        }
    }
    
    /// 执行测试模板
    pub fn execute_template(
        &mut self,
        template: TestTemplate,
        target_config: TargetConfig,
    ) -> Result<ExecutionResult> {
        // 1. 创建执行上下文
        let mut context = TemplateContext::new(template.clone());
        
        // 2. 填充特殊变量
        self.populate_special_vars(&mut context, &target_config)?;
        
        // 2.1 加载外部引用模板的变量（新增功能）
        self.load_external_references(&mut context)?;
        
        // 3. 构建依赖图并生成执行顺序
        let execution_order = self.build_execution_order(&template)?;
        
        // 打印所有步骤ID，帮助诊断
        let step_ids: Vec<String> = template.steps.iter().map(|s| s.id.clone()).collect();
        debug!("所有定义的步骤ID: {:?}", step_ids);
        
        // 记录所有可执行步骤
        let executable_step_ids: Vec<String> = template.steps.iter()
            .filter(|s| s.executable)
            .map(|s| s.id.clone())
            .collect();
        debug!("可执行的步骤ID: {:?}", executable_step_ids);
        
        // 记录所有引用命令的步骤
        let ref_step_ids: Vec<(String, String)> = template.steps.iter()
            .filter_map(|s| s.ref_command.clone().map(|ref_id| (s.id.clone(), ref_id)))
            .collect();
        debug!("引用命令的步骤ID: {:?}", ref_step_ids);
        
        // 4. 按执行顺序执行步骤
        for step_id in &execution_order {
            // 查找步骤
            let step = template.steps.iter()
                .find(|s| &s.id == step_id)
                .ok_or_else(|| anyhow!("找不到步骤ID: {}", step_id))?;
            
            // 检查依赖状态，必要时跳过
            if !self.check_dependencies(&context, step)? {
                // 存储跳过状态
                let result = StepResult {
                    id: step.id.clone(),
                    status: StepStatus::Skipped,
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: -1,
                    extracted_vars: HashMap::new(),
                };
                context.results.insert(step.id.clone(), result);
                continue;
            }
            
            // 执行步骤
            let result = self.execute_step(&mut context, step, &target_config)?;
            
            // 记录步骤执行结果
            debug!("步骤 {} 执行完成: 状态={:?}, stdout长度={}, stderr长度={}",
                   step.id, result.status, result.stdout.len(), result.stderr.len());
            
            // 存储结果
            context.results.insert(step.id.clone(), result);
        }
        
        // 5. 获取总体状态
        let overall_status = self.calculate_overall_status(&context);
        
        // 6. 构建执行结果
        let template_id = template.file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // 使用目标配置的testing_type作为目标名称，因为TargetConfig没有name字段
        let target_name = target_config.testing_type.clone();
        
        let execution_result = ExecutionResult {
            template_id,
            template_title: template.metadata.title.clone(),
            unit_name: template.metadata.unit_name.clone(),
            target_name,
            overall_status,
            step_results: context.results,
            variables: context.variables,
            special_vars: context.special_vars,
            report_path: None,
        };
        
        Ok(execution_result)
    }
    
    /// 加载外部引用模板的变量
    fn load_external_references(&mut self, context: &mut TemplateContext) -> Result<()> {
        // 检查是否有外部引用
        if context.template.metadata.references.is_empty() {
            debug!("没有外部引用模板需要加载");
            return Ok(());
        }
        
        debug!("开始加载 {} 个外部引用模板的变量", context.template.metadata.references.len());
        
        // Clone the references to avoid borrowing context immutably inside the loop
        let references = context.template.metadata.references.clone();
        
        for reference in &references { // Iterate over the cloned vector
            // Assume the referenced template path is relative to the './' directory in work_dir
            // If reference.template_path is absolute, join typically does nothing useful unless it's relative to root.
            // Consider adding logic here to check if reference.template_path is already absolute.
            let tests_dir = self.work_dir.clone();
            let full_template_path = tests_dir.join(&reference.template_path);
            // Convert the resolved path to a string for use in the rest of the function
            let template_path = full_template_path.to_string_lossy().to_string();
            let namespace = reference.namespace.clone();
            
            debug!("尝试加载外部模板: {}, 命名空间: {}", template_path, namespace);
            
            // 可能的报告目录路径列表
            let mut possible_report_paths = Vec::new();
            
            // 1. reports目录 (默认)
            let mut report_path = self.work_dir.clone();
            report_path.push("reports");
            possible_report_paths.push(report_path.clone());
            debug!("搜索路径1 (reports目录): {}", report_path.display());
            
            // 2. 与当前测试模板相同目录下的reports子目录
            if let Some(parent) = context.template.file_path.parent() {
                let mut local_report_path = parent.to_path_buf();
                local_report_path.push("reports");
                possible_report_paths.push(local_report_path.clone());
                debug!("搜索路径2 (模板目录下的reports): {}", local_report_path.display());
            }
            
            // 3. 模板所在的父目录
            if let Some(parent) = context.template.file_path.parent() {
                possible_report_paths.push(parent.to_path_buf());
                debug!("搜索路径3 (模板所在目录): {}", parent.display());
            }
            
            // 4. 工作目录的根目录
            possible_report_paths.push(self.work_dir.clone());
            debug!("搜索路径4 (工作目录): {}", self.work_dir.display());
            
            // 如果命名空间包含路径分隔符，进行分割处理
            let components: Vec<&str> = template_path.split('/').collect();
            let template_path_str = template_path.as_str(); // Create a longer-lived binding
            let filename = components.last().unwrap_or(&template_path_str); // Use the binding
            
            // 根据约定，报告文件名是<template_name>_<target_name>.report.md
            // 我们只取template_name部分，然后找到对应的报告文件
            let report_file_prefix = if filename.ends_with(".test.md") {
                filename[..filename.len() - 8].to_string() // 去掉 .test.md
            } else if filename.ends_with(".md") {
                filename[..filename.len() - 3].to_string() // 去掉 .md
            } else {
                filename.to_string()
            };
            
            debug!("开始在多个可能位置查找以 {} 开头的报告文件", report_file_prefix);
            
            // 在所有可能路径中查找匹配的报告文件
            let mut found_report = None;
            
            for path in &possible_report_paths {
                debug!("在 {} 目录中查找报告文件", path.display());
                
                // 检查目录是否存在
                if !path.exists() || !path.is_dir() {
                    debug!("目录 {} 不存在或不是一个目录", path.display());
                    continue;
                }
                
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let entry_path = entry.path();
                            if entry_path.is_file() {
                                if let Some(file_name) = entry_path.file_name() {
                                    let file_name_str = file_name.to_string_lossy();
                                    
                                    // 支持多种命名格式:
                                    // 1. [prefix]_*.report.md
                                    // 2. [prefix].report.md
                                    // 3. [prefix]*.md (如果在报告目录下)
                                    if (file_name_str.starts_with(&report_file_prefix) && 
                                       (file_name_str.ends_with(".report.md") || 
                                        path.ends_with("reports") && file_name_str.ends_with(".md"))) ||
                                       file_name_str == format!("{}.report.md", report_file_prefix) {
                                        
                                        debug!("找到匹配的报告文件: {}", entry_path.display());
                                        found_report = Some(entry_path);
                                        break;
                                    } else {
                                        // 记录所有检查过的文件名，方便调试
                                        debug!("检查文件: {}, 但不匹配模式", file_name_str);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    debug!("无法读取目录: {}", path.display());
                }
                
                if found_report.is_some() {
                    break;
                }
            }
            
            // 如果找到匹配的报告文件，提取其中的变量
            if let Some(report_path) = found_report {
                debug!("从报告文件 {} 中提取变量", report_path.display());
                
                // 读取报告文件内容
                let content = std::fs::read_to_string(&report_path)
                    .with_context(|| format!("无法读取报告文件: {}", report_path.display()))?;
                
                // 提取变量
                self.extract_variables_from_report(&content, namespace, context)?;
            } else {
                warn!("未找到匹配的报告文件: {}*.report.md 或 {}.report.md", 
                     report_file_prefix, report_file_prefix);
                debug!("已尝试查找的目录: {}", possible_report_paths
                      .iter()
                      .map(|p| p.display().to_string())
                      .collect::<Vec<_>>()
                      .join(", "));
                
                // 找不到报告文件时，尝试直接加载并解析模板文件
                debug!("尝试直接解析模板文件：{}", template_path);
                self.extract_variables_from_template(&template_path, namespace, context)?;
            }
        }
        
        Ok(())
    }
    
    /// 尝试从模板文件直接加载变量
    fn extract_variables_from_template(
        &mut self,
        template_path: &str,
        namespace: String,
        context: &mut TemplateContext
    ) -> Result<()> {
        let mut possible_paths = Vec::new();
        
        debug!("尝试从模板文件加载变量: {}, 命名空间: {}", template_path, namespace);
        
        // 1. 相对于当前模板的路径
        if let Some(parent) = context.template.file_path.parent() {
            let mut relative_path = parent.to_path_buf();
            relative_path.push(template_path);
            debug!("搜索路径1 (相对于当前模板): {}", relative_path.display());
            possible_paths.push(relative_path);
        }
        
        // 2. 相对于工作目录的路径
        let mut work_dir_path = self.work_dir.clone();
        work_dir_path.push(template_path);
        debug!("搜索路径2 (相对于工作目录): {}", work_dir_path.display());
        possible_paths.push(work_dir_path);
        
        // 3. 如果路径中包含模板目录名，尝试在模板目录下查找
        let mut test_dir_path = self.work_dir.clone();
        test_dir_path.push("tests");
        test_dir_path.push(template_path);
        debug!("搜索路径3 (相对于测试目录): {}", test_dir_path.display());
        possible_paths.push(test_dir_path);
        
        // 查找模板文件
        let mut template_file = None;
        
        for path in &possible_paths {
            debug!("检查路径: {}", path.display());
            if path.exists() && path.is_file() {
                template_file = Some(path.clone());
                debug!("找到模板文件: {}", path.display());
                break;
            }
        }
        
        if let Some(file_path) = template_file {
            debug!("找到模板文件: {}", file_path.display());
            
            // 读取模板文件内容
            let content = std::fs::read_to_string(&file_path)
                .with_context(|| format!("无法读取模板文件: {}", file_path.display()))?;
            
            // 1. 首先，解析模板内容提取变量
            // 提取YAML前置数据
            if let Some(yaml_content) = Self::extract_front_matter_from_report(&content) {
                debug!("从模板中提取YAML前置数据, 长度: {}字节", yaml_content.len());
                
                // 解析YAML内容
                if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(&yaml_content) {
                    // 2. 检查该模板是否也有自己的引用（递归处理）
                    if let Some(refs) = yaml_value.get("references") {
                        if let Some(refs_array) = refs.as_sequence() {
                            debug!("模板 {} 中找到 {} 个递归引用", file_path.display(), refs_array.len());
                            
                            // 处理每一个引用，采用递归方式
                            for ref_item in refs_array {
                                if let Some(ref_obj) = ref_item.as_mapping() {
                                    // 提取模板路径和命名空间
                                    let ref_template = ref_obj.get(&serde_yaml::Value::String("template".to_string()))
                                        .and_then(|v| v.as_str());
                                    let ref_namespace = ref_obj.get(&serde_yaml::Value::String("as".to_string()))
                                        .and_then(|v| v.as_str());
                                    
                                    if let (Some(ref_path), Some(ref_ns)) = (ref_template, ref_namespace) {
                                        // 构造完整命名空间路径：当前命名空间.引用命名空间
                                        let nested_namespace = format!("{}.{}", namespace, ref_ns);
                                        debug!("处理递归引用: {} 作为命名空间 {}", ref_path, nested_namespace);
                                        
                                        // 递归处理引用的模板
                                        self.extract_variables_from_template(ref_path, nested_namespace, context)?;
                                    }
                                }
                            }
                        }
                    }
                    
                    // 3. 从当前模板中提取变量
                    if let Some(obj) = yaml_value.as_mapping() {
                        let mut extracted = HashMap::new();
                        
                        // 提取所有字符串类型的字段作为变量
                        for (key, value) in obj {
                            if let (Some(key_str), Some(value_str)) = (key.as_str(), value.as_str()) {
                                // 排除一些特殊字段
                                if !["title", "target_config", "unit_name", "tags", "references"].contains(&key_str) {
                                    extracted.insert(key_str.to_string(), value_str.to_string());
                                    debug!("从模板YAML中提取变量: {}.{} = {}", namespace, key_str, value_str);
                                }
                            }
                        }
                        
                        if !extracted.is_empty() {
                            context.set_external_variables(&namespace, extracted);
                        }
                    }
                }
            }
            
            // 4. 尝试从模板正文中提取更多变量
            self.extract_variables_from_content(&content, &namespace, context)?;
        } else {
            debug!("未找到模板文件: {}", template_path);
            debug!("已尝试的路径: {}", possible_paths
                   .iter()
                   .map(|p| p.display().to_string())
                   .collect::<Vec<_>>()
                   .join(", "));
        }
        
        Ok(())
    }

    /// 从文本内容中提取变量
    fn extract_variables_from_content(
        &self,
        content: &str,
        namespace: &str,
        context: &mut TemplateContext
    ) -> Result<()> {
        // 从正文中提取显式声明的变量
        // 例如：`变量名: value` 或 `Variable: value` 格式的行
        let variable_patterns = [
            r"^([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*(.+)$",
            r"^([a-zA-Z_][a-zA-Z0-9_]*)值?\s*[:：]\s*(.+)$", // 支持中文冒号
        ];
        
        let mut extracted = HashMap::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            for pattern in &variable_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if let Some(caps) = re.captures(line) {
                        if caps.len() >= 3 {
                            let key = caps.get(1).unwrap().as_str();
                            let value = caps.get(2).unwrap().as_str().trim();
                            
                            // 排除明显的标题或其他非变量内容
                            if !key.starts_with("#") && !key.contains(" ") && value.len() <= 100 {
                                extracted.insert(key.to_string(), value.to_string());
                                debug!("从模板正文中提取变量: {}.{} = {}", namespace, key, value);
                            }
                        }
                    }
                }
            }
        }
        
        // 合并到已有的外部变量中
        if !extracted.is_empty() {
            // 如果已经有该命名空间的变量，则合并；否则新建
            if let Some(existing) = context.external_vars.get_mut(namespace) {
                for (k, v) in extracted {
                    existing.insert(k, v);
                }
            } else {
                context.set_external_variables(namespace, extracted);
            }
        }
        
        Ok(())
    }

    /// 从报告内容中提取变量
    fn extract_variables_from_report(
        &self,
        content: &str,
        namespace: String,
        context: &mut TemplateContext
    ) -> Result<()> {
        debug!("开始从报告内容中提取变量，长度: {}字节", content.len());
        
        // 提取YAML前置数据（可能包含变量）
        if let Some(yaml_content) = Self::extract_front_matter_from_report(content) {
            debug!("提取到YAML前置数据，长度: {}字节", yaml_content.len());
            
            // 解析YAML
            if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(&yaml_content) {
                // 首先尝试从variables字段中提取变量
                if let Some(variables) = yaml_value.get("variables") {
                    if let Some(obj) = variables.as_mapping() {
                        let mut extracted = HashMap::new();
                        
                        for (key, value) in obj {
                            if let (Some(key_str), Some(value_str)) = (key.as_str(), value.as_str()) {
                                extracted.insert(key_str.to_string(), value_str.to_string());
                                debug!("从YAML中提取变量: {} = {}", key_str, value_str);
                            }
                        }
                        
                        // 将提取的变量添加到上下文中
                        context.set_external_variables(&namespace, extracted);
                    }
                } else {
                    debug!("YAML前置数据中没有找到variables字段");
                    
                    // 尝试从整个YAML中提取字符串值作为变量（备用方案）
                    if let Some(obj) = yaml_value.as_mapping() {
                        let mut extracted = HashMap::new();
                        
                        for (key, value) in obj {
                            if let (Some(key_str), Some(value_str)) = (key.as_str(), value.as_str()) {
                                // 只提取字符串类型的值
                                extracted.insert(key_str.to_string(), value_str.to_string());
                                debug!("从YAML中提取变量(备用方案): {} = {}", key_str, value_str);
                            }
                        }
                        
                        if !extracted.is_empty() {
                            context.set_external_variables(&namespace, extracted);
                        }
                    }
                }
            } else {
                warn!("无法解析YAML前置数据为有效的YAML");
            }
        } else {
            debug!("未在报告内容中找到YAML前置数据");
        }
        
        // 此外，从正文中提取显式声明的变量
        // 例如：`变量名: value` 或 `Variable: value` 格式的行
        let variable_patterns = [
            r"^([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*(.+)$",
            r"^([a-zA-Z_][a-zA-Z0-9_]*)值?\s*[:：]\s*(.+)$", // 支持中文冒号
        ];
        
        let mut extracted = HashMap::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            for pattern in &variable_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if let Some(caps) = re.captures(line) {
                        if caps.len() >= 3 {
                            let key = caps.get(1).unwrap().as_str();
                            let value = caps.get(2).unwrap().as_str().trim();
                            
                            // 排除明显的标题或其他非变量内容
                            if !key.starts_with("#") && !key.contains(" ") && value.len() <= 100 {
                                extracted.insert(key.to_string(), value.to_string());
                                debug!("从正文中提取变量: {} = {}", key, value);
                            }
                        }
                    }
                }
            }
        }
        
        // 合并到已有的外部变量中
        if !extracted.is_empty() {
            // 如果已经有该命名空间的变量，则合并；否则新建
            if let Some(existing) = context.external_vars.get_mut(&namespace) {
                for (k, v) in extracted {
                    existing.insert(k, v);
                }
            } else {
                context.set_external_variables(&namespace, extracted);
            }
        }
        
        Ok(())
    }

/// 从报告内容中提取YAML前置数据
fn extract_front_matter_from_report(content: &str) -> Option<String> {
    let pattern = r"(?s)^---\s*\n(.*?)\n---\s*\n";
    if let Ok(re) = Regex::new(pattern) {
        if let Some(caps) = re.captures(content) {
            if caps.len() >= 2 {
                return Some(caps.get(1).unwrap().as_str().to_string());
            }
        }
    }
    None
}
    
    /// 填充特殊变量
    fn populate_special_vars(
        &mut self, 
        context: &mut TemplateContext,
        target_config: &TargetConfig
    ) -> Result<()> {
        // 已经在TemplateContext::new中添加了execution_date
        
        // 添加target_info（因为TargetConfig中没有info_command字段，我们使用测试类型作为基本信息）
        let target_info = format!("测试类型: {}", target_config.testing_type);
        context.special_vars.insert("target_info".to_string(), target_info);
        
        // 添加unit_version（如果模板中有定义unit_version_command）
        if let Some(version_cmd) = &context.template.metadata.unit_version_command {
            // 执行version_command
            let output = self.connection_manager.execute_command(version_cmd, None)?;
            if output.exit_code == 0 {
                context.special_vars.insert("unit_version".to_string(), output.stdout.trim().to_string());
            } else {
                warn!("获取单元版本失败，unit_version_command返回非零退出码: {}", output.exit_code);
                context.special_vars.insert("unit_version".to_string(), "获取单元版本失败".to_string());
            }
        }
        
        Ok(())
    }
    
    /// 构建依赖图并生成执行顺序
    fn build_execution_order(&self, template: &TestTemplate) -> Result<Vec<String>> {
        // 使用拓扑排序生成执行顺序
        debug!("开始构建执行顺序");
        
        // 1. 构建依赖图
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        
        // 收集所有步骤ID (无论是显式定义还是被引用的)
        let mut all_steps: HashSet<&str> = HashSet::new();
        
        // 先收集所有显式定义的步骤ID
        let existing_steps: HashSet<&str> = template.steps.iter()
            .map(|s| s.id.as_str())
            .collect();
        
        // 记录所有步骤ID和它们的依赖关系
        for step in &template.steps {
            // 将当前步骤添加到全部步骤集合中
            all_steps.insert(step.id.as_str());
            
            // 初始化图和入度
            graph.entry(step.id.as_str()).or_insert_with(Vec::new);
            in_degree.entry(step.id.as_str()).or_insert(0);
            
            // 更新依赖关系
            for dep in &step.depends_on {
                debug!("步骤 {} 依赖于 {}", step.id, dep);
                all_steps.insert(dep.as_str()); // 确保所有被依赖的步骤也在集合中
                graph.entry(dep.as_str()).or_insert_with(Vec::new).push(step.id.as_str());
                *in_degree.entry(step.id.as_str()).or_insert(0) += 1;
            }
        }
        
        // 记录被引用但未定义的步骤
        let mut undefined_steps = HashSet::new();
        for step_id in &all_steps {
            if !existing_steps.contains(step_id) {
                undefined_steps.insert(*step_id);
            }
        }
        
        debug!("步骤总数: {}, 已定义步骤数: {}, 未定义步骤数: {}", 
               all_steps.len(), existing_steps.len(), undefined_steps.len());
        
        debug!("All steps considered during dependency check: {:?}", all_steps);
        debug!("Defined steps: {:?}", existing_steps);
        if !undefined_steps.is_empty() {
            debug!("Undefined steps: {:?}", undefined_steps);
        }
        
        // 检查步骤依赖关系
        debug!("依赖关系图构建完成，检查依赖项");
        
        // 过滤掉带有命名空间的未定义步骤（这些可能是跨模板引用）
        let critical_undefined_steps: HashSet<_> = undefined_steps.iter()
            .filter(|&&step_id| !step_id.contains("::"))
            .collect();
            
        // 只检查不包含命名空间的未定义步骤
        for &step_id in &critical_undefined_steps {
            // 找出引用了此步骤的其他步骤，提供更详细的错误信息
            let mut referencing_steps = Vec::new();
            for step in &template.steps {
                if step.depends_on.iter().any(|dep| dep.as_str() == *step_id) {
                    referencing_steps.push(&step.id);
                }
                if let Some(ref_cmd) = &step.ref_command {
                    if ref_cmd.as_str() == *step_id {
                        referencing_steps.push(&step.id);
                    }
                }
            }
            
            // 获取文件名信息
            let file_info = if let Some(path) = template.file_path.file_name() {
                path.to_string_lossy().to_string()
            } else {
                "未知文件".to_string()
            };
            
            // 给出详细错误信息，包括引用关系
            let error_detail = if !referencing_steps.is_empty() {
                let referencing_steps_str: Vec<&str> = referencing_steps.iter().map(|s| s.as_str()).collect();
                format!("模板中引用了不存在的步骤ID: `{}` (在文件 {} 中). 此ID被以下步骤引用: {}", 
                        step_id, file_info, referencing_steps_str.join(", "))
            } else {
                format!("模板中引用了不存在的步骤ID: `{}` (在文件 {} 中)", step_id, file_info)
            };

            warn!("{}", error_detail);
            bail!("{}", error_detail);
        }
        
        // 2. 拓扑排序
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        
        // 加入所有入度为0的节点
        for (step_id, degree) in &in_degree {
            if *degree == 0 {
                debug!("入度为0的节点: {}", step_id);
                queue.push_back(*step_id);
            }
        }
        
        debug!("初始队列大小: {}", queue.len());
        
        // 执行拓扑排序
        while let Some(step_id) = queue.pop_front() {
            result.push(step_id.to_string());
            debug!("处理节点: {}, 当前结果大小: {}", step_id, result.len());
            
            if let Some(deps) = graph.get(step_id) {
                for &dep in deps {
                    *in_degree.get_mut(dep).unwrap() -= 1;
                    debug!("  更新依赖节点 {} 的入度为 {}", dep, in_degree[dep]);
                    if in_degree[dep] == 0 {
                        queue.push_back(dep);
                        debug!("  入度为0，加入队列: {}", dep);
                    }
                }
            }
        }
        
        debug!("拓扑排序结果大小: {}, 总步骤数: {}", result.len(), all_steps.len());
        
        // 检查是否有循环依赖
        if result.len() != all_steps.len() {
            // 找出潜在的循环依赖
            let mut unprocessed = all_steps.iter()
                .filter(|&id| !result.contains(&id.to_string()))
                .map(|&id| id.to_string())
                .collect::<Vec<_>>();
            
            let error_msg = format!("测试模板中存在循环依赖，无法确定执行顺序。可能涉及的步骤: {}", 
                                   unprocessed.join(", "));
            bail!(error_msg);
        }
        
        Ok(result)
    }
    
    /// 检查步骤的依赖是否已满足
    fn check_dependencies(&self, context: &TemplateContext, step: &TestStep) -> Result<bool> {
        if step.depends_on.is_empty() {
            return Ok(true);
        }
        
        for dep_id in &step.depends_on {
            match context.get_step_status(dep_id) {
                StepStatus::Pass => {
                    // 依赖成功，继续检查下一个
                    continue;
                },
                StepStatus::NotRun => {
                    // 依赖尚未执行，阻塞当前步骤
                    warn!("步骤 {} 的依赖 {} 尚未执行", step.id, dep_id);
                    return Ok(false);
                },
                status => {
                    // 依赖执行失败或被跳过，跳过当前步骤
                    warn!("步骤 {} 的依赖 {} 状态为 {:?}，因此跳过当前步骤", step.id, dep_id, status);
                    return Ok(false);
                }
            }
        }
        
        Ok(true)
    }
    
    /// 执行单个测试步骤
    fn execute_step(
        &mut self,
        context: &mut TemplateContext,
        step: &TestStep,
        _target_config: &TargetConfig
    ) -> Result<StepResult> {
        // 检查是否是可执行步骤
        if !step.executable {
            // 处理引用命令的输出块
            if let Some(ref_id) = &step.ref_command {
                // 记录正在处理的引用命令
                debug!("处理引用命令步骤: {} -> 引用 {}", step.id, ref_id);
                
                // 创建一个引用结果
                if let Some(ref_result) = context.results.get(ref_id) {
                    debug!("找到引用的命令结果: {} (输出长度: {}字节)", ref_id, ref_result.stdout.len());
                    return Ok(StepResult {
                        id: step.id.clone(),
                        status: StepStatus::Pass,
                        stdout: ref_result.stdout.clone(),
                        stderr: ref_result.stderr.clone(),
                        exit_code: ref_result.exit_code,
                        extracted_vars: HashMap::new(),
                    });
                } else {
                    // 如果引用的命令不存在，提供更有帮助的错误信息
                    let error_msg = format!("引用的命令结果不存在: {}。这可能是因为该命令未定义或者执行顺序有问题。", ref_id);
                    warn!("{}", error_msg);
                    
                    // 显示所有已有的命令结果ID，帮助诊断
                    let available_ids: Vec<&String> = context.results.keys().collect();
                    debug!("当前可用的命令结果ID: {:?}", available_ids);
                    
                    // 返回一个包含明确错误信息的结果，而不是空输出
                    return Ok(StepResult {
                        id: step.id.clone(),
                        status: StepStatus::Fail,
                        stdout: format!("错误: {}\n可用的命令ID: {:?}", error_msg, available_ids),
                        stderr: error_msg,
                        exit_code: -1,
                        extracted_vars: HashMap::new(),
                    });
                }
            }
            
            // 非可执行步骤，直接返回通过
            return Ok(StepResult {
                id: step.id.clone(),
                status: StepStatus::Pass,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                extracted_vars: HashMap::new(),
            });
        }
        
        // 确保有命令
        let command = match &step.command {
            Some(cmd) => cmd,
            None => {
                return Ok(StepResult {
                    id: step.id.clone(),
                    status: StepStatus::Fail,
                    stdout: String::new(),
                    stderr: "步骤标记为可执行但没有命令".to_string(),
                    exit_code: -1,
                    extracted_vars: HashMap::new(),
                });
            }
        };
        
        // 替换命令中的变量
        let command = context.replace_variables(command);
        
        debug!("执行命令: {}", command);
        
        // 设置命令超时
        let timeout = Duration::from_secs(self.options.command_timeout);
        
        // 执行命令
        let output = self.connection_manager.execute_command(&command, Some(timeout))?;
        
        debug!("命令执行结果: exit_code={}, stdout={}, stderr={}", 
               output.exit_code, output.stdout.len(), output.stderr.len());
        
        // 提取变量
        let mut extracted_vars = HashMap::new();
        for extraction in &step.extractions {
            match extract_variable(&output.stdout, &extraction.regex) {
                Ok(value) => {
                    extracted_vars.insert(extraction.variable.clone(), value.clone());
                    context.variables.insert(extraction.variable.clone(), value);
                },
                Err(e) => {
                    warn!("从输出中提取变量失败: {}", e);
                }
            }
        }
        
        // 评估断言
        let status = self.evaluate_assertions(&output, &step.assertions);
        
        // 创建步骤结果
        let result = StepResult {
            id: step.id.clone(),
            status,
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.exit_code,
            extracted_vars,
        };
        
        Ok(result)
    }
    
    /// 评估断言
    /// 
    /// 此函数根据用户定义的断言来评估命令执行结果是否通过。
    /// 注意：测试步骤的通过与否完全取决于用户定义的断言，而不是stderr是否有内容。
    /// 即使stderr有错误信息，只要断言全部通过，步骤仍然被视为通过。
    /// 如果用户期望检查stderr，应该使用`assert.stderr_contains`等断言。
    fn evaluate_assertions(
        &self,
        output: &crate::connection::CommandOutput,
        assertions: &[AssertionType]
    ) -> StepStatus {
        if assertions.is_empty() {
            // 如果没有断言，默认检查退出码是否为0
            return if output.exit_code == 0 {
                StepStatus::Pass
            } else {
                StepStatus::Fail
            };
        }
        
        // 检查所有断言
        for assertion in assertions {
            match assertion {
                AssertionType::ExitCode(expected) => {
                    if output.exit_code != *expected {
                        debug!("断言失败: exit_code={}, expected={}", output.exit_code, expected);
                        return StepStatus::Fail;
                    }
                    debug!("断言通过: exit_code={}", expected);
                },
                AssertionType::StdoutContains(text) => {
                    if !output.stdout.contains(text) {
                        debug!("断言失败: stdout不包含'{}'", text);
                        return StepStatus::Fail;
                    }
                    debug!("断言通过: stdout包含'{}'", text);
                },
                AssertionType::StdoutNotContains(text) => {
                    if output.stdout.contains(text) {
                        debug!("断言失败: stdout包含'{}'，但期望不包含", text);
                        return StepStatus::Fail;
                    }
                    debug!("断言通过: stdout不包含'{}'", text);
                },
                AssertionType::StdoutMatches(regex_str) => {
                    match Regex::new(regex_str) {
                        Ok(re) => {
                            if !re.is_match(&output.stdout) {
                                debug!("断言失败: stdout不匹配正则表达式'{}'", regex_str);
                                return StepStatus::Fail;
                            }
                            debug!("断言通过: stdout匹配正则表达式'{}'", regex_str);
                        },
                        Err(e) => {
                            warn!("无效的正则表达式: {}, 错误: {}", regex_str, e);
                            return StepStatus::Fail;
                        }
                    }
                },
                AssertionType::StderrContains(text) => {
                    if !output.stderr.contains(text) {
                        debug!("断言失败: stderr不包含'{}'", text);
                        return StepStatus::Fail;
                    }
                    debug!("断言通过: stderr包含'{}'", text);
                },
                AssertionType::StderrNotContains(text) => {
                    if output.stderr.contains(text) {
                        debug!("断言失败: stderr包含'{}'，但期望不包含", text);
                        return StepStatus::Fail;
                    }
                    debug!("断言通过: stderr不包含'{}'", text);
                },
                AssertionType::StderrMatches(regex_str) => {
                    match Regex::new(regex_str) {
                        Ok(re) => {
                            if !re.is_match(&output.stderr) {
                                debug!("断言失败: stderr不匹配正则表达式'{}'", regex_str);
                                return StepStatus::Fail;
                            }
                            debug!("断言通过: stderr匹配正则表达式'{}'", regex_str);
                        },
                        Err(e) => {
                            warn!("无效的正则表达式: {}, 错误: {}", regex_str, e);
                            return StepStatus::Fail;
                        }
                    }
                },
            }
        }
        
        // 所有断言都通过
        debug!("所有断言都通过");
        StepStatus::Pass
    }
    
    /// 计算整体状态
    fn calculate_overall_status(&self, context: &TemplateContext) -> StepStatus {
        let mut has_fail = false;
        let mut has_skip = false;
        
        for (_, result) in &context.results {
            match result.status {
                StepStatus::Fail => {
                    has_fail = true;
                },
                StepStatus::Skipped | StepStatus::Blocked => {
                    has_skip = true;
                },
                _ => {}
            }
        }
        
        if has_fail {
            StepStatus::Fail
        } else if has_skip {
            StepStatus::Skipped
        } else {
            StepStatus::Pass
        }
    }
}

/// 从文本中提取变量值
fn extract_variable(text: &str, regex_str: &str) -> Result<String> {
    let re = Regex::new(regex_str)
        .with_context(|| format!("无效的正则表达式: {}", regex_str))?;
    
    match re.captures(text) {
        Some(caps) => {
            if caps.len() > 1 {
                // 使用第一个捕获组
                Ok(caps.get(1).unwrap().as_str().to_string())
            } else {
                // 使用整个匹配
                Ok(caps.get(0).unwrap().as_str().to_string())
            }
        },
        None => bail!("正则表达式没有匹配: {}", regex_str),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 添加测试...
}