//! Test runner for remote boardtest server test environments.
use crate::config::boardtest_config::BoardtestConfig;
use crate::test_runner::TestRunner;
use crate::utils::{CommandOutput, PackageMetadata, Report, TestResult, REMOTE_TMP_DIR};
use anyhow::Context as _;
use log::{debug, error, info};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::{Error, Error as StdError};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use async_trait::async_trait;

#[derive(Debug)]
pub struct BoardtestRunner {
    config: BoardtestConfig,
}

#[derive(Debug, Deserialize)]
struct CreateTestResponse {
    test_id: String,
}

#[derive(Debug, Deserialize)]
struct TestStatusResponse {
    status: String,
}

#[derive(Debug, Serialize)]
struct TestConfig {
    tests: Vec<TestCase>,
}

#[derive(Debug, Serialize)]
struct TestCase {
    name: String,
    command: String,
    expected_output: String,
    method: String,
    timeout: u64,
}

impl BoardtestRunner {
    pub fn new(config: &BoardtestConfig) -> Self {
        BoardtestRunner { config: config.clone() }
    }

    fn create_test_client(&self) -> Result<Client, anyhow::Error> {
        Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")
    }

    fn create_test(&self, client: &Client, base64_content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // First write the test configuration
        let test_config = TestConfig {
            tests: vec![
                TestCase {
                    name: "extract_and_run".to_string(),
                    command: format!(
                        "echo '{}' | base64 -d | tar xz -C {} && cd {}/test && bash test.sh",
                        base64_content, REMOTE_TMP_DIR, REMOTE_TMP_DIR
                    ),
                    expected_output: "0".to_string(), // We expect the test script to return 0
                    method: "exit_code".to_string(),
                    timeout: self.config.timeout_secs,
                }
            ]
        };

        let write_test_resp = client
            .post(format!("{}/write_test", self.config.api_url))
            .json(&json!({
                "token": self.config.token,
                "test_name": "package_test",
                "test_content": test_config
            }))
            .send()
            .context("Failed to write test configuration")?;

        if !write_test_resp.status().is_success() {
            return Err(anyhow!("Failed to write test configuration: {}", write_test_resp.text()?).into());
        }

        // Then create the test
        let create_test_args = format!(
            "-f -t -s -b {} -S {}{}",
            self.config.board_config,
            self.config.serial,
            if self.config.mi_sdk_enabled { " -M" } else { "" }
        );

        let create_resp = client
            .post(format!("{}/create_test", self.config.api_url))
            .json(&json!({
                "token": self.config.token,
                "args": create_test_args
            }))
            .send()
            .context("Failed to create test")?;

        if !create_resp.status().is_success() {
            return Err(anyhow!("Failed to create test: {}", create_resp.text()?).into());
        }

        let create_test_response: CreateTestResponse = create_resp.json()?;
        Ok(create_test_response.test_id)
    }

    fn start_test(&self, client: &Client, test_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let resp = client
            .post(format!("{}/start_test/{}", self.config.api_url, test_id))
            .json(&json!({
                "token": self.config.token
            }))
            .send()
            .context("Failed to start test")?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to start test: {}", resp.text()?).into());
        }
        Ok(())
    }

    fn wait_for_test_completion(&self, client: &Client, test_id: &str) -> Result<bool, anyhow::Error> {
        let start_time = Instant::now();
        
        while start_time.elapsed() < Duration::from_secs(self.config.timeout_secs) {
            let resp = client
                .get(format!("{}/test_status/{}", self.config.api_url, test_id))
                .send()
                .context("Failed to get test status")?;

            if !resp.status().is_success() {
                return Err(anyhow!("Failed to get test status: {}", resp.text()?));
            }

            let status: TestStatusResponse = resp.json()?;
            match status.status.as_str() {
                "completed" => return Ok(true),
                "failed" => return Ok(false),
                "stopped" => return Ok(false),
                _ => {
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
            }
        }

        Err(anyhow!("Test timeout after {} seconds", self.config.timeout_secs))
    }

    fn get_test_output(&self, client: &Client, test_id: &str) -> Result<String, anyhow::Error> {
        let resp = client
            .get(format!("{}/test_output/{}", self.config.api_url, test_id))
            .send()
            .context("Failed to get test output")?;

        if !resp.status().is_success() {
            return Err(anyhow!("Failed to get test output: {}", resp.text()?));
        }

        Ok(resp.text()?)
    }
}

#[async_trait::async_trait]
impl TestRunner for BoardtestRunner {
    fn run_test(
        &self,
        distro: &str,
        package: &str,
        skip_scripts: Vec<String>,
        dir: &Path,
    ) -> Result<(), Box<(dyn StdError + 'static)>> {
        info!("Starting boardtest for {}/{}", distro, package);
        
        // Create HTTP client
        let client = self.create_test_client()?;

        // Compress local test directory
        let local_dir = Path::new(dir).join(format!("{}/{}", distro, package));
        let mut tar_buffer = Vec::new();
        
        Command::new("tar")
            .arg("czf")
            .arg("-")
            .arg("-C")
            .arg(&local_dir)
            .arg(".")
            .output()
            .and_then(|output| {
                if output.status.success() {
                    tar_buffer = output.stdout;
                    Ok(())
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("tar command failed: {}", String::from_utf8_lossy(&output.stderr))
                    ))
                }
            })
            .context("Failed to create tar archive")?;

        // Convert to base64
        let base64_content = BASE64.encode(&tar_buffer);

        // Create and start test
        let test_id = self.create_test(&client, &base64_content)?;
        self.start_test(&client, &test_id)?;

        // Wait for test completion and get results
        let test_passed = self.wait_for_test_completion(&client, &test_id)?;
        let test_output = self.get_test_output(&client, &test_id)?;

        // Create test result
        let test_results = vec![TestResult {
            test_name: format!("{}/{}", distro, package),
            output: test_output,
            passed: test_passed,
        }];

        // Generate report
        let report = Report {
            distro: distro.to_string(),
            os_version: String::new(), // Could be fetched from board if needed
            kernel_version: String::new(), // Could be fetched from board if needed
            package_name: package.to_string(),
            package_metadata: PackageMetadata {
                package_pretty_name: package.to_string(),
                ..Default::default()
            },
            test_results,
            all_tests_passed: test_passed,
        };

        // Write report
        let report_path = local_dir.join("report.json");
        crate::aggregator::generate_report(&report_path, report)?;

        if !test_passed {
            return Err(anyhow!("Not all tests passed for {}/{}", distro, package).into());
        }

        Ok(())
    }
}