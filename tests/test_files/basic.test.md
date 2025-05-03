---
title: "基本功能测试"
target_config: "targets/local/config.toml"
unit_name: "BasicTest"
tags: ["basic", "example"]
---

# {{ title }}

*测试执行于: {{ execution_date }}*

这是一个最基本的测试模板示例，演示以下功能：
- 基本命令执行
- 断言功能（检查退出码）
- 命令输出引用

## 执行基本命令 {id="basic-command"}

下面是一个简单的命令示例：

```bash {id="echo-cmd" exec=true description="Echo 命令演示" assert.exit_code=0}
echo "Hello, Lintestor!"
echo "当前日期: $(date)"
```

**命令输出:**
```output {ref="echo-cmd"}
命令输出将显示在这里
```

## 检查系统信息 {id="system-info"}

获取一些基本的系统信息：

```bash {id="sys-info" exec=true description="系统信息" assert.exit_code=0}
uname -a
echo "----------------"
echo "内存信息:"
free -h | head -3
```

**命令输出:**
```output {ref="sys-info"}
系统信息将显示在这里
```

## 测试结果摘要 {id="summary" generate_summary=true}

| 步骤描述 | 状态 |
|---------|------|
| Echo 命令演示 | {{ status.echo-cmd }} |
| 系统信息 | {{ status.sys-info }} |