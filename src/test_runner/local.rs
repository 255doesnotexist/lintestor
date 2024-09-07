use crate::aggregator::generate_report;
use crate::test_runner::TestRunner;
use crate::testscript_manager::TestScriptManager;
use crate::utils::{Report, TestResult, REMOTE_TMP_DIR};
use log::{log_enabled, Level};
use std::fs::read_to_string;
use std::process::{Command, Stdio};
pub struct LocalTestRunner {}

impl LocalTestRunner {
    pub fn new(_distro: &str, _package: &str) -> Self {
        LocalTestRunner {}
    }
}

impl TestRunner for LocalTestRunner {
    /// Runs a test for a specific distribution and package.
    ///
    /// # Arguments
    ///
    /// * `distro` - The name of the distribution.
    /// * `package` - The name of the package.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the following occurs:
    ///
    /// * The test script manager fails to initialize.
    /// * Reading the OS version from `/proc/version` fails.
    /// * Running the `uname -r` command to get the kernel version fails.
    /// * Writing the package version to the temporary file fails.
    /// * Running the test script fails.
    /// * Reading the package version from the temporary file fails.
    /// * Generating the report fails.
    /// * Not all tests passed for the given distribution and package.
    fn run_test(&self, distro: &str, package: &str) -> Result<(), Box<dyn std::error::Error>> {
        let script_manager = TestScriptManager::new(distro, package);

        let os_version = read_to_string("/proc/version")?;
        let kernelver_output = Command::new("uname").arg("-r").output()?;
        let kernel_version = String::from_utf8_lossy(&kernelver_output.stdout).to_string();
        let mut all_tests_passed = true;
        let mut test_results = Vec::new();

        let pkgver_tmpfile = format!("{}/pkgver", REMOTE_TMP_DIR);

        for script in script_manager?.get_test_scripts() {
            let output = Command::new("bash")
                .arg("-c")
                .arg(&format!(
                    "mkdir -p {} && source {} && echo -n $PACKAGE_VERSION > {}",
                    REMOTE_TMP_DIR, script, pkgver_tmpfile
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
                    String::from_utf8_lossy(&output.stdout).to_string(),
                    String::from_utf8_lossy(&output.stderr).to_string()
                ),
                passed: test_passed,
            });
        }

        let package_version = read_to_string(&pkgver_tmpfile)?;

        let report = Report {
            distro: distro.to_string(),
            os_version,
            kernel_version,
            package_name: package.to_string(),
            package_type: String::from("package"),
            package_version, // partially removed
            // TODO: add a metadata.sh script for every package
            // which generate a metadata.json file containing package version
            // and other metadata (different distros / packages have really different
            // metadata fetching methods so it is essential to write a metadata.sh for each one seperately)
            test_results,
            all_tests_passed,
        };

        let report_path = format!("{}/{}/report.json", distro, package);
        generate_report(report_path, report)?;

        if !all_tests_passed {
            return Err(format!("Not all tests passed for {}/{}", distro, package).into());
        }

        Ok(())
    }
}
