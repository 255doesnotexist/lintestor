//! Test runner for remote test environments.
//!
//! This module implements the `TestRunner` trait for the `RemoteTestRunner` struct.
use crate::aggregator::generate_report;
use crate::test_runner::TestRunner;
use crate::testscript_manager::TestScriptManager;
use crate::utils::{CommandOutput, PackageMetadata, Report, TestResult, REMOTE_TMP_DIR};
use log::{debug, log_enabled, Level};
use ssh2::Session;
use std::{
    fs::File,
    io::{Read, Write},
    net::TcpStream,
    path::Path,
    process::Command,
};

pub struct RemoteTestRunner {
    remote_ip: String,
    port: u16,
    username: String,
    password: Option<String>,
}

impl RemoteTestRunner {
    pub fn new(remote_ip: String, port: u16, username: String, password: Option<String>) -> Self {
        RemoteTestRunner {
            remote_ip,
            port,
            username,
            password,
        }
    }

    /// Prints an SSH message. (maybe remove later)
    fn print_ssh_msg(&self, msg: &str) {
        // PRINT_SSH_MSG is deprecated, use RUST_LOG=debug
        if std::env::var("PRINT_SSH_MSG").is_ok() || log_enabled!(Level::Debug) {
            debug!("{}", msg);
        }
    }

    /// Runs a command on the remote server.
    /// # Arguments
    ///
    /// * `sess` - The SSH session.
    /// * `command` - The command to run.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails or encounters any issues.
    /// # Returns
    ///
    /// A `CommandOutput` struct containing the command, exit status, and output.
    ///
    fn run_command(
        &self,
        sess: &Session,
        command: &str,
    ) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        let mut channel = sess.channel_session()?;
        channel.exec(command)?;

        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        channel.send_eof()?;
        channel.wait_close()?;
        let command_output = CommandOutput {
            command: command.to_string(),
            exit_status: channel.exit_status()?,
            output: s,
        };
        self.print_ssh_msg(&format!("{:?}", command_output));
        Ok(command_output)
    }
}

/// Implements the `TestRunner` trait for the `RemoteTestRunner` struct.
///
/// This struct allows running tests on a remote server using SSH.
impl TestRunner for RemoteTestRunner {
    /// Runs a test on a remote server.
    ///
    /// # Arguments
    ///
    /// * `distro` - The name of the distribution.
    /// * `package` - The name of the package.
    /// * `dir` - Working directory which contains the test folders and files, defaults to env::current_dir()
    ///
    /// # Errors
    ///
    /// Returns an error if the test fails or encounters any issues.
    fn run_test(
        &self,
        distro: &str,
        package: &str,
        dir: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create SSH session
        let tcp = TcpStream::connect((self.remote_ip.as_str(), self.port))?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        self.print_ssh_msg("SSH handshake completed");

        // Authentication
        if let Some(password) = &self.password {
            sess.userauth_password(&self.username, password)?;
            self.print_ssh_msg("SSH password authentication completed");
        } else {
            sess.userauth_agent(&self.username)?;
            self.print_ssh_msg("SSH agent authentication completed");
        }
        if !sess.authenticated() {
            return Err("Authentication failed".into());
        }

        // Compress local test directory
        let local_dir = Path::new(dir).join(format!("{}/{}", distro, package));
        let tar_file_path_relative = format!("{}.tar.gz", package);
        let tar_file = Path::new(dir).join(tar_file_path_relative.clone());
        // let _temp_tar = TempFile::new(tar_file.clone());
        Command::new("tar")
            .arg("czf")
            .arg(&tar_file)
            .arg("-C")
            .arg(&local_dir)
            .arg(".")
            .output()?;
        self.print_ssh_msg(&format!(
            "Local directory {} compressed into {}",
            local_dir.display(),
            tar_file.display()
        ));

        // Make preparations on the remote server
        self.run_command(&sess, &format!("mkdir -p {}", REMOTE_TMP_DIR))?;

        // Upload compressed file to remote server
        let remote_tar_path = format!("{}/{}", REMOTE_TMP_DIR, tar_file_path_relative);
        let mut remote_file = sess.scp_send(
            Path::new(&remote_tar_path),
            0o644,
            std::fs::metadata(&tar_file)?.len(),
            None,
        )?;
        let mut local_file = File::open(&tar_file)?;
        let mut buffer = Vec::new();
        local_file.read_to_end(&mut buffer)?;
        remote_file.write_all(&buffer)?;
        self.print_ssh_msg(&format!(
            "File {} uploaded to remote server",
            tar_file_path_relative
        ));

        // Upload prerequisite.sh (optional) to remote server
        let prerequisite_path = Path::new(dir).join(format!("{}/prerequisite.sh", distro));
        if Path::new(&prerequisite_path).exists() {
            let remote_prerequisite_path = "/tmp/prerequisite.sh".to_string();
            let mut remote_file = sess.scp_send(
                Path::new(&remote_prerequisite_path),
                0o644,
                std::fs::metadata(&prerequisite_path)?.len(),
                None,
            )?;
            let mut local_file = File::open(&prerequisite_path)?;
            let mut buffer = Vec::new();
            local_file.read_to_end(&mut buffer)?;
            remote_file.write_all(&buffer)?;
            self.print_ssh_msg(&format!(
                "File {} uploaded to remote server",
                prerequisite_path.display()
            ));
        }
        // Ensure remote file is closed before proceeding
        drop(remote_file);

        // Clean up remote directory, extract files, and run tests on remote server
        let remote_dir = format!("{}/{}/{}", REMOTE_TMP_DIR, distro, package);
        self.print_ssh_msg(&format!(
            "Extracting file {} on remote server at {}",
            tar_file_path_relative, remote_dir
        ));
        if let Ok(CommandOutput {
            exit_status: 0,
            output: _,
            ..
        }) = self.run_command(
            &sess,
            &format!(
                "rm -rf {}; mkdir -p {} && tar xzf {} -C {} --overwrite",
                remote_dir, remote_dir, remote_tar_path, remote_dir
            ),
        ) {
            self.print_ssh_msg(&format!(
                "Successfully extracted file {} on remote server at {}",
                tar_file_path_relative, remote_dir
            ));
        } else {
            return Err("Failed to extract test files on remote server".into());
        }

        // Run test commands
        self.print_ssh_msg(&format!("Running tests in directory {}", remote_dir));

        let script_manager = TestScriptManager::new(distro, package, dir.to_string())?;
        let mut all_tests_passed = true;
        let mut test_results = Vec::new();
        for script in script_manager.get_test_script_names() {
            let remote_prerequisite_path = "/tmp/prerequisite.sh";
            let result = self.run_command(
                &sess,
                &format!(
                    "cd {}; {} source {}",
                    remote_dir,
                    if Path::new(&prerequisite_path).exists() {
                        format!("source {} &&", remote_prerequisite_path)
                    } else {
                        String::from("")
                    },
                    script
                ),
            )?;

            let test_passed = result.exit_status == 0;
            all_tests_passed &= test_passed;

            let output = &result.output;
            debug!("Command: {}", result.command);
            debug!("{:?}", &result);
            test_results.push(TestResult {
                test_name: script.to_string(),
                output: output.to_string(),
                passed: test_passed,
            });
        }

        if all_tests_passed {
            self.print_ssh_msg(&format!("Test successful for {}/{}", distro, package));
        } else {
            self.print_ssh_msg(&format!("Test failed for {}/{}", distro, package));
        }

        // Get OS version and package metadata
        let os_version = self.run_command(&sess, "cat /proc/version")?;
        let kernel_version = self.run_command(&sess, "uname -r")?;

        let package_metadata =
            if let Some(metadata_script_name) = script_manager.get_metadata_script_name() {
                let metadata_command = format!("source {}/{}", remote_dir, metadata_script_name);
                let metadata_output = self.run_command(&sess, &metadata_command)?;
                let metadata_vec: Vec<String> = metadata_output
                    .output
                    .to_string()
                    .lines()
                    .map(|line| line.to_string())
                    .collect();
                debug!(
                    "Collected metadata for {}/{} from remote stream: {:?}",
                    distro, package, metadata_vec
                );
                if let [version, pretty_name, package_type, description] = &metadata_vec[..] {
                    PackageMetadata {
                        package_version: version.to_owned(),
                        package_pretty_name: pretty_name.to_owned(),
                        package_type: package_type.to_owned(),
                        package_description: description.to_owned(),
                    }
                } else {
                    // 处理错误情况，如果 metadata_vec 不包含四个元素
                    panic!("Unexpected metadata format: not enough elements in metadata_vec");
                }
            } else {
                PackageMetadata {
                    package_pretty_name: package.to_string(),
                    ..Default::default()
                }
            };

        let report = Report {
            distro: distro.to_string(),
            os_version: os_version.output,
            kernel_version: kernel_version.output,
            package_name: package.to_string(),
            package_metadata,
            test_results,
            all_tests_passed,
        };

        // Compress remote test directory
        let remote_tar_file = format!("{}/../{}_result.tar.gz", remote_dir, package);
        self.print_ssh_msg(&format!(
            "Compressing remote directory {} into {}",
            remote_dir, remote_tar_file
        ));
        if let Ok(CommandOutput {
            exit_status: 0,
            output: _,
            ..
        }) = self.run_command(
            &sess,
            &format!(
                "cd {}/.. && tar czf {} -C {} . --overwrite",
                remote_dir, remote_tar_file, package
            ),
        ) {
            self.print_ssh_msg(&format!(
                "Successfully compressed remote test result at {} into {}",
                remote_dir, remote_tar_file
            ));
        } else {
            return Err("Failed to compress test results on remote server".into());
        }

        // Download compressed test directory
        let local_result_tar_file = local_dir.join(format!("{}_result.tar.gz", package));
        // let _temp_result_tar = TempFile::new(local_result_tar_file.clone());
        let (mut remote_file, _) = sess.scp_recv(Path::new(&remote_tar_file))?;
        let mut local_file = File::create(&local_result_tar_file)?;
        let mut buffer = Vec::new();
        remote_file.read_to_end(&mut buffer)?;
        local_file.write_all(&buffer)?;
        self.print_ssh_msg(&format!(
            "Downloaded test results to local file {}",
            local_result_tar_file.display()
        ));

        // Extract downloaded test results
        Command::new("tar")
            .arg("xzf")
            .arg(&local_result_tar_file)
            .arg("-C")
            .arg(&local_dir)
            .output()?;
        self.print_ssh_msg(&format!(
            "Extracted test results into local directory {}",
            local_dir.display()
        ));

        // Generate report locally

        let report_path = local_dir.join("report.json");
        generate_report(report_path, report.clone())?;
        debug!("{}-{} report:\n {:?}", distro, package, report);

        if !all_tests_passed {
            return Err(format!("Not all tests passed for {}/{}", distro, package).into());
        }
        Ok(())
    }
}
