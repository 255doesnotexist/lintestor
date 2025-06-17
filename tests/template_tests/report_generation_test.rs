use assert_cmd::Command;
use std::{fs, io::Write, path::PathBuf};
use tempfile::tempdir;

// 测试报告生成功能
#[test]
fn test_report_generation() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let template_path = temp_dir.path().join("report_test.test.md");
    let report_path = temp_dir.path().join("report_test.report.md");

    // 创建测试模板目录
    fs::create_dir_all(temp_dir.path().join("targets/local")).unwrap();

    // 创建一个简单的本地配置文件
    let config_content = r#"
enabled = true
testing_type = "locally"
"#;
    fs::write(
        temp_dir.path().join("targets/local/config.toml"),
        config_content,
    )
    .unwrap();

    // 创建一个用于测试报告生成的模板
    let template_content = r#"---
title: "报告生成测试"
target_config: "targets/local/config.toml"
unit_name: "ReportUnit"
tags: ["test", "report"]
---

# 报告生成测试

测试执行于: {{ execution_date }}

## 执行命令获取数据 {id="data-cmd"}

```bash {id="get-data" exec=true description="获取测试数据" assert.exit_code=0 extract.system_info=/系统信息：(.+)/}
echo "系统信息：Linux测试环境 x86_64"
echo "内核版本：$(uname -r)"
```

**结果:**
```output {ref="get-data"}
占位符输出
```

## 摘要 {id="summary" generate_summary=true}

| 命令描述        | 状态            |
|----------------|----------------|
| 获取测试数据     | {{ status.get-data }} |

系统信息: {{ system_info }}
"#;

    // 写入测试模板到临时文件
    fs::write(&template_path, template_content).unwrap();

    // 执行lintestor命令运行测试并生成报告
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("--local") // 使用本地执行
        .arg("--template") // 指定模板文件
        .arg(template_path.to_str().unwrap())
        .arg("--report-path") // 指定报告输出路径
        .arg(report_path.to_str().unwrap())
        .env("RUST_LOG", "debug") // 启用调试日志
        .assert();

    // 验证命令执行成功
    result.success();

    // 检查报告文件是否已生成
    assert!(report_path.exists(), "报告文件未生成");

    // 读取报告内容
    let report_content = fs::read_to_string(&report_path).unwrap();

    // 验证报告内容包含关键信息
    assert!(report_content.contains("# 报告生成测试"), "报告缺少标题");
    assert!(report_content.contains("测试执行于:"), "报告缺少执行日期");
    assert!(
        report_content.contains("系统信息：Linux测试环境"),
        "报告缺少命令输出"
    );
    assert!(report_content.contains("系统信息:"), "报告缺少提取的变量");
}
