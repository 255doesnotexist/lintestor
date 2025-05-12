---
title: "SpacemiT K1/M1 (X60) GNU Toolchain (gnu-upstream) 测试"
target_config: "target/k1.toml"
unit_name: "gnu-upstream"
unit_version_command: "riscv64-unknown-linux-gnu-gcc -v 2>&1 | grep 'gcc version' | awk '{print $3}'"
tags: ["toolchain", "gcc", "gnu-upstream", "K1"]
---


# SpacemiT K1/M1 (X60) GNU Toolchain (gnu-upstream) 测试

## 环境信息

* **测试日期:** `2025-05-09`
* **单元版本:** `{ unit_version }`
* **目标信息:** `{ target_info }`

## 0. 初始化 ruyi 包管理器

```bash
rm -r ~/venv-gnu-upstream
rm -r ~/ruyi
curl -o ruyi-0.32.0.amd64 https://mirror.iscas.ac.cn/ruyisdk/ruyi/tags/0.32.0/ruyi-0.32.0.riscv64
mv ruyi-0.32.0.amd64 ruyi
chmod +x ruyi
```

```output

```

## 1. 安装工具链

安装 Upstream GNU Toolchain。

```bash
~/ruyi install toolchain/gnu-upstream
```

**结果:**
```output

```

## 2. 创建虚拟环境

创建用于测试的虚拟环境。

```bash
~/ruyi venv -t toolchain/gnu-upstream generic venv-gnu-upstream
```

**结果:**
```output

```

## 3. 激活环境

激活虚拟环境以进行后续测试。

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
echo "环境已激活"
```

**结果:**
```output
环境已激活

```

## 4. 编译器版本检查

检查编译器版本以确认安装正确。

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
echo $PATH
ls /home/ezra/venv-gnu-upstream/bin
riscv64-unknown-linux-gnu-gcc -v
```

**结果:**
```output
/home/ezra/venv-gnu-upstream/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin
riscv64-unknown-linux-gnu-addr2line
riscv64-unknown-linux-gnu-ar
riscv64-unknown-linux-gnu-as
riscv64-unknown-linux-gnu-c++
riscv64-unknown-linux-gnu-cc
riscv64-unknown-linux-gnu-c++filt
riscv64-unknown-linux-gnu-cpp
riscv64-unknown-linux-gnu-elfedit
riscv64-unknown-linux-gnu-g++
riscv64-unknown-linux-gnu-gcc
riscv64-unknown-linux-gnu-gcc-ar
riscv64-unknown-linux-gnu-gcc-nm
riscv64-unknown-linux-gnu-gcc-ranlib
riscv64-unknown-linux-gnu-gcov
riscv64-unknown-linux-gnu-gcov-dump
riscv64-unknown-linux-gnu-gcov-tool
riscv64-unknown-linux-gnu-gdb
riscv64-unknown-linux-gnu-gdb-add-index
riscv64-unknown-linux-gnu-gfortran
riscv64-unknown-linux-gnu-gp-archive
riscv64-unknown-linux-gnu-gp-collect-app
riscv64-unknown-linux-gnu-gp-display-html
riscv64-unknown-linux-gnu-gp-display-src
riscv64-unknown-linux-gnu-gp-display-text
riscv64-unknown-linux-gnu-gprof
riscv64-unknown-linux-gnu-gprofng
riscv64-unknown-linux-gnu-gstack
riscv64-unknown-linux-gnu-ld
riscv64-unknown-linux-gnu-ld.bfd
riscv64-unknown-linux-gnu-ldd
riscv64-unknown-linux-gnu-lto-dump
riscv64-unknown-linux-gnu-nm
riscv64-unknown-linux-gnu-objcopy
riscv64-unknown-linux-gnu-objdump
riscv64-unknown-linux-gnu-ranlib
riscv64-unknown-linux-gnu-readelf
riscv64-unknown-linux-gnu-size
riscv64-unknown-linux-gnu-strings
riscv64-unknown-linux-gnu-strip
ruyi-activate

```

编译器版本: { gcc_version }

## 5. Hello World 测试

创建并编译一个简单的 Hello World 程序。

```bash
cat > hello.c << 'EOF'
#include <stdio.h>

int main() {
 printf("Hello, world!\n");
 return 0;
}
EOF
```

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
riscv64-unknown-linux-gnu-gcc hello.c -o hello_upstream
file hello_upstream
```

**结果:**
```output
hello_upstream: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=3f957a7dffdbebbda3524d8c601b149ef231839c, for GNU/Linux 4.15.0, with debug_info, not stripped

```

```bash
./hello_upstream
```

**结果:**
```output
Hello, world!

```

## 6. CoreMark 基准测试 (默认优化)

使用默认优化选项编译和运行 CoreMark 基准测试。

```bash
mkdir coremark
cd coremark
~/ruyi extract coremark
```

**结果:**
```output

```

```bash
cd coremark
sed -i 's/\bgcc\b/riscv64-unknown-linux-gnu-gcc/g' linux64/core_portme.mak
```

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
cd coremark
make PORT_DIR=linux64 link
file coremark.exe
```

**结果:**
```output
riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -lrt"\" -DITERATIONS=0 core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
Link performed along with compile
coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=5c8618cf62e0f1f7dd462ba5bddb03479631d0e9, for GNU/Linux 4.15.0, with debug_info, not stripped

```

```bash
cd coremark
./coremark.exe
```

**结果:**
```output
2K performance run parameters for coremark.
CoreMark Size : 666
Total ticks : 20008
Total time (secs): 20.008000
Iterations/Sec : 5497.800880
Iterations : 110000
Compiler version : GCC14.2.0
Compiler flags : -O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt
Memory location : Please put data memory location here
 (e.g. code in flash, data on heap etc)
seedcrc : 0xe9f5
[0]crclist : 0xe714
[0]crcmatrix : 0x1fd7
[0]crcstate : 0x8e3a
[0]crcfinal : 0x33ff
Correct operation validated. See readme.txt for run and reporting rules.
CoreMark 1.0 : 5497.800880 / GCC14.2.0 -O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt / Heap

```

CoreMark 默认优化分数: { coremark_score }

## 7. CoreMark 基准测试 (向量扩展)

使用向量扩展优化选项编译和运行 CoreMark 基准测试。

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
cd coremark
make PORT_DIR=linux64 XCFLAGS="-march=rv64gcv_zvl256b -mabi=lp64d" link
file coremark.exe
```

**结果:**
```output
riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt"\" -DITERATIONS=0 -march=rv64gcv_zvl256b -mabi=lp64d core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
Link performed along with compile
coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=26ed89acf90a9b7d555582be7602c422e6aeccbe, for GNU/Linux 4.15.0, with debug_info, not stripped

```

```bash
cd coremark
./coremark.exe
```

**结果:**
```output
2K performance run parameters for coremark.
CoreMark Size : 666
Total ticks : 20007
Total time (secs): 20.007000
Iterations/Sec : 5498.075674
Iterations : 110000
Compiler version : GCC14.2.0
Compiler flags : -O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt
Memory location : Please put data memory location here
 (e.g. code in flash, data on heap etc)
seedcrc : 0xe9f5
[0]crclist : 0xe714
[0]crcmatrix : 0x1fd7
[0]crcstate : 0x8e3a
[0]crcfinal : 0x33ff
Correct operation validated. See readme.txt for run and reporting rules.
CoreMark 1.0 : 5498.075674 / GCC14.2.0 -O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt / Heap

```

CoreMark 向量扩展优化分数: { coremark_vector_score }

## 8. 测试总结


| 步骤ID | 描述 | 状态 | 退出码 | 输出摘要 | 错误信息 |
|--------|------|------|--------|----------|----------|
| install | 步骤 install | ✅ Pass | 0 | - | - |
| create-env-block-2 | 步骤 create-env-block-2 | ✅ Pass | 0 | - | - |
| run-hello | 步骤 run-hello | ✅ Pass | 0 | Hello, world! | - |
| install-toolchain | 步骤 install-toolchain | ✅ Pass | 0 | - | warn: this ruyi installation h... |
| coremark-default-block-7 | 步骤 coremark-default-block-7 | ✅ Pass | 0 | - | - |
| section-37 | 步骤 section-37 | ✅ Pass | 0 | - | - |
| section-2 | 步骤 section-2 | ✅ Pass | 0 | - | - |
| section-1 | 步骤 section-1 | ✅ Pass | 0 | - | - |
| compiler-check-block-4 | 步骤 compiler-check-block-4 | ✅ Pass | 0 | - | - |
| hello-world | 步骤 hello-world | ✅ Pass | 0 | - | - |
| hello-world-block-6 | 步骤 hello-world-block-6 | ✅ Pass | 0 | - | - |
| install-block-1 | 步骤 install-block-1 | ✅ Pass | 0 | - | - |
| coremark-default | 步骤 coremark-default | ✅ Pass | 0 | - | - |
| build-coremark | 步骤 build-coremark | ✅ Pass | 0 | riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -D... | - |
| activate-env-block-3 | 步骤 activate-env-block-3 | ✅ Pass | 0 | - | - |
| init-ruyi | 步骤 init-ruyi | ✅ Pass | 0 | - | rm: 无法删除 '/home/ezra/v... |
| coremark-vector | 步骤 coremark-vector | ✅ Pass | 0 | - | - |
| create-venv | 步骤 create-venv | ✅ Pass | 0 | - | warn: this ruyi installation h... |
| coremark-default-block-9 | 步骤 coremark-default-block-9 | ✅ Pass | 0 | - | - |
| build-coremark-vector | 步骤 build-coremark-vector | ✅ Pass | 0 | riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -D... | - |
| extract-coremark | 步骤 extract-coremark | ✅ Pass | 0 | - | mkdir: 无法创建目录 "cor... |
| config-coremark | 步骤 config-coremark | ✅ Pass | 0 | - | - |
| coremark-vector-block-10 | 步骤 coremark-vector-block-10 | ✅ Pass | 0 | - | - |
| activate-venv | 步骤 activate-venv | ✅ Pass | 0 | 环境已激活 | - |
| performance | 步骤 performance | ✅ Pass | 0 | - | - |
| init | 步骤 init | ✅ Pass | 0 | - | - |
| activate-env | 步骤 activate-env | ✅ Pass | 0 | - | - |
| check-version | 步骤 check-version | ✅ Pass | 0 | /home/ezra/venv-gnu-upstream/bin:/usr/local/sbin:/... | Using built-in specs. |
| run-coremark-vector | 步骤 run-coremark-vector | ✅ Pass | 0 | 2K performance run parameters for coremark. | bash: 第 1 行：export: "-ma... |
| init-block-0 | 步骤 init-block-0 | ✅ Pass | 0 | - | - |
| coremark-default-block-8 | 步骤 coremark-default-block-8 | ✅ Pass | 0 | - | - |
| compile-hello | 步骤 compile-hello | ✅ Pass | 0 | hello_upstream: ELF 64-bit LSB executable, UCB RIS... | - |
| create-hello | 步骤 create-hello | ✅ Pass | 0 | - | - |
| compiler-check | 步骤 compiler-check | ✅ Pass | 0 | - | - |
| run-coremark | 步骤 run-coremark | ✅ Pass | 0 | 2K performance run parameters for coremark. | bash: 第 1 行：export: "-ma... |
| hello-world-block-5 | 步骤 hello-world-block-5 | ✅ Pass | 0 | - | - |
| coremark-vector-block-11 | 步骤 coremark-vector-block-11 | ✅ Pass | 0 | - | - |
| create-env | 步骤 create-env | ✅ Pass | 0 | - | - |


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