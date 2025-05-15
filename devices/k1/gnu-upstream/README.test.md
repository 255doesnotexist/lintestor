---
title: "SpacemiT K1/M1 (X60) GNU Toolchain (gnu-upstream) 测试"
target_config: "target/k1.toml"
unit_name: "gnu-upstream"
unit_version: "0.1.0"
tags: ["toolchain", "gcc", "gnu-upstream", "K1"]
---

# {{ title }}

## 环境信息

* **测试日期:** `{{ execution_date }}`
* **目标配置:** `target/k1.toml`
* **工具链版本:** `{{ check-version::gcc_version }}`
* **单元版本:** `{{ metadata.unit_version }}`

## 0. 初始化 ruyi 包管理器 {id="init"}

```bash {id="init-ruyi" exec=true description="安装 RuyiSDK CLI" assert.exit_code=0}
rm -r ~/venv-gnu-upstream
rm -r ~/ruyi
curl -o ruyi-0.32.0.amd64 https://mirror.iscas.ac.cn/ruyisdk/ruyi/tags/0.32.0/ruyi-0.32.0.riscv64
mv ruyi-0.32.0.amd64 ruyi
chmod +x ruyi
```

```output {ref="init-ruyi"}
# lintestor 将在此处插入 init-ruyi 命令的输出
```

## 1. 安装工具链 {id="install"}

安装 Upstream GNU Toolchain。

```bash {id="install-toolchain" exec=true description="安装 Upstream GNU Toolchain" assert.exit_code=0 depends_on=["init-ruyi"]}
~/ruyi install toolchain/gnu-upstream
```

**结果:**
```output {ref="install-toolchain"}
# lintestor 将在此处插入 install-toolchain 命令的输出
```

## 2. 创建虚拟环境 {id="create-env"}

创建用于测试的虚拟环境。

```bash {id="create-venv" exec=true description="创建虚拟环境" assert.exit_code=0 depends_on=["install-toolchain"]}
~/ruyi venv -t toolchain/gnu-upstream generic venv-gnu-upstream
```

**结果:**
```output {ref="create-venv"}
# lintestor 将在此处插入 create-venv 命令的输出
```

## 3. 激活环境 {id="activate-env"}

激活虚拟环境以进行后续测试。

```bash {id="activate-venv" exec=true description="激活虚拟环境" assert.exit_code=0 depends_on=["create-venv"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
echo "环境已激活"
```

**结果:**
```output {ref="activate-venv"}
# lintestor 将在此处插入 activate-venv 命令的输出
```

## 4. 获取工具链版本 {id="compiler-check"}

检查编译器版本以确认安装正确。

```bash {id="check-version" exec=true description="检查编译器版本" assert.exit_code=0 assert.stderr_contains="gcc version" extract.gcc_version=/gcc version ([0-9.]+)/ depends_on=["activate-venv"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
riscv64-unknown-linux-gnu-gcc -v
```

**结果:**
```output {ref="check-version"}
# lintestor 将在此处插入 check-version 命令的输出
```

编译器版本: {{ check-version::gcc_version }}

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
. ~/venv-gnu-upstream/bin/ruyi-activate
riscv64-unknown-linux-gnu-gcc hello.c -o hello_upstream
file hello_upstream
```

**结果:**
```output {ref="compile-hello"}
# lintestor 将在此处插入 compile-hello 命令的输出
```

```bash {id="run-hello" exec=true description="运行 Hello World 程序" assert.exit_code=0 assert.stdout_contains="Hello, world!" depends_on=["compile-hello"]}
./hello_upstream
```

**结果:**
```output {ref="run-hello"}
# lintestor 将在此处插入 run-hello 命令的输出
```

## 6. CoreMark 基准测试 (默认优化) {id="coremark-default"}

使用默认优化选项编译和运行 CoreMark 基准测试。

```bash {id="extract-coremark" exec=true description="提取 CoreMark 包" assert.exit_code=0 depends_on=["activate-venv"]}
mkdir coremark
cd coremark
~/ruyi extract coremark
```

**结果:**
```output {ref="extract-coremark"}
# lintestor 将在此处插入 extract-coremark 命令的输出
```

```bash {id="config-coremark" exec=true description="配置 CoreMark 编译" assert.exit_code=0 depends_on=["extract-coremark"]}
cd coremark
sed -i 's/\bgcc\b/riscv64-unknown-linux-gnu-gcc/g' linux64/core_portme.mak
```

```bash {id="build-coremark" exec=true description="编译 CoreMark (默认优化)" assert.exit_code=0 depends_on=["config-coremark"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
cd coremark
make PORT_DIR=linux64 link
file coremark.exe
```

**结果:**
```output {ref="build-coremark"}
# lintestor 将在此处插入 build-coremark 命令的输出
```

```bash {id="run-coremark" exec=true description="运行 CoreMark (默认优化)" assert.exit_code=0 assert.stdout_contains="CoreMark 1.0" extract.coremark_score=/Iterations\/Sec\s+:\s+([0-9.]+)/ depends_on=["build-coremark"]}
cd coremark
./coremark.exe
```

**结果:**
```output {ref="run-coremark"}
# lintestor 将在此处插入 run-coremark 命令的输出
```

CoreMark 默认优化分数: {{ run-coremark::coremark_score }}

## 7. CoreMark 基准测试 (向量扩展) {id="coremark-vector"}

使用向量扩展优化选项编译和运行 CoreMark 基准测试。

```bash {id="build-coremark-vector" exec=true description="编译 CoreMark (向量扩展)" assert.exit_code=0 depends_on=["config-coremark"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
cd coremark
make PORT_DIR=linux64 XCFLAGS="-march=rv64gcv_zvl256b -mabi=lp64d" link
file coremark.exe
```

**结果:**
```output {ref="build-coremark-vector"}
# lintestor 将在此处插入 build-coremark-vector 命令的输出
```

```bash {id="run-coremark-vector" exec=true description="运行 CoreMark (向量扩展)" assert.exit_code=0 assert.stdout_contains="CoreMark 1.0" extract.coremark_vector_score=/Iterations\/Sec\s+:\s+([0-9.]+)/ depends_on=["build-coremark-vector"]}
cd coremark
./coremark.exe
```

**结果:**
```output {ref="run-coremark-vector"}
# lintestor 将在此处插入 run-coremark-vector 命令的输出
```

CoreMark 向量扩展优化分数: {{ run-coremark-vector::coremark_vector_score }}

## 8. 测试总结 {id="summary" generate_summary=true}

| 步骤ID | 描述 | 状态 | 退出码 | 输出摘要 | 错误信息 |
|--------|------|------|--------|----------|----------|
| install-toolchain | 安装 Upstream GNU Toolchain | {{ install-toolchain::status.execution }} | {{ install-toolchain::exit_code }} | {{ install-toolchain::output_summary }} | {{ install-toolchain::error }} |
| create-venv | 创建测试虚拟环境 | {{ create-venv::status.execution }} | {{ create-venv::exit_code }} | {{ create-venv::output_summary }} | {{ create-venv::error }} |
| activate-venv | 激活虚拟环境 | {{ activate-venv::status.execution }} | {{ activate-venv::exit_code }} | {{ activate-venv::output_summary }} | {{ activate-venv::error }} |
| check-version | 检查编译器版本 | {{ check-version::status.execution }} | {{ check-version::exit_code }} | {{ check-version::output_summary }} | {{ check-version::error }} |
| create-hello | 创建 Hello World 源文件 | {{ create-hello::status.execution }} | {{ create-hello::exit_code }} | {{ create-hello::output_summary }} | {{ create-hello::error }} |
| compile-hello | 编译 Hello World 程序 | {{ compile-hello::status.execution }} | {{ compile-hello::exit_code }} | {{ compile-hello::output_summary }} | {{ compile-hello::error }} |
| run-hello | 运行 Hello World 程序 | {{ run-hello::status.execution }} | {{ run-hello::exit_code }} | {{ run-hello::output_summary }} | {{ run-hello::error }} |
| extract-coremark | 提取 CoreMark 包 | {{ extract-coremark::status.execution }} | {{ extract-coremark::exit_code }} | {{ extract-coremark::output_summary }} | {{ extract-coremark::error }} |
| config-coremark | 配置 CoreMark 编译 | {{ config-coremark::status.execution }} | {{ config-coremark::exit_code }} | {{ config-coremark::output_summary }} | {{ config-coremark::error }} |
| build-coremark | 编译 CoreMark (默认优化) | {{ build-coremark::status.execution }} | {{ build-coremark::exit_code }} | {{ build-coremark::output_summary }} | {{ build-coremark::error }} |
| run-coremark | 运行 CoreMark (默认优化) | {{ run-coremark::status.execution }} | {{ run-coremark::exit_code }} | {{ run-coremark::output_summary }} | {{ run-coremark::error }} |
| build-coremark-vector | 编译 CoreMark (向量扩展) | {{ build-coremark-vector::status.execution }} | {{ build-coremark-vector::exit_code }} | {{ build-coremark-vector::output_summary }} | {{ build-coremark-vector::error }} |
| run-coremark-vector | 运行 CoreMark (向量扩展) | {{ run-coremark-vector::status.execution }} | {{ run-coremark-vector::exit_code }} | {{ run-coremark-vector::output_summary }} | {{ run-coremark-vector::error }} |

## 9. 性能比较 {id="performance"}

| 优化选项 | CoreMark 分数 |
|---------|-------------|
| 默认优化 (-O2 -lrt) | {{ run-coremark::coremark_score }} |
| 向量扩展 (-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt) | {{ run-coremark-vector::coremark_vector_score }} |