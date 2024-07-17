// File: scheduler.rs
// Description: 调度器模块，负责调用测试并生成测试报告。

use std::process::Command;
use std::fs::File;
use std::io::prelude::*;
use crate::utils::{Report, TestResult};

pub fn run_test(distro: &str, package: &str) -> Result<(), Box<dyn std::error::Error>> {
    let makefile_path = format!("{}/{}/Makefile", distro, package);
    if !std::path::Path::new(&makefile_path).exists() {
        return Err(format!("Makefile not found for {}/{}", distro, package).into());
    }

    let output = Command::new("make")
        .arg("test")
        .current_dir(format!("{}/{}", distro, package))
        .output()?;

    if !output.status.success() {
        return Err(format!("Test failed for {}/{}", distro, package).into());
    }

    let report_path = format!("{}/{}/report.toml", distro, package);
    let mut file = File::open(&report_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let report: Report = toml::from_str(&contents)?;
    println!("{:?}", report);
    Ok(())
}
