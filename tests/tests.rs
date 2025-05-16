use assert_cmd::Command;
use std::{
    env, fs,
    io::{self, Write},
    path::Path,
};
mod tests {
    use super::*;

    #[test]
    fn integration_test() {
        // 测试文件目录
        let test_dir = "tests/test_files";

        // 预期的报告文件路径
        let reports_json_path = format!("{}/reports.json", test_dir);
        let summary_md_path = format!("{}/summary.md", test_dir);

        // 运行命令
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let output = cmd
            .arg("-tas")
            .arg("-D")
            .arg(test_dir)
            .env("RUST_LOG", "debug")
            .output()
            .expect("failed to execute process");

        // 输出命令执行结果
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        // 输出报告文件内容到标准输出
        println!("\n--- Reports JSON Content ---");
        if let Ok(reports_json) = fs::read_to_string(&reports_json_path) {
            println!("{}", reports_json);
        } else {
            println!("报告文件 '{}' 不存在或无法读取", reports_json_path);
        }

        println!("\n--- Summary MD Content ---");
        if let Ok(summary_md) = fs::read_to_string(&summary_md_path) {
            println!("{}", summary_md);
        } else {
            println!("汇总报告 '{}' 不存在或无法读取", summary_md_path);
        }

        // 验证报告文件是否存在
        let reports_json_exists = Path::new(&reports_json_path).exists();
        let summary_md_exists = Path::new(&summary_md_path).exists();

        // 验证测试执行状态和报告文件存在性
        assert!(output.status.success(), "程序执行失败");
        assert!(
            reports_json_exists,
            "reports.json 文件未生成在预期位置: {}",
            reports_json_path
        );
        assert!(
            summary_md_exists,
            "summary.md 文件未生成在预期位置: {}",
            summary_md_path
        );
    }
}
