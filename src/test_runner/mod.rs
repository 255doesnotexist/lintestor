//! Test runner for local and remote test environments.

use std::path::Path;

pub mod local;
pub mod remote;
pub mod boardtest;

/// Test runner for local and remote test environments.
pub trait TestRunner {
    fn run_test(
        &self,
        distro: &str,
        package: &str,
        skip_scripts: Vec<String>,
        dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
