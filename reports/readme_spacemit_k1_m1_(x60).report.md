---
title: "SpacemiT K1/M1 (X60) GNU Toolchain (gnu-upstream) Test Report"
target_config: "target/k1.toml"
unit_name: "gnu-upstream"
unit_version: "0.1.0" # This is the version of this test unit itself
tags: ["toolchain", "gcc", "gnu-upstream", "K1"]
---

# SpacemiT K1/M1 (X60) GNU Toolchain (gnu-upstream) Test Report

This report details a series of basic functionality and performance tests conducted on the SpacemiT K1/M1 (X60) development board using the Upstream GNU Toolchain (`gnu-upstream`) provided by RuyiSDK. The tests aim to verify the correct installation of the toolchain, its basic compilation and linking capabilities, and its performance on a standard benchmark (CoreMark).

## Environment Information

The following hardware and software environment was used for this test:

### System Information

* **Test Date:** `2025-05-21`
* **Target Configuration:** `target/k1.toml`
* **Test Unit Name:** `gnu-upstream`
* **Test Unit Version:** `0.1.0`
* **Installed Toolchain Package:** `gnu-upstream-0.20250401.0`
* **GCC Version (from -v):** `14.2.0`
* RuyiSDK running on a `Banana Pi BPI-F3 with SpacemiT K1/M1 (X60) SoC`.

### Hardware Information

* Banana Pi BPI-F3 board
* SpacemiT K1/M1 SoC (RISC-V SpacemiT X60 core)

## Installation

This section documents the process of installing and setting up the Upstream GNU Toolchain using RuyiSDK.

### 0. Initialize Ruyi Package Manager

Clean up any potentially existing old environments and download/prepare the Ruyi CLI tool.

```bash
rm -rf ~/venv-gnu-upstream ~/ruyi /tmp/coremark_* ~/.local/share/ruyi/
curl -Lo ~/ruyi https://mirror.iscas.ac.cn/ruyisdk/ruyi/tags/0.33.0/ruyi.riscv64
chmod +x ~/ruyi
echo "Ruyi CLI downloaded and prepared. Old directories cleaned."
```

**Command Output:**

```output {ref="init-ruyi"}
[stdout]
Ruyi CLI downloaded and prepared. Old directories cleaned.
[stderr]
 % Total % Received % Xferd Average Speed Time Time Time Current
 Dload Upload Total Spent Left Speed

 0 0 0 0 0 0 0 0 --:--:-- --:--:-- --:--:-- 0
 0 0 0 0 0 0 0 0 --:--:-- --:--:-- --:--:-- 0
100 23.9M 100 23.9M 0 0 15.5M 0 0:00:01 0:00:01 --:--:-- 15.6M
```

Ruyi CLI Initialization Status (based on `assert.exit_code`): Pass

### 1. Install Toolchain

Install the Upstream GNU Toolchain package using Ruyi.

```bash
~/ruyi install toolchain/gnu-upstream
```

**Command Output:**

```output {ref="install-toolchain"}
[stderr]
warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday
info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800
info: in order to hide this banner:
info: - opt out with ruyi telemetry optout
info: - or give consent with ruyi telemetry consent
info: extracting RuyiSDK-20250401-Upstream-Sources-HOST-riscv64-linux-gnu-riscv64-unknown-linux-gnu.tar.xz for package gnu-upstream-0.20250401.0
info: package gnu-upstream-0.20250401.0 installed to /home/ezra/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0
```

This command downloaded the toolchain package `gnu-upstream-0.20250401.0` (approx. 161MB) from the configured mirror and installed it into Ruyi's local repository.
Actual installation path: `/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0`.
Installation Status (based on `assert.exit_code`): Pass

### 2. Create Virtual Environment

Create an isolated virtual environment for this test.

```bash
~/ruyi venv -t toolchain/gnu-upstream generic venv-gnu-upstream
```

**Command Output:**

```output {ref="create-venv"}
[stderr]
warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday
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
```

This step created a virtual environment directory at `venv-gnu-upstream`.
Creation Status (based on `assert.exit_code`): Pass

**Verify Created Environment Contents:**

```bash
ls ~/venv-gnu-upstream/
ls ~/venv-gnu-upstream/bin/
```

**Command Output:**

```output {ref="verify-venv"}
[stdout]
bin
meson-cross.ini
meson-cross.riscv64-unknown-linux-gnu.ini
ruyi-cache.v2.toml
ruyi-venv.toml
sysroot
sysroot.riscv64-unknown-linux-gnu
toolchain.cmake
toolchain.riscv64-unknown-linux-gnu.cmake
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

Environment Contents Verification (based on `assert.stdout_contains`):
- `assert.exit_code=0`: Pass
- `bin` `ruyi-venv.toml` `toolchain.cmake` `riscv64-unknown-linux-gnu-gcc` `ruyi-activate` exists: Pass

### 3. Activate Environment

Activate the virtual environment.

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
echo "Environment activated. Current PATH: $PATH"
```

**Command Output:**

```output {ref="activate-venv"}
[stdout]
Environment activated. Current PATH: /home/ezra/venv-gnu-upstream/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin
```

**Command Output (Activate Environment):**

```output {ref="activate-venv"}
[stdout]
Environment activated. Current PATH: /home/ezra/venv-gnu-upstream/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin
```

Environment Activation Status (based on `assert.exit_code` and `assert.stdout_contains`): Success

## Tests & Results

### 1. Compiler Version Check

Check the version information of `riscv64-unknown-linux-gnu-gcc`.

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
riscv64-unknown-linux-gnu-gcc -v 2>&1
```

**Command Output:**

```output {ref="check-version"}
[stdout]
Using built-in specs.
COLLECT_GCC=/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0/bin/riscv64-unknown-linux-gnu-gcc
COLLECT_LTO_WRAPPER=/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0/bin/../libexec/gcc/riscv64-unknown-linux-gnu/14.2.0/lto-wrapper
Target: riscv64-unknown-linux-gnu
Configured with: /work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/src/gcc/configure --build=x86_64-build_pc-linux-gnu --host=riscv64-host_unknown-linux-gnu --target=riscv64-unknown-linux-gnu --prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu --exec_prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu --with-sysroot=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/riscv64-unknown-linux-gnu/sysroot --enable-languages=c,c++,fortran,objc,obj-c++ --with-arch=rv64gc --with-abi=lp64d --with-pkgversion='RuyiSDK 20250401 Upstream-Sources' --with-bugurl=https://github.com/ruyisdk/ruyisdk/issues --enable-__cxa_atexit --disable-libmudflap --disable-libgomp --disable-libquadmath --disable-libquadmath-support --disable-libmpx --with-gmp=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --with-mpfr=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --with-mpc=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --with-isl=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --enable-lto --enable-threads=posix --enable-target-optspace --enable-linker-build-id --with-linker-hash-style=gnu --enable-plugin --disable-nls --disable-multilib --with-local-prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/riscv64-unknown-linux-gnu/sysroot --enable-long-long
Thread model: posix
Supported LTO compression algorithms: zlib zstd
gcc version 14.2.0 (RuyiSDK 20250401 Upstream-Sources)
```

**Analysis:**
Exit code check: Pass
GCC version check (`assert.stdout_contains="gcc version"`): Pass
The output shows detailed GCC version and configuration options:
* GCC Version: `14.2.0` (Matches expected version 14.2.0)
* Target Triple: `riscv64-unknown-linux-gnu` (Matches expected)
* Configured Arch: `rv64gc` (Matches expected rv64gc)
* Configured ABI: `lp64d` (Matches expected lp64d)

### 2. Hello World Program

Compile and run a simple "Hello, world!" C program.

```bash
cd /tmp
cat > hello.c << 'EOF'
#include <stdio.h>

int main() {printf("Hello, world!\n"); return 0;}
EOF
```

**Command Output (Create source file):**

```output {ref="create-hello"}
```

Source file `hello.c` creation status (based on `assert.exit_code`): Pass

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
cd /tmp
riscv64-unknown-linux-gnu-gcc hello.c -o hello_upstream
file hello_upstream
```

**Command Output (Compile and check ELF format):**

```output {ref="compile-hello"}
[stdout]
hello_upstream: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=3f957a7dffdbebbda3524d8c601b149ef231839c, for GNU/Linux 4.15.0, with debug_info, not stripped
```

**Analysis:**
Compilation and linking status (based on `assert.exit_code`): Pass.
ELF format checks (based on `assert.stdout_contains`):
- "ELF 64-bit LSB executable", "RISC-V", "dynamically linked": Pass
Generated `hello_upstream` file type: `ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1`.
Interpreter: `/lib/ld-linux-riscv64-lp64d.so.1` (Matches expected).

```bash
cd /tmp
./hello_upstream
```

**Command Output (Run program):**

```output {ref="run-hello"}
[stdout]
Hello, world!
[stderr]
bash: 第 1 行：export: "-mabi=lp64d link
file": 不是有效的标识符
```

**Analysis:**
Program execution status (based on `assert.exit_code`): Pass.
Program output: `Hello, world!`.
Output correctness (based on `assert.stdout`): Correct (Output matches 'Hello, world!').

### 3. CoreMark Benchmark (Default Optimizations)

Compile and run CoreMark using default optimization options (`-O2 -lrt`).

**Command (Extract CoreMark - Default):**

```bash
mkdir -p /tmp/coremark_default
cd /tmp/coremark_default
~/ruyi extract coremark
```

**Command Output (Extract CoreMark - Default):**

```output {ref="extract-coremark-default"}
[stderr]
warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday
info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800
info: in order to hide this banner:
info: - opt out with ruyi telemetry optout
info: - or give consent with ruyi telemetry consent
info: extracting coremark-1.01.tar.gz for package coremark-1.0.1
info: package coremark-1.0.1 extracted to current working directory
```

CoreMark (Default) extraction status (based on `assert.exit_code`): Pass.

**Command (Configure Makefile - Default):**

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
cd /tmp/coremark_default
sed -i 's/\bgcc\b/riscv64-unknown-linux-gnu-gcc/g' linux64/core_portme.mak
```

**Command Output (Configure Makefile - Default):**

```output {ref="config-coremark-default"}
```

Makefile (Default) configuration status (based on `assert.exit_code`): Pass.

**Command (Compile CoreMark - Default Optimizations):**

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
cd /tmp/coremark_default
make PORT_DIR=linux64 link
file coremark.exe
```

**Command Output (Compile CoreMark - Default Optimizations):**

```output {ref="build-coremark-default"}
[stdout]
riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -lrt"\" -DITERATIONS=0 core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
Link performed along with compile
coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=5c8618cf62e0f1f7dd462ba5bddb03479631d0e9, for GNU/Linux 4.15.0, with debug_info, not stripped
```

CoreMark (Default Opt.) compilation status (based on `assert.exit_code` and `assert.stdout_contains`): ✅ PASS.

**Command (Run CoreMark - Default Optimizations):**

```bash
cd /tmp/coremark_default
./coremark.exe
```

**Command Output (Run CoreMark - Default Optimizations):**

```output {ref="run-coremark-default"}
[stdout]
2K performance run parameters for coremark.
CoreMark Size : 666
Total ticks : 19446
Total time (secs): 19.446000
Iterations/Sec : 5656.690322
Iterations : 110000
Compiler version : GCC14.2.0
Compiler flags : -O2 -lrt
Memory location : Please put data memory location here
 (e.g. code in flash, data on heap etc)
seedcrc : 0xe9f5
[0]crclist : 0xe714
[0]crcmatrix : 0x1fd7
[0]crcstate : 0x8e3a
[0]crcfinal : 0x33ff
Correct operation validated. See readme.txt for run and reporting rules.
CoreMark 1.0 : 5656.690322 / GCC14.2.0 -O2 -lrt / Heap
[stderr]
bash: 第 1 行：export: "-mabi=lp64d link
file": 不是有效的标识符
```

**Results (Default Optimizations):**
CoreMark run status (based on `assert.stdout_contains="CoreMark 1.0"`): Pass
CoreMark Score (Iterations/Sec): `5656.690322`
Reported Compiler Version: `GCC14.2.0` (Does not match -v or not extracted)
Reported Compiler Flags: `-O2 -lrt`

### 4. CoreMark Benchmark (Vector Extension Optimizations)

Compile and run CoreMark using `-march=rv64gcv_zvl256b -mabi=lp64d`.

**Command (Extract CoreMark - Vector):**

```bash
mkdir -p /tmp/coremark_vector
cd /tmp/coremark_vector
~/ruyi extract coremark
```

**Command Output (Extract CoreMark - Vector):**

```output {ref="extract-coremark-vector"}
[stderr]
warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday
info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800
info: in order to hide this banner:
info: - opt out with ruyi telemetry optout
info: - or give consent with ruyi telemetry consent
info: extracting coremark-1.01.tar.gz for package coremark-1.0.1
info: package coremark-1.0.1 extracted to current working directory
```

CoreMark (Vector) extraction status (based on `assert.exit_code`): Pass.

**Command (Configure Makefile - Vector):**

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
cd /tmp/coremark_vector
sed -i 's/\bgcc\b/riscv64-unknown-linux-gnu-gcc/g' linux64/core_portme.mak
```

**Command Output (Configure Makefile - Vector):**

```output {ref="config-coremark-vector"}
```

Makefile (Vector) configuration status (based on `assert.exit_code`): Pass.

**Command (Compile CoreMark - Vector Optimizations):**

```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
cd /tmp/coremark_vector
make PORT_DIR=linux64 XCFLAGS="-march=rv64gcv_zvl256b -mabi=lp64d" link
file coremark.exe
```

**Command Output (Compile CoreMark - Vector Optimizations):**

```output {ref="build-coremark-vector"}
[stdout]
riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt"\" -DITERATIONS=0 -march=rv64gcv_zvl256b -mabi=lp64d core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
Link performed along with compile
coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=26ed89acf90a9b7d555582be7602c422e6aeccbe, for GNU/Linux 4.15.0, with debug_info, not stripped
```

CoreMark (Vector Opt.) compilation status (based on `assert.exit_code` and `assert.stdout_contains`): ✅ PASS.

**Command (Run CoreMark - Vector Optimizations):**

```bash
cd /tmp/coremark_vector
./coremark.exe
```

**Command Output (Run CoreMark - Vector Optimizations):**

```output {ref="run-coremark-vector"}
[stdout]
2K performance run parameters for coremark.
CoreMark Size : 666
Total ticks : 20024
Total time (secs): 20.024000
Iterations/Sec : 5493.407911
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
CoreMark 1.0 : 5493.407911 / GCC14.2.0 -O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt / Heap
[stderr]
bash: 第 1 行：export: "-mabi=lp64d link
file": 不是有效的标识符
```

**Results (Vector Optimizations):**
CoreMark run status (based on `assert.stdout_contains="CoreMark 1.0"`): Pass
CoreMark Score (Iterations/Sec): `5493.407911`
Reported Compiler Version: `GCC14.2.0`
Reported Compiler Flags: `-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt`

## Performance Comparison

Higher CoreMark scores (Iterations/Sec) indicate better performance.

| Metric | Default Optimizations | Vector Extension Optimizations|
|-------------------------------|-----------------------------------------------------|-----------------------------------------------------------------------|
| **Iterations/Sec** | `5656.690322` | `5493.407911` |
| **Total ticks** | `19446` | `20024` |
| **Total time (secs)** | `19.446000` | `20.024000` |
| **Iterations** | `110000` | `110000` |
| **Compiler (Reported)** | `GCC14.2.0` | `GCC14.2.0` |
| **Compiler Flags (Reported)** | `-O2 -lrt` | `-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt` |
| **Compiler (from -v)** | `14.2.0` | `14.2.0` |

**Performance Analysis:**
In this test, CoreMark on the SpacemiT K1/M1 (X60) SoC using GCC `14.2.0` achieved scores of `5656.690322` (Default Optimizations) and `5493.407911` (Vector Extension Optimizations).
Further analysis of these scores can reveal the specific impact of vector extensions for the CoreMark workload with this compiler version.

## Test Summary

The following table, automatically generated by `lintestor`, summarizes the execution status of each step:

| Step ID | Description | Status | Exit Code | Stdout Summary | Stderr Summary |
|-------------------------|---------------------------------------------------|----------------------------------------|-----------------------------------|------------------------------------------|------------------------------------------|
| init-ruyi | Download and prepare RuyiSDK CLI | Pass | 0 | Ruyi CLI downloaded and prepared. Old directories cleaned. | % Total % Received % Xferd Average Speed Time Time Time Current Dload Upload Total Spent Left Speed 
 0 0 0 0 0 0 0 0 --:--:-- --:--:-- --:--:-- 0
 0 0 0 0 0 0 0 0 --:--:-- --:--:-- --:--:-- 0
100 23.9M 100 23.9M 0 0 15.5M ... ... |
| install-toolchain | Install Upstream GNU Toolchain | Pass | 0 | | warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800 info: in order to hide this banner: info: - opt out with ruyi telemetry optout info: - or give consent with ruyi telemetry consent ... |
| create-venv | Create virtual environment | Pass | 0 | | warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800 info: in order to hide this banner: info: - opt out with ruyi telemetry optout info: - or give consent with ruyi telemetry consent ... |
| verify-venv | Verify virtual environment contents | Pass | 0 | bin meson-cross.ini meson-cross.riscv64-unknown-linux-gnu.ini ruyi-cache.v2.toml ruyi-venv.toml ... | |
| activate-venv | Activate virtual environment | Pass | 0 | Environment activated. Current PATH: /home/ezra/venv-gnu-upstream/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin | |
| check-version | Check compiler version | Pass | 0 | Using built-in specs. COLLECT_GCC=/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0/bin/riscv64-unknown-linux-gnu-gcc COLLECT_LTO_WRAPPER=/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0/bin/../libexec/gcc/riscv64-unknown-linux-gnu/14.2.0/lto-wrapper Target: riscv64-unknown-linux-gnu Configured with: /work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/src/gcc/configure --build=x86_64-build_pc-linux-gnu --host=riscv64-host_unknown-linux-gnu --target=riscv64-unknown-linux-gnu --p... ... | |
| create-hello | Create Hello World source file | Pass | 0 | | |
| compile-hello | Compile Hello World program and check ELF format | Pass | 0 | hello_upstream: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=3f957a7dffdbebbda3524d8c... ... | |
| run-hello | Run Hello World program | Pass | 0 | Hello, world! | bash: 第 1 行：export: "-mabi=lp64d link file": 不是有效的标识符 |
| extract-coremark-default| Create directory and extract CoreMark (Default Opt.)| Pass | 0 | | warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800 info: in order to hide this banner: info: - opt out with ruyi telemetry optout info: - or give consent with ruyi telemetry consent ... |
| config-coremark-default | Configure CoreMark Makefile (Default Opt.) | Pass | 0 | | |
| build-coremark-default | Compile CoreMark (Default Opt. -O2 -lrt) | Pass | 0 | riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -lrt"\" -DITERATIONS=0 core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe... Link performed along with compile coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=5c8618cf62e0f1f7dd462ba5bd... ... | |
| run-coremark-default | Run CoreMark (Default Opt.) | Pass | 0 | 2K performance run parameters for coremark. CoreMark Size : 666 Total ticks : 19446 Total time (secs): 19.446000 Iterations/Sec : 5656.690322 ... | bash: 第 1 行：export: "-mabi=lp64d link file": 不是有效的标识符 |
| extract-coremark-vector | Create directory and extract CoreMark (Vector Opt.) | Pass | 0 | | warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800 info: in order to hide this banner: info: - opt out with ruyi telemetry optout info: - or give consent with ruyi telemetry consent ... |
| config-coremark-vector | Configure CoreMark Makefile (Vector Opt.) | Pass | 0 | | |
| build-coremark-vector | Compile CoreMark (Vector Extension Opt.) | Pass | 0 | riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt"\" -DITERATIONS=0 -march=rv64gcv_zvl256b -mabi=lp64d core_list_join.c core_main.c core_matri... Link performed along with compile coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=26ed89acf90a9b7d555582be76... ... | |
| run-coremark-vector | Run CoreMark (Vector Extension Opt.) | Pass | 0 | 2K performance run parameters for coremark. CoreMark Size : 666 Total ticks : 20024 Total time (secs): 20.024000 Iterations/Sec : 5493.407911 ... | bash: 第 1 行：export: "-mabi=lp64d link file": 不是有效的标识符 |

---
**Manual-Style Test Conclusions Overview (based on exit_code and assertion status strings):**

| Test Item | Expected Result | Actual Status (template logic) |
|---------------------------------------------------|-------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Ruyi CLI Initialization (`init-ruyi`) | Ruyi CLI successfully prepared | ✅ PASS |
| Toolchain Installation (`install-toolchain`) | Successfully installed | ✅ PASS (Path: `/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0`) |
| Compiler Version Check (`check-version`) | GCC 14.2.0, riscv64-unknown-linux-gnu, rv64gc, lp64d | ✅ PASS (Version: `14.2.0`) |
| Hello World Compilation (`compile-hello`) | Successfully compiled ELF 64-bit | ✅ PASS |
| Hello World Execution (`run-hello`) | Outputs "Hello, world!" | ✅ PASS (Output: `Hello, world!`) |
| CoreMark (Default) Compilation (`build-coremark-default`) | Successfully compiled | ✅ PASS |
| CoreMark (Default) Execution (`run-coremark-default`) | Successfully runs, "CoreMark 1.0" flag present | ✅ PASS (Score: `5656.690322`) |
| CoreMark (Vector) Compilation (`build-coremark-vector`) | Successfully compiled | ✅ PASS |
| CoreMark (Vector) Execution (`run-coremark-vector`) | Successfully runs, "CoreMark 1.0" flag present | ✅ PASS (Score: `5493.407911`) |
