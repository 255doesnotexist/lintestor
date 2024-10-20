//! Utility types and constants for the lintestor project.
//!
//! This module provides common structures and utilities used across the project,
//! including report structures, temporary file management, and command output handling.

use log::{debug, error};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashSet,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use crate::config::distro_config::DistroConfig;

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
pub fn get_distros(dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut distros = Vec::new();
    debug!("Scanning distros in directory {}", dir.display());
    for subdir in dir.read_dir()? {
        let distro = subdir?;
        debug!("Scanning subdirectory {}", distro.path().display());
        let distro_dir_path = distro.path();
        if distro_dir_path.is_dir() {
            debug!("Discovered distro directory {}", distro_dir_path.display());
            let distro_dir_name = distro.file_name().into_string().unwrap();
            let distro_config_path = distro_dir_path.join("config.toml");
            let distro_config: DistroConfig = match read_toml_from_file(&distro_config_path) {
                Ok(config) => {
                    debug!(
                        "Loaded config for distro directory {}",
                        distro_dir_path.display()
                    );
                    config
                }
                Err(_) => {
                    debug!(
                        "Cannot load config for distro directory {}",
                        distro_dir_path.display()
                    );
                    continue;
                }
            };
            debug!(
                "Loaded config for distro {}: \n{:?}",
                distro_dir_name, distro_config
            );
            if distro_config.enabled {
                distros.push(distro_dir_name);
            }
        }
    }
    Ok(distros)
}

/// Discover available package tests of the given distribution.
///
/// # Parameters
///
/// - `distro`: The name of the distribution.
/// - `dir`: The path of the program's working directory.
///
/// # Returns
///
/// Returns a vector of strings containing the paths of the discovered package
/// test directories under the given distribution's directory if successful, otherwise returns an error.
///
/// # Errors
///
/// Returns an error if directory traversal fails.
pub fn get_packages(distro: &str, dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let directory = dir.join(distro);
    let mut packages = Vec::new();
    for subdir in directory.read_dir()? {
        let package = subdir?;
        let package_dir_path = package.path();
        if package_dir_path.is_dir() {
            let package_dir_name = package.file_name().into_string().unwrap();
            packages.push(package_dir_name);
        }
    }
    Ok(packages)
}

/// Discover available package test directories under the given distribution directory.
///
/// # Parameters
///
/// - `distros`: Array of distribution names.
/// - `dir`: The path of the program's working directory.
///
/// # Returns
///
/// Returns a vector of strings containing the names of all the package tests discovered,
/// otherwise returns an error. Note that duplicates would be removed from the list beforehand.
///
/// # Errors
///
/// Returns an error if the process fails.
pub fn get_all_packages(distros: &[&str], dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut packages = HashSet::new();
    for distro in distros {
        let current_packages = get_packages(distro, dir).unwrap_or_default();
        packages.extend(current_packages);
    }
    let mut packages_vec: Vec<String> = packages.into_iter().collect();
    packages_vec.sort(); // do we really need sorting?
    Ok(packages_vec)
}
