pub mod local;
pub mod remote;

use crate::utils::{CommandOutput, Report, TempFile, TestResult, REMOTE_TMP_DIR};
use ssh2::Session;
use std::fs::{read_to_string, File};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};

pub trait TestRunner {
    fn run_test(
        &self,
        distro: &str,
        package: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}