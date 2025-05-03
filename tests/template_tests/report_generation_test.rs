use assert_cmd::Command;
use std::{fs, path::PathBuf, io::Write};
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
    fs::write(temp_dir.path().join("targets/local/config.toml"), config_content).unwrap();
    
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
        .arg("--local")        // 使用本地执行
        .arg("--template")     // 指定模板文件
        .arg(template_path.to_str().unwrap())
        .arg("--report-path")  // 指定报告输出路径
        .arg(report_path.to_str().unwrap())
        .env("RUST_LOG", "debug")  // 启用调试日志
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
    assert!(report_content.contains("系统信息：Linux测试环境"), "报告缺少命令输出");
    assert!(report_content.contains("系统信息:"), "报告缺少提取的变量");
}

// 测试聚合报告功能
#[test]
fn test_report_aggregation() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let reports_dir = temp_dir.path().join("reports");
    fs::create_dir_all(&reports_dir).unwrap();
    
    // 创建测试模板目录
    fs::create_dir_all(temp_dir.path().join("targets/local")).unwrap();
    
    // 创建一个简单的本地配置文件
    let config_content = r#"
enabled = true
testing_type = "locally"
"#;
    fs::write(temp_dir.path().join("targets/local/config.toml"), config_content).unwrap();
    
    // 创建几个模拟报告文件
    let report1_content = r#"---
title: "测试报告1"
target_config: "targets/local/config.toml"
unit_name: "Unit1"
tags: ["test", "report"]
status: "pass"
---

# 测试报告1

测试执行于: 2025-05-04T10:00:00Z

## 执行结果 {id="results"}

所有测试通过。

## 摘要 {id="summary"}

| 测试 | 状态 |
|------|------|
| 测试1 | ✅ Pass |
| 测试2 | ✅ Pass |
"#;
    
    let report2_content = r#"---
title: "测试报告2"
target_config: "targets/local/config.toml"
unit_name: "Unit2"
tags: ["test", "report"]
status: "fail"
---

# 测试报告2

测试执行于: 2025-05-04T10:05:00Z

## 执行结果 {id="results"}

部分测试失败。

## 摘要 {id="summary"}

| 测试 | 状态 |
|------|------|
| 测试1 | ✅ Pass |
| 测试2 | ❌ Fail |
"#;

    // 写入模拟报告文件
    fs::write(reports_dir.join("unit1_local.report.md"), report1_content).unwrap();
    fs::write(reports_dir.join("unit2_local.report.md"), report2_content).unwrap();

    // 执行lintestor命令生成聚合报告
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("--aggregate")           // 聚合报告
        .arg("--reports-dir")         // 指定报告目录
        .arg(reports_dir.to_str().unwrap())
        .arg("--output")              // 指定输出文件
        .arg(temp_dir.path().join("reports.json").to_str().unwrap())
        .env("RUST_LOG", "debug")     // 启用调试日志
        .assert();

    // 验证命令执行成功
    result.success();

    // 检查聚合报告文件是否已生成
    let reports_json_path = temp_dir.path().join("reports.json");
    assert!(reports_json_path.exists(), "聚合报告文件未生成");

    // 读取聚合报告内容
    let reports_json = fs::read_to_string(&reports_json_path).unwrap();
    
    // 验证聚合报告包含关键信息
    assert!(reports_json.contains("Unit1"), "聚合报告缺少Unit1");
    assert!(reports_json.contains("Unit2"), "聚合报告缺少Unit2");
    assert!(reports_json.contains("pass"), "聚合报告缺少通过状态");
    assert!(reports_json.contains("fail"), "聚合报告缺少失败状态");
}

// 测试摘要报告生成
#[test]
fn test_summary_generation() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let reports_json_path = temp_dir.path().join("reports.json");
    let summary_path = temp_dir.path().join("summary.md");
    
    // 创建模拟的reports.json文件
    let reports_json_content = r#"{
  "reports": [
    {
      "template_id": "unit1_local",
      "template_title": "测试报告1",
      "unit_name": "Unit1",
      "target_name": "locally",
      "overall_status": "pass",
      "execution_date": "2025-05-04T10:00:00Z",
      "report_path": "reports/unit1_local.report.md"
    },
    {
      "template_id": "unit2_local",
      "template_title": "测试报告2",
      "unit_name": "Unit2",
      "target_name": "locally",
      "overall_status": "fail",
      "execution_date": "2025-05-04T10:05:00Z",
      "report_path": "reports/unit2_local.report.md"
    },
    {
      "template_id": "unit1_qemu",
      "template_title": "测试报告3",
      "unit_name": "Unit1",
      "target_name": "qemu_vm",
      "overall_status": "pass",
      "execution_date": "2025-05-04T10:10:00Z",
      "report_path": "reports/unit1_qemu.report.md"
    }
  ]
}"#;

    // 写入模拟的reports.json文件
    fs::write(&reports_json_path, reports_json_content).unwrap();

    // 执行lintestor命令生成摘要报告
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("--summarize")           // 生成摘要
        .arg("--reports-json")        // 指定报告JSON文件
        .arg(reports_json_path.to_str().unwrap())
        .arg("--summary-path")        // 指定摘要输出路径
        .arg(summary_path.to_str().unwrap())
        .env("RUST_LOG", "debug")     // 启用调试日志
        .assert();

    // 验证命令执行成功
    result.success();

    // 检查摘要文件是否已生成
    assert!(summary_path.exists(), "摘要文件未生成");

    // 读取摘要内容
    let summary_content = fs::read_to_string(&summary_path).unwrap();
    
    // 验证摘要包含关键信息
    assert!(summary_content.contains("# 测试摘要"), "摘要缺少标题");
    assert!(summary_content.contains("Unit1"), "摘要缺少Unit1");
    assert!(summary_content.contains("Unit2"), "摘要缺少Unit2");
    assert!(summary_content.contains("locally"), "摘要缺少locally目标");
    assert!(summary_content.contains("qemu_vm"), "摘要缺少qemu_vm目标");
    assert!(summary_content.contains("✅"), "摘要缺少通过状态标记");
    assert!(summary_content.contains("❌"), "摘要缺少失败状态标记");
}