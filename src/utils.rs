//! Utility types and constants for the lintestor project.
//!
//! This module provides common structures and utilities used across the project,
//! including report structures, temporary file management, and command output handling.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{error::Error, fs};

/// The remote temporary directory used for operations.
pub static REMOTE_TMP_DIR: &str = "/tmp/lintestor";

/// Represents a complete test report for a package on a specific distribution.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Report {
    /// The name of the distribution being tested.
    pub distro: String,
    /// The version of the operating system.
    pub os_version: String,
    /// The version of the kernel (deprecated).
    pub kernel_version: String,
    /// The name of the package being tested.
    pub package_name: String,
    /// A collection of extra metadata for the package,
    /// defined by `metadata.sh` in the package's subdirectory.
    pub package_metadata: PackageMetadata,
    /// A collection of individual test results.
    pub test_results: Vec<TestResult>,
    /// Indicates whether all tests passed.
    pub all_tests_passed: bool,
}

/// Represents package specific extra metadata information to be used in test reports
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageMetadata {
    /// The version of the package being tested.
    pub package_version: String,
    /// The pretty (formal) name of the package.
    pub package_pretty_name: String,
    /// The type of the package (temporarily deprecated).
    pub package_type: String,
    /// Brief description of the package (optional).
    pub package_description: String,
}

impl Default for PackageMetadata {
    fn default() -> PackageMetadata {
        PackageMetadata {
            package_pretty_name: String::new(),
            package_type: String::from("package"),
            package_version: String::from("Unknown"),
            package_description: String::new(),
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

/// A utility struct for managing temporary files.
///
/// This struct ensures that the file is deleted when it goes out of scope.
pub struct TempFile {
    path: String,
}

impl TempFile {
    /// Creates a new `TempFile` instance.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the temporary file.
    ///
    /// # Returns
    ///
    /// A new `TempFile` instance.
    pub fn new(path: String) -> Self {
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

pub fn read_toml_from_file<T>(path: &str) -> Result<T, Box<dyn Error>>
where
    T: DeserializeOwned,
{
    let content = fs::read_to_string(path)?;
    let config: T = toml::de::from_str(&content)?;
    Ok(config)
}
