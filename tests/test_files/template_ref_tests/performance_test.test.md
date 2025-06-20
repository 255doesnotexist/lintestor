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

## 性能基准测试 {id="performance-benchmark"}

```bash {id="run-benchmark" exec=true extract.perf_score=/Performance score:\s+(\d+)/}
echo "Running performance benchmark..."
echo "Using base score: {{ base::calc-base-values::base_score }} as reference"
echo "Test started at: {{ base::get-timestamp::time }}"
sleep 2
# 基于基础分数计算性能分数
CALC=$(({{ base::calc-base-values::base_score }} + 12))
echo "Performance score: $CALC"
```

```output {ref="run-benchmark"}
output
```

## 系统负载检测 {id="system-load"}

```bash {id="check-load" exec=true extract.system_load=/Average load:\s+([\d\.]+)/}
echo "Checking system load..."
LOAD=$(cat /proc/loadavg | awk '{print $1}')
echo "Average load: $LOAD"
echo "Test executed by: {{ base::get-user::username }}"
```

```output {ref="check-load"}
output
```

## 性能评估报告 {id="performance-report"}

基于基础测试的分数 {{ base::calc-base-values::base_score }} 和当前性能测试分数 {{ run-benchmark::perf_score }}，
系统性能表现为: {{ run-benchmark::perf_score > 90 ? "优秀" : "一般" }}

系统信息:
- 内核版本: {{ base::collect-system::kernel }}
- 当前负载: {{ check-load::system_load }}
- 测试时间: {{ base::get-timestamp::time }}
- 执行用户: {{ base::get-user::username }}