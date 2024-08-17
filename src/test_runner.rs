use crate::utils::{Report, TempFile};
use ssh2::Session;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;

pub trait TestRunner {
    fn run_test(&self, distro: &str, package: &str) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct LocalTestRunner;

impl LocalTestRunner {
    pub fn new(_distro: &str, _package: &str) -> Self {
        LocalTestRunner
    }
}

impl TestRunner for LocalTestRunner {
    fn run_test(&self, distro: &str, package: &str) -> Result<(), Box<dyn std::error::Error>> {
        let script_path = format!("{}/{}/test.sh", distro, package);
        let output = Command::new("bash").arg("-c").arg(&script_path).output()?;

        if !output.status.success() {
            return Err(format!(
                "Test failed for {}/{}: {}",
                distro,
                package,
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        // 读取并解析 report.json
        let report_path = format!("{}/{}/report.json", distro, package);
        let report_file = std::fs::File::open(Path::new(&report_path))?;
        let report: Report = serde_json::from_reader(report_file)?;

        if !report.all_tests_passed {
            return Err(format!("Not all tests passed for {}/{}", distro, package).into());
        }

        Ok(())
    }
}

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

    fn print_ssh_msg(&self, msg: &str) {
        if std::env::var("PRINT_SSH_MSG").is_ok() {
            println!("{}", msg);
        }
    }
}

impl TestRunner for RemoteTestRunner {
    fn run_test(&self, distro: &str, package: &str) -> Result<(), Box<dyn std::error::Error>> {
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

        // 上传压缩文件到远程服务器
        let remote_tar_path = format!("/tmp/lintestor/{}", tar_file);
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

        // 确保远程文件在继续之前关闭
        drop(remote_file);

        // 清理远程目录，解压文件并在远程服务器上运行测试
        let remote_dir = format!("/tmp/lintestor/{}", package);
        let mut channel = sess.channel_session()?;

        channel.exec(&format!(
            "rm -rf {}; mkdir -p {} && tar xzf {} -C {} --overwrite",
            remote_dir, remote_dir, tar_file, remote_dir
        ))?;
        self.print_ssh_msg(&format!(
            "Extracting file {} on remote server at {}",
            tar_file, remote_dir
        ));

        // 读取远程命令的输出以防止死锁
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        self.print_ssh_msg(&format!("Command output: {}", s));

        channel.send_eof()?;
        channel.wait_close()?;
        let exit_status = channel.exit_status()?;
        if exit_status != 0 {
            return Err("Failed to extract test files on remote server".into());
        }

        // 运行测试命令
        let mut channel = sess.channel_session()?;
        channel.exec(&format!("cd {} && ./test.sh", remote_dir))?;
        self.print_ssh_msg(&format!("Running tests in directory {}", remote_dir));
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        self.print_ssh_msg(&format!("Test command output: {}", s));
        channel.send_eof()?;
        channel.wait_close()?;
        let exit_status = channel.exit_status()?;
        if exit_status != 0 {
            return Err(format!("Test failed for {}/{}: {}", distro, package, s).into());
        }

        // 压缩远程测试目录
        let remote_tar_file = format!("/tmp/lintestor/{}_result.tar.gz", package);
        let mut channel = sess.channel_session()?;
        channel.exec(&format!(
            "cd /tmp/lintestor && tar czf {} -C {} . --overwrite",
            remote_tar_file, package
        ))?;
        self.print_ssh_msg(&format!(
            "Compressing remote directory {} into {}",
            remote_dir, remote_tar_file
        ));

        // 读取远程命令的输出以防止死锁
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        self.print_ssh_msg(&format!("Command output: {}", s));
        channel.send_eof()?;
        channel.wait_close()?;
        let exit_status = channel.exit_status()?;
        if exit_status != 0 {
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
        let report_file_path = Path::new(&report_path);
        if report_file_path.exists() {
            let mut file = File::open(&report_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            self.print_ssh_msg(&format!("Downloaded test report: {}", report_path));

            // 解析并打印报告
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
