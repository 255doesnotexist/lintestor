use crate::test_runner::TestRunner;
use crate::testscript_manager::TestScriptManager;
use crate::aggregator::generate_report;
use crate::utils::{CommandOutput, Report, TempFile, TestResult, REMOTE_TMP_DIR};
use ssh2::Session;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;

pub struct RemoteTestRunner {
    remote_ip: String,
    port: u16,
    username: String,
    password: Option<String>,
    verbose: bool,
}

impl RemoteTestRunner {
    pub fn new(remote_ip: String, port: u16, username: String, password: Option<String>, verbose: bool) -> Self {
        RemoteTestRunner {
            remote_ip,
            port,
            username,
            password,
            verbose,
        }
    }

    fn print_ssh_msg(&self, msg: &str) {
        if std::env::var("PRINT_SSH_MSG").is_ok() || self.verbose {
            println!("{}", msg);
        }
    }

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
    ///
    /// # Errors
    ///
    /// Returns an error if the test fails or encounters any issues.
    fn run_test(
        &self,
        distro: &str,
        package: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 创建 SSH 会话
        let tcp = TcpStream::connect((self.remote_ip.as_str(), self.port))?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        self.print_ssh_msg("SSH handshake completed");

        // 认证
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

        // 压缩本地测试目录
        let local_dir = format!("{}/{}", distro, package);
        let tar_file = format!("{}.tar.gz", package);
        let _temp_tar = TempFile::new(tar_file.clone());
        Command::new("tar")
            .arg("czf")
            .arg(&tar_file)
            .arg("-C")
            .arg(&local_dir)
            .arg(".")
            .output()?;
        self.print_ssh_msg(&format!(
            "Local directory {} compressed into {}",
            local_dir, tar_file
        ));

        // make preparations on the remote server
        self.run_command(&sess, &format!("mkdir -p {}", REMOTE_TMP_DIR))?;

        // 上传压缩文件到远程服务器
        let remote_tar_path = format!("{}/{}", REMOTE_TMP_DIR, tar_file);
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
        self.print_ssh_msg(&format!("File {} uploaded to remote server", tar_file));

        // 上传 prerequisite.sh 到远程服务器
        let prerequisite_path = format!("{}/prerequisite.sh", distro);
        let remote_prerequisite_path = "/tmp/prerequisite.sh".to_string();
        let mut remote_file = sess.scp_send(Path::new(&remote_prerequisite_path), 0o644, std::fs::metadata(&prerequisite_path)?.len(), None)?;
        let mut local_file = File::open(&prerequisite_path)?;
        let mut buffer = Vec::new();
        local_file.read_to_end(&mut buffer)?;
        remote_file.write_all(&buffer)?;
        self.print_ssh_msg(&format!("File {} uploaded to remote server", prerequisite_path));

        // 确保远程文件在继续之前关闭
        drop(remote_file);

        // 清理远程目录，解压文件并在远程服务器上运行测试
        let remote_dir = format!("{}/{}", REMOTE_TMP_DIR, package);
        self.print_ssh_msg(&format!(
            "Extracting file {} on remote server at {}",
            tar_file, remote_dir
        ));
        if let Ok(CommandOutput {
            exit_status: 0,
            output: _,
        }) = self.run_command(
            &sess,
            &format!(
                "rm -rf {}; mkdir -p {} && tar xzf {} -C {} --overwrite",
                remote_dir, remote_dir, remote_tar_path, remote_dir
            ),
        ) {
            self.print_ssh_msg(&format!(
                "Successfully extracted file {} on remote server at {}",
                tar_file, remote_dir
            ));
        } else {
            return Err("Failed to extract test files on remote server".into());
        }

        // 运行测试命令
        self.print_ssh_msg(&format!("Running tests in directory {}", remote_dir));
        let script_manager = TestScriptManager::new(distro, package);
        let mut all_tests_passed = true;
        let mut test_results = Vec::new();
        let pkgver_tmpfile = format!("{}/pkgver", REMOTE_TMP_DIR);
        for script in script_manager?.get_test_scripts() {
            let result = self.run_command(
                &sess,
                &format!("source {} && echo -n $PACKAGE_VERSION > {}", script, pkgver_tmpfile),
            );
            let test_passed = result.is_ok();
            all_tests_passed &= test_passed;
            test_results.push(TestResult {
                test_name: script.to_string(),
                output: result.unwrap().output,
                passed: test_passed,
            });
        }

        // 获取系统版本和包版本
        let os_version = self.run_command(&sess, "cat /proc/version")?;
        let package_version = self.run_command(
            &sess,
            &format!("cat {} && rm {}", pkgver_tmpfile, pkgver_tmpfile),
        )?;
        let kernel_version = self.run_command(&sess, "uname -r")?;
        if all_tests_passed {
            self.print_ssh_msg(&format!("Test successful for {}/{}", distro, package));
        } else {
            self.print_ssh_msg(&format!(
                "Test failed for {}/{}",
                distro, package
            ));
        }
        let report: Report = Report {
            distro: distro.to_string(),
            os_version: os_version.output,
            kernel_version: kernel_version.output,
            package_name: package.to_string(),
            package_type: String::from("package"),
            package_version: package_version.output,
            test_results,
            all_tests_passed,
        };

        // 压缩远程测试目录
        let remote_tar_file = format!("{}/{}_result.tar.gz", REMOTE_TMP_DIR, package);
        self.print_ssh_msg(&format!(
            "Compressing remote directory {} into {}",
            remote_dir, remote_tar_file
        ));
        if let Ok(CommandOutput {
            exit_status: 0,
            output: _,
        }) = self.run_command(
            &sess,
            &format!(
                "cd {} && tar czf {} -C {} . --overwrite",
                REMOTE_TMP_DIR, remote_tar_file, package
            ),
        ) {
            self.print_ssh_msg(&format!(
                "Successfully compressed remote test result at {} into {}",
                remote_dir, remote_tar_file
            ));
        } else {
            return Err("Failed to compress test results on remote server".into());
        }

        // 下载压缩的测试目录
        let local_result_tar_file = format!("{}/{}_result.tar.gz", local_dir, package);
        let _temp_result_tar = TempFile::new(local_result_tar_file.clone());
        let (mut remote_file, _) = sess.scp_recv(Path::new(&remote_tar_file))?;
        let mut local_file = File::create(&local_result_tar_file)?;
        let mut buffer = Vec::new();
        remote_file.read_to_end(&mut buffer)?;
        local_file.write_all(&buffer)?;
        self.print_ssh_msg(&format!(
            "Downloaded test results to local file {}",
            local_result_tar_file
        ));

        // 解压下载的测试结果
        Command::new("tar")
            .arg("xzf")
            .arg(&local_result_tar_file)
            .arg("-C")
            .arg(&local_dir)
            .output()?;
        self.print_ssh_msg(&format!(
            "Extracted test results into local directory {}",
            local_dir
        ));

        // 下载测试报告
        let report_path = format!("{}/report.json", local_dir);
        generate_report(report_path.clone(), report)?;
        let report_file_path = Path::new(&report_path);
        if report_file_path.exists() {
            let mut file = File::open(&report_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            self.print_ssh_msg(&format!("Downloaded test report: {}", report_path));
            let report: Report = serde_json::from_str(&contents)?;
            println!("{}-{} report:\n {:?}", distro, package, report);
            if !report.all_tests_passed {
                return Err(format!("Not all tests passed {}/{}", distro, package).into());
            }
        } else {
            self.print_ssh_msg("Test report not found.");
        }
        Ok(())
    }
}