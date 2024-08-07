use std::env;
use crate::test_runner::{TestRunner, LocalTestRunner, RemoteTestRunner};

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

pub fn run_test(
    remote_ip: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    distro: &str,
    package: &str,
    run_locally: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let runner: Box<dyn TestRunner> = if run_locally {
        Box::new(LocalTestRunner)
    } else {
        Box::new(RemoteTestRunner::new(
            remote_ip.to_string(),
            port,
            username.to_string(),
            password.map(String::from),
        ))
    };

    runner.run_test(distro, package)
}