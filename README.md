# Lintestor (v0.2.0+)

[Docs](https://255doesnotexist.github.io/lintestor/) | [English README](README_en.md)

Lintestor 是一个基于 Rust 的测试工具，它使用部分可执行的 Markdown 文件来定义和执行自动化测试。

## 静态构建

```
cargo build --release --target x86_64-unknown-linux-musl
```

## 主要功能

-   使用 Markdown 定义测试: 将测试描述、命令、断言和报告结构整合在 `.test.md` 文件中。
-   通过 TOML 配置环境: 管理不同的测试目标（如 QEMU、远程SSH、本地等）。
-   处理依赖关系: 自动按顺序执行测试步骤。
-   断言与数据提取: 支持验证命令退出码和输出内容，并能通过正则表达式提取变量。
-   生成报告: 为每次执行创建 Markdown 格式的测试报告和摘要。
-   测试筛选: 可根据目标、单元或标签选择要运行的测试。

## 使用

关于如何配置和编写测试的说明，请参见：

-   [使用指南 (USAGE.md)](USAGE.md)
-   [Usage Guide (USAGE_en.md)](USAGE_en.md)

## 预构建二进制

实验性的 Nightly 构建请见 [Releases](https://github.com/255doesnotexist/lintestor/releases)。
