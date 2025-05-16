---
title: "综合功能测试"
target_config: "targets/local/config.toml"
unit_name: "ComprehensiveTest"
unit_version: "v1.0.0-test"
tags: ["comprehensive", "all-features", "advanced"]
custom_field: "自定义字段值"
---

# 综合功能测试模板

**测试标题:** 综合功能测试
**执行时间:** 2025-05-16
**单元名称:** ComprehensiveTest
**单元版本:** v1.0.0-test
**目标环境:** config.toml
**自定义字段:** {{metadata.custom_field}}

> 本测试模板演示了 Lintestor 的所有主要功能，包括但不限于：变量提取、断言验证、依赖关系、特殊属性等。

## 准备测试环境

创建测试目录和基础文件：

```bash
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
total 8
drwxrwxr-x 2 ezra ezra 80 May 16 11:18 .
drwxrwxrwt 164 root root 3820 May 16 11:18 ..
-rw-rw-r-- 1 ezra ezra 20 May 16 11:18 test.txt
-rw-rw-r-- 1 ezra ezra 48 May 16 11:18 version.env
This is a test file
VERSION=1.2.3
CONFIG=production
BUILD_NUMBER=42
```

## 提取版本信息

从版本文件中提取变量：

```bash
cd /tmp/comprehensive_test
cat version.env
echo "提取完成"
```

**命令输出:**

```output {ref="version-extract"}
VERSION=1.2.3
CONFIG=production
BUILD_NUMBER=42
提取完成
```

## 测试文件操作

执行一些文件操作并验证结果：

```bash
cd /tmp/comprehensive_test
echo "Additional content" >> test.txt
wc -l test.txt
echo "File updated successfully"
```

**命令输出:**

```output {ref="file-operations"}
2 test.txt
File updated successfully
```

## 复杂断言测试

测试多种断言类型：

```bash
echo "This test should pass"
echo "Errors should be present here, expected" >&2
```

**命令输出:**

```output {ref="complex-assert"}
This test should pass
```

## 使用提取的变量

使用之前提取的变量：

```bash
echo "软件版本: 1.2.3"
echo "构建编号: 42"
echo "配置环境: production"
echo "当前工作目录: $(pwd)"
```

**命令输出:**

```output {ref="use-variables"}
软件版本: 1.2.3
构建编号: 42
配置环境: production
当前工作目录: /home/ezra/lintestor
```

## 组合测试

组合多个步骤的结果：

```bash
cd /tmp/comprehensive_test
echo "综合报告:"
echo "----------------------------------------"
echo "软件版本: 1.2.3"
echo "构建编号: 42"
echo "配置模式: production"
echo "----------------------------------------"
echo "文件内容:"
cat test.txt
echo "----------------------------------------"
```

**命令输出:**

```output {ref="combined"}
综合报告:
----------------------------------------
软件版本: 1.2.3
构建编号: 42
配置模式: production
----------------------------------------
文件内容:
This is a test file
Additional content
----------------------------------------
```

## 清理环境

清理测试环境：

```bash
rm -rf /tmp/comprehensive_test
echo "测试环境已清理"
```

**命令输出:**

```output {ref="cleanup-env"}
测试环境已清理
```

## 测试报告

### 提取的变量

| 变量名 | 值 |
|-------|-----|
| version | 1.2.3 |
| build | 42 |
| config | production |

### 特殊变量

| 变量名 | 值 |
|-------|-----|
| execution_date | 2025-05-16 |
| target_name | config.toml |
| unit_version | v1.0.0-test |

## 测试结果摘要

| 步骤描述 | 状态 |
|---------|------|
| 环境准备 | Pass |
| 提取版本 | Pass |
| 文件操作 | Pass |
| 复杂断言 | Pass |
| 使用变量 | Pass |
| 组合测试 | Pass |
| 清理环境 | Pass |
