//! Utility types and constants for the lintestor project.
//!
//! This module provides common structures and utilities used across the project,
//! including report structures, temporary file management, and command output handling.

use crate::template::{self, BatchOptions};
use log::{error, warn};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
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
        warn!("Template ID should not contain '::' separator: {clean_id}, cleaning up"); // 模板ID不应包含'::'分隔符: {clean_id}, 进行清理
        clean_id.split("::").next().unwrap_or(clean_id).to_string()
    } else {
        clean_id.to_string()
    }
}

/// 从文件路径获取模板ID
///
/// 从文件路径提取模板ID，考虑相对路径以避免冲突，移除扩展名和.test后缀
/// 路径分割符会被替换为点（.），以符合模板ID的规范。
///
/// # 参数
///
/// - `tests_dir`: 测试模板的根目录
/// - `file_path`: 文件路径
///
/// # 返回值
///
/// 返回提取的模板ID
pub fn get_template_id_from_path(tests_dir: &Path, file_path: &Path) -> String {
    let relative_path = file_path.strip_prefix(tests_dir).unwrap_or(file_path);
    let path_str = relative_path.to_string_lossy().to_string();
    let cleaned_id = if let Some(stripped) = path_str.strip_suffix(".test.md") {
        stripped.to_string()
    } else if let Some(stripped) = path_str.strip_suffix(".test") {
        stripped.to_string()
    } else {
        path_str
    };
    let final_id = cleaned_id.replace(std::path::MAIN_SEPARATOR, ".");
    normalize_template_id(&final_id)
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
    let content: String = fs::read_to_string(path)?
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n");
    let config: T = match toml::de::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to parse TOML file: {e}");
            return Err(Box::new(e));
        }
    };
    Ok(config)
}

/// Generate report file path (supports keeping directory structure or flat structure)
///
/// # Parameters
/// * `batch_options` - Batch execution options
/// * `report_dir` - Report output directory
/// * `template_path` - Template file path
/// * `template_id` - Template ID
/// * `test_dir_root` - Test directory root path (optional)
///
/// # Returns
/// Returns the generated report file path
pub fn generate_report_path(
    batch_options: &Option<BatchOptions>,
    template_arc: &std::sync::Arc<template::TestTemplate>,
) -> anyhow::Result<PathBuf> {
    let options = match batch_options {
        Some(options) => options,
        None => {
            warn!("Batch options not provided, defaulting to default BatchOptions.");
            &BatchOptions::default()
        }
    };
    let (report_dir, test_dir) = (
        options.report_directory.as_deref(),
        options.test_directory.as_deref(),
    );
    let report_dir = match report_dir {
        Some(dir) => dir,
        None => {
            return Err(anyhow::anyhow!(
                "report_directory is not set in BatchOptions"
            ));
        }
    };
    let template_path = template_arc.file_path.as_path();

    let file_stem = template_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let report_file_name = format!("{}.report.md", file_stem.trim_end_matches(".test.md"));
    let keep_structure = options.keep_template_directory_structure;

    let final_report_path = if keep_structure {
        if let Some(test_dir_root) = test_dir {
            match template_path.strip_prefix(test_dir_root) {
                Ok(relative_template_path) => {
                    if let Some(relative_template_dir) = relative_template_path.parent() {
                        report_dir
                            .join(relative_template_dir)
                            .join(&report_file_name)
                    } else {
                        report_dir.join(&report_file_name)
                    }
                }
                Err(_) => {
                    warn!(
                        "Template path {} is not relative to test_dir_root {}. Falling back to flat report structure.",
                        template_path.display(),
                        test_dir_root.display()
                    );
                    report_dir.join(&report_file_name)
                }
            }
        } else {
            warn!(
                "test_dir_root not provided but keep_template_directory_structure is true. Falling back to flat report structure."
            );
            report_dir.join(&report_file_name)
        }
    } else {
        report_dir.join(&report_file_name)
    };
    // Ensure the parent directory of the report file exists
    if let Some(parent_dir) = final_report_path.parent() {
        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to create report directory {}: {}",
                    parent_dir.display(),
                    e
                )
            })?;
        }
    }
    Ok(final_report_path)
}
