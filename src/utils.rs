use serde::{Deserialize, Serialize};

pub static REMOTE_TMP_DIR: &str = "/tmp/lintestor";
#[derive(Serialize, Deserialize, Debug)]
pub struct Report {
    pub distro: String,
    pub os_version: String,
    pub kernel_version: String, // deprecated
    pub package_name: String,
    pub package_type: String, // temporarily deprecated
    pub package_version: String,
    pub test_results: Vec<TestResult>,
    pub all_tests_passed: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestResult {
    pub test_name: String,
    pub output: String,
    pub passed: bool,
}

pub struct TempFile {
    path: String,
}

impl TempFile {
    pub fn new(path: String) -> Self {
        TempFile { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[derive(Debug)]
pub struct CommandOutput {
    pub exit_status: i32,
    pub output: String,
}
