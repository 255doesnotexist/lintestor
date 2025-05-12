//! Utility types and constants for the lintestor project.
//!
//! This module provides common structures and utilities used across the project,
//! including report structures, temporary file management, and command output handling.

use log::{debug, error, warn};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::config::target_config::TargetConfig;

/// The remote temporary directory used for operations.
pub static REMOTE_TMP_DIR: &str = "/tmp/lintestor";

/// 标准化模板ID
/// 
/// 移除末尾的 `.test` 后缀，并确保ID不包含分隔符
/// 
/// # 参数
/// 
/// - `template_id`: 原始模板ID
/// 
/// # 返回值
/// 
/// 返回标准化后的模板ID
pub fn normalize_template_id(template_id: &str) -> String {
    let clean_id = if template_id.ends_with(".test") {
        let len = template_id.len();
        &template_id[0..len-5]
    } else {
        template_id
    };
    
    if clean_id.contains("::") {
        warn!("模板ID不应包含'::'分隔符: {}, 进行清理", clean_id);
        clean_id.split("::").next().unwrap_or(clean_id).to_string()
    } else {
        clean_id.to_string()
    }
}

/// 从文件路径获取模板ID
/// 
/// 从文件名提取模板ID，移除扩展名和.test后缀
/// 
/// # 参数
/// 
/// - `file_path`: 文件路径
/// 
/// # 返回值
/// 
/// 返回提取的模板ID
pub fn get_template_id_from_path(file_path: &Path) -> String {
    let file_stem = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    normalize_template_id(file_stem)
}

/// 构建变量标识符
/// 
/// 根据模板ID和步骤ID构建标准化的变量标识符
/// 
/// # 参数
/// 
/// - `name`: 变量名
/// - `template_id`: 可选的模板ID
/// - `step_id`: 可选的步骤ID
/// 
/// # 返回值
/// 
/// 返回构建的变量标识符
pub fn build_variable_key(name: &str, template_id: Option<&str>, step_id: Option<&str>) -> String {
    // 确保变量有完整的命名空间
    // 如果模板ID和步骤ID都为None，则将变量视为系统特殊变量，放入特殊的全局命名空间
    let (template_id_str, step_id_str) = match (template_id, step_id) {
        (Some(tid), Some(sid)) => (normalize_template_id(tid), sid.to_string()),
        (Some(tid), None) => (normalize_template_id(tid), "GLOBAL".to_string()),
        (None, Some(sid)) => {
            warn!("设置变量时提供了步骤ID但未提供模板ID，使用GLOBAL作为默认模板ID: {:?}::{:?}::{}", "GLOBAL", sid, name);
            ("GLOBAL".to_string(), sid.to_string())
        },
        (None, None) => {
            // 系统全局变量，如 execution_date、execution_time 等
            if name.starts_with("execution_") || name == "title" || name == "unit_name" || name == "target_name" {
                debug!("系统特殊变量: {}, 放入全局命名空间", name);
                ("GLOBAL".to_string(), "GLOBAL".to_string())
            } else {
                warn!("设置非系统变量时未提供模板ID和步骤ID，使用GLOBAL作为默认命名空间: {:?}::{:?}::{}", "GLOBAL", "GLOBAL", name);
                ("GLOBAL".to_string(), "GLOBAL".to_string())
            }
        }
    };
    
    // 生成标准化的变量标识符 (模板ID::步骤ID::变量名)
    format!("{}::{}::{}", template_id_str, step_id_str, name)
}

/// 规范化变量键名
/// 
/// 分析键名，去除重复部分，确保它符合模板ID::步骤ID::变量名的格式
/// 
/// # 参数
/// 
/// - `key`: 原始变量键名
/// 
/// # 返回值
/// 
/// 返回规范化后的键名
pub fn normalize_variable_key(key: &str) -> String {
    // 如果键不包含分隔符，直接返回
    if !key.contains("::") {
        return key.to_string();
    }
    
    let parts: Vec<&str> = key.split("::").collect();
    
    // 如果只有一个或两个部分(如template::var或var)，直接返回
    if parts.len() <= 2 {
        return key.to_string();
    }
    
    // 检查是否存在重复部分（如README::README::var）
    let mut result_parts = Vec::new();
    let mut seen_parts = HashSet::new();
    
    // 处理模板ID和步骤ID部分(除了最后一个变量名)
    for part in &parts[..parts.len()-1] {  
        if !seen_parts.contains(*part) {
            seen_parts.insert(*part);
            result_parts.push(*part);
        } else {
            debug!("检测到重复部分在键名中: {}", part);
            // 重复部分不添加
        }
    }
    
    // 添加变量名部分
    result_parts.push(parts[parts.len()-1]);
    
    // 重新组装键名
    let normalized = result_parts.join("::");
    if normalized != key {
        debug!("规范化键名: {} -> {}", key, normalized);
    }
    
    normalized
}

/// 将点符号转换为双冒号符号
/// 
/// 用于处理命名空间引用，将a.b格式转换为a::b格式
/// 
/// # 参数
/// 
/// - `name`: 包含点符号的变量引用
/// 
/// # 返回值
/// 
/// 返回转换后的字符串
pub fn convert_dot_to_colon(name: &str) -> String {
    name.replace('.', "::")
}

/// 生成变量哈希标识符
/// 
/// 为变量创建唯一的哈希标识符，可用于快速比较和存储
/// 
/// # 参数
/// 
/// - `template_id`: 模板ID
/// - `step_id`: 步骤ID
/// - `name`: 变量名
/// 
/// # 返回值
/// 
/// 返回变量的哈希标识符
pub fn hash_variable(template_id: &str, step_id: &str, name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    format!("{}::{}::{}", template_id, step_id, name).hash(&mut hasher);
    hasher.finish()
}

/// Represents a complete test report for a unit on a specific distribution.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Report {
    /// The name of the distribution being tested.
    pub target: String,
    /// The version of the operating system.
    pub os_version: String,
    /// The version of the kernel (deprecated).
    pub kernel_version: String,
    /// The name of the unit being tested.
    pub unit_name: String,
    /// A collection of extra metadata for the unit,
    /// defined by `metadata.sh` in the unit's subdirectory.
    pub unit_metadata: PackageMetadata,
    /// A collection of individual test results.
    pub test_results: Vec<TestResult>,
    /// Indicates whether all tests passed.
    pub all_tests_passed: bool,
}

/// Represents unit specific extra metadata information to be used in test reports
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageMetadata {
    /// The version of the unit being tested.
    pub unit_version: String,
    /// The pretty (formal) name of the unit.
    pub unit_pretty_name: String,
    /// The type of the unit (temporarily deprecated).
    pub unit_type: String,
    /// Brief detemplateion of the unit (optional).
    pub unit_detemplateion: String,
}

impl Default for PackageMetadata {
    fn default() -> PackageMetadata {
        PackageMetadata {
            unit_pretty_name: String::new(),
            unit_type: String::from("unit"),
            unit_version: String::from("Unknown"),
            unit_detemplateion: String::new(),
        }
    }
}

/// Represents the result of an individual test.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestResult {
    /// The name of the test.
    pub test_name: String,
    /// The output produced by the test.
    pub output: String,
    /// Indicates whether the test passed.
    pub passed: bool,
}

/// A utility struct for managing temporary files. (deprecated)
///
/// This struct ensures that the file is deleted when it goes out of scope.
pub struct TempFile {
    path: String,
}

impl TempFile {
    /// Creates a new `TempFile` instance.
    ///
    /// # Parameters
    ///
    /// - `path`: The path of the temporary file.
    ///
    /// # Returns
    ///
    /// A new `TempFile` instance.
    pub fn _new(path: String) -> Self {
        TempFile { path }
    }
}

impl Drop for TempFile {
    /// Attempts to remove the file when the `TempFile` instance is dropped.
    ///
    /// If the file removal fails, the error is silently ignored.
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Represents the output of a command execution.

#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// The command executed.
    pub command: String,
    /// The exit status of the command.
    pub exit_status: i32,
    /// The output (stdout and stderr) of the command.
    pub output: String,
}

/// Reads a TOML file into an arbitrary struct.
///
/// # Parameters
///
/// - `path`: The path of the TOML file.
///
/// # Returns
///
/// Returns a struct of the specified type containing deserialized data.
///
/// # Errors
///
/// Returns an error if data parsing fails.
pub fn read_toml_from_file<T>(path: &PathBuf) -> Result<T, Box<dyn Error>>
where
    T: DeserializeOwned,
{
    let content = fs::read_to_string(path)?;
    let config: T = match toml::de::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to parse TOML file: {}", e);
            return Err(Box::new(e));
        }
    };
    Ok(config)
}

/// Discover available distribution test directories under the working directory.
///
/// # Parameters
///
/// - `dir`: The path of the program's working directory.
///
/// # Returns
///
/// Returns a vector of strings containing the paths of the discovered distribution
/// test directories if successful, otherwise returns an error.
///
/// # Errors
///
/// Returns an error if directory traversal fails.
pub fn get_targets(dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut targets = Vec::new();
    debug!("Scanning targets in directory {}", dir.display());
    for subdir in dir.read_dir()? {
        let target = subdir?;
        debug!("Scanning subdirectory {}", target.path().display());
        let target_dir_path = target.path();
        if target_dir_path.is_dir() {
            debug!("Discovered target directory {}", target_dir_path.display());
            let target_dir_name = target.file_name().into_string().unwrap();
            let target_config_path = target_dir_path.join("config.toml");
            let target_config: TargetConfig = match read_toml_from_file(&target_config_path) {
                Ok(config) => {
                    debug!(
                        "Loaded config for target directory {}",
                        target_dir_path.display()
                    );
                    config
                }
                Err(_) => {
                    debug!(
                        "Cannot load config for target directory {}",
                        target_dir_path.display()
                    );
                    continue;
                }
            };
            debug!(
                "Loaded config for target {}: \n{:?}",
                target_dir_name, target_config
            );
            if true {
                targets.push(target_dir_name);
            }
        }
    }
    Ok(targets)
}

/// Discover available unit tests of the given distribution.
///
/// # Parameters
///
/// - `target`: The name of the distribution.
/// - `dir`: The path of the program's working directory.
///
/// # Returns
///
/// Returns a vector of strings containing the paths of the discovered unit
/// test directories under the given distribution's directory if successful, otherwise returns an error.
///
/// # Errors
///
/// Returns an error if directory traversal fails.
pub fn get_units(target: &str, dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let directory = dir.join(target);
    let mut units = Vec::new();
    for subdir in directory.read_dir()? {
        let unit = subdir?;
        let unit_dir_path = unit.path();
        if unit_dir_path.is_dir() {
            let unit_dir_name = unit.file_name().into_string().unwrap();
            units.push(unit_dir_name);
        }
    }
    Ok(units)
}

/// Discover available unit test directories under the given distribution directory.
///
/// # Parameters
///
/// - `targets`: Array of distribution names.
/// - `dir`: The path of the program's working directory.
///
/// # Returns
///
/// Returns a vector of strings containing the names of all the unit tests discovered,
/// otherwise returns an error. Note that duplicates would be removed from the list beforehand.
///
/// # Errors
///
/// Returns an error if the process fails.
pub fn get_all_units(targets: &[&str], dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut units = HashSet::new();
    for target in targets {
        let current_units = get_units(target, dir).unwrap_or_default();
        units.extend(current_units);
    }
    let mut units_vec: Vec<String> = units.into_iter().collect();
    units_vec.sort(); // do we really need sorting?
    Ok(units_vec)
}
