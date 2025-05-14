---
title: "SpacemiT K1/M1 (X60) GNU Toolchain (gnu-upstream) 测试"
target_config: "target/k1.toml"
unit_name: "gnu-upstream"
unit_version_command: "riscv64-unknown-linux-gnu-gcc -v 2>&1 | grep 'gcc version' | awk '{print $3}'"
tags: ["toolchain", "gcc", "gnu-upstream", "K1"]
---

# { title }

## 环境信息

* **测试日期:** `2025-05-13`
* **单元版本:** `{ unit_version }`
* **目标信息:** `{ target_info }`

## 0. 初始化 ruyi 包管理器

```bash {id="init-ruyi" exec=true description="安装 RuyiSDK CLI" assert.exit_code=0}
rm -r ~/venv-gnu-upstream
rm -r ~/ruyi
curl -o ruyi-0.32.0.amd64 https://mirror.iscas.ac.cn/ruyisdk/ruyi/tags/0.32.0/ruyi-0.32.0.riscv64
mv ruyi-0.32.0.amd64 ruyi
chmod +x ruyi
```

```output {ref="init-ruyi"}
[Output for step 'init-ruyi' not found]
```

## 1. 安装工具链

安装 Upstream GNU Toolchain。

```bash {id="install-toolchain" exec=true description="安装 Upstream GNU Toolchain" assert.exit_code=0 depends_on=["init-ruyi"]}
~/ruyi install toolchain/gnu-upstream
```

**结果:**

```output {ref="install-toolchain"}
[Output for step 'install-toolchain' not found]
```

## 2. 创建虚拟环境

创建用于测试的虚拟环境。

```bash {id="create-venv" exec=true description="创建虚拟环境" assert.exit_code=0 depends_on=["install-toolchain"]}
~/ruyi venv -t toolchain/gnu-upstream generic venv-gnu-upstream
```

**结果:**

```output {ref="create-venv"}
[Output for step 'create-venv' not found]
```

## 3. 激活环境

激活虚拟环境以进行后续测试。

```bash {id="activate-venv" exec=true description="激活虚拟环境" assert.exit_code=0 depends_on=["create-venv"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
echo "环境已激活"
```

**结果:**

```output {ref="activate-venv"}
[Output for step 'activate-venv' not found]
```

## 4. 编译器版本检查

检查编译器版本以确认安装正确。

```bash {id="check-version" exec=true description="检查编译器版本" assert.exit_code=0 assert.stderr_contains="gcc version" extract.gcc_version="/gcc version ([0-9.]+)/" depends_on=["activate-venv"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
echo $PATH
ls /home/ezra/venv-gnu-upstream/bin
riscv64-unknown-linux-gnu-gcc -v
```

**结果:**

```output {ref="check-version"}
[Output for step 'check-version' not found]
```

编译器版本: { gcc_version }

## 5. Hello World 测试

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
[Output for step 'compile-hello' not found]
```

```bash {id="run-hello" exec=true description="运行 Hello World 程序" assert.exit_code=0 assert.stdout_contains="Hello, world!" depends_on=["compile-hello"]}
./hello_upstream
```

**结果:**

```output {ref="run-hello"}
[Output for step 'run-hello' not found]
```

## 6. CoreMark 基准测试 (默认优化)

使用默认优化选项编译和运行 CoreMark 基准测试。

```bash {id="extract-coremark" exec=true description="提取 CoreMark 包" assert.exit_code=0 depends_on=["activate-venv"]}
mkdir coremark
cd coremark
~/ruyi extract coremark
```

**结果:**

```output {ref="extract-coremark"}
[Output for step 'extract-coremark' not found]
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
[Output for step 'build-coremark' not found]
```

```bash {id="run-coremark" exec=true description="运行 CoreMark (默认优化)" assert.exit_code=0 assert.stdout_contains="CoreMark 1.0" extract.coremark_score=/Iterations\/Sec\s+:\s+([0-9.]+)/ depends_on=["build-coremark"]}
cd coremark
./coremark.exe
```

**结果:**

```output {ref="run-coremark"}
[Output for step 'run-coremark' not found]
```

CoreMark 默认优化分数: { coremark_score }

## 7. CoreMark 基准测试 (向量扩展)

使用向量扩展优化选项编译和运行 CoreMark 基准测试。

```bash {id="build-coremark-vector" exec=true description="编译 CoreMark (向量扩展)" assert.exit_code=0 depends_on=["config-coremark"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
cd coremark
make PORT_DIR=linux64 XCFLAGS="-march=rv64gcv_zvl256b -mabi=lp64d" link
file coremark.exe
```

**结果:**

```output {ref="build-coremark-vector"}
[Output for step 'build-coremark-vector' not found]
```

```bash {id="run-coremark-vector" exec=true description="运行 CoreMark (向量扩展)" assert.exit_code=0 assert.stdout_contains="CoreMark 1.0" extract.coremark_vector_score=/Iterations\/Sec\s+:\s+([0-9.]+)/ depends_on=["build-coremark-vector"]}
cd coremark
./coremark.exe
```

**结果:**

```output {ref="run-coremark-vector"}
[Output for step 'run-coremark-vector' not found]
```

CoreMark 向量扩展优化分数: { coremark_vector_score }

## 8. 测试总结 {id="summary" generate_summary=true}

| 步骤ID | 描述 | 状态 | 退出码 | 输出摘要 | 错误信息 |
|--------|------|------|--------|----------|----------|
| install-toolchain | 安装 Upstream GNU Toolchain | {{ status.install-toolchain }} | {{ exit_code.install-toolchain }} | {{ output_summary.install-toolchain }} | {{ error.install-toolchain }} |
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

## 9. 性能比较

| 优化选项 | CoreMark 分数 |
|---------|-------------|
| 默认优化 (-O2 -lrt) | { coremark_score } |
| 向量扩展 (-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt) | { coremark_vector_score } |
