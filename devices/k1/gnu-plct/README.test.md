---
title: "SpacemiT K1/M1 (X60) GNU Toolchain (gnu-plct) 测试"
target_config: "target/k1.toml"
unit_name: "gnu-plct"
unit_version_command: "riscv64-plct-linux-gnu-gcc -v 2>&1 | grep 'gcc version' | awk '{print $3}'"
tags: ["toolchain", "gcc", "gnu-plct", "K1"]
---

# {{ title }}

## 环境信息

* **测试日期:** `{{ execution_date }}`
* **单元版本:** `{{ unit_version }}`
* **目标信息:** `{{ target_info }}`

## 0. 初始化 ruyi 包管理器 {id="init"}

```bash {id="init-ruyi" exec=true description="安装 RuyiSDK CLI" assert.exit_code=0}
rm -r ~/venv-gnu-upstream
rm -r ~/ruyi
curl -o ruyi-0.32.0.amd64 https://mirror.iscas.ac.cn/ruyisdk/ruyi/tags/0.32.0/ruyi-0.32.0.amd64
mv ruyi-0.32.0.amd64 ruyi
chmod +x ruyi
```

```output {ref="init-ruyi"}
output
```

## 1. 安装工具链 {id="install" depends_on=["init-ruyi"]}

安装 PLCT GNU Toolchain。

```bash {id="install-toolchain" exec=true description="安装 PLCT GNU Toolchain" assert.exit_code=0}
ruyi install toolchain/gnu-plct
```

**结果:**
```output {ref="install-toolchain"}
# 将显示安装日志
```

## 2. 创建虚拟环境 {id="create-env"}

创建用于测试的虚拟环境。

```bash {id="create-venv" exec=true description="创建虚拟环境" assert.exit_code=0 depends_on=["install-toolchain"]}
ruyi venv -t toolchain/gnu-plct generic venv-gnu-plct
```

**结果:**
```output {ref="create-venv"}
# 将显示虚拟环境创建日志
```

## 3. 激活环境 {id="activate-env"}

激活虚拟环境以进行后续测试。

```bash {id="activate-venv" exec=true description="激活虚拟环境" assert.exit_code=0 depends_on=["create-venv"]}
. ~/venv-gnu-plct/bin/ruyi-activate
echo "环境已激活"
```

**结果:**
```output {ref="activate-venv"}
# 将显示环境激活结果
```

## 4. 编译器版本检查 {id="compiler-check"}

检查编译器版本以确认安装正确。

```bash {id="check-version" exec=true description="检查编译器版本" assert.exit_code=0 assert.stdout_contains="gcc version" extract.gcc_version=/gcc version ([0-9.]+)/ depends_on=["activate-venv"]}
riscv64-plct-linux-gnu-gcc -v
```

**结果:**
```output {ref="check-version"}
# 将显示编译器版本信息
```

编译器版本: {{ gcc_version }}

## 5. Hello World 测试 {id="hello-world"}

创建并编译一个简单的 Hello World 程序。

```bash {id="create-hello" exec=true description="创建 Hello World 源文件" assert.exit_code=0 depends_on=["activate-venv"]}
cat > hello.c << 'EOF'
#include <stdio.h>

int main() {
    printf("Hello, world!\n");
    return 0;
}
EOF
```

```bash {id="compile-hello" exec=true description="编译 Hello World 程序" assert.exit_code=0 depends_on=["create-hello"]}
riscv64-plct-linux-gnu-gcc hello.c -o hello_plct
file hello_plct
```

**结果:**
```output {ref="compile-hello"}
# 将显示编译结果和文件类型信息
```

```bash {id="run-hello" exec=true description="运行 Hello World 程序" assert.exit_code=0 assert.stdout_contains="Hello, world!" depends_on=["compile-hello"]}
./hello_plct
```

**结果:**
```output {ref="run-hello"}
# 将显示程序运行输出
```

## 6. CoreMark 基准测试 (默认优化) {id="coremark-default"}

使用默认优化选项编译和运行 CoreMark 基准测试。

```bash {id="extract-coremark" exec=true description="提取 CoreMark 包" assert.exit_code=0 depends_on=["activate-venv"]}
ruyi extract coremark
cd coremark
```

**结果:**
```output {ref="extract-coremark"}
# 将显示 CoreMark 提取过程
```

```bash {id="config-coremark" exec=true description="配置 CoreMark 编译" assert.exit_code=0 depends_on=["extract-coremark"]}
sed -i 's/\bgcc\b/riscv64-plct-linux-gnu-gcc/g' linux64/core_portme.mak
```

```bash {id="build-coremark" exec=true description="编译 CoreMark (默认优化)" assert.exit_code=0 depends_on=["config-coremark"]}
make PORT_DIR=linux64 link
file coremark.exe
```

**结果:**
```output {ref="build-coremark"}
# 将显示编译过程和文件类型信息
```

```bash {id="run-coremark" exec=true description="运行 CoreMark (默认优化)" assert.exit_code=0 assert.stdout_contains="CoreMark 1.0" extract.coremark_score=/Iterations\/Sec\s+:\s+([0-9.]+)/ depends_on=["build-coremark"]}
./coremark.exe
```

**结果:**
```output {ref="run-coremark"}
# 将显示 CoreMark 运行结果
```

CoreMark 默认优化分数: {{ coremark_score }}

## 7. CoreMark 基准测试 (向量扩展) {id="coremark-vector"}

使用向量扩展优化选项编译和运行 CoreMark 基准测试。

```bash {id="build-coremark-vector" exec=true description="编译 CoreMark (向量扩展)" assert.exit_code=0 depends_on=["config-coremark"]}
make PORT_DIR=linux64 XCFLAGS="-march=rv64gcv_zvl256b -mabi=lp64d" link
file coremark.exe
```

**结果:**
```output {ref="build-coremark-vector"}
# 将显示编译过程和文件类型信息
```

```bash {id="run-coremark-vector" exec=true description="运行 CoreMark (向量扩展)" assert.exit_code=0 assert.stdout_contains="CoreMark 1.0" extract.coremark_vector_score=/Iterations\/Sec\s+:\s+([0-9.]+)/ depends_on=["build-coremark-vector"]}
./coremark.exe
```

**结果:**
```output {ref="run-coremark-vector"}
# 将显示 CoreMark 运行结果
```

CoreMark 向量扩展优化分数: {{ coremark_vector_score }}

## 8. 测试总结 {id="summary" generate_summary=true}

| 步骤ID | 描述 | 状态 | 退出码 | 输出摘要 | 错误信息 |
|--------|------|------|--------|----------|----------|
| install-toolchain | 安装 PLCT GNU Toolchain | {{ status.install-toolchain }} | {{ exit_code.install-toolchain }} | {{ output_summary.install-toolchain }} | {{ error.install-toolchain }} |
| create-venv | 创建测试虚拟环境 | {{ status.create-venv }} | {{ exit_code.create-venv }} | {{ output_summary.create-venv }} | {{ error.create-venv }} |
| activate-venv | 激活虚拟环境 | {{ status.activate-venv }} | {{ exit_code.activate-venv }} | {{ output_summary.activate-venv }} | {{ error.activate-venv }} |
| check-version | 检查编译器版本 | {{ status.check-version }} | {{ exit_code.check-version }} | {{ output_summary.check-version }} | {{ error.check-version }} |
| create-hello | 创建 Hello World 源文件 | {{ status.create-hello }} | {{ exit_code.create-hello }} | {{ output_summary.create-hello }} | {{ error.create-hello }} |
| compile-hello | 编译 Hello World 程序 | {{ status.compile-hello }} | {{ exit_code.compile-hello }} | {{ output_summary.compile-hello }} | {{ error.compile-hello }} |
| run-hello | 运行 Hello World 程序 | {{ status.run-hello }} | {{ exit_code.run-hello }} | {{ output_summary.run-hello }} | {{ error.run-hello }} |
| extract-coremark | 提取 CoreMark 包 | {{ status.extract-coremark }} | {{ exit_code.extract-coremark }} | {{ output_summary.extract-coremark }} | {{ error.extract-coremark }} |
| config-coremark | 配置 CoreMark 编译 | {{ status.config-coremark }} | {{ exit_code.config-coremark }} | {{ output_summary.config-coremark }} | {{ error.config-coremark }} |
| build-coremark | 编译 CoreMark (默认优化) | {{ status.build-coremark }} | {{ exit_code.build-coremark }} | {{ output_summary.build-coremark }} | {{ error.build-coremark }} |
| run-coremark | 运行 CoreMark (默认优化) | {{ status.run-coremark }} | {{ exit_code.run-coremark }} | {{ output_summary.run-coremark }} | {{ error.run-coremark }} |
| build-coremark-vector | 编译 CoreMark (向量扩展) | {{ status.build-coremark-vector }} | {{ exit_code.build-coremark-vector }} | {{ output_summary.build-coremark-vector }} | {{ error.build-coremark-vector }} |
| run-coremark-vector | 运行 CoreMark (向量扩展) | {{ status.run-coremark-vector }} | {{ exit_code.run-coremark-vector }} | {{ output_summary.run-coremark-vector }} | {{ error.run-coremark-vector }} |

## 9. 性能比较 {id="performance"}

| 优化选项 | CoreMark 分数 |
|---------|-------------|
| 默认优化 (-O2 -lrt) | {{ coremark_score }} |
| 向量扩展 (-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt) | {{ coremark_vector_score }} |