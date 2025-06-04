//! Utility types and constants for the lintestor project.
//!
//! This module provides common structures and utilities used across the project,
//! including report structures, temporary file management, and command output handling.

use log::{error, warn};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

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
        &template_id[0..len - 5]
    } else {
        template_id
    };

    if clean_id.contains("::") {
        warn!("模板ID不应包含'::'分隔符: {clean_id}, 进行清理");
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
    let file_stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    normalize_template_id(file_stem)
}

/// 从模板 Arc 引用和本地步骤 ID 获取 step 对应的 ResultID
///
/// # 参数
///
/// - `template`: 模板引用
/// - `local_step_id`: 本地步骤 ID
///
/// # 返回值
///
/// 返回 ResultID
pub fn get_result_id(template_id: &str, local_step_id: &str) -> String {
    let template_id = template_id.to_string();
    let step_id = local_step_id.to_string();
    format!("{template_id}::{step_id}")
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
            error!("Failed to parse TOML file: {e}");
            return Err(Box::new(e));
        }
    };
    Ok(config)
}
