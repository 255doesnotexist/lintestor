//! Test runner for local and remote test environments.

use std::path::Path;

pub mod boardtest;
pub mod local;
pub mod remote;

/// Test runner for local and remote test environments.
pub trait TestRunner {
    fn run_test(
        &self,
        target: &str,
        unit: &str,
        skip_scripts: Vec<String>,
        dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
