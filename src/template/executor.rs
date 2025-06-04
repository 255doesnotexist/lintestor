//! 测试模板执行相关的定义和辅助函数
//!
//! 这个模块包含执行结果、选项，以及命令断言和变量提取的辅助逻辑。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::{Result, Context, bail};
use regex::Regex;

use crate::template::{
    StepStatus, AssertionType, TestTemplate
};

/// 测试执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 模板引用
    pub template: Arc<TestTemplate>,
    /// 测试单元名称
    pub unit_name: String,
    /// 目标名称
    pub target_name: String,
    /// 总体状态
    pub overall_status: StepStatus,
    /// 步骤结果 - Keyed by Step ID (e.g., "SECTION_ID" or "BLOCK_ID")
    pub step_results: HashMap<String, StepResult>,
    /// 从该模板执行中提取的变量（临时存储，查询还是请去 VariableManager）
    pub variables: HashMap<String, String>,
    /// 报告文件路径
    pub report_path: Option<PathBuf>,
}

impl ExecutionResult {
    /// 获取模板ID
    pub fn template_id(&self) -> String {
        self.template.get_template_id()
    }
    
    /// 获取模板标题
    pub fn template_title(&self) -> &str {
        &self.template.metadata.title
    }
    
    /// 获取模板文件路径
    pub fn template_path(&self) -> &Path {
        &self.template.file_path
    }
}

/// 单个测试步骤的执行结果
#[derive(Debug, Clone)]
pub struct StepResult {
    /// 步骤ID
    pub id: String,
    /// 步骤描述 (可选)
    pub description: Option<String>,
    /// 状态
    pub status: StepStatus,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: i32,
    /// 执行耗时 (可选)
    pub duration_ms: Option<u128>,
    /// 断言失败信息 (可选)
    pub assertion_error: Option<String>,
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
    /// 是否保持连接会话状态 (通常用于SSH)
    pub maintain_session: bool,
    /// 是否在出错时继续执行（尽可能多地执行其他独立步骤）
    pub continue_on_error: bool,
}

impl Default for ExecutorOptions {
    fn default() -> Self {
        Self {
            command_timeout: 300, // 5 minutes
            retry_count: 1,       // Default to 1 try (no retries beyond the first attempt)
            retry_interval: 5,
            maintain_session: true,
            continue_on_error: false,
        }
    }
}

/// 检查断言
///
/// # Arguments
/// * `assertion` - 要检查的断言类型
/// * `stdout` - 命令的标准输出
/// * `stderr` - 命令的标准错误
/// * `exit_code` - 命令的退出码
///
/// # Returns
/// * `Ok(())` - 如果断言通过
/// * `Err(anyhow::Error)` - 如果断言失败，包含失败信息
pub fn check_assertion(assertion: &AssertionType, stdout: &str, stderr: &str, exit_code: i32) -> Result<()> {
    match assertion {
        AssertionType::ExitCode(expected) => {
            if exit_code != *expected {
                bail!("退出码不匹配: 期望 {}, 实际 {}", expected, exit_code);
            }
        }
        AssertionType::StdoutContains(pattern) => {
            if !stdout.contains(pattern) {
                bail!("标准输出不包含期望的模式: '{}'", pattern);
            }
        }
        AssertionType::StdoutNotContains(pattern) => {
            if stdout.contains(pattern) {
                bail!("标准输出包含了不期望的模式: '{}'", pattern);
            }
        }
        AssertionType::StdoutMatches(pattern) => {
            let re = Regex::new(pattern)
                .with_context(|| format!("无效的正则表达式 (stdout): {pattern}"))?;
            
            if !re.is_match(stdout) {
                bail!("标准输出不匹配正则表达式: '{}'", pattern);
            }
        }
        AssertionType::StderrContains(pattern) => {
            if !stderr.contains(pattern) {
                bail!("标准错误不包含期望的模式: '{}'", pattern);
            }
        }
        AssertionType::StderrNotContains(pattern) => {
            if stderr.contains(pattern) {
                bail!("标准错误包含了不期望的模式: '{}'", pattern);
            }
        }
        AssertionType::StderrMatches(pattern) => {
            let re = Regex::new(pattern)
                .with_context(|| format!("无效的正则表达式 (stderr): {pattern}"))?;
            
            if !re.is_match(stderr) {
                bail!("标准错误不匹配正则表达式: '{}'", pattern);
            }
        }
    }
    Ok(())
}

/// 从文本中提取变量值
///
/// # Arguments
/// * `text` - 从中提取变量的文本 (通常是 stdout 或 stderr)
/// * `regex_str` - 用于提取的正则表达式字符串。如果包含捕获组，则使用第一个捕获组；否则使用整个匹配。
///
/// # Returns
/// * `Ok(String)` - 提取到的变量值
/// * `Err(anyhow::Error)` - 如果正则表达式无效或没有匹配
pub fn extract_variable(text: &str, regex_str: &str) -> Result<String> {
    let re = Regex::new(regex_str)
        .with_context(|| format!("无效的提取正则表达式: {regex_str}"))?;
    
    match re.captures(text) {
        Some(caps) => {
            if caps.len() > 1 && caps.get(1).is_some() {
                // 使用第一个捕获组
                Ok(caps.get(1).unwrap().as_str().to_string())
            } else if caps.get(0).is_some() {
                // 使用整个匹配
                Ok(caps.get(0).unwrap().as_str().to_string())
            } else {
                bail!("正则表达式 '{}' 匹配成功，但无法提取捕获组", regex_str)
            }
        },
        None => bail!("正则表达式 '{}' 在文本中没有匹配", regex_str),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_assertion_exit_code_pass() {
        assert!(check_assertion(&AssertionType::ExitCode(0), "", "", 0).is_ok());
    }

    #[test]
    fn test_check_assertion_exit_code_fail() {
        let result = check_assertion(&AssertionType::ExitCode(0), "", "", 1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "退出码不匹配: 期望 0, 实际 1");
    }

    #[test]
    fn test_check_assertion_stdout_contains_pass() {
        assert!(check_assertion(&AssertionType::StdoutContains("hello".to_string()), "hello world", "", 0).is_ok());
    }

    #[test]
    fn test_check_assertion_stdout_contains_fail() {
        let result = check_assertion(&AssertionType::StdoutContains("goodbye".to_string()), "hello world", "", 0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "标准输出不包含期望的模式: 'goodbye'");
    }

    #[test]
    fn test_extract_variable_with_capture_group() {
        let text = "Version: 1.2.3";
        let regex = r"Version: (\d+\.\d+\.\d+)";
        assert_eq!(extract_variable(text, regex).unwrap(), "1.2.3");
    }

    #[test]
    fn test_extract_variable_full_match() {
        let text = "ID: user123";
        let regex = r"user\d+"; // No capture group, but matches "user123"
        assert_eq!(extract_variable(text, regex).unwrap(), "user123");
    }

    #[test]
    fn test_extract_variable_no_match() {
        let text = "Hello world";
        let regex = r"Version: (\d+)";
        assert!(extract_variable(text, regex).is_err());
    }

    #[test]
    fn test_extract_variable_empty_capture_group() {
        // This case should ideally not happen with well-formed regex, but tests robustness
        let text = "key=";
        let regex = r"key=(.*)";
        assert_eq!(extract_variable(text, regex).unwrap(), "");
    }
}