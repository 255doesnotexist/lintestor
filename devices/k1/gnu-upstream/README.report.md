# SpacemiT K1/M1 (X60) GNU Toolchain (gnu-upstream) Test Report

## Environment

### System Information
- RuyiSDK on Banana Pi BPI-F3 with SpacemiT K1/M1 (X60) SoC
- Testing date: April 21th, 2025

### Hardware Information
- Banana Pi BPI-F3 board
- SpacemiT K1/M1 SoC (RISC-V SpacemiT X60 core)

## Installation

### 1. Install Toolchain

**Command:**
```bash
ruyi install toolchain/gnu-upstream
```

**Result:**
```bash
warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Thursday
info: the next upload will happen anytime ruyi is executed between 2025-04-24 08:00:00 +0800 and 2025-04-25 08:00:00 +0800
info: in order to hide this banner:
info: - opt out with ruyi telemetry optout
info: - or give consent with ruyi telemetry consent
info: downloading https://mirror.iscas.ac.cn/ruyisdk/dist/RuyiSDK-20250401-Upstream-Sources-HOST-riscv64-linux-gnu-riscv64-unknown-linux-gnu.tar.xz to /root/.cache/ruyi/distfiles/RuyiSDK-20250401-Upstream-Sources-HOST-riscv64-linux-gnu-riscv64-unknown-linux-gnu.tar.xz
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100  161M  100  161M    0     0  35.7M      0  0:00:04  0:00:04 --:--:-- 35.7M
info: extracting RuyiSDK-20250401-Upstream-Sources-HOST-riscv64-linux-gnu-riscv64-unknown-linux-gnu.tar.xz for package gnu-upstream-0.20250401.0
info: package gnu-upstream-0.20250401.0 installed to /root/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0
```

This command downloaded the toolchain package (~161MB) from ISCAS mirror and installed it to `/root/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0`.

### 2. Create Virtual Environment

**Command:**
```bash
ruyi venv -t toolchain/gnu-upstream generic venv-gnu-upstream
```

**Result:**
```bash
«Ruyi venv-gnu-plct» root@k1:~# ruyi venv -t toolchain/gnu-upstream generic venv-gnu-upstream
warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Thursday
info: the next upload will happen anytime ruyi is executed between 2025-04-24 08:00:00 +0800 and 2025-04-25 08:00:00 +0800
info: in order to hide this banner:
info: - opt out with ruyi telemetry optout
info: - or give consent with ruyi telemetry consent
info: Creating a Ruyi virtual environment at venv-gnu-upstream...
info: The virtual environment is now created.

You may activate it by sourcing the appropriate activation script in the
bin directory, and deactivate by invoking `ruyi-deactivate`.

A fresh sysroot/prefix is also provisioned in the virtual environment.
It is available at the following path:

    /root/venv-gnu-upstream/sysroot

The virtual environment also comes with ready-made CMake toolchain file
and Meson cross file. Check the virtual environment root for those;
comments in the files contain usage instructions.
```

**Verification of created environment contents:**
```bash
«Ruyi venv-gnu-plct» root@k1:~# ls ~/venv-gnu-upstream/
bin                                        ruyi-cache.v2.toml  sysroot.riscv64-unknown-linux-gnu
meson-cross.ini                            ruyi-venv.toml      toolchain.cmake
meson-cross.riscv64-unknown-linux-gnu.ini  sysroot             toolchain.riscv64-unknown-linux-gnu.cmake
«Ruyi venv-gnu-plct» root@k1:~# ls ~/venv-gnu-upstream/bin/
riscv64-unknown-linux-gnu-addr2line   riscv64-unknown-linux-gnu-gcov-dump        riscv64-unknown-linux-gnu-ld.bfd
riscv64-unknown-linux-gnu-ar          riscv64-unknown-linux-gnu-gcov-tool        riscv64-unknown-linux-gnu-ldd
riscv64-unknown-linux-gnu-as          riscv64-unknown-linux-gnu-gdb              riscv64-unknown-linux-gnu-lto-dump
riscv64-unknown-linux-gnu-c++         riscv64-unknown-linux-gnu-gdb-add-index    riscv64-unknown-linux-gnu-nm
riscv64-unknown-linux-gnu-cc          riscv64-unknown-linux-gnu-gfortran         riscv64-unknown-linux-gnu-objcopy
riscv64-unknown-linux-gnu-c++filt     riscv64-unknown-linux-gnu-gp-archive       riscv64-unknown-linux-gnu-objdump
riscv64-unknown-linux-gnu-cpp         riscv64-unknown-linux-gnu-gp-collect-app   riscv64-unknown-linux-gnu-ranlib
riscv64-unknown-linux-gnu-elfedit     riscv64-unknown-linux-gnu-gp-display-html  riscv64-unknown-linux-gnu-readelf
riscv64-unknown-linux-gnu-g++         riscv64-unknown-linux-gnu-gp-display-src   riscv64-unknown-linux-gnu-size
riscv64-unknown-linux-gnu-gcc         riscv64-unknown-linux-gnu-gp-display-text  riscv64-unknown-linux-gnu-strings
riscv64-unknown-linux-gnu-gcc-ar      riscv64-unknown-linux-gnu-gprof            riscv64-unknown-linux-gnu-strip
riscv64-unknown-linux-gnu-gcc-nm      riscv64-unknown-linux-gnu-gprofng          ruyi-activate
riscv64-unknown-linux-gnu-gcc-ranlib  riscv64-unknown-linux-gnu-gstack
riscv64-unknown-linux-gnu-gcov        riscv64-unknown-linux-gnu-ld
```

This step created a virtual environment at `/root/venv-gnu-upstream/` with all necessary configuration files, including Meson cross-compilation files, CMake toolchain files, a ready-to-use sysroot, and binary tools with the prefix `riscv64-unknown-linux-gnu-`.

### 3. Activate Environment

**Command:**
```bash
. ~/venv-gnu-upstream/bin/ruyi-activate
```

**Result:**
The prompt changed to indicate active environment:
```bash
«Ruyi venv-gnu-upstream» root@k1:~#
```

This initialized the environment and provided access to all cross-compilation tools.

## Tests & Results

### 1. Compiler Version Check

**Command:**
```bash
riscv64-unknown-linux-gnu-gcc -v
```

**Result:**
```bash
Using built-in specs.
COLLECT_GCC=/root/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0/bin/riscv64-unknown-linux-gnu-gcc
COLLECT_LTO_WRAPPER=/root/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0/bin/../libexec/gcc/riscv64-unknown-linux-gnu/14.2.0/lto-wrapper
Target: riscv64-unknown-linux-gnu
Configured with: /work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/src/gcc/configure --build=x86_64-build_pc-linux-gnu --host=riscv64-host_unknown-linux-gnu --target=riscv64-unknown-linux-gnu --prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu --exec_prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu --with-sysroot=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/riscv64-unknown-linux-gnu/sysroot --enable-languages=c,c++,fortran,objc,obj-c++ --with-arch=rv64gc --with-abi=lp64d --with-pkgversion='RuyiSDK 20250401 Upstream-Sources' --with-bugurl=https://github.com/ruyisdk/ruyisdk/issues --enable-__cxa_atexit --disable-libmudflap --disable-libgomp --disable-libquadmath --disable-libquadmath-support --disable-libmpx --with-gmp=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --with-mpfr=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --with-mpc=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --with-isl=/work/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/buildtools/complibs-host --enable-lto --enable-threads=posix --enable-target-optspace --enable-linker-build-id --with-linker-hash-style=gnu --enable-plugin --disable-nls --disable-multilib --with-local-prefix=/opt/ruyi/HOST-riscv64-linux-gnu/riscv64-unknown-linux-gnu/riscv64-unknown-linux-gnu/sysroot --enable-long-long
Thread model: posix
Supported LTO compression algorithms: zlib zstd
gcc version 14.2.0 (RuyiSDK 20250401 Upstream-Sources) 
```

The output confirmed a successful installation showing:
- GCC version: 14.2.0 (RuyiSDK 20250401 Upstream-Sources) 
- Target architecture: riscv64-unknown-linux-gnu
- Thread model: posix
- Configured with appropriate RISC-V specific options (rv64gc architecture, lp64d ABI)

### 2. Hello World Program

**Commands:**
```bash
«Ruyi venv-gnu-upstream» root@k1:~# nano hello.c
«Ruyi venv-gnu-upstream» root@k1:~# riscv64-unknown-linux-gnu-gcc hello.c -o hello_upstream
«Ruyi venv-gnu-upstream» root@k1:~# ./hello_upstream 
Hello, world!
```

The program executed successfully and output "Hello, world!", confirming basic functionality of the toolchain and proper integration with the system libraries.

### 3. CoreMark Benchmark (-O2 -lrt)

**Commands and Results:**

1. Extract the CoreMark package with Ruyi:
```bash
«Ruyi venv-gnu-upstream» root@k1:~/coremark# ruyi extract coremark
warn: this ruyi installation has telemetry mode set to on, and will upload non-tracking usage information to RuyiSDK-managed servers every Thursday
info: the next upload will happen anytime ruyi is executed between 2025-04-24 08:00:00 +0800 and 2025-04-25 08:00:00 +0800
info: in order to hide this banner:
info: - opt out with ruyi telemetry optout
info: - or give consent with ruyi telemetry consent
info: extracting coremark-1.01.tar.gz for package coremark-1.0.1
info: package coremark-1.0.1 extracted to current working directory
```

2. Modify the build configuration to use the RISC-V compiler:
```bash
«Ruyi venv-gnu-upstream» root@k1:~/coremark# sed -i 's/\bgcc\b/riscv64-unknown-linux-gnu-gcc/g' linux64/core_portme.mak
```

3. Build CoreMark:
```bash
«Ruyi venv-gnu-upstream» root@k1:~/coremark# make PORT_DIR=linux64 link
riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2   -lrt"\" -DITERATIONS=0  core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2   -lrt"\" -DITERATIONS=0  core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
Link performed along with compile
```

4. Verify the resulting binary is a RISC-V executable:
```bash
«Ruyi venv-gnu-upstream» root@k1:~/coremark# ls -al
总计 160
drwxr-xr-x  8 root root  4096  4月21日 05:12 .
drwx------ 17 root root  4096  4月21日 05:09 ..
drwxrwxr-x  2 root root  4096 2018年 5月23日 barebones
-rw-rw-r--  1 root root 14651 2018年 5月23日 core_list_join.c
-rw-rw-r--  1 root root 12503 2018年 5月23日 core_main.c
-rwxr-xr-x  1 root root 27000  4月21日 05:12 coremark.exe
-rw-rw-r--  1 root root  4373 2018年 5月23日 coremark.h
-rw-rw-r--  1 root root  8097 2018年 5月23日 core_matrix.c
-rw-rw-r--  1 root root  7186 2018年 5月23日 core_state.c
-rw-rw-r--  1 root root  5171 2018年 5月23日 core_util.c
drwxrwxr-x  2 root root  4096 2018年 5月23日 cygwin
drwxrwxr-x  3 root root  4096 2018年 5月23日 docs
-rw-rw-r--  1 root root  9416 2018年 5月23日 LICENSE.md
drwxrwxr-x  2 root root  4096 2018年 5月23日 linux
drwxrwxr-x  2 root root  4096  4月21日 05:11 linux64
-rw-rw-r--  1 root root  3678 2018年 5月23日 Makefile
-rw-rw-r--  1 root root 18799 2018年 5月23日 README.md
drwxrwxr-x  2 root root  4096 2018年 5月23日 simple
«Ruyi venv-gnu-upstream» root@k1:~/coremark# file coremark.exe 
coremark.exe: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-riscv64-lp64d.so.1, BuildID[sha1]=5c8618cf62e0f1f7dd462ba5bddb03479631d0e9, for GNU/Linux 4.15.0, with debug_info, not stripped
```

5. CoreMark score

```bash
«Ruyi venv-gnu-upstream» root@k1:~/coremark# ./coremark.exe 
2K performance run parameters for coremark.
CoreMark Size    : 666
Total ticks      : 19404
Total time (secs): 19.404000
Iterations/Sec   : 5668.934240
Iterations       : 110000
Compiler version : GCC14.2.0
Compiler flags   : -O2   -lrt
Memory location  : Please put data memory location here
                        (e.g. code in flash, data on heap etc)
seedcrc          : 0xe9f5
[0]crclist       : 0xe714
[0]crcmatrix     : 0x1fd7
[0]crcstate      : 0x8e3a
[0]crcfinal      : 0x33ff
Correct operation validated. See readme.txt for run and reporting rules.
CoreMark 1.0 : 5668.934240 / GCC14.2.0 -O2   -lrt / Heap
```

### 4. CoreMark Benchmark (-O2 -march=rv64gcv_zvl256b -mabi=lp64d  -lrt)

**Commands and Results:**

1. Modify the build configuration to use the RISC-V compiler with specific flags:
```bash
«Ruyi venv-gnu-upstream» root@k1:~/coremark# make PORT_DIR=linux64 XCFLAGS="-march=rv64gcv_zvl256b -mabi=lp64d" link
riscv64-unknown-linux-gnu-gcc -O2 -Ilinux64 -I. -DFLAGS_STR=\""-O2 -march=rv64gcv_zvl256b -mabi=lp64d  -lrt"\" -DITERATIONS=0 -march=rv64gcv_zvl256b -mabi=lp64d core_list_join.c core_main.c core_matrix.c core_state.c core_util.c linux64/core_portme.c -o ./coremark.exe -lrt
Link performed along with compile
```

2. Run CoreMark:
```bash
«Ruyi venv-gnu-upstream» root@k1:~/coremark# ./coremark.exe 
2K performance run parameters for coremark.
CoreMark Size    : 666
Total ticks      : 19936
Total time (secs): 19.936000
Iterations/Sec   : 5517.656501
Iterations       : 110000
Compiler version : GCC14.2.0
Compiler flags   : -O2 -march=rv64gcv_zvl256b -mabi=lp64d  -lrt
Memory location  : Please put data memory location here
                        (e.g. code in flash, data on heap etc)
seedcrc          : 0xe9f5
[0]crclist       : 0xe714
[0]crcmatrix     : 0x1fd7
[0]crcstate      : 0x8e3a
[0]crcfinal      : 0x33ff
Correct operation validated. See readme.txt for run and reporting rules.
CoreMark 1.0 : 5517.656501 / GCC14.2.0 -O2 -march=rv64gcv_zvl256b -mabi=lp64d  -lrt / Heap
```

**CoreMark Results Comparison:**

CoreMark is a benchmark used to evaluate embedded processor performance. A higher score indicates better processor performance. The results from two different compiler configurations are summarized in the table below:

| Metric | Default Flags | Vector Extension Flags | Description |
|--------|--------------|------------------------|-------------|
| **Iterations/Sec** | 5668.934240 | 5517.656501 | Number of iterations completed per second (higher is better) |
| **Total ticks** | 19404 | 19936 | Total number of clock cycles |
| **Total time (secs)** | 19.404000 | 19.936000 | Total execution time in seconds |
| **Iterations** | 110000 | 110000 | Total number of iterations performed |
| **Compiler version** | GCC14.2.0 | GCC14.2.0 | Compiler used for the test |
| **Compiler flags** | -O2   -lrt | -O2 -march=rv64gcv_zvl256b -mabi=lp64d  -lrt | Compilation flags used |
| **Memory location** | Heap | Heap | Where data is stored during execution |

These results demonstrate good performance of the SpacemiT K1/M1 (X60) SoC when running CoreMark compiled with the RuyiSDK toolchain (GCC 14.2.0), with slightly better performance using the default optimization flags compared to the vector extension flags in this particular test case.

## Test Summary

The following table summarizes the test results for GNU Upstream Toolchain on SpacemiT K1/M1 (X60):

| Test Case | Expected Result | Actual Result | Status |
|-----------|----------------|---------------|--------|
| **Toolchain Installation** | Successfully installed toolchain | Installed to `/root/.local/share/ruyi/binaries/riscv64/gnu-upstream-0.20250401.0` | ✅ PASS |
| **Compiler Verification** | GCC 14.2.0 for RISC-V architecture | GCC 14.2.0 with rv64gc architecture, lp64d ABI | ✅ PASS |
| **Hello World Test** | Successful compilation and execution | Successfully compiled and executed | ✅ PASS |
| **CoreMark Benchmark (Default)** | Successfully compile and run benchmark | Successfully compiled and completed benchmark with score of 5668.934240 | ✅ PASS |
| **CoreMark Benchmark (Vector Extension)** | Successfully compile and run benchmark with vector flags | Successfully compiled and completed benchmark with score of 5517.656501 | ✅ PASS |

All tests passed successfully, confirming that the GNU Upstream Toolchain (GCC 14.2.0) works correctly on the SpacemiT K1/M1 (X60) SoC. It's worth noting that enabling the vector extension (`-march=rv64gcv_zvl256b`) did not improve performance in the CoreMark benchmark; in fact, it resulted in a slightly lower score. This suggests that either the CoreMark benchmark code doesn't benefit from vectorization, or the compiler's auto-vectorization capabilities for this specific workload need improvement.