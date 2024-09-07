use crate::test_runner::TestRunner;
use crate::testscript_manager::TestScriptManager;
use crate::utils::{Report, TestResult, REMOTE_TMP_DIR};
use crate::aggregator::generate_report;
use std::fs::read_to_string;
use std::process::{Command, Stdio};

pub struct LocalTestRunner {
    verbose: bool,
}

impl LocalTestRunner {
    pub fn new(_distro: &str, _package: &str, _verbose: bool) -> Self {
        LocalTestRunner { verbose: _verbose }
    }
}

impl TestRunner for LocalTestRunner {
    fn run_test(
        &self,
        distro: &str,
        package: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
                .arg(&format!("source {} && echo -n $PACKAGE_VERSION > {}", script, pkgver_tmpfile))
                .stdout(if self.verbose {
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
                    "{}{}",
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
            package_version: package_version, // partially removed
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