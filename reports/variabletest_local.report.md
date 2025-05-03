---
title: "变量提取和替换测试"
target_config: "targets/local/config.toml"
unit_name: "VariableTest"
tags: ["variables", "extraction", "example"]
---


# 变量提取和替换测试

*测试执行于: 2025-05-04 02:56:52*

这个测试模板演示如何从命令输出中提取变量，并在后续步骤中使用这些变量。

## 生成数据

生成一些包含可提取数据的输出：

```bash
echo "内存总量: 8192MB"
echo "内核版本: $(uname -r)"
echo "主机名: $(hostname)"
echo "当前用户: $(whoami)"
```

**命令输出:**
```output \n内存总量: 8192MB
内核版本: 6.11.10-amd64
主机名: debian
当前用户: ezra
\n```

## 使用提取的变量

使用前一步提取的变量：

```bash
echo "使用提取的变量:"
echo "系统内存: 8192MB"
echo "Linux内核版本: {{ kernel_version }}"
```

**命令输出:**
```output \n使用提取的变量:
系统内存: 8192MB
Linux内核版本: {{ kernel_version }}
\n```

## 其他变量测试

使用更复杂的正则表达式提取变量：

```bash
echo "系统信息:"
echo "可用内存: 4096KB"
echo "系统负载: 0.15"
echo "磁盘使用率: 65%"
```

**命令输出:**
```output \n系统信息:
可用内存: 4096KB
系统负载: 0.15
磁盘使用率: 65%
\n```

## 综合使用变量

综合使用前面提取的所有变量：

```bash
echo "系统摘要报告:"
echo "----------------------------------------"
echo "内核版本: {{ kernel_version }}"
echo "总内存: 8192MB"
echo "可用内存: 4096KB"
echo "当前负载: 0.15"
echo "----------------------------------------"
```

**命令输出:**
```output \n系统摘要报告:
----------------------------------------
内核版本: {{ kernel_version }}
总内存: {{ memory_total }}MB
可用内存: {{ free_memory }}KB
当前负载: {{ load_avg }}
----------------------------------------
\n```

## 变量替换摘要

**提取的变量值:**

- 内核版本: {{ kernel_version }}
- 总内存: 8192MB
- 可用内存: 4096KB
- 系统负载: 0.15

## 测试结果摘要


| 步骤描述 | 状态 |
|---------|------|
| use-vars-output | ✅ Pass |
| 综合使用变量 | ✅ Pass |
| 生成测试数据 | ✅ Pass |
| | ✅ Pass |
| all-vars-output | ✅ Pass |
| generate-output | ✅ Pass |
| | ✅ Pass |
| complex-extract-output | ✅ Pass |
| | ✅ Pass |
| 使用提取的变量 | ✅ Pass |
| | ✅ Pass |
| 复杂变量提取 | ✅ Pass |

| 步骤描述 | 状态 |
|---------|------|
| 生成测试数据 | ✅ Pass |
| 使用提取的变量 | ✅ Pass |
| 复杂变量提取 | ✅ Pass |
| 综合使用变量 | ✅ Pass |