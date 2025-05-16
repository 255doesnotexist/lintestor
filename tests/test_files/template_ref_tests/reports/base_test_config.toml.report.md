---
title: "基础测试"
target_config: "targets/local/config.toml"
unit_name: "基础模块"
unit_version: 0.1.0
tags: ["base", "foundation"]
---

# 基础测试

这个测试模板定义了其他测试可能需要的基础变量。

## 系统信息收集

```bash
uname -a
```

```output {ref="collect-system"}
Linux debian 6.11.10-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.11.10-1 (2024-11-23) x86_64 GNU/Linux
```

## 时间戳生成

```bash
echo "Current time: $(date '+%Y-%m-%d %H:%M:%S')"
```

```output {ref="get-timestamp"}
Current time: 2025-05-17 00:23:35
```

## 用户信息获取

```bash
echo "Current user: $(whoami)"
```

```output {ref="get-user"}
Current user: ezra
```

## 计算基准数值

```bash
echo "Performing base calculations..."
sleep 1
echo "Base score: 85"
```

```output {ref="calc-base-values"}
Performing base calculations...
Base score: 85
```

系统信息: 内核版本 6.11.10-amd64
当前时间: 2025-05-17 00:23:35
当前用户: ezra
基准分数: 85
