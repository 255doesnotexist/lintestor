---
title: "SpacemiT K1/M1 (X60) GNU Toolchain (gnu-upstream) 测试"
target_config: "target/k1.toml"
unit_name: "gnu-upstream"
unit_version: "0.1.0"
tags: ["toolchain", "gcc", "gnu-upstream", "K1"]
---

# SpacemiT K1/M1 (X60) GNU Toolchain (gnu-upstream) 测试

## 环境信息

* **测试日期:** `2025-05-16`
* **目标配置:** `target/k1.toml`
* **工具链版本:** `14.2.0`
* **单元版本:** `0.1.0`

## 0. 初始化 ruyi 包管理器

```bash
rm -r ~/venv-gnu-upstream
rm -r ~/ruyi
curl -o ruyi-0.32.0.amd64 https://mirror.iscas.ac.cn/ruyisdk/ruyi/tags/0.32.0/ruyi-0.32.0.riscv64
mv ruyi-0.32.0.amd64 ruyi
chmod +x ruyi
```

```output {ref="init-ruyi"}
```

## 1. 安装工具链

安装 Upstream GNU Toolchain。

```bash
~/ruyi install toolchain/gnu-upstream
```

**结果:**

```output {ref="install-toolchain"}
```

## 2. 创建虚拟环境

创建用于测试的虚拟环境。

```bash
~/ruyi venv -t toolchain/gnu-upstream generic venv-gnu-upstream
```

**结果:**

```output {ref="create-venv"}
```

## 3. 激活环境

激活虚拟环境以进行后续测试。

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
echo "环境已激活"
```

**结果:**

```output {ref="activate-venv"}
环境已激活
```

## 4. 获取工具链版本

检查编译器版本以确认安装正确。

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
riscv64-unknown-linux-gnu-gcc -v 2>&1
```

**结果:**

```output {ref="check-version"}
Using built-in specs.
COLLECT_GCC=/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0/bin/riscv64-unknown-linux-gnu-gcc
COLLECT_LTO_WRAPPER=/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0/bin/../libexec/gcc/riscv64-unknown-linux-gnu/14.2.0/lto-wrapper
Target: riscv64-unknown-linux-gnu
Configured with: /work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/src/gcc/configure --build=x86_64-build_pc-linux-gnu --host=riscv64-host_unknown-linux-gnu --target=riscv64-unknown-linux-gnu --prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu --exec_prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu --with-sysroot=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/riscv64-unknown-linux-gnu/sysroot --enable-languages=c,c++,fortran,objc,obj-c++ --with-arch=rv64gc --with-abi=lp64d --with-pkgversion='RuyiSDK 20250401 Upstream-Sources' --with-bugurl=https://github.com/ruyisdk/ruyisdk/issues --enable-__cxa_atexit --disable-libmudflap --disable-libgomp --disable-libquadmath --disable-libquadmath-support --disable-libmpx --with-gmp=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --with-mpfr=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --with-mpc=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --with-isl=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --enable-lto --enable-threads=posix --enable-target-optspace --enable-linker-build-id --with-linker-hash-style=gnu --enable-plugin --disable-nls --disable-multilib --with-local-prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/riscv64-unknown-linux-gnu/sysroot --enable-long-long
Thread model: posix
Supported LTO compression algorithms: zlib zstd
gcc version 14.2.0 (RuyiSDK 20250401 Upstream-Sources)
```

编译器版本: 14.2.0

## 5. Hello World 测试

创建并编译一个简单的 Hello World 程序。

```bash
cat > hello.c << 'EOF'
#include <stdio.h>

int main()
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

```output {ref="compile-hello"}
hello_upstream: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=3f957a7dffdbebbda3524d8c601b149ef231839c, for GNU/Linux 4.15.0, with debug_info, not stripped
```

```bash
./hello_upstream
```

**结果:**

```output {ref="run-hello"}
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

```output {ref="extract-coremark"}
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

```output {ref="build-coremark"}
riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -lrt"\" -DITERATIONS=0 core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
Link performed along with compile
coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=5c8618cf62e0f1f7dd462ba5bddb03479631d0e9, for GNU/Linux 4.15.0, with debug_info, not stripped
```

```bash
cd coremark
./coremark.exe
```

**结果:**

```output {ref="run-coremark"}
2K performance run parameters for coremark.
CoreMark Size : 666
Total ticks : 20005
Total time (secs): 20.005000
Iterations/Sec : 5498.625344
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
CoreMark 1.0 : 5498.625344 / GCC14.2.0 -O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt / Heap
```

CoreMark 默认优化分数: 5498.625344

## 7. CoreMark 基准测试 (向量扩展)

使用向量扩展优化选项编译和运行 CoreMark 基准测试。

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
cd coremark
make PORT_DIR=linux64 XCFLAGS="-march=rv64gcv_zvl256b -mabi=lp64d" link
file coremark.exe
```

**结果:**

```output {ref="build-coremark-vector"}
riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt"\" -DITERATIONS=0 -march=rv64gcv_zvl256b -mabi=lp64d core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
Link performed along with compile
coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=26ed89acf90a9b7d555582be7602c422e6aeccbe, for GNU/Linux 4.15.0, with debug_info, not stripped
```

```bash
cd coremark
./coremark.exe
```

**结果:**

```output {ref="run-coremark-vector"}
2K performance run parameters for coremark.
CoreMark Size : 666
Total ticks : 20009
Total time (secs): 20.009000
Iterations/Sec : 5497.526113
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
CoreMark 1.0 : 5497.526113 / GCC14.2.0 -O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt / Heap
```

CoreMark 向量扩展优化分数: 5497.526113

## 8. 测试总结

| 步骤ID | 描述 | 状态 | 退出码 | 输出摘要 | 错误信息 |
|--------|------|------|--------|----------|----------|
| install-toolchain | 安装 Upstream GNU Toolchain | Pass | 0 | | warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday
info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800
info: in order to hide this banner:
info: - opt out with ruyi telemetry optout
info: - or give consent with ruyi telemetry consent
info: skipping already installed package gnu-upstream-0.20250401.0
 |
| create-venv | 创建测试虚拟环境 | Pass | 0 | | warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday
info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800
info: in order to hide this banner:
info: - opt out with ruyi telemetry optout
info: - or give consent with ruyi telemetry consent
info: Creating a Ruyi virtual environment at venv-gnu-upstream...
info: The virtual environment is now created.

You may activate it by sourcing the appropriate activation script in the
bin directory, and deactivate by invoking `ruyi-deactivate`.

A fresh sysroot/prefix is also provisioned in the virtual environment.
It is available at the following path:

 /home/ezra/venv-gnu-upstream/sysroot

The virtual environment also comes with ready-made CMake toolchain file
and Meson cross file. Check the virtual environment root for those;
comments in the files contain usage instructions.

 |
| activate-venv | 激活虚拟环境 | Pass | 0 | 环境已激活
 | |
| check-version | 检查编译器版本 | Pass | 0 | Using built-in specs.
COLLECT_GCC=/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0/bin/riscv64-unknown-linux-gnu-gcc
COLLECT_LTO_WRAPPER=/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0/bin/../libexec/gcc/riscv64-unknown-linux-gnu/14.2.0/lto-wrapper
Target: riscv64-unknown-linux-gnu
Configured with: /work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/src/gcc/configure --build=x86_64-build_pc-linux-gnu --host=riscv64-host_unknown-linux-gnu --target=riscv64-unknown-linux-gnu --prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu --exec_prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu --with-sysroot=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/riscv64-unknown-linux-gnu/sysroot --enable-languages=c,c++,fortran,objc,obj-c++ --with-arch=rv64gc --with-abi=lp64d --with-pkgversion='RuyiSDK 20250401 Upstream-Sources' --with-bugurl=https://github.com/ruyisdk/ruyisdk/issues --enable-__cxa_atexit --disable-libmudflap --disable-libgomp --disable-libquadmath --disable-libquadmath-support --disable-libmpx --with-gmp=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --with-mpfr=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --with-mpc=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --with-isl=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --enable-lto --enable-threads=posix --enable-target-optspace --enable-linker-build-id --with-linker-hash-style=gnu --enable-plugin --disable-nls --disable-multilib --with-local-prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/riscv64-unknown-linux-gnu/sysroot --enable-long-long
Thread model: posix
Supported LTO compression algorithms: zlib zstd
gcc version 14.2.0 (RuyiSDK 20250401 Upstream-Sources)
 | |
| create-hello | 创建 Hello World 源文件 | Pass | 0 | | |
| compile-hello | 编译 Hello World 程序 | Pass | 0 | hello_upstream: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=3f957a7dffdbebbda3524d8c601b149ef231839c, for GNU/Linux 4.15.0, with debug_info, not stripped
 | hello.c:4:12: error: expected declaration specifiers or '...' before string constant
 4 | printf("Hello, world!\n");
 | ^~~~~~~~~~~~~~~~~
hello.c:5:5: error: expected identifier or '(' before 'return'
 5 | return 0;
 | ^~~~~~
hello.c:6:1: error: expected identifier or '(' before '}' token
 6 | }
 | ^
 |
| run-hello | 运行 Hello World 程序 | Pass | 0 | Hello, world!
 | |
| extract-coremark | 提取 CoreMark 包 | Pass | 0 | | mkdir: 无法创建目录 "coremark": File exists
warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday
info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800
info: in order to hide this banner:
info: - opt out with ruyi telemetry optout
info: - or give consent with ruyi telemetry consent
info: extracting coremark-1.01.tar.gz for package coremark-1.0.1
info: package coremark-1.0.1 extracted to current working directory
 |
| config-coremark | 配置 CoreMark 编译 | Pass | 0 | | |
| build-coremark | 编译 CoreMark (默认优化) | Pass | 0 | riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -lrt"\" -DITERATIONS=0 core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
Link performed along with compile
coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=5c8618cf62e0f1f7dd462ba5bddb03479631d0e9, for GNU/Linux 4.15.0, with debug_info, not stripped
 | |
| run-coremark | 运行 CoreMark (默认优化) | Pass | 0 | 2K performance run parameters for coremark.
CoreMark Size : 666
Total ticks : 20005
Total time (secs): 20.005000
Iterations/Sec : 5498.625344
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
CoreMark 1.0 : 5498.625344 / GCC14.2.0 -O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt / Heap
 | |
| build-coremark-vector | 编译 CoreMark (向量扩展) | Pass | 0 | riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt"\" -DITERATIONS=0 -march=rv64gcv_zvl256b -mabi=lp64d core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
Link performed along with compile
coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=26ed89acf90a9b7d555582be7602c422e6aeccbe, for GNU/Linux 4.15.0, with debug_info, not stripped
 | |
| run-coremark-vector | 运行 CoreMark (向量扩展) | Pass | 0 | 2K performance run parameters for coremark.
CoreMark Size : 666
Total ticks : 20009
Total time (secs): 20.009000
Iterations/Sec : 5497.526113
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
CoreMark 1.0 : 5497.526113 / GCC14.2.0 -O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt / Heap
 | |

## 9. 性能比较

| 优化选项 | CoreMark 分数 |
|---------|-------------|
| 默认优化 (-O2 -lrt) | 5498.625344 |
| 向量扩展 (-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt) | 5497.526113 |
