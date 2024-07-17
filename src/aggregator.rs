// File: aggregator.rs
// Description: 汇总模块，负责汇总所有测试报告并生成综合报告。

use std::fs::File;
use std::io::prelude::*;
use crate::utils::Report;
use toml;

pub fn aggregate_reports(distros: &[&str], packages: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let mut consolidated_report = vec![];

    for distro in distros {
        for package in packages {
            let report_path = format!("{}/{}/report.toml", distro, package);
            if let Ok(mut file) = File::open(&report_path) {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let report: Report = toml::from_str(&contents)?;
                consolidated_report.push(report);
            }
        }
    }

    let consolidated_toml = toml::to_string(&consolidated_report)?;
    let mut file = File::create("reports.toml")?;
    file.write_all(consolidated_toml.as_bytes())?;
    Ok(())
}
