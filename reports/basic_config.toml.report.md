---
title: "基本功能测试"
target_config: "targets/local/config.toml"
unit_name: "BasicTest"
tags: ["basic", "example"]
---

# 基本功能测试

*测试执行于: 2025-05-14*

这是一个最基本的测试模板示例，演示以下功能：
- 基本命令执行
- 断言功能（检查退出码）
- 命令输出引用

## 执行基本命令

下面是一个简单的命令示例：

```bash {id="echo-cmd" exec=true description="Echo 命令演示" assert.exit_code=0}
echo "Hello, Lintestor!"
echo "当前日期: $(date)"
```

**命令输出:**

```output {ref="echo-cmd"}
Hello, Lintestor!
当前日期: Wed May 14 09:37:30 PM CST 2025
```

## 检查系统信息

获取一些基本的系统信息：

```bash {id="sys-info" exec=true description="系统信息" assert.exit_code=0}
uname -a
echo "----------------"
echo "内存信息:"
free -h | head -3
```

**命令输出:**

```output {ref="sys-info"}
Linux debian 6.11.10-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.11.10-1 (2024-11-23) x86_64 GNU/Linux
----------------
内存信息:
 total used free shared buff/cache available
Mem: 15Gi 10Gi 2.8Gi 79Mi 2.5Gi 4.9Gi
Swap: 0B 0B 0B
```

## 测试结果摘要 {id="summary" generate_summary=true}

| 步骤描述 | 状态 |
|---------|------|
| Echo 命令演示 | {{ status.echo-cmd }} |
| 系统信息 | {{ status.sys-info }} |
