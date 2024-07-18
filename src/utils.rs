// File: utils.rs
// Description: 工具模块，包含辅助函数和结构体定义。

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Report {
    pub os_version: String,
    pub kernel_version: String,
    pub package_version: String,
    pub test_results: Vec<TestResult>,
    pub all_tests_passed: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
}
