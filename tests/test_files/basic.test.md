---
title: "基本功能测试"
target_config: "targets/local/config.toml"
unit_name: "BasicTest"
unit_version: 0.1.0
tags: ["basic", "example"]
---

# {{ metadata.title }}

*测试执行于: {{ execution_date }}*

这是一个最基本的测试模板示例，演示以下功能：
- 基本命令执行
- 断言功能（检查退出码）
- 命令输出引用

## 执行基本命令 {id="basic-command"}

下面是一个简单的命令示例：

```bash {id="echo-cmd" exec=true description="Echo 命令演示" assert.exit_code=0 extract.lintestor=/Lintestor/}
echo "Hello, Lintestor!"
echo "当前日期: $(date)"
```

**命令输出:**
```output {ref="echo-cmd"}
命令输出将显示在这里
```

## 检查系统信息 {id="system-info" depends_on=["basic-command"]}

获取一些基本的系统信息：
（代码块的依赖写 depends_on=["step_id","template_id::step_id"]）
（以及会自动隐式依赖属于它这一级的标题，（不对吧？应该是标题依赖于底下所有代码块））

```bash {id="sys-info" exec=true description="系统信息" assert.exit_code=0 depends_on=["echo-cmd"]}
uname -a
echo "----------------"
echo "内存信息:"
free -h | head -3
echo "----------------"
echo "{{ echo-cmd::lintestor }}"
```

**命令输出:**
```output {ref="sys-info"}
系统信息将显示在这里
```

## 测试结果摘要 {id="summary" generate_summary=true}

| 步骤描述 | 状态 |
|---------|------|
| Echo 命令演示 | {{ echo-cmd::status.execution }} |
| 系统信息 | {{ sys-info::status.execution }} |