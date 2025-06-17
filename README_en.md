# Lintestor (v0.2.0+)

[Docs](https://255doesnotexist.github.io/lintestor/) | [中文 README](README.md)

Lintestor is a Rust-based testing tool that uses partially executable Markdown files to define and execute automated tests.

## Features

-   Define tests using Markdown: Integrate test descriptions, commands, assertions, and report structure into `.test.md` files.
-   Configure environments via TOML: Manage different test targets (e.g., QEMU, remote SSH, local).
-   Handle dependencies: Automatically execute test steps in order.
-   Assertions and Data Extraction: Supports validating command exit codes and output, and can extract variables using regular expressions.
-   Generate reports: Create Markdown-format test reports and summaries for each execution.
-   Test filtering: Select tests to run based on target, unit, or tags.

## Usage

For instructions on how to configure and write tests, please see:

-   [使用指南 (USAGE.md)](USAGE.md)
-   [Usage Guide (USAGE_en.md)](USAGE_en.md)

## Prebuilt binaries

For experimental Nightly builds, see [Releases](https://github.com/255doesnotexist/lintestor/releases).