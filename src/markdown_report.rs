use std::fs::File;
use std::io::prelude::*;
use crate::utils::Report;
use serde_json;

pub fn generate_markdown_report(distros: &[&str], packages: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    // Open the reports.json file containing the test results
    let file = File::open("reports.json")?;
    // Deserialize the JSON file into a vector of Report objects
    let reports: Vec<Report> = serde_json::from_reader(file)?;

    // Initialize an empty string for the markdown content
    let mut markdown = String::new();
    // Write the title of the markdown file
    markdown.push_str("# 软件包测试结果矩阵\n\n");
    // Write the header row of the table
    markdown.push_str("| 软件包 | 种类 | ");
    // Iterate over the distros and add them to the header row
    for distro in distros {
        markdown.push_str(&format!("{} | ", distro));
    }
    // Remove the last pipe character from the header row
    markdown.pop();
    // Write a newline after the header row
    markdown.push_str("\n");
    // Write the separator row for the table
    markdown.push_str("|:------|:-----| ");
    // Iterate over the distros and add them to the separator row
    for distro in distros {
        markdown.push_str(&format!(":-------| "));
    }
    // Remove the last pipe character from the separator row
    markdown.pop();
    // Write a newline after the separator row
    markdown.push_str("\n");

    // Iterate over the reports and add them to the markdown content
    for report in reports {
        // Write a row for the package name, version, and type
        markdown.push_str(&format!("| {} | {} ",
            report.package_name,
            report.package_type
        ));

        // println!("{:?}", report);
        // Write a row for the test name and result
        markdown.push_str(&format!("| {} {}-{} |\n",
            if report.all_tests_passed { "✅" } else { "⚠️" },
            report.package_name, report.package_version
        ));
            
        // Write a newline after each report
        markdown.push_str("\n");
    }

    // Open a new file for writing the markdown content
    let mut file = File::create("summary.md")?;
    // Write the markdown content to the file
    file.write_all(markdown.as_bytes())?;
    // Return a result indicating success or an error
    Ok(())
}