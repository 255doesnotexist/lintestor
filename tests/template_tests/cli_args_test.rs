use assert_cmd::Command;
use std::{fs, io::Write, path::PathBuf};
use tempfile::tempdir;

// 测试基本的命令行参数处理
#[test]
fn test_basic_cli_args() {
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd.arg("--help").assert();

    result.success();
    result.stdout(predicates::str::contains("USAGE:"));
    result.stdout(predicates::str::contains("-t, --test"));
}

// 测试组合参数（-tas）功能
#[test]
fn test_combined_flags() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let test_dir = temp_dir.path().join("tests");
    fs::create_dir_all(&test_dir).unwrap();

    // 创建一个简单的测试模板
    let template_content = r#"---
title: "简单测试"
target_config: "targets/local/config.toml"
unit_name: "SimpleUnit"
tags: ["test"]
---

# 简单测试

## 测试命令 {id="test-cmd"}

```bash {id="cmd" exec=true description="测试命令" assert.exit_code=0}
echo "Hello, world!"
```

**结果:**
```output {ref="cmd"}
占位符输出
```
"#;

    // 创建目标配置目录
    let target_dir = temp_dir.path().join("targets/local");
    fs::create_dir_all(&target_dir).unwrap();

    // 创建简单配置文件
    let config_content = r#"
enabled = true
testing_type = "locally"
"#;
    fs::write(target_dir.join("config.toml"), config_content).unwrap();

    // 写入测试模板
    fs::write(test_dir.join("simple.test.md"), template_content).unwrap();

    // 执行测试、聚合、汇总组合命令
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("-tas")
        .arg("-D")
        .arg(test_dir.to_str().unwrap())
        .env("RUST_LOG", "debug")
        .assert();

    result.success();
}

// 测试单独模板执行参数
#[test]
fn test_single_template_execution() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let template_path = temp_dir.path().join("single.test.md");

    // 创建测试模板目录
    fs::create_dir_all(temp_dir.path().join("targets/local")).unwrap();

    // 创建配置文件
    let config_content = r#"
enabled = true
testing_type = "locally"
"#;
    fs::write(
        temp_dir.path().join("targets/local/config.toml"),
        config_content,
    )
    .unwrap();

    // 创建测试模板
    let template_content = r#"---
title: "单一模板测试"
target_config: "targets/local/config.toml"
unit_name: "SingleTemplate"
tags: ["test"]
---

# 单一模板测试

## 测试命令 {id="test-cmd"}

```bash {id="cmd" exec=true description="测试命令" assert.exit_code=0}
echo "这是单一模板测试"
```

**结果:**
```output {ref="cmd"}
占位符输出
```
"#;

    // 写入测试模板
    fs::write(&template_path, template_content).unwrap();

    // 执行单一模板测试
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("-t")
        .arg("--local")
        .arg("--template")
        .arg(template_path.to_str().unwrap())
        .env("RUST_LOG", "debug")
        .assert();

    result.success();
}

// 测试筛选参数
#[test]
fn test_filter_parameters() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let test_dir = temp_dir.path().join("tests");
    fs::create_dir_all(&test_dir).unwrap();

    // 创建目标配置目录
    let target_dir = temp_dir.path().join("targets/local");
    fs::create_dir_all(&target_dir).unwrap();

    // 创建配置文件
    let config_content = r#"
enabled = true
testing_type = "locally"
"#;
    fs::write(target_dir.join("config.toml"), config_content).unwrap();

    // 创建多个测试模板
    let template1_content = r#"---
title: "单元A测试"
target_config: "targets/local/config.toml"
unit_name: "UnitA"
tags: ["tag1", "tag2"]
---

# 单元A测试

## 测试命令 {id="test-cmd"}

```bash {id="cmd" exec=true description="测试命令" assert.exit_code=0}
echo "这是单元A测试"
```

**结果:**
```output {ref="cmd"}
占位符输出
```
"#;

    let template2_content = r#"---
title: "单元B测试"
target_config: "targets/local/config.toml"
unit_name: "UnitB"
tags: ["tag1", "tag3"]
---

# 单元B测试

## 测试命令 {id="test-cmd"}

```bash {id="cmd" exec=true description="测试命令" assert.exit_code=0}
echo "这是单元B测试"
```

**结果:**
```output {ref="cmd"}
占位符输出
```
"#;

    // 写入测试模板
    fs::write(test_dir.join("unit_a.test.md"), template1_content).unwrap();
    fs::write(test_dir.join("unit_b.test.md"), template2_content).unwrap();

    // 测试通过单元筛选
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("-t")
        .arg("--local")
        .arg("-D")
        .arg(test_dir.to_str().unwrap())
        .arg("--unit")
        .arg("UnitA")
        .env("RUST_LOG", "debug")
        .assert();

    result.success();

    // 测试通过标签筛选
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("-t")
        .arg("--local")
        .arg("-D")
        .arg(test_dir.to_str().unwrap())
        .arg("--tag")
        .arg("tag3")
        .env("RUST_LOG", "debug")
        .assert();

    result.success();
}

// 测试仅解析模式
#[test]
fn test_parse_only_mode() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let template_path = temp_dir.path().join("parse_test.test.md");

    // 创建测试模板
    let template_content = r#"---
title: "解析测试"
target_config: "targets/nonexistent/config.toml"
unit_name: "ParseTest"
tags: ["test"]
---

# 解析测试

## 测试命令 {id="test-cmd"}

```bash {id="cmd" exec=true description="测试命令" assert.exit_code=0}
echo "此命令不会执行"
```

**结果:**
```output {ref="cmd"}
占位符输出
```
"#;

    // 写入测试模板
    fs::write(&template_path, template_content).unwrap();

    // 执行仅解析模式（不应尝试执行命令或连接到目标）
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("-p")
        .arg("--template")
        .arg(template_path.to_str().unwrap())
        .env("RUST_LOG", "debug")
        .assert();

    result.success();
}

// 测试报告路径参数
#[test]
fn test_report_path_parameter() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let template_path = temp_dir.path().join("report_test.test.md");
    let report_path = temp_dir.path().join("custom_report.md");

    // 创建测试模板目录
    fs::create_dir_all(temp_dir.path().join("targets/local")).unwrap();

    // 创建配置文件
    let config_content = r#"
enabled = true
testing_type = "locally"
"#;
    fs::write(
        temp_dir.path().join("targets/local/config.toml"),
        config_content,
    )
    .unwrap();

    // 创建测试模板
    let template_content = r#"---
title: "报告路径测试"
target_config: "targets/local/config.toml"
unit_name: "ReportPathTest"
tags: ["test"]
---

# 报告路径测试

## 测试命令 {id="test-cmd"}

```bash {id="cmd" exec=true description="测试命令" assert.exit_code=0}
echo "测试自定义报告路径"
```

**结果:**
```output {ref="cmd"}
占位符输出
```
"#;

    // 写入测试模板
    fs::write(&template_path, template_content).unwrap();

    // 执行测试并指定自定义报告路径
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("-t")
        .arg("--local")
        .arg("--template")
        .arg(template_path.to_str().unwrap())
        .arg("--report-path")
        .arg(report_path.to_str().unwrap())
        .env("RUST_LOG", "debug")
        .assert();

    result.success();

    // 验证报告是否生成在指定路径
    assert!(report_path.exists(), "报告未生成在自定义路径");
}

// 测试详细模式参数
#[test]
fn test_verbose_mode() {
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("-v")
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stderr.contains("INFO") || stderr.contains("DEBUG"),
        "详细模式未生效，日志级别未调整"
    );
}

// 测试安静模式参数
#[test]
fn test_quiet_mode() {
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("-q")
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        !stderr.contains("INFO") && !stderr.contains("DEBUG"),
        "安静模式未生效，仍有常规日志输出"
    );
}
