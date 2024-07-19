use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use ssh2::Session;
use std::net::TcpStream;
use std::io::Read;
use std::io;
use std::env;
use crate::utils::{Report, TestResult};

struct TempFile {
    path: String,
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn print_ssh_msg(msg: &str) {
    if env::var("PRINT_SSH_MSG").is_ok() {
        println!("{}", msg);
        // let _ = io::stdin().read(&mut [0u8]).unwrap();
    }
}

pub fn run_test(remote_ip: &str, port: u16, username: &str, password: Option<&str>, distro: &str, package: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create SSH session
    let tcp = TcpStream::connect((remote_ip, port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    print_ssh_msg("SSH handshake completed");

    // Authenticate
    if let Some(password) = password {
        sess.userauth_password(username, password)?;
        print_ssh_msg("SSH password authentication completed");
    } else {
        sess.userauth_agent(username)?;
        print_ssh_msg("SSH agent authentication completed");
    }

    if !sess.authenticated() {
        return Err("Authentication failed".into());
    }

    // Compress the local test directory
    let local_dir = format!("{}/{}", distro, package);
    let tar_file = format!("{}.tar.gz", package);
    Command::new("tar")
        .arg("czf")
        .arg(&tar_file)
        .arg("-C")
        .arg(&local_dir)
        .arg(".")
        .output()?;
    print_ssh_msg(&format!("Local directory {} compressed into {}", local_dir, tar_file));

    // Create TempFile instance to ensure cleanup
    let _temp_file = TempFile { path: tar_file.clone() };

    // Upload the compressed file to the remote server
    let mut remote_file = sess.scp_send(Path::new(&tar_file), 0o644, std::fs::metadata(&tar_file)?.len(), None)?;
    let mut local_file = File::open(&tar_file)?;
    let mut buffer = Vec::new();
    local_file.read_to_end(&mut buffer)?;
    remote_file.write_all(&buffer)?;
    print_ssh_msg(&format!("File {} uploaded to remote server", tar_file));

    // Ensure remote file is closed before proceeding
    drop(remote_file);

    // Extract the file and run the tests on the remote server
    let remote_dir = format!("/tmp/{}", package);
    let mut channel = sess.channel_session()?;
    channel.exec(&format!("mkdir -p {} && tar xzf {} -C {}  --overwrite", remote_dir, tar_file, remote_dir))?;
    print_ssh_msg(&format!("Extracting file {} on remote server at {}", tar_file, remote_dir));

    // Read the remote command's output to prevent deadlock
    let mut s = String::new();
    channel.read_to_string(&mut s)?;
    print_ssh_msg(&format!("Command output: {}", s));

    channel.send_eof();
    channel.wait_close()?;  // Ensure the channel is properly closed
    let exit_status = channel.exit_status()?;
    if exit_status != 0 {
        return Err("Failed to extract test files on remote server".into());
    }

    // Run the test command
    let mut channel = sess.channel_session()?;
    channel.exec(&format!("cd {} && ./test.sh", remote_dir))?;
    print_ssh_msg(&format!("Running tests in directory {}", remote_dir));
    let mut s = String::new();
    channel.read_to_string(&mut s)?;
    print_ssh_msg(&format!("Test command output: {}", s));
    channel.send_eof();
    channel.wait_close()?;  // Ensure the channel is properly closed
    let exit_status = channel.exit_status()?;
    if exit_status != 0 {
        return Err(format!("Test failed for {}/{}: {}", distro, package, s).into());
    }

    // Compress the remote test directory
    let remote_tar_file = format!("/tmp/{}_result.tar.gz", package);
    let mut channel = sess.channel_session()?;
    channel.exec(&format!("cd /tmp && tar czf {} -C {} .  --overwrite", remote_tar_file, package))?;
    print_ssh_msg(&format!("Compressing remote directory {} into {}", remote_dir, remote_tar_file));

    // Read the remote command's output to prevent deadlock
    let mut s = String::new();
    channel.read_to_string(&mut s)?;
    print_ssh_msg(&format!("Command output: {}", s));
    channel.send_eof();
    channel.wait_close()?;  // Ensure the channel is properly closed
    let exit_status = channel.exit_status()?;
    if exit_status != 0 {
        return Err("Failed to compress test results on remote server".into());
    }

    // Download the compressed test directory
    let local_result_tar_file = format!("{}/{}_result.tar.gz", local_dir, package);
    let (mut remote_file, _) = sess.scp_recv(Path::new(&remote_tar_file))?;
    let mut local_file = File::create(&local_result_tar_file)?;
    let mut buffer = Vec::new();
    remote_file.read_to_end(&mut buffer)?;
    local_file.write_all(&buffer)?;
    print_ssh_msg(&format!("Downloaded test results to local file {}", local_result_tar_file));

    // Extract the downloaded test results
    Command::new("tar")
        .arg("xzf")
        .arg(&local_result_tar_file)
        .arg("-C")
        .arg(&local_dir)
        .output()?;
    print_ssh_msg(&format!("Extracted test results into local directory {}", local_dir));

    // Clean up
    let _ = std::fs::remove_file(&local_result_tar_file);

    // Download the test report
    let report_path = format!("{}/report.json", local_dir);
    let report_file_path = Path::new(&report_path);
    if report_file_path.exists() {
        let mut file = File::open(&report_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        print_ssh_msg(&format!("Downloaded test report: {}", report_path));

        // Parse and print the report
        let report: Report = serde_json::from_str(&contents)?;
        println!("{}-{} report:\n {:?}", distro, package, report);

        if !report.all_tests_passed {
            return Err(format!("Not all tests passed {}/{}", distro, package).into());
        }
    } else {
        print_ssh_msg("Test report not found.");
    }

    Ok(())
}
