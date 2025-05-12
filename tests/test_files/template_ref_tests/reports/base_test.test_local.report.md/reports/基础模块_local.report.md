---
title: "基础测试"
target_config: "../../../targets/local/config.toml"
unit_name: "基础模块"
tags: ["base", "foundation"]
---


# 基础测试

这个测试模板定义了其他测试可能需要的基础变量。

## 系统信息收集

```bash
uname -a
```

```output
Linux debian 6.11.10-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.11.10-1 (2024-11-23) x86_64 GNU/Linux

```

## 时间戳生成

```bash
echo "Current time: $(date '+%Y-%m-%d %H:%M:%S')"
```

```output
Current time: 2025-05-08 22:10:28

```

## 用户信息获取

```bash
echo "Current user: $(whoami)"
```

```output
Current user: ezra

```

## 计算基准数值

```bash
echo "Performing base calculations..."
sleep 1
echo "Base score: 85"
```

```output

```

系统信息: 内核版本 debian
当前时间: 2025-05-08 22:10:28
当前用户: ezra
基准分数: 85