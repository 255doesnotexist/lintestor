//! Test runner for local test environments.
//!
//! This module implements the `TestRunner` trait for the `LocalTestRunner` struct.
use crate::aggregator::generate_report;
use crate::test_runner::TestRunner;
use crate::testscript_manager::TestScriptManager;
use crate::utils::{PackageMetadata, Report, TestResult, REMOTE_TMP_DIR};
use log::{debug, log_enabled, Level};
use std::fs::read_to_string;
use std::path::Path;
use std::process::{Command, Stdio};
pub struct LocalTestRunner {}

impl LocalTestRunner {
    pub fn new(_target: &str, _unit: &str) -> Self {
        LocalTestRunner {}
    }
}

impl TestRunner for LocalTestRunner {
    /// Runs a test for a specific distribution and unit.
    ///
    /// # Arguments
    ///
    /// * `target` - The name of the distribution.
    /// * `unit` - The name of the unit.
    /// * `skip_scripts` - Some scripts skiped by use --skip-successful
    /// * `dir` - Working directory which contains the test folders and files, defaults to env::current_dir()
    ///
    /// # Errors
    ///
    /// Returns an error if any of the following occurs:
    ///
    /// * The test script manager fails to initialize.
    /// * Reading the OS version from `/proc/version` fails.
    /// * Running the `uname -r` command to get the kernel version fails.
    /// * Writing the unit version to the temporary file fails.
    /// * Running the test script fails.
    /// * Reading the unit version from the temporary file fails.
    /// * Generating the report fails.
    /// * Not all tests passed for the given distribution and unit.
    fn run_test(
        &self,
        target: &str,
        unit: &str,
        skip_scripts: Vec<String>,
        dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let script_manager = TestScriptManager::new(target, unit, skip_scripts, dir)?;

        let os_version = read_to_string("/proc/version")?;
        let kernelver_output = Command::new("uname").arg("-r").output()?;
        let kernel_version = String::from_utf8_lossy(&kernelver_output.stdout).to_string();
        let mut all_tests_passed = true;
        let mut test_results = Vec::new();

        let prerequisite_path = dir.join(format!("{}/prerequisite.sh", target));

        for script in script_manager.get_test_scripts() {
            let output = Command::new("bash")
                .arg("-c")
                .arg(format!(
                    "mkdir -p {} {} && source {}",
                    REMOTE_TMP_DIR,
                    if Path::new(&prerequisite_path).exists() {
                        format!("&& source {}", prerequisite_path.display())
                    } else {
                        String::from("")
                    },
                    script
                ))
                .stdout(if log_enabled!(Level::Debug) {
                    Stdio::inherit()
                } else {
                    Stdio::null()
                })
                .output()?;

            let test_passed = output.status.success();
            all_tests_passed &= test_passed;

            test_results.push(TestResult {
                test_name: script.to_string(),
                output: format!(
                    "stdout:'{}', stderr:'{}'",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ),
                passed: test_passed,
            });
        }

        let unit_metadata = if let Some(metadata_script) = script_manager.get_metadata_script() {
            let metadata_command = format!(
                "source {} && echo $PACKAGE_VERSION && echo $PACKAGE_PRETTY_NAME && echo $PACKAGE_TYPE && echo $PACKAGE_DESCRIPTION",
                metadata_script
            );
            let metadata_output = Command::new("bash")
                .arg("-c")
                .arg(metadata_command)
                .output()?;
            let metadata_vec: Vec<String> = String::from_utf8_lossy(&metadata_output.stdout)
                .lines()
                .map(|line| line.to_string())
                .collect();
            debug!("Collected metadata: {:?}", metadata_vec);
            if let [version, pretty_name, unit_type, description] = &metadata_vec[..] {
                PackageMetadata {
                    unit_version: version.to_owned(),
                    unit_pretty_name: pretty_name.to_owned(),
                    unit_type: unit_type.to_owned(),
                    unit_description: description.to_owned(),
                }
            } else {
                // 处理错误情况，向量长度不足
                panic!("Unexpected metadata format: not enough elements in metadata_vec");
            }
        } else {
            PackageMetadata {
                unit_pretty_name: unit.to_string(),
                ..Default::default()
            }
        };

        let report = Report {
            target: target.to_string(),
            os_version,
            kernel_version,
            unit_name: unit.to_string(),
            unit_metadata,
            test_results,
            all_tests_passed,
        };

        let report_path = dir.join(format!("{}/{}/report.json", target, unit));
        generate_report(&report_path, report)?;

        if !all_tests_passed {
            return Err(format!("Not all tests passed for {}/{}", target, unit).into());
        }

        Ok(())
    }
}
