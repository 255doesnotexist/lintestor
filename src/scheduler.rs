// File: scheduler.rs
// Description: Scheduler module responsible for running tests and generating test reports.

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use ssh2::Session;
use std::net::TcpStream;
use std::io::Read;
use crate::utils::{Report, TestResult};

pub fn run_test(remote_ip: &str, port: u16, username: &str, password: Option<&str>, distro: &str, package: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create SSH session
    let tcp = TcpStream::connect((remote_ip, port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Authenticate
    if let Some(password) = password {
        sess.userauth_password(username, password)?;
    } else {
        sess.userauth_agent(username)?;
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
        .arg(&local_dir)
        .output()?;

    // Upload the compressed file to the remote server
    let mut remote_file = sess.scp_send(Path::new(&tar_file), 0o644, std::fs::metadata(&tar_file)?.len(), None)?;
    let mut local_file = File::open(&tar_file)?;
    let mut buffer = Vec::new();
    local_file.read_to_end(&mut buffer)?;
    remote_file.write_all(&buffer)?;

    // Extract the file and run the tests on the remote server
    let remote_dir = format!("/tmp/{}", package);
    let mut channel = sess.channel_session()?;
    channel.exec(&format!("mkdir -p {} && tar xzf {} -C {}", remote_dir, tar_file, remote_dir))?;
    channel.send_eof();
    channel.wait_close()?;
    let exit_status = channel.exit_status()?;
    if exit_status != 0 {
        return Err("Failed to extract test files on remote server".into());
    }

    // Run the test command
    let mut channel = sess.channel_session()?;
    channel.exec(&format!("cd {}/{} && make test", remote_dir, package))?;
    let mut s = String::new();
    channel.read_to_string(&mut s)?;
    channel.send_eof();
    channel.wait_close()?;
    let exit_status = channel.exit_status()?;
    if exit_status != 0 {
        return Err(format!("Test failed for {}/{}: {}", distro, package, s).into());
    }

    // Download the test report
    let report_path = format!("{}/{}/report.toml", remote_dir, package);
    let (mut remote_file, _) = sess.scp_recv(Path::new(&report_path))?;
    let mut contents = String::new();
    remote_file.read_to_string(&mut contents)?;

    // Parse and print the report
    let report: Report = toml::from_str(&contents)?;
    println!("{:?}", report);

    // Clean up temporary files
    std::fs::remove_file(&tar_file)?;

    Ok(())
}
