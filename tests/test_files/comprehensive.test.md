---
title: "综合功能测试"
target_config: "targets/local/config.toml"
unit_name: "ComprehensiveTest"
unit_version: "v1.0.0-test"
tags: ["comprehensive", "all-features", "advanced"]
custom_field: "自定义字段值"
---

# 综合功能测试模板

**测试标题:** {{ metadata.title }}  
**执行时间:** {{ execution_date }}  
**单元名称:** {{ metadata.unit_name }}  
**单元版本:** {{ metadata.unit_version }}  
**目标环境:** {{ metadata.target_name }}  
**自定义字段:** {{ metadata.custom_field }}

> 本测试模板演示了 Lintestor 的所有主要功能，包括但不限于：变量提取、断言验证、依赖关系、特殊属性等。

## 准备测试环境 {id="setup"}

创建测试目录和基础文件：

```bash {id="setup-env" exec=true description="环境准备" assert.exit_code=0}
# 创建测试目录
mkdir -p /tmp/comprehensive_test
cd /tmp/comprehensive_test

# 创建一些基础文件
echo "This is a test file" > test.txt
echo "VERSION=1.2.3" > version.env
echo "CONFIG=production" >> version.env
echo "BUILD_NUMBER=42" >> version.env

# 显示创建的内容
ls -la
cat test.txt
cat version.env
```

**命令输出:**
```output {ref="setup-env"}
命令输出将显示在这里
```

## 提取版本信息 {id="extract-version"}

从版本文件中提取变量：

```bash {id="version-extract" exec=true description="提取版本" assert.exit_code=0 extract.version=/VERSION=([0-9.]+)/ extract.build=/BUILD_NUMBER=(\d+)/ extract.config=/CONFIG=(\w+)/ depends_on=["setup-env"]}
cd /tmp/comprehensive_test
cat version.env
echo "提取完成"
```

**命令输出:**
```output {ref="version-extract"}
版本文件内容将显示在这里
```

## 测试文件操作 {id="file-ops"}

执行一些文件操作并验证结果：

```bash {id="file-operations" exec=true description="文件操作" assert.exit_code=0 assert.stdout_contains="successfully" depends_on=["setup-env"]}
cd /tmp/comprehensive_test
echo "Additional content" >> test.txt
wc -l test.txt
echo "File updated successfully"
```

**命令输出:**
```output {ref="file-operations"}
文件操作结果将显示在这里
```

## 复杂断言测试 {id="assertions"}

测试多种断言类型：

```bash {id="complex-assert" exec=true description="复杂断言" assert.exit_code=0 assert.stdout_contains="pass" assert.stderr_contains="Error" assert.stderr_not_contains="Error" assert.stderr_not_contains="pass"}
echo "This test should pass"
echo "Errors should be present here, expected" >&2
```

**命令输出:**
```output {ref="complex-assert"}
断言测试输出将显示在这里
```

## 使用提取的变量 {id="use-vars"}

使用之前提取的变量：

```bash {id="use-variables" exec=true description="使用变量" assert.exit_code=0 depends_on=["version-extract"]}
echo "软件版本: {{ version-extract::version }}"
echo "构建编号: {{ version-extract::build }}"
echo "配置环境: {{ version-extract::config }}"
echo "当前工作目录: $(pwd)"
```

**命令输出:**
```output {ref="use-variables"}
变量使用结果将显示在这里
```

## 组合测试 {id="combined-test"}

组合多个步骤的结果：

```bash {id="combined" exec=true description="组合测试" assert.exit_code=0 depends_on=["version-extract", "file-operations"]}
cd /tmp/comprehensive_test
echo "综合报告:"
echo "----------------------------------------"
echo "软件版本: {{ version-extract::version }}"
echo "构建编号: {{ version-extract::build }}"
echo "配置模式: {{ version-extract::config }}"
echo "----------------------------------------"
echo "文件内容:"
cat test.txt
echo "----------------------------------------"
```

**命令输出:**
```output {ref="combined"}
组合测试结果将显示在这里
```

## 清理环境 {id="cleanup"}

清理测试环境：

```bash {id="cleanup-env" exec=true description="清理环境" assert.exit_code=0 depends_on=["combined"]}
rm -rf /tmp/comprehensive_test
echo "测试环境已清理"
```

**命令输出:**
```output {ref="cleanup-env"}
清理结果将显示在这里
```

## 测试报告 {id="report"}

### 提取的变量

| 变量名 | 值 |
|-------|-----|
| version | {{ version-extract::version }} |
| build | {{ version-extract::build }} |
| config | {{ version-extract::config }} |

### 特殊变量

| 变量名 | 值 |
|-------|-----|
| execution_date | {{ execution_date }} |
| target_name | {{ metadata.target_name }} |
| unit_version | {{ metadata.unit_version }} |

## 测试结果摘要 {id="summary" generate_summary=true}

| 步骤描述 | 状态 |
|---------|------|
| 环境准备 | {{ setup-env::status.execution }} |
| 提取版本 | {{ version-extract::status.execution }} |
| 文件操作 | {{ file-operations::status.execution }} |
| 复杂断言 | {{ complex-assert::status.execution }} |
| 使用变量 | {{ use-variables::status.execution }} |
| 组合测试 | {{ combined::status.execution }} |
| 清理环境 | {{ cleanup-env::status.execution }} |