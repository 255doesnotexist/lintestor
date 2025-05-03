---
title: "依赖关系测试"
target_config: "targets/local/config.toml"
unit_name: "DependencyTest"
tags: ["dependencies", "example"]
---

# {{ title }}

*测试执行于: {{ execution_date }}*

这个测试模板演示步骤之间的依赖关系，确保某些步骤在其他步骤完成后才执行。

## 环境准备 {id="setup"}

首先创建一个临时工作目录并进入：

```bash {id="setup-env" exec=true description="准备环境" assert.exit_code=0}
mkdir -p /tmp/lintestor_test
cd /tmp/lintestor_test
echo "准备工作完成，创建时间: $(date)" > setup.log
echo "环境准备完成"
```

**命令输出:**
```output {ref="setup-env"}
环境准备输出将显示在这里
```

## 创建测试文件 {id="create-file" depends_on=["setup"]}

此步骤依赖于环境准备完成：

```bash {id="create-test-file" exec=true description="创建测试文件" assert.exit_code=0}
echo "这是测试内容" > test_file.txt
echo "文件已创建: $(date)" >> setup.log
cat test_file.txt
```

**命令输出:**
```output {ref="create-test-file"}
创建文件输出将显示在这里
```

## 修改测试文件 {id="modify-file" depends_on=["create-file"]}

此步骤依赖于文件创建步骤：

```bash {id="modify-test-file" exec=true description="修改测试文件" assert.exit_code=0}
echo "这是附加内容" >> test_file.txt
echo "文件已修改: $(date)" >> setup.log
cat test_file.txt
```

**命令输出:**
```output {ref="modify-test-file"}
修改文件输出将显示在这里
```

## 检查日志文件 {id="check-log" depends_on=["setup", "create-file", "modify-file"]}

此步骤依赖于所有前面的步骤完成：

```bash {id="check-log-file" exec=true description="检查日志文件" assert.exit_code=0}
cat setup.log
echo "测试流程已完成"
```

**命令输出:**
```output {ref="check-log-file"}
日志文件内容将显示在这里
```

## 清理环境 {id="cleanup"}

清理测试环境：

```bash {id="cleanup-env" exec=true description="清理环境" assert.exit_code=0}
cd /
rm -rf /tmp/lintestor_test
echo "清理完成"
```

**命令输出:**
```output {ref="cleanup-env"}
清理环境输出将显示在这里
```

## 测试结果摘要 {id="summary" generate_summary=true}

| 步骤描述 | 状态 |
|---------|------|
| 准备环境 | {{ status.setup-env }} |
| 创建测试文件 | {{ status.create-test-file }} |
| 修改测试文件 | {{ status.modify-test-file }} |
| 检查日志文件 | {{ status.check-log-file }} |
| 清理环境 | {{ status.cleanup-env }} |