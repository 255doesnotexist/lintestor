---
title: "性能测试"
target_config: "targets/local/config.toml"
unit_name: "性能模块"
unit_version: 0.1.0
tags: ["performance"]
references:
 - template: "base_test.test.md"
 as: "base"
---

# 性能测试

这个测试模板引用了基础测试中的变量来进行性能评估。

## 性能基准测试

```bash
echo "Running performance benchmark..."
echo "Using base score: 85 as reference"
echo "Test started at: 2025-05-17 00:23:35"
sleep 2
# 基于基础分数计算性能分数
CALC=$((85 + 12))
echo "Performance score: $CALC"
```

```output {ref="run-benchmark"}
Running performance benchmark...
Using base score: 85 as reference
Test started at: 2025-05-17 00:23:35
Performance score: 97
```

## 系统负载检测

```bash
echo "Checking system load..."
LOAD=$(cat /proc/loadavg | awk '{print $1}')
echo "Average load: $LOAD"
echo "Test executed by: ezra"
```

```output {ref="check-load"}
Checking system load...
Average load: 0.15
Test executed by: ezra
```

## 性能评估报告

基于基础测试的分数 85 和当前性能测试分数 97，
系统性能表现为: 优秀

系统信息:
- 内核版本: 6.11.10-amd64
- 当前负载: 0.15
- 测试时间: 2025-05-17 00:23:35
- 执行用户: ezra
