---
title: "基本功能测试"
target_config: "targets/local/config.toml"
unit_name: "BasicTest"
tags: ["basic", "example"]
---


# 基本功能测试

*测试执行于: 2025-05-04 02:58:31*

这是一个最基本的测试模板示例，演示以下功能：
- 基本命令执行
- 断言功能（检查退出码）
- 命令输出引用

## 执行基本命令

下面是一个简单的命令示例：

```bash
echo "Hello, Lintestor!"
echo "当前日期: $(date)"
```

**命令输出:**
```output \nHello, Lintestor!
当前日期: Sun May 4 02:58:31 AM CST 2025
\n```

## 检查系统信息

获取一些基本的系统信息：

```bash
uname -a
echo "----------------"
echo "内存信息:"
free -h | head -3
```

**命令输出:**
```output \nLinux debian 6.11.10-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.11.10-1 (2024-11-23) x86_64 GNU/Linux
----------------
内存信息:
 total used free shared buff/cache available
Mem: 15Gi 12Gi 988Mi 78Mi 2.5Gi 2.8Gi
Swap: 0B 0B 0B
\n```

## 测试结果摘要


| 步骤描述 | 状态 |
|---------|------|
| echo-cmd-output | ✅ Pass |
| 系统信息 | ✅ Pass |
| | ✅ Pass |
| Echo | ✅ Pass |
| | ✅ Pass |
| sys-info-output | ✅ Pass |

| 步骤描述 | 状态 |
|---------|------|
| Echo 命令演示 | ✅ Pass |
| 系统信息 | ✅ Pass |