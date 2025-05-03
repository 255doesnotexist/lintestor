---
title: "变量提取和替换测试"
target_config: "targets/local/config.toml"
unit_name: "VariableTest"
tags: ["variables", "extraction", "example"]
---

# {{ title }}

*测试执行于: {{ execution_date }}*

这个测试模板演示如何从命令输出中提取变量，并在后续步骤中使用这些变量。

## 生成数据 {id="generate-data"}

生成一些包含可提取数据的输出：

```bash {id="generate" exec=true description="生成测试数据" assert.exit_code=0 extract.memory_total=/内存总量:\s+(\d+)MB/ extract.kernel_version=/内核版本:\s+(.+)/}
echo "内存总量: 8192MB"
echo "内核版本: $(uname -r)"
echo "主机名: $(hostname)"
echo "当前用户: $(whoami)"
```

**命令输出:**
```output {ref="generate"}
生成的数据将显示在这里
```

## 使用提取的变量 {id="use-variables" depends_on=["generate-data"]}

使用前一步提取的变量：

```bash {id="use-vars" exec=true description="使用提取的变量" assert.exit_code=0}
echo "使用提取的变量:"
echo "系统内存: {{ memory_total }}MB"
echo "Linux内核版本: {{ kernel_version }}"
```

**命令输出:**
```output {ref="use-vars"}
使用变量的输出将显示在这里
```

## 其他变量测试 {id="more-variables"}

使用更复杂的正则表达式提取变量：

```bash {id="complex-extract" exec=true description="复杂变量提取" assert.exit_code=0 extract.free_memory=/可用内存:\s+(\d+)KB/ extract.load_avg=/系统负载:\s+([0-9.]+)/}
echo "系统信息:"
echo "可用内存: 4096KB"
echo "系统负载: 0.15"
echo "磁盘使用率: 65%"
```

**命令输出:**
```output {ref="complex-extract"}
复杂提取的输出将显示在这里
```

## 综合使用变量 {id="combined-vars" depends_on=["generate-data", "more-variables"]}

综合使用前面提取的所有变量：

```bash {id="all-vars" exec=true description="综合使用变量" assert.exit_code=0}
echo "系统摘要报告:"
echo "----------------------------------------"
echo "内核版本: {{ kernel_version }}"
echo "总内存: {{ memory_total }}MB"
echo "可用内存: {{ free_memory }}KB"
echo "当前负载: {{ load_avg }}"
echo "----------------------------------------"
```

**命令输出:**
```output {ref="all-vars"}
综合使用变量的输出将显示在这里
```

## 变量替换摘要

**提取的变量值:**

- 内核版本: {{ kernel_version }}
- 总内存: {{ memory_total }}MB
- 可用内存: {{ free_memory }}KB
- 系统负载: {{ load_avg }}

## 测试结果摘要 {id="summary" generate_summary=false}

| 步骤描述 | 状态 |
|---------|------|
| 生成测试数据 | {{ status.generate }} |
| 使用提取的变量 | {{ status.use-vars }} |
| 复杂变量提取 | {{ status.complex-extract }} |
| 综合使用变量 | {{ status.all-vars }} |