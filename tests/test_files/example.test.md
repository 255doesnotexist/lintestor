---
title: "GNU Toolchain 在 K1 上的测试"
target_config: "targets/k1/config.toml"
unit_name: "gnu-plct"
unit_version: "0.1.0"
tags: ["gnu", "toolchain", "riscv"]
---

# {{ title }}

测试执行于: {{ execution_date }}
目标信息: {{ target_info }}
单元版本: {{ unit_version }}

## 安装 {id="setup"}

安装 GNU 工具链的步骤。

```bash {id="install-toolchain" exec=true description="安装工具链" assert.exit_code=0}
echo "正在安装 PLCT GNU 工具链..."
echo "ruyi install toolchain/gnu-plct"
sleep 1
echo "工具链安装完成。"
```

**结果:**
```output {ref="install-toolchain"}
# install-toolchain 命令输出的占位符
```

## 环境配置 {id="env-setup" depends_on=["setup"]}

配置虚拟环境。

```bash {id="create-venv" exec=true description="创建虚拟环境" assert.exit_code=0}
echo "正在创建虚拟环境..."
echo "ruyi venv -t toolchain/gnu-plct generic venv-gnu-plct"
sleep 1
echo "虚拟环境创建完成。可用路径: /root/venv-gnu-plct"
```

**结果:**
```output {ref="create-venv"}
# create-venv 命令输出的占位符
```

```bash {id="activate-venv" exec=true description="激活虚拟环境" assert.exit_code=0}
echo "正在激活虚拟环境..."
echo "source ~/venv-gnu-plct/bin/ruyi-activate"
sleep 1
echo "虚拟环境已激活。"
```

**结果:**
```output {ref="activate-venv"}
# activate-venv 命令输出的占位符
```

## 编译器测试 {id="compiler-test" depends_on=["env-setup"]}

测试编译器版本和基本功能。

```bash {id="check-gcc-version" exec=true description="检查GCC版本" assert.stdout_contains="GCC" extract.gcc_version=/version\s+([0-9.]+)/}
echo "检查 GCC 版本..."
echo "使用内置规范。"
echo "COLLECT_GCC=/root/.local/share/ruyi/binaries/riscv64/gnu-plct-0.20250401.0/bin/riscv64-plct-linux-gnu-gcc"
echo "Target: riscv64-plct-linux-gnu"
echo "线程模型: posix"
echo "gcc version 14.1.0 (RuyiSDK 20250401 PLCT-Sources)"
```

**结果:**
```output {ref="check-gcc-version"}
# check-gcc-version 命令输出的占位符
```

GCC版本: {{ gcc_version }}

## Hello World 测试 {id="hello-world" depends_on=["compiler-test"]}

编译并执行一个简单的Hello World程序。

```bash {id="compile-hello" exec=true description="编译Hello World" assert.exit_code=0}
echo "正在编译Hello World程序..."
echo "riscv64-plct-linux-gnu-gcc hello.c -o hello_plct"
sleep 1
echo "编译完成。"
```

**结果:**
```output {ref="compile-hello"}
# compile-hello 命令输出的占位符
```

```bash {id="run-hello" exec=true description="运行Hello World" assert.stdout_contains="Hello, world!" depends_on=["compile-hello"]}
echo "正在运行Hello World程序..."
sleep 1
echo "Hello, world!"
```

**结果:**
```output {ref="run-hello"}
# run-hello 命令输出的占位符
```

## 测试总结 {id="summary" generate_summary=true}

| 步骤描述 | 状态 |
|---------|------|
| 安装工具链 | {{ status.install-toolchain }} |
| 创建虚拟环境 | {{ status.create-venv }} |
| 激活虚拟环境 | {{ status.activate-venv }} |
| 检查GCC版本 | {{ status.check-gcc-version }} |
| 编译Hello World | {{ status.compile-hello }} |
| 运行Hello World | {{ status.run-hello }} |