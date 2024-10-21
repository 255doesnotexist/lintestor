use assert_cmd::Command;
use std::{
    env,
    io::{self, Write},
};
#[test]
fn integration_test() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let output = cmd
        .arg("-tas")
        .arg("-D")
        .arg("tests/test_files")
        .env("RUST_LOG", "debug")
        .output()
        .expect("failed to execute process");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    // TODO: append contents of reports.json and summary.md to stdout
    assert!(output.status.success());
}
