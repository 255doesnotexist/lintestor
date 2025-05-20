---
title: "SpacemiT K1/M1 (X60) GNU Toolchain (gnu-plct) Test Report"
target_config: "target/k1.toml"
unit_name: "gnu-plct"
unit_version: "0.1.0" # This is the version of this test unit itself
tags: ["toolchain", "gcc", "gnu-plct", "K1"]
gcc_name: "riscv64-plct-linux-gnu-gcc"
---

# SpacemiT K1/M1 (X60) GNU Toolchain (gnu-plct) Test Report

This report details a series of basic functionality and performance tests conducted on the SpacemiT K1/M1 (X60) development board using the PLCT GNU Toolchain (`gnu-plct`) provided by RuyiSDK. The tests aim to verify the correct installation of the toolchain, its basic compilation and linking capabilities, and its performance on a standard benchmark (CoreMark).

## Environment Information

The following hardware and software environment was used for this test:

### System Information

* **Test Date:** `2025-05-21`
* **Target Configuration:** `target/k1.toml`
* **Test Unit Name:** `gnu-plct`
* **Test Unit Version:** `0.1.0`
* **Installed Toolchain Package:** `gnu-plct-0.20250401.0`
* **GCC Version (from -v):** `14.1.0`
* RuyiSDK running on a `Banana Pi BPI-F3 with SpacemiT K1/M1 (X60) SoC`.

### Hardware Information

* Banana Pi BPI-F3 board
* SpacemiT K1/M1 SoC (RISC-V SpacemiT X60 core)

## Installation

This section documents the process of installing and setting up the PLCT GNU Toolchain using RuyiSDK.

### 0. Initialize Ruyi Package Manager

Clean up any potentially existing old environments and download/prepare the Ruyi CLI tool.

```bash
rm -rf ~/venv-gnu-plct ~/ruyi /tmp/coremark_* ~/.local/share/ruyi/
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
 0 23.9M 0 72400 0 0 97846 0 0:04:16 --:--:-- 0:04:16 97837
100 23.9M 100 23.9M 0 0 15.1M 0 0:00:01 0:00:01 --:--:-- 15.1M
```

Ruyi CLI Initialization Status (based on `assert.exit_code`): Pass

### 1. Install Toolchain

Install the PLCT GNU Toolchain package using Ruyi.

```bash
~/ruyi install toolchain/gnu-plct
```

**Command Output:**

```output {ref="install-toolchain"}
[stderr]
warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday
info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800
info: in order to hide this banner:
info: - opt out with ruyi telemetry optout
info: - or give consent with ruyi telemetry consent
info: extracting RuyiSDK-20250401-PLCT-Sources-HOST-riscv64-linux-gnu-riscv64-plct-linux-gnu.tar.xz for package gnu-plct-0.20250401.0
info: package gnu-plct-0.20250401.0 installed to /home/ezra/.local/share/ruyi/binaries/riscv64/gnu-plct-0.20250401.0
```

This command downloaded the toolchain package `gnu-plct-0.20250401.0` (approx. 161MB) from the configured mirror and installed it into Ruyi's local repository.
Actual installation path: `/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-plct-0.20250401.0`.
Installation Status (based on `assert.exit_code`): Pass

### 2. Create Virtual Environment

Create an isolated virtual environment for this test.

```bash
~/ruyi venv -t toolchain/gnu-plct generic venv-gnu-plct
```

**Command Output:**

```output {ref="create-venv"}
[stderr]
warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday
info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800
info: in order to hide this banner:
info: - opt out with ruyi telemetry optout
info: - or give consent with ruyi telemetry consent
info: Creating a Ruyi virtual environment at venv-gnu-plct...
info: The virtual environment is now created.

You may activate it by sourcing the appropriate activation script in the
bin directory, and deactivate by invoking `ruyi-deactivate`.

A fresh sysroot/prefix is also provisioned in the virtual environment.
It is available at the following path:

 /home/ezra/venv-gnu-plct/sysroot

The virtual environment also comes with ready-made CMake toolchain file
and Meson cross file. Check the virtual environment root for those;
comments in the files contain usage instructions.
```

This step created a virtual environment directory at `venv-gnu-plct`.
Creation Status (based on `assert.exit_code`): Pass

**Verify Created Environment Contents:**

```bash
ls ~/venv-gnu-plct/
ls ~/venv-gnu-plct/bin/
```

**Command Output:**

```output {ref="verify-venv"}
[stdout]
bin
meson-cross.ini
meson-cross.riscv64-plct-linux-gnu.ini
ruyi-cache.v2.toml
ruyi-venv.toml
sysroot
sysroot.riscv64-plct-linux-gnu
toolchain.cmake
toolchain.riscv64-plct-linux-gnu.cmake
riscv64-plct-linux-gnu-addr2line
riscv64-plct-linux-gnu-ar
riscv64-plct-linux-gnu-as
riscv64-plct-linux-gnu-c++
riscv64-plct-linux-gnu-cc
riscv64-plct-linux-gnu-c++filt
riscv64-plct-linux-gnu-cpp
riscv64-plct-linux-gnu-elfedit
riscv64-plct-linux-gnu-g++
riscv64-plct-linux-gnu-gcc
riscv64-plct-linux-gnu-gcc-ar
riscv64-plct-linux-gnu-gcc-nm
riscv64-plct-linux-gnu-gcc-ranlib
riscv64-plct-linux-gnu-gcov
riscv64-plct-linux-gnu-gcov-dump
riscv64-plct-linux-gnu-gcov-tool
riscv64-plct-linux-gnu-gdb
riscv64-plct-linux-gnu-gdb-add-index
riscv64-plct-linux-gnu-gfortran
riscv64-plct-linux-gnu-gprof
riscv64-plct-linux-gnu-ld
riscv64-plct-linux-gnu-ld.bfd
riscv64-plct-linux-gnu-ldd
riscv64-plct-linux-gnu-lto-dump
riscv64-plct-linux-gnu-nm
riscv64-plct-linux-gnu-objcopy
riscv64-plct-linux-gnu-objdump
riscv64-plct-linux-gnu-ranlib
riscv64-plct-linux-gnu-readelf
riscv64-plct-linux-gnu-size
riscv64-plct-linux-gnu-strings
riscv64-plct-linux-gnu-strip
ruyi-activate
```

Environment Contents Verification (based on `assert.stdout_contains`):
- `assert.exit_code=0`: Pass
- `bin` `ruyi-venv.toml` `toolchain.cmake` `riscv64-plct-linux-gnu-gcc` `ruyi-activate` exists: Pass

### 3. Activate Environment

Activate the virtual environment.

```bash
. ~/venv-gnu-plct/bin/ruyi-activate
echo "Environment activated. Current PATH: $PATH"
```

**Command Output:**

```output {ref="activate-venv"}
[stdout]
Environment activated. Current PATH: /home/ezra/venv-gnu-plct/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin
```

**Command Output (Activate Environment):**

```output {ref="activate-venv"}
[stdout]
Environment activated. Current PATH: /home/ezra/venv-gnu-plct/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin
```

Environment Activation Status (based on `assert.exit_code` and `assert.stdout_contains`): Success

## Tests & Results

### 1. Compiler Version Check

Check the version information of `riscv64-plct-linux-gnu-gcc`.

```bash
. ~/venv-gnu-plct/bin/ruyi-activate
riscv64-plct-linux-gnu-gcc -v 2>&1
```

**Command Output:**

```output {ref="check-version"}
[stdout]
Using built-in specs.
COLLECT_GCC=/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-plct-0.20250401.0/bin/riscv64-plct-linux-gnu-gcc
COLLECT_LTO_WRAPPER=/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-plct-0.20250401.0/bin/../libexec/gcc/riscv64-plct-linux-gnu/14.1.0/lto-wrapper
Target: riscv64-plct-linux-gnu
Configured with: /work/HOST-riscv64-linux-gnu/riscv64-plct-linux-gnu/src/gcc/configure --build=x86_64-build_pc-linux-gnu --host=riscv64-host_unknown-linux-gnu --target=riscv64-plct-linux-gnu --prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-plct-linux-gnu --exec_prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-plct-linux-gnu --with-sysroot=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-plct-linux-gnu/riscv64-plct-linux-gnu/sysroot --enable-languages=c,c++,fortran,objc,obj-c++ --with-arch=rv64gc --with-abi=lp64d --with-pkgversion='RuyiSDK 20250401 PLCT-Sources' --with-bugurl=https://github.com/ruyisdk/ruyisdk/issues --enable-__cxa_atexit --disable-libmudflap --disable-libgomp --disable-libquadmath --disable-libquadmath-support --disable-libmpx --with-gmp=/work/HOST-riscv64-linux-gnu/riscv64-plct-linux-gnu/buildtools/complibs-host --with-mpfr=/work/HOST-riscv64-linux-gnu/riscv64-plct-linux-gnu/buildtools/complibs-host --with-mpc=/work/HOST-riscv64-linux-gnu/riscv64-plct-linux-gnu/buildtools/complibs-host --with-isl=/work/HOST-riscv64-linux-gnu/riscv64-plct-linux-gnu/buildtools/complibs-host --enable-lto --enable-threads=posix --enable-target-optspace --enable-linker-build-id --with-linker-hash-style=gnu --enable-plugin --disable-nls --disable-multilib --with-local-prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-plct-linux-gnu/riscv64-plct-linux-gnu/sysroot --enable-long-long
Thread model: posix
Supported LTO compression algorithms: zlib zstd
gcc version 14.1.0 (RuyiSDK 20250401 PLCT-Sources)
```

**Analysis:**
Exit code check: Pass
GCC version check (`assert.stdout_contains="gcc version"`): Pass
The output shows detailed GCC version and configuration options:
* GCC Version: `14.1.0` (Does not match expected version 14.2.0 or not extracted)
* Target Triple: `riscv64-plct-linux-gnu` (Does not match expected riscv64-unknown-linux-gnu)
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
. ~/venv-gnu-plct/bin/ruyi-activate
cd /tmp
riscv64-plct-linux-gnu-gcc hello.c -o hello_plct
file hello_plct
```

**Command Output (Compile and check ELF format):**

```output {ref="compile-hello"}
[stdout]
hello_plct: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=f6dc2eb15f6d8844b93ab461c4625fd3cc70d8c8, for GNU/Linux 4.15.0, with debug_info, not stripped
```

**Analysis:**
Compilation and linking status (based on `assert.exit_code`): Pass.
ELF format checks (based on `assert.stdout_contains`):
- "ELF 64-bit LSB executable", "RISC-V", "dynamically linked": Pass
Generated `hello_plct` file type: `ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1`.
Interpreter: `/lib/ld-linux-riscv64-lp64d.so.1` (Matches expected).

```bash
cd /tmp
./hello_plct
```

**Command Output (Run program):**

```output {ref="run-hello"}
[stdout]
Hello, world!
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
. ~/venv-gnu-plct/bin/ruyi-activate
cd /tmp/coremark_default
sed -i 's/\bgcc\b/riscv64-plct-linux-gnu-gcc/g' linux64/core_portme.mak
```

**Command Output (Configure Makefile - Default):**

```output {ref="config-coremark-default"}
```

Makefile (Default) configuration status (based on `assert.exit_code`): Pass.

**Command (Compile CoreMark - Default Optimizations):**

```bash
. ~/venv-gnu-plct/bin/ruyi-activate
cd /tmp/coremark_default
make PORT_DIR=linux64 link
file coremark.exe
```

**Command Output (Compile CoreMark - Default Optimizations):**

```output {ref="build-coremark-default"}
[stdout]
riscv64-plct-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -lrt"\" -DITERATIONS=0 core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
Link performed along with compile
coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=db453fedfca2f8f9d2f6567a949c0147daf8218f, for GNU/Linux 4.15.0, with debug_info, not stripped
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
Total ticks : 19407
Total time (secs): 19.407000
Iterations/Sec : 5668.057917
Iterations : 110000
Compiler version : GCC14.1.0
Compiler flags : -O2 -lrt
Memory location : Please put data memory location here
 (e.g. code in flash, data on heap etc)
seedcrc : 0xe9f5
[0]crclist : 0xe714
[0]crcmatrix : 0x1fd7
[0]crcstate : 0x8e3a
[0]crcfinal : 0x33ff
Correct operation validated. See readme.txt for run and reporting rules.
CoreMark 1.0 : 5668.057917 / GCC14.1.0 -O2 -lrt / Heap
```

**Results (Default Optimizations):**
CoreMark run status (based on `assert.stdout_contains="CoreMark 1.0"`): Pass
CoreMark Score (Iterations/Sec): `5668.057917`
Reported Compiler Version: `GCC14.1.0` (Does not match -v or not extracted)
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
. ~/venv-gnu-plct/bin/ruyi-activate
cd /tmp/coremark_vector
sed -i 's/\bgcc\b/riscv64-plct-linux-gnu-gcc/g' linux64/core_portme.mak
```

**Command Output (Configure Makefile - Vector):**

```output {ref="config-coremark-vector"}
```

Makefile (Vector) configuration status (based on `assert.exit_code`): Pass.

**Command (Compile CoreMark - Vector Optimizations):**

```bash
. ~/venv-gnu-plct/bin/ruyi-activate
cd /tmp/coremark_vector
make PORT_DIR=linux64 XCFLAGS="-march=rv64gcv_zvl256b -mabi=lp64d" link
file coremark.exe
```

**Command Output (Compile CoreMark - Vector Optimizations):**

```output {ref="build-coremark-vector"}
[stdout]
riscv64-plct-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt"\" -DITERATIONS=0 -march=rv64gcv_zvl256b -mabi=lp64d core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
Link performed along with compile
coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=8a8e2adba38393d260ae24ad6e438951b0c9a9fd, for GNU/Linux 4.15.0, with debug_info, not stripped
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
Total ticks : 20009
Total time (secs): 20.009000
Iterations/Sec : 5497.526113
Iterations : 110000
Compiler version : GCC14.1.0
Compiler flags : -O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt
Memory location : Please put data memory location here
 (e.g. code in flash, data on heap etc)
seedcrc : 0xe9f5
[0]crclist : 0xe714
[0]crcmatrix : 0x1fd7
[0]crcstate : 0x8e3a
[0]crcfinal : 0x33ff
Correct operation validated. See readme.txt for run and reporting rules.
CoreMark 1.0 : 5497.526113 / GCC14.1.0 -O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt / Heap
```

**Results (Vector Optimizations):**
CoreMark run status (based on `assert.stdout_contains="CoreMark 1.0"`): Pass
CoreMark Score (Iterations/Sec): `5497.526113`
Reported Compiler Version: `GCC14.1.0`
Reported Compiler Flags: `-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt`

## Performance Comparison

Higher CoreMark scores (Iterations/Sec) indicate better performance.

| Metric | Default Optimizations | Vector Extension Optimizations|
|-------------------------------|-----------------------------------------------------|-----------------------------------------------------------------------|
| **Iterations/Sec** | `5668.057917` | `5497.526113` |
| **Total ticks** | `19407` | `20009` |
| **Total time (secs)** | `19.407000` | `20.009000` |
| **Iterations** | `110000` | `110000` |
| **Compiler (Reported)** | `GCC14.1.0` | `GCC14.1.0` |
| **Compiler Flags (Reported)** | `-O2 -lrt` | `-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt` |
| **Compiler (from -v)** | `14.1.0` | `14.1.0` |

**Performance Analysis:**
In this test, CoreMark on the SpacemiT K1/M1 (X60) SoC using GCC `14.1.0` achieved scores of `5668.057917` (Default Optimizations) and `5497.526113` (Vector Extension Optimizations).
Further analysis of these scores can reveal the specific impact of vector extensions for the CoreMark workload with this compiler version.

## Test Summary

The following table, automatically generated by `lintestor`, summarizes the execution status of each step:

| Step ID | Description | Status | Exit Code | Stdout Summary | Stderr Summary |
|-------------------------|---------------------------------------------------|----------------------------------------|-----------------------------------|------------------------------------------|------------------------------------------|
| init-ruyi | Download and prepare RuyiSDK CLI | Pass | 0 | Ruyi CLI downloaded and prepared. Old directories cleaned. | % Total % Received % Xferd Average Speed Time Time Time Current Dload Upload Total Spent Left Speed 0 0 0 0 0 0 0 0 --:--:-- --:--:-- --:--:-- 0 0 23.9M 0 72400 0 0 97846 0 0:04:16 --:--:-- 0:04:16 97837 100 23.9M 100 23.9M 0 0 15.1M ... ... |
| install-toolchain | Install PLCT GNU Toolchain | Pass | 0 | | warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800 info: in order to hide this banner: info: - opt out with ruyi telemetry optout info: - or give consent with ruyi telemetry consent ... |
| create-venv | Create virtual environment | Pass | 0 | | warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800 info: in order to hide this banner: info: - opt out with ruyi telemetry optout info: - or give consent with ruyi telemetry consent ... |
| verify-venv | Verify virtual environment contents | Pass | 0 | bin meson-cross.ini meson-cross.riscv64-plct-linux-gnu.ini ruyi-cache.v2.toml ruyi-venv.toml ... | |
| activate-venv | Activate virtual environment | Pass | 0 | Environment activated. Current PATH: /home/ezra/venv-gnu-plct/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin | |
| check-version | Check compiler version | Pass | 0 | Using built-in specs. COLLECT_GCC=/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-plct-0.20250401.0/bin/riscv64-plct-linux-gnu-gcc COLLECT_LTO_WRAPPER=/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-plct-0.20250401.0/bin/../libexec/gcc/riscv64-plct-linux-gnu/14.1.0/lto-wrapper Target: riscv64-plct-linux-gnu Configured with: /work/HOST-riscv64-linux-gnu/riscv64-plct-linux-gnu/src/gcc/configure --build=x86_64-build_pc-linux-gnu --host=riscv64-host_unknown-linux-gnu --target=riscv64-plct-linux-gnu --prefix=... ... | |
| create-hello | Create Hello World source file | Pass | 0 | | |
| compile-hello | Compile Hello World program and check ELF format | Pass | 0 | hello_plct: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=f6dc2eb15f6d8844b93ab461c462... ... | {{ metadata.gcc_name }} |
| run-hello | Run Hello World program | Pass | 0 | Hello, world! | |
| extract-coremark-default| Create directory and extract CoreMark (Default Opt.)| Pass | 0 | | warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800 info: in order to hide this banner: info: - opt out with ruyi telemetry optout info: - or give consent with ruyi telemetry consent ... |
| config-coremark-default | Configure CoreMark Makefile (Default Opt.) | Pass | 0 | | |
| build-coremark-default | Compile CoreMark (Default Opt. -O2 -lrt) | Pass | 0 | riscv64-plct-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -lrt"\" -DITERATIONS=0 core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -l... Link performed along with compile coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=db453fedfca2f8f9d2f6567a94... ... | {{ metadata.gcc_name }} |
| run-coremark-default | Run CoreMark (Default Opt.) | Pass | 0 | 2K performance run parameters for coremark. CoreMark Size : 666 Total ticks : 19407 Total time (secs): 19.407000 Iterations/Sec : 5668.057917 ... | |
| extract-coremark-vector | Create directory and extract CoreMark (Vector Opt.) | Pass | 0 | | warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Wednesday info: the next upload will happen anytime ruyi is executed between 2025-05-21 08:00:00 +0800 and 2025-05-22 08:00:00 +0800 info: in order to hide this banner: info: - opt out with ruyi telemetry optout info: - or give consent with ruyi telemetry consent ... |
| config-coremark-vector | Configure CoreMark Makefile (Vector Opt.) | Pass | 0 | | |
| build-coremark-vector | Compile CoreMark (Vector Extension Opt.) | Pass | 0 | riscv64-plct-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -march=rv64gcv_zvl256b -mabi=lp64d -lrt"\" -DITERATIONS=0 -march=rv64gcv_zvl256b -mabi=lp64d core_list_join.c core_main.c core_matrix.c... Link performed along with compile coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=8a8e2adba38393d260ae24ad6e... ... | |
| run-coremark-vector | Run CoreMark (Vector Extension Opt.) | Pass | 0 | 2K performance run parameters for coremark. CoreMark Size : 666 Total ticks : 20009 Total time (secs): 20.009000 Iterations/Sec : 5497.526113 ... | |

---
**Manual-Style Test Conclusions Overview (based on exit_code and assertion status strings):**

| Test Item | Expected Result | Actual Status (template logic) |
|---------------------------------------------------|-------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Ruyi CLI Initialization (`init-ruyi`) | Ruyi CLI successfully prepared | ✅ PASS |
| Toolchain Installation (`install-toolchain`) | Successfully installed | ✅ PASS (Path: `/home/ezra/.local/share/ruyi/binaries/riscv64/gnu-plct-0.20250401.0`) |
| Compiler Version Check (`check-version`) | GCC 14.2.0, riscv64-unknown-linux-gnu, rv64gc, lp64d | ✅ PASS (Version: `14.1.0`) |
| Hello World Compilation (`compile-hello`) | Successfully compiled ELF 64-bit | ✅ PASS |
| Hello World Execution (`run-hello`) | Outputs "Hello, world!" | ✅ PASS (Output: `Hello, world!`) |
| CoreMark (Default) Compilation (`build-coremark-default`) | Successfully compiled | ✅ PASS |
| CoreMark (Default) Execution (`run-coremark-default`) | Successfully runs, "CoreMark 1.0" flag present | ✅ PASS (Score: `5668.057917`) |
| CoreMark (Vector) Compilation (`build-coremark-vector`) | Successfully compiled | ✅ PASS |
| CoreMark (Vector) Execution (`run-coremark-vector`) | Successfully runs, "CoreMark 1.0" flag present | ✅ PASS (Score: `5497.526113`) |
