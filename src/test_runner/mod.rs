//! Test runner for local and remote test environments.

pub mod local;
pub mod remote;

/// Test runner for local and remote test environments.
pub trait TestRunner {
    fn run_test(&self, distro: &str, package: &str) -> Result<(), Box<dyn std::error::Error>>;
}
