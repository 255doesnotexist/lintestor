# Lintestor (v0.2.0+ 架构)

[Docs](https://255doesnotexist.github.io/lintestor/) | [English README](README_en.md)

Lintestor 是一个基于 Rust 的自动化测试框架，旨在通过**可执行的 Markdown 测试模板**来灵活地定义、执行和报告针对特定**目标 (Target)** 环境上的特定**单元 (Unit)** 的测试。

## 功能 (v0.2.0+)

*   **基于 Markdown 的测试定义:** 使用 `.test.md` 文件作为单一事实来源，包含描述、命令、断言、数据提取和报告结构。
*   **目标环境管理:** 通过配置文件 (`targets/**/config.toml`) 管理不同测试环境（QEMU, 远程 SSH, Boardtest, 本地）的连接和设置。
*   **依赖感知的测试执行:** 自动处理测试步骤之间的依赖关系 (`depends_on`)。
*   **高级断言能力:** 支持多种断言类型，包括退出码、标准输出/错误内容验证，且正确处理多个相同类型断言（如多个 `assert.stderr_not_contains`）。
*   **自动化报告生成:**
    *   为每个执行的测试模板生成详细的、包含执行输出和结果的 Markdown **测试报告 (`.report.md`)**。
    *   聚合所有测试结果到 `reports.json`。
    *   生成 Target x Unit 的 Markdown **摘要矩阵 (`summary.md`)**，包含丰富的状态和输出信息。
*   **灵活的测试筛选:** 通过目标、单元或标签 (`--target`, `--unit`, `--tag`) 精确选择要运行的测试。

## 使用

参见 [USAGE.md](USAGE.md) 获取详细的使用说明和模板编写指南。

See [USAGE_en.md](USAGE_en.md) for English usage.

## 预构建二进制
实验性的 Nightly 构建请见 [Releases](https://github.com/255doesnotexist/lintestor/releases) 。