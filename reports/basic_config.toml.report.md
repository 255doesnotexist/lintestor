---
title: "基本功能测试"
target_config: "targets/local/config.toml"
unit_name: "BasicTest"
unit_version: 0.1.0
tags: ["basic", "example"]
---

# 基本功能测试

*测试执行于: 2025-05-16*

这是一个最基本的测试模板示例，演示以下功能：
- 基本命令执行
- 断言功能（检查退出码）
- 命令输出引用

## 执行基本命令

下面是一个简单的命令示例：

```bash {exec="true" extract.lintestor="/Lintestor/"}
echo "Hello, Lintestor!"
echo "当前日期: $(date)"
```

**命令输出:**

```output {ref="echo-cmd"}
Hello, Lintestor!
当前日期: Fri May 16 10:39:28 AM CST 2025
```

## 检查系统信息

获取一些基本的系统信息：
（代码块的依赖写 depends_on=["step_id","template_id::step_id"]）
（以及会自动隐式依赖属于它这一级的标题，（不对吧？应该是标题依赖于底下所有代码块））

```bash {depends_on=""echo-cmd"]" exec="true"}
uname -a
echo "----------------"
echo "内存信息:"
free -h | head -3
echo "----------------"
echo "Lintestor"
```

**命令输出:**

```output {ref="sys-info"}
Linux debian 6.11.10-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.11.10-1 (2024-11-23) x86_64 GNU/Linux
----------------
内存信息:
 total used free shared buff/cache available
Mem: 15Gi 9.8Gi 4.6Gi 55Mi 1.7Gi 5.8Gi
Swap: 0B 0B 0B
----------------
Lintestor
```

## 测试结果摘要

| 步骤描述 | 状态 |
|---------|------|
| Echo 命令演示 | Pass |
| 系统信息 | Pass |
