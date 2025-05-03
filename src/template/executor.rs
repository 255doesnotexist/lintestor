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
        
        // 3. 构建依赖图并生成执行顺序
        let execution_order = self.build_execution_order(&template)?;
        
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
        
        // 1. 构建依赖图
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        
        // 所有步骤ID
        let mut all_steps: HashSet<&str> = template.steps.iter()
            .map(|s| s.id.as_str())
            .collect();
        
        // 初始化图和入度
        for step in &template.steps {
            graph.entry(step.id.as_str()).or_insert_with(Vec::new);
            in_degree.entry(step.id.as_str()).or_insert(0);
            
            // 更新依赖关系
            for dep in &step.depends_on {
                graph.entry(dep.as_str()).or_insert_with(Vec::new).push(step.id.as_str());
                *in_degree.entry(step.id.as_str()).or_insert(0) += 1;
                all_steps.insert(dep.as_str()); // 确保所有被依赖的步骤也在集合中
            }
        }
        
        // 检查是否有引用了不存在的步骤
        for step_id in &all_steps {
            let exists = template.steps.iter().any(|s| &s.id == step_id);
            if !exists {
                bail!("模板中引用了不存在的步骤ID: {}", step_id);
            }
        }
        
        // 2. 拓扑排序
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        
        // 加入所有入度为0的节点
        for (step_id, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(*step_id);
            }
        }
        
        // 执行拓扑排序
        while let Some(step_id) = queue.pop_front() {
            result.push(step_id.to_string());
            
            if let Some(deps) = graph.get(step_id) {
                for &dep in deps {
                    *in_degree.get_mut(dep).unwrap() -= 1;
                    if in_degree[dep] == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }
        
        // 检查是否有循环依赖
        if result.len() != all_steps.len() {
            bail!("测试模板中存在循环依赖");
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
                // 创建一个引用结果
                if let Some(ref_result) = context.results.get(ref_id) {
                    return Ok(StepResult {
                        id: step.id.clone(),
                        status: StepStatus::Pass,
                        stdout: ref_result.stdout.clone(),
                        stderr: ref_result.stderr.clone(),
                        exit_code: ref_result.exit_code,
                        extracted_vars: HashMap::new(),
                    });
                } else {
                    return Ok(StepResult {
                        id: step.id.clone(),
                        status: StepStatus::Fail,
                        stdout: String::new(),
                        stderr: format!("引用的命令结果不存在: {}", ref_id),
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
                        return StepStatus::Fail;
                    }
                },
                AssertionType::StdoutContains(text) => {
                    if !output.stdout.contains(text) {
                        return StepStatus::Fail;
                    }
                },
                AssertionType::StdoutMatches(regex_str) => {
                    match Regex::new(regex_str) {
                        Ok(re) => {
                            if !re.is_match(&output.stdout) {
                                return StepStatus::Fail;
                            }
                        },
                        Err(_) => {
                            warn!("无效的正则表达式: {}", regex_str);
                            return StepStatus::Fail;
                        }
                    }
                },
                AssertionType::StderrContains(text) => {
                    if !output.stderr.contains(text) {
                        return StepStatus::Fail;
                    }
                },
                AssertionType::StderrMatches(regex_str) => {
                    match Regex::new(regex_str) {
                        Ok(re) => {
                            if !re.is_match(&output.stderr) {
                                return StepStatus::Fail;
                            }
                        },
                        Err(_) => {
                            warn!("无效的正则表达式: {}", regex_str);
                            return StepStatus::Fail;
                        }
                    }
                },
            }
        }
        
        // 所有断言都通过
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