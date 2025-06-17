use assert_cmd::Command;
use std::{fs, path::Path, io::Write};
use tempfile::tempdir;

// 测试本地执行基本功能
#[test]
fn test_local_execution_basic() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let template_path = temp_dir.path().join("local_basic.test.md");
    
    // 创建测试模板目录
    fs::create_dir_all(temp_dir.path().join("targets/local")).unwrap();
    
    // 创建一个简单的本地配置文件
    let config_content = r#"
enabled = true
testing_type = "locally"
"#;
    fs::write(temp_dir.path().join("targets/local/config.toml"), config_content).unwrap();
    
    // 创建一个简单的测试模板
    let template_content = r#"---
title: "本地执行测试"
target_config: "targets/local/config.toml"
unit_name: "LocalUnit"
tags: ["test", "local"]
---

# 本地执行测试

## 执行简单命令 {id="simple-command"}

```bash {id="echo-command" exec=true description="Echo命令" assert.exit_code=0}
echo "Hello from local execution!"
```

**结果:**
```output {ref="echo-command"}
占位符输出
```

## 检查环境变量 {id="env-check"}

```bash {id="env-command" exec=true description="环境变量检查" assert.exit_code=0}
export TEST_VAR="test_value"
echo "TEST_VAR=${TEST_VAR}"
```

**结果:**
```output {ref="env-command"}
占位符输出
```
"#;

    // 写入测试模板到临时文件
    fs::write(&template_path, template_content).unwrap();

    // 执行lintestor命令运行本地测试
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("--local")  // 使用本地执行
        .arg("--template")  // 指定模板文件
        .arg(template_path.to_str().unwrap())
        .env("RUST_LOG", "debug")  // 启用调试日志
        .assert();

    // 验证命令执行成功
    result.success();
}

// 测试本地执行的变量提取和替换功能
#[test]
fn test_local_execution_variables() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let template_path = temp_dir.path().join("local_vars.test.md");
    
    // 创建测试模板目录
    fs::create_dir_all(temp_dir.path().join("targets/local")).unwrap();
    
    // 创建一个简单的本地配置文件
    let config_content = r#"
enabled = true
testing_type = "locally"
"#;
    fs::write(temp_dir.path().join("targets/local/config.toml"), config_content).unwrap();
    
    // 创建一个测试变量提取和替换的模板
    let template_content = r#"---
title: "变量测试"
target_config: "targets/local/config.toml"
unit_name: "VarUnit"
tags: ["test", "variables"]
---

# 变量提取和替换测试

## 第一步：提取变量 {id="extract-step"}

```bash {id="extract-cmd" exec=true description="提取变量" assert.exit_code=0 extract.version=/版本：(\d+\.\d+\.\d+)/}
echo "版本：1.2.3"
echo "其他信息：测试数据"
```

**结果:**
```output {ref="extract-cmd"}
占位符输出
```

## 第二步：使用变量 {id="use-var-step" depends_on=["extract-step"]}

```bash {id="use-var-cmd" exec=true description="使用变量" assert.exit_code=0 assert.stdout_contains="1.2.3"}
echo "提取的版本是：{{ version }}"
```

**结果:**
```output {ref="use-var-cmd"}
占位符输出
```

## 摘要 {id="summary"}

提取的版本：{{ version }}
"#;

    // 写入测试模板到临时文件
    fs::write(&template_path, template_content).unwrap();

    // 执行lintestor命令运行本地测试
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("--local")  // 使用本地执行
        .arg("--template")  // 指定模板文件
        .arg(template_path.to_str().unwrap())
        .env("RUST_LOG", "debug")  // 启用调试日志
        .assert();

    // 验证命令执行成功
    result.success();
}

// 测试本地执行的断言功能
#[test]
fn test_local_execution_assertions() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let template_path = temp_dir.path().join("local_assert.test.md");
    
    // 创建测试模板目录
    fs::create_dir_all(temp_dir.path().join("targets/local")).unwrap();
    
    // 创建一个简单的本地配置文件
    let config_content = r#"
enabled = true
testing_type = "locally"
"#;
    fs::write(temp_dir.path().join("targets/local/config.toml"), config_content).unwrap();
    
    // 创建一个测试断言功能的模板
    let template_content = r#"---
title: "断言测试"
target_config: "targets/local/config.toml"
unit_name: "AssertUnit"
tags: ["test", "assert"]
---

# 断言测试

## 测试退出码断言 {id="exit-code-test"}

```bash {id="exit-cmd" exec=true description="退出码测试" assert.exit_code=0}
echo "成功的命令"
exit 0
```

**结果:**
```output {ref="exit-cmd"}
占位符输出
```

## 测试标准输出包含断言 {id="stdout-contains-test"}

```bash {id="stdout-cmd" exec=true description="标准输出包含测试" assert.stdout_contains="成功"}
echo "这是一个成功的输出"
```

**结果:**
```output {ref="stdout-cmd"}
占位符输出
```

## 测试标准输出匹配正则 {id="stdout-regex-test"}

```bash {id="regex-cmd" exec=true description="正则匹配测试" assert.stdout_matches="^结果：\\d+$"}
echo "结果：42"
```

**结果:**
```output {ref="regex-cmd"}
占位符输出
```
"#;

    // 写入测试模板到临时文件
    fs::write(&template_path, template_content).unwrap();

    // 执行lintestor命令运行本地测试
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("--local")  // 使用本地执行
        .arg("--template")  // 指定模板文件
        .arg(template_path.to_str().unwrap())
        .env("RUST_LOG", "debug")  // 启用调试日志
        .assert();

    // 验证命令执行成功
    result.success();
}

// 测试本地执行的依赖关系处理
#[test]
fn test_local_execution_dependencies() {
    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let template_path = temp_dir.path().join("local_deps.test.md");
    
    // 创建测试模板目录
    fs::create_dir_all(temp_dir.path().join("targets/local")).unwrap();
    
    // 创建一个简单的本地配置文件
    let config_content = r#"
enabled = true
testing_type = "locally"
"#;
    fs::write(temp_dir.path().join("targets/local/config.toml"), config_content).unwrap();
    
    // 创建一个测试依赖关系的模板
    let template_content = r#"---
title: "依赖关系测试"
target_config: "targets/local/config.toml"
unit_name: "DepsUnit"
tags: ["test", "dependencies"]
---

# 依赖关系测试

## 前置步骤 {id="setup"}

```bash {id="setup-cmd" exec=true description="设置环境" assert.exit_code=0}
echo "设置环境中..." > /tmp/lintestor_test_deps.txt
```

**结果:**
```output {ref="setup-cmd"}
占位符输出
```

## 成功依赖的步骤 {id="success-step" depends_on=["setup"]}

```bash {id="success-cmd" exec=true description="成功的依赖步骤" assert.exit_code=0}
echo "添加成功步骤内容" >> /tmp/lintestor_test_deps.txt
cat /tmp/lintestor_test_deps.txt
```

**结果:**
```output {ref="success-cmd"}
占位符输出
```

## 失败依赖的步骤 {id="fail-step" depends_on=["non-existent-step"]}

```bash {id="fail-cmd" exec=true description="失败的依赖步骤" assert.exit_code=0}
echo "这个步骤不应该执行，因为依赖不存在"
```

**结果:**
```output {ref="fail-cmd"}
占位符输出
```

## 最终步骤 {id="final-step" depends_on=["success-step"]}

```bash {id="final-cmd" exec=true description="最终步骤" assert.exit_code=0 assert.stdout_contains="设置环境中"}
cat /tmp/lintestor_test_deps.txt
rm /tmp/lintestor_test_deps.txt
```

**结果:**
```output {ref="final-cmd"}
占位符输出
```
"#;

    // 写入测试模板到临时文件
    fs::write(&template_path, template_content).unwrap();

    // 执行lintestor命令运行本地测试
    let mut cmd = Command::cargo_bin("lintestor").unwrap();
    let result = cmd
        .arg("--local")  // 使用本地执行
        .arg("--template")  // 指定模板文件
        .arg(template_path.to_str().unwrap())
        .env("RUST_LOG", "debug")  // 启用调试日志
        .assert();

    // 验证命令执行成功（即使有一个步骤依赖不满足）
    result.success();
}