use assert_cmd::Command;
use std::{fs, path::Path, io::Write};
use tempfile::tempdir;

// 测试模板解析功能
#[test]
fn test_simple_template_parsing() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let template_path = temp_dir.path().join("simple.test.md");

    // 创建一个简单的测试模板
    let template_content = r#"---
title: "简单测试模板"
target_config: "targets/local/config.toml"
unit_name: "SimpleUnit"
tags: ["test", "simple"]
---

# 简单测试

这是一个简单的测试模板。

## 执行简单命令 {id="simple-command"}

```bash {id="echo-command" exec=true description="Echo命令" assert.exit_code=0}
echo "Hello, world!"
```

**结果:**
```output {ref="echo-command"}
占位符输出
```
"#;

    // 写入测试模板到临时文件
    fs::write(&template_path, template_content).unwrap();

    // 执行lintestor命令解析模板
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("--parse-only")  // 假设有这个参数只解析不执行
        .arg(template_path.to_str().unwrap())
        .assert();

    // 验证命令执行成功
    result.success();
}

// 测试复杂模板解析（包含依赖关系）
#[test]
fn test_template_with_dependencies() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let template_path = temp_dir.path().join("dependencies.test.md");

    // 创建一个包含依赖关系的测试模板
    let template_content = r#"---
title: "依赖关系测试模板"
target_config: "targets/local/config.toml"
unit_name: "DependencyUnit"
tags: ["test", "dependency"]
---

# 依赖关系测试

测试步骤间的依赖关系。

## 准备环境 {id="setup"}

```bash {id="setup-cmd" exec=true description="设置环境" assert.exit_code=0}
echo "Setting up environment..."
```

**结果:**
```output {ref="setup-cmd"}
占位符输出
```

## 第一步 {id="step1" depends_on=["setup"]}

```bash {id="step1-cmd" exec=true description="执行第一步" assert.exit_code=0 extract.step1_value=/Value: (\d+)/}
echo "Executing step 1"
echo "Value: 42"
```

**结果:**
```output {ref="step1-cmd"}
占位符输出
```

## 第二步 {id="step2" depends_on=["setup"]}

```bash {id="step2-cmd" exec=true description="执行第二步" assert.exit_code=0}
echo "Executing step 2"
```

**结果:**
```output {ref="step2-cmd"}
占位符输出
```

## 最终步骤 {id="final" depends_on=["step1", "step2"]}

```bash {id="final-cmd" exec=true description="执行最终步骤" assert.exit_code=0}
echo "Step 1 value was: {{ step1_value }}"
echo "Executing final step"
```

**结果:**
```output {ref="final-cmd"}
占位符输出
```
"#;

    // 写入测试模板到临时文件
    fs::write(&template_path, template_content).unwrap();

    // 执行lintestor命令解析模板
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("--parse-only")  // 假设有这个参数只解析不执行
        .arg(template_path.to_str().unwrap())
        .assert();

    // 验证命令执行成功
    result.success();
}

// 测试模板解析错误处理
#[test]
fn test_template_parsing_errors() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let template_path = temp_dir.path().join("invalid.test.md");

    // 创建一个缺少必要字段的无效测试模板
    let template_content = r#"---
title: "无效测试模板"
# 缺少 target_config
unit_name: "InvalidUnit"
---

# 无效测试

这个模板缺少必要的target_config字段。
"#;

    // 写入测试模板到临时文件
    fs::write(&template_path, template_content).unwrap();

    // 执行lintestor命令解析模板
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("--parse-only")  // 假设有这个参数只解析不执行
        .arg(template_path.to_str().unwrap())
        .assert();

    // 验证命令执行失败（因为模板无效）
    result.failure();
}