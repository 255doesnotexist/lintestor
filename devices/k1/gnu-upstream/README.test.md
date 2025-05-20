---
title: "SpacemiT K1/M1 (X60) GNU Toolchain (gnu-upstream) Test Report"
target_config: "target/k1.toml"
unit_name: "gnu-upstream"
unit_version: "0.1.0" # This is the version of this test unit itself
tags: ["toolchain", "gcc", "gnu-upstream", "K1"]
---

# {{ metadata.title }}

This report details a series of basic functionality and performance tests conducted on the {{ metadata.target_name }} development board using the Upstream GNU Toolchain (`{{ metadata.unit_name }}`) provided by RuyiSDK. The tests aim to verify the correct installation of the toolchain, its basic compilation and linking capabilities, and its performance on a standard benchmark (CoreMark).

## Environment Information {id="env_info" depends_on=["check-version", "install-toolchain"]}

The following hardware and software environment was used for this test:

### System Information {id="system-info" depends_on=["check-version"]}

*   **Test Date:** `{{ execution_date }}`
*   **Target Configuration:** `target/k1.toml`
*   **Test Unit Name:** `{{ metadata.unit_name }}`
*   **Test Unit Version:** `{{ metadata.unit_version }}`
*   **Installed Toolchain Package:** `{{ install-toolchain::installed_package_name_version }}`
*   **GCC Version (from -v):** `{{ check-version::gcc_version }}`
*   RuyiSDK running on a `{{ metadata.target_description }}`.

### Hardware Information

*   Banana Pi BPI-F3 board
*   SpacemiT K1/M1 SoC (RISC-V SpacemiT X60 core)

## Installation {id="installation"}

This section documents the process of installing and setting up the Upstream GNU Toolchain using RuyiSDK.

### 0. Initialize Ruyi Package Manager {id="init"}

Clean up any potentially existing old environments and download/prepare the Ruyi CLI tool.

```bash {id="init-ruyi" exec=true description="Download and prepare RuyiSDK CLI" assert.exit_code=0}
rm -rf ~/venv-gnu-upstream ~/ruyi /tmp/coremark_* ~/.local/share/ruyi/
curl -Lo ~/ruyi https://mirror.iscas.ac.cn/ruyisdk/ruyi/tags/0.33.0/ruyi.riscv64
chmod +x ~/ruyi
echo "Ruyi CLI downloaded and prepared. Old directories cleaned."
```

**Command Output:**
```output {ref="init-ruyi" stream="both"}
# lintestor will insert the stdout and stderr of the init-ruyi command here
```
Ruyi CLI Initialization Status (based on `assert.exit_code`): {{ init-ruyi::status.assertion.0 }}

### 1. Install Toolchain {id="install"}

Install the Upstream GNU Toolchain package using Ruyi.

```bash {id="install-toolchain" exec=true description="Install Upstream GNU Toolchain" assert.exit_code=0 depends_on=["init-ruyi"] extract.installed_package_path=/\s*info: package [\S.-]+ installed to (\S+)/ extract.installed_package_name_version=/\s*info: package ([\S.-]+) installed to/}
~/ruyi install toolchain/gnu-upstream
```

**Command Output:**
```output {ref="install-toolchain" stream="both"}
# lintestor will insert the stdout and stderr of the install-toolchain command here
```

This command downloaded the toolchain package `{{ install-toolchain::installed_package_name_version }}` (approx. 161MB) from the configured mirror and installed it into Ruyi's local repository.
Actual installation path: `{{ install-toolchain::installed_package_path }}`.
Installation Status (based on `assert.exit_code`): {{ install-toolchain::status.assertion.0 }}

### 2. Create Virtual Environment {id="create-env"}

Create an isolated virtual environment for this test.

```bash {id="create-venv" exec=true description="Create virtual environment" assert.exit_code=0 depends_on=["install-toolchain"] extract.venv_path=/Creating a Ruyi virtual environment at (\S+)\.\.\./}
~/ruyi venv -t toolchain/gnu-upstream generic venv-gnu-upstream
```

**Command Output:**
```output {ref="create-venv" stream="both"}
# lintestor will insert the stdout and stderr of the create-venv command here
```

This step created a virtual environment directory at `{{ create-venv::venv_path }}`.
Creation Status (based on `assert.exit_code`): {{ create-venv::status.assertion.0 }}

**Verify Created Environment Contents:**

```bash {id="verify-venv" exec=true description="Verify virtual environment contents" assert.exit_code=0 assert.stdout_contains="bin" assert.stdout_contains="ruyi-venv.toml" assert.stdout_contains="toolchain.cmake" assert.stdout_contains="riscv64-unknown-linux-gnu-gcc" assert.stdout_contains="ruyi-activate" depends_on=["create-venv"]}
ls ~/venv-gnu-upstream/
ls ~/venv-gnu-upstream/bin/
```

**Command Output:**
```output {ref="verify-venv" stream="both"}
# lintestor will insert the stdout and stderr of the verify-venv command here
```
Environment Contents Verification (based on `assert.stdout_contains`):
- `assert.exit_code=0`: {{ verify-venv::status.assertion.0 }}
- `bin` `ruyi-venv.toml` `toolchain.cmake` `riscv64-unknown-linux-gnu-gcc` `ruyi-activate` exists: {{ verify-venv::status.assertion.1 }}

### 3. Activate Environment {id="activate-env"}

Activate the virtual environment.

```bash {id="activate-venv" exec=true description="Activate virtual environment" assert.exit_code=0 assert.stdout_contains="Environment activated" depends_on=["create-venv"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
echo "Environment activated. Current PATH: $PATH"
```

**Command Output:**
```output {ref="activate-venv" stream="both"}
# lintestor will insert the stdout and stderr of the activate-venv command here
```

**Command Output (Activate Environment):**
```output {ref="activate-venv" stream="both"}
# lintestor will insert the stdout and stderr of the activate-venv command here
```
Environment Activation Status (based on `assert.exit_code` and `assert.stdout_contains`): {{ activate-venv::status.assertion == "Pass" ? "Success" : "Failure" }}

## Tests & Results {id="tests_results"}

### 1. Compiler Version Check {id="compiler-check"}

Check the version information of `riscv64-unknown-linux-gnu-gcc`.

```bash {id="check-version" exec=true description="Check compiler version" assert.exit_code=0 assert.stdout_contains="gcc version" extract.gcc_version=/gcc version ([0-9.]+)/ extract.target_triple=/Target: (\S+)/ extract.configured_arch=/--with-arch=([^ ]+)/ extract.configured_abi=/--with-abi=([^ ]+)/ depends_on=["activate-venv"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
riscv64-unknown-linux-gnu-gcc -v 2>&1
```

**Command Output:**
```output {ref="check-version" stream="both"}
# lintestor will insert the stdout and stderr of the check-version command here
```

**Analysis:**
Exit code check: {{ check-version::status.assertion.0 }}
GCC version check (`assert.stdout_contains="gcc version"`): {{ check-version::status.assertion.1 }}
The output shows detailed GCC version and configuration options:
*   GCC Version: `{{ check-version::gcc_version }}` ({{ check-version::gcc_version == "14.2.0" ? "Matches expected version 14.2.0" : "Does not match expected version 14.2.0 or not extracted" }})
*   Target Triple: `{{ check-version::target_triple }}` ({{ check-version::target_triple == "riscv64-unknown-linux-gnu" ? "Matches expected" : "Does not match expected riscv64-unknown-linux-gnu" }})
*   Configured Arch: `{{ check-version::configured_arch }}` ({{ check-version::configured_arch == "rv64gc" ? "Matches expected rv64gc" : "Does not match expected rv64gc" }})
*   Configured ABI: `{{ check-version::configured_abi }}` ({{ check-version::configured_abi == "lp64d" ? "Matches expected lp64d" : "Does not match expected lp64d" }})

### 2. Hello World Program {id="hello-world"}

Compile and run a simple "Hello, world!" C program.

```bash {id="create-hello" exec=true description="Create Hello World source file" assert.exit_code=0 depends_on=["check-version"]}
cd /tmp
cat > hello.c << 'EOF'
#include <stdio.h>

int main() { printf("Hello, world!\n"); return 0; }
EOF
```

**Command Output (Create source file):**
```output {ref="create-hello" stream="both"}
# lintestor will insert the stdout and stderr of the create-hello command here (cat command usually has no output)
```
Source file `hello.c` creation status (based on `assert.exit_code`): {{ create-hello::status.assertion.0 }}

```bash {id="compile-hello" exec=true description="Compile Hello World program and check ELF format" assert.exit_code=0 assert.stdout_contains="ELF 64-bit LSB executable" assert.stdout_contains="RISC-V" assert.stdout_contains="dynamically linked" extract.elf_details=/hello_upstream: (ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 \(SYSV\), dynamically linked, interpreter \S+ld-linux-riscv64-lp64d\.so\.1)/ extract.elf_interpreter=/interpreter (\S+ld-linux-riscv64-lp64d\.so\.1)/ depends_on=["create-hello"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
cd /tmp
riscv64-unknown-linux-gnu-gcc hello.c -o hello_upstream
file hello_upstream
```

**Command Output (Compile and check ELF format):**
```output {ref="compile-hello" stream="both"}
# lintestor will insert the stdout and stderr of the compile-hello command here
```
**Analysis:**
Compilation and linking status (based on `assert.exit_code`): {{ compile-hello::status.assertion.0 }}.
ELF format checks (based on `assert.stdout_contains`):
- "ELF 64-bit LSB executable", "RISC-V", "dynamically linked": {{ compile-hello::status.assertion }}
Generated `hello_upstream` file type: `{{ compile-hello::elf_details }}`.
Interpreter: `{{ compile-hello::elf_interpreter }}` ({{ compile-hello::elf_interpreter == "/lib/ld-linux-riscv64-lp64d.so.1" ? "Matches expected" : "Does not match expected interpreter" }}).

```bash {id="run-hello" exec=true description="Run Hello World program" assert.exit_code=0 assert.stdout_contains="Hello, world!" depends_on=["compile-hello"]}
cd /tmp
./hello_upstream
```

**Command Output (Run program):**
```output {ref="run-hello" stream="both"}
# lintestor will insert the stdout and stderr of the run-hello command here
```
**Analysis:**
Program execution status (based on `assert.exit_code`): {{ run-hello::status.assertion.0 }}.
Program output: `{{ run-hello::stdout_summary }}`.
Output correctness (based on `assert.stdout`): {{ run-hello::status.assertion.1 == "Pass" ? "Correct (Output matches 'Hello, world!')" : "Incorrect (Output does not match)" }}.

### 3. CoreMark Benchmark (Default Optimizations) {id="coremark-default" depends_on=["check-version"]}

Compile and run CoreMark using default optimization options (`-O2 -lrt`).

**Command (Extract CoreMark - Default):**

```bash {id="extract-coremark-default" exec=true description="Create directory and extract CoreMark package (Default Opt.)" assert.exit_code=0 depends_on=["activate-venv"]}
mkdir -p /tmp/coremark_default
cd /tmp/coremark_default
~/ruyi extract coremark
```

**Command Output (Extract CoreMark - Default):**
```output {ref="extract-coremark-default" stream="both"}
# lintestor will insert the stdout and stderr of the extract-coremark-default command here
```
CoreMark (Default) extraction status (based on `assert.exit_code`): {{ extract-coremark-default::status.assertion.0 }}.

**Command (Configure Makefile - Default):**

```bash {id="config-coremark-default" exec=true description="Configure CoreMark Makefile (Default Opt.)" assert.exit_code=0 workdir="/tmp/coremark_default" depends_on=["extract-coremark-default"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
cd /tmp/coremark_default
sed -i 's/\bgcc\b/riscv64-unknown-linux-gnu-gcc/g' linux64/core_portme.mak
```

**Command Output (Configure Makefile - Default):**
```output {ref="config-coremark-default" stream="both"}
# lintestor will insert the stdout and stderr of the config-coremark-default command here
```
Makefile (Default) configuration status (based on `assert.exit_code`): {{ config-coremark-default::status.assertion.0 }}.

**Command (Compile CoreMark - Default Optimizations):**

```bash {id="build-coremark-default" exec=true description="Compile CoreMark (Default Opt. -O2 -lrt)" assert.exit_code=0 assert.stdout_contains="coremark.exe: ELF 64-bit LSB executable" workdir="/tmp/coremark_default" depends_on=["config-coremark-default"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
cd /tmp/coremark_default
make PORT_DIR=linux64 link
file coremark.exe
```

**Command Output (Compile CoreMark - Default Optimizations):**
```output {ref="build-coremark-default" stream="both"}
# lintestor will insert the stdout and stderr of the build-coremark-default command here
```
CoreMark (Default Opt.) compilation status (based on `assert.exit_code` and `assert.stdout_contains`): {{ build-coremark-default::status.assertion == "Pass" ? "✅ PASS" : "❌ FAIL" }}.

**Command (Run CoreMark - Default Optimizations):**

```bash {id="run-coremark-default" exec=true description="Run CoreMark (Default Opt.)" assert.exit_code=0 assert.stdout_contains="CoreMark 1.0" extract.coremark_score=/Iterations\/Sec\s+:\s+([0-9.]+)/ extract.cm_total_ticks=/Total ticks\s+:\s+([0-9]+)/ extract.cm_total_time_secs=/Total time \(secs\):\s+([0-9.]+)/ extract.cm_iterations=/Iterations\s+:\s+([0-9]+)/ extract.cm_compiler_version_reported=/Compiler version\s+:\s+(GCC[0-9.]+)/ extract.cm_compiler_flags_reported=/Compiler flags\s+:\s+(.+)/ workdir="/tmp/coremark_default" depends_on=["build-coremark-default"]}
cd /tmp/coremark_default
./coremark.exe
```

**Command Output (Run CoreMark - Default Optimizations):**
```output {ref="run-coremark-default" stream="both"}
# lintestor will insert the stdout and stderr of the run-coremark-default command here
```
**Results (Default Optimizations):**
CoreMark run status (based on `assert.stdout_contains="CoreMark 1.0"`): {{ run-coremark-default::status.assertion.1 }}
CoreMark Score (Iterations/Sec): `{{ run-coremark-default::coremark_score }}`
Reported Compiler Version: `{{ run-coremark-default::cm_compiler_version_reported }}` ({{ run-coremark-default::cm_compiler_version_reported == check-version::gcc_version ? "Matches -v" : "Does not match -v or not extracted" }})
Reported Compiler Flags: `{{ run-coremark-default::cm_compiler_flags_reported }}`

### 4. CoreMark Benchmark (Vector Extension Optimizations) {id="coremark-vector"}

Compile and run CoreMark using `-march=rv64gcv_zvl256b -mabi=lp64d`.

**Command (Extract CoreMark - Vector):**

```bash {id="extract-coremark-vector" exec=true description="Create directory and extract CoreMark package (Vector Opt.)" assert.exit_code=0 depends_on=["activate-venv"]}
mkdir -p /tmp/coremark_vector
cd /tmp/coremark_vector
~/ruyi extract coremark
```

**Command Output (Extract CoreMark - Vector):**
```output {ref="extract-coremark-vector" stream="both"}
# lintestor will insert the stdout and stderr of the extract-coremark-vector command here
```
CoreMark (Vector) extraction status (based on `assert.exit_code`): {{ extract-coremark-vector::status.assertion.0 }}.

**Command (Configure Makefile - Vector):**

```bash {id="config-coremark-vector" exec=true description="Configure CoreMark Makefile (Vector Opt.)" assert.exit_code=0 workdir="/tmp/coremark_vector" depends_on=["extract-coremark-vector"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
cd /tmp/coremark_vector
sed -i 's/\bgcc\b/riscv64-unknown-linux-gnu-gcc/g' linux64/core_portme.mak
```

**Command Output (Configure Makefile - Vector):**
```output {ref="config-coremark-vector" stream="both"}
# lintestor will insert the stdout and stderr of the config-coremark-vector command here
```
Makefile (Vector) configuration status (based on `assert.exit_code`): {{ config-coremark-vector::status.assertion.0 }}.

**Command (Compile CoreMark - Vector Optimizations):**

```bash {id="build-coremark-vector" exec=true description="Compile CoreMark (Vector Extension Opt.)" assert.exit_code=0 assert.stdout_contains="coremark.exe: ELF 64-bit LSB executable" workdir="/tmp/coremark_vector" depends_on=["config-coremark-vector"]}
. ~/venv-gnu-upstream/bin/ruyi-activate
cd /tmp/coremark_vector
make PORT_DIR=linux64 XCFLAGS="-march=rv64gcv_zvl256b -mabi=lp64d" link
file coremark.exe
```

**Command Output (Compile CoreMark - Vector Optimizations):**
```output {ref="build-coremark-vector" stream="both"}
# lintestor will insert the stdout and stderr of the build-coremark-vector command here
```
CoreMark (Vector Opt.) compilation status (based on `assert.exit_code` and `assert.stdout_contains`): {{ build-coremark-vector::status.assertion == "Pass" ? "✅ PASS" : "❌ FAIL" }}.

**Command (Run CoreMark - Vector Optimizations):**

```bash {id="run-coremark-vector" exec=true description="Run CoreMark (Vector Extension Opt.)" assert.exit_code=0 assert.stdout_contains="CoreMark 1.0" extract.coremark_vector_score=/Iterations\/Sec\s+:\s+([0-9.]+)/ extract.cm_vec_total_ticks=/Total ticks\s+:\s+([0-9]+)/ extract.cm_vec_total_time_secs=/Total time \(secs\):\s+([0-9.]+)/ extract.cm_vec_iterations=/Iterations\s+:\s+([0-9]+)/ extract.cm_vec_compiler_version_reported=/Compiler version\s+:\s+(GCC[0-9.]+)/ extract.cm_vec_compiler_flags_reported=/Compiler flags\s+:\s+(.+)/ workdir="/tmp/coremark_vector" depends_on=["build-coremark-vector"]}
cd /tmp/coremark_vector
./coremark.exe
```

**Command Output (Run CoreMark - Vector Optimizations):**
```output {ref="run-coremark-vector" stream="both"}
# lintestor will insert the stdout and stderr of the run-coremark-vector command here
```
**Results (Vector Optimizations):**
CoreMark run status (based on `assert.stdout_contains="CoreMark 1.0"`): {{ run-coremark-vector::status.assertion.1 }}
CoreMark Score (Iterations/Sec): `{{ run-coremark-vector::coremark_vector_score }}`
Reported Compiler Version: `{{ run-coremark-vector::cm_vec_compiler_version_reported }}`
Reported Compiler Flags: `{{ run-coremark-vector::cm_vec_compiler_flags_reported }}`

## Performance Comparison {id="performance" depends_on=["run-coremark-default", "run-coremark-vector", "check-version"]}

Higher CoreMark scores (Iterations/Sec) indicate better performance.

| Metric                        | Default Optimizations                               | Vector Extension Optimizations|
|-------------------------------|-----------------------------------------------------|-----------------------------------------------------------------------|
| **Iterations/Sec**            | `{{ run-coremark-default::coremark_score }}`        | `{{ run-coremark-vector::coremark_vector_score }}`          |
| **Total ticks**               | `{{ run-coremark-default::cm_total_ticks }}`        | `{{ run-coremark-vector::cm_vec_total_ticks }}`           |
| **Total time (secs)**         | `{{ run-coremark-default::cm_total_time_secs }}`  | `{{ run-coremark-vector::cm_vec_total_time_secs }}`     |
| **Iterations**                | `{{ run-coremark-default::cm_iterations }}`         | `{{ run-coremark-vector::cm_vec_iterations }}`            |
| **Compiler (Reported)**       | `{{ run-coremark-default::cm_compiler_version_reported }}` | `{{ run-coremark-vector::cm_vec_compiler_version_reported }}` |
| **Compiler Flags (Reported)** | `{{ run-coremark-default::cm_compiler_flags_reported }}` | `{{ run-coremark-vector::cm_vec_compiler_flags_reported }}` |
| **Compiler (from -v)**        | `{{ check-version::gcc_version }}`                  | `{{ check-version::gcc_version }}`                                    |

**Performance Analysis:**
In this test, CoreMark on the SpacemiT K1/M1 (X60) SoC using GCC `{{ check-version::gcc_version }}` achieved scores of `{{ run-coremark-default::coremark_score }}` (Default Optimizations) and `{{ run-coremark-vector::coremark_vector_score }}` (Vector Extension Optimizations).
Further analysis of these scores can reveal the specific impact of vector extensions for the CoreMark workload with this compiler version.

## Test Summary {id="summary" depends_on=["check-version"]}

The following table, automatically generated by `lintestor`, summarizes the execution status of each step:

| Step ID                 | Description                                       | Status                                 | Exit Code                         | Stdout Summary                           | Stderr Summary                           |
|-------------------------|---------------------------------------------------|----------------------------------------|-----------------------------------|------------------------------------------|------------------------------------------|
| init-ruyi               | Download and prepare RuyiSDK CLI                  | {{ init-ruyi::status.execution }}      | {{ init-ruyi::exit_code }}        | {{ init-ruyi::stdout_summary }}          | {{ init-ruyi::stderr_summary }}          |
| install-toolchain       | Install Upstream GNU Toolchain                    | {{ install-toolchain::status.execution }} | {{ install-toolchain::exit_code }} | {{ install-toolchain::stdout_summary }}  | {{ install-toolchain::stderr_summary }}  |
| create-venv             | Create virtual environment                        | {{ create-venv::status.execution }}    | {{ create-venv::exit_code }}       | {{ create-venv::stdout_summary }}        | {{ create-venv::stderr_summary }}        |
| verify-venv             | Verify virtual environment contents               | {{ verify-venv::status.execution }}    | {{ verify-venv::exit_code }}       | {{ verify-venv::stdout_summary }}        | {{ verify-venv::stderr_summary }}        |
| activate-venv           | Activate virtual environment                      | {{ activate-venv::status.execution }}  | {{ activate-venv::exit_code }}     | {{ activate-venv::stdout_summary }}      | {{ activate-venv::stderr_summary }}      |
| check-version           | Check compiler version                            | {{ check-version::status.execution }}  | {{ check-version::exit_code }}     | {{ check-version::stdout_summary }}      | {{ check-version::stderr_summary }}      |
| create-hello            | Create Hello World source file                    | {{ create-hello::status.execution }}   | {{ create-hello::exit_code }}      | {{ create-hello::stdout_summary }}       | {{ create-hello::stderr_summary }}       |
| compile-hello           | Compile Hello World program and check ELF format  | {{ compile-hello::status.execution }}  | {{ compile-hello::exit_code }}     | {{ compile-hello::stdout_summary }}      | {{ compile-hello::stderr_summary }}      |
| run-hello               | Run Hello World program                           | {{ run-hello::status.execution }}      | {{ run-hello::exit_code }}         | {{ run-hello::stdout_summary }}          | {{ run-hello::stderr_summary }}          |
| extract-coremark-default| Create directory and extract CoreMark (Default Opt.)| {{ extract-coremark-default::status.execution }} | {{ extract-coremark-default::exit_code }} | {{ extract-coremark-default::stdout_summary }} | {{ extract-coremark-default::stderr_summary }} |
| config-coremark-default | Configure CoreMark Makefile (Default Opt.)        | {{ config-coremark-default::status.execution }} | {{ config-coremark-default::exit_code }} | {{ config-coremark-default::stdout_summary }} | {{ config-coremark-default::stderr_summary }} |
| build-coremark-default  | Compile CoreMark (Default Opt. -O2 -lrt)          | {{ build-coremark-default::status.execution }} | {{ build-coremark-default::exit_code }} | {{ build-coremark-default::stdout_summary }} | {{ build-coremark-default::stderr_summary }} |
| run-coremark-default    | Run CoreMark (Default Opt.)                       | {{ run-coremark-default::status.execution }} | {{ run-coremark-default::exit_code }} | {{ run-coremark-default::stdout_summary }} | {{ run-coremark-default::stderr_summary }} |
| extract-coremark-vector | Create directory and extract CoreMark (Vector Opt.) | {{ extract-coremark-vector::status.execution }} | {{ extract-coremark-vector::exit_code }} | {{ extract-coremark-vector::stdout_summary }} | {{ extract-coremark-vector::stderr_summary }} |
| config-coremark-vector  | Configure CoreMark Makefile (Vector Opt.)         | {{ config-coremark-vector::status.execution }} | {{ config-coremark-vector::exit_code }} | {{ config-coremark-vector::stdout_summary }} | {{ config-coremark-vector::stderr_summary }} |
| build-coremark-vector   | Compile CoreMark (Vector Extension Opt.)          | {{ build-coremark-vector::status.execution }} | {{ build-coremark-vector::exit_code }} | {{ build-coremark-vector::stdout_summary }} | {{ build-coremark-vector::stderr_summary }} |
| run-coremark-vector     | Run CoreMark (Vector Extension Opt.)              | {{ run-coremark-vector::status.execution }} | {{ run-coremark-vector::exit_code }} | {{ run-coremark-vector::stdout_summary }} | {{ run-coremark-vector::stderr_summary }} |

---
**Manual-Style Test Conclusions Overview (based on exit_code and assertion status strings):**

| Test Item | Expected Result                                       | Actual Status (template logic) |
|---------------------------------------------------|-------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Ruyi CLI Initialization (`init-ruyi`)               | Ruyi CLI successfully prepared                        | {{ init-ruyi::status.assertion.0 == "Pass" ? "✅ PASS" : "❌ FAIL" }}    |
| Toolchain Installation (`install-toolchain`)        | Successfully installed                                | {{ install-toolchain::status.assertion.0 == "Pass" ? "✅ PASS" : "❌ FAIL" }} (Path: `{{ install-toolchain::installed_package_path }}`)              |
| Compiler Version Check (`check-version`)            | GCC 14.2.0, riscv64-unknown-linux-gnu, rv64gc, lp64d  | {{ check-version::status.assertion == "Pass"  ? "✅ PASS" : "⚠️  CHECK" }} (Version: `{{ check-version::gcc_version }}`) |
| Hello World Compilation (`compile-hello`)           | Successfully compiled ELF 64-bit                      | {{ compile-hello::status.assertion == "Pass" ? "✅ PASS" : "❌ FAIL" }}                            |
| Hello World Execution (`run-hello`)                 | Outputs "Hello, world!"                               | {{ run-hello::status.assertion == "Pass" ? "✅ PASS" : "❌ FAIL" }} (Output: `{{ run-hello::stdout_summary }}`)                                   |
| CoreMark (Default) Compilation (`build-coremark-default`) | Successfully compiled                                 | {{ build-coremark-default::status.assertion == "Pass" ? "✅ PASS" : "❌ FAIL" }}                              |
| CoreMark (Default) Execution (`run-coremark-default`)     | Successfully runs, "CoreMark 1.0" flag present      | {{ run-coremark-default::status.assertion == "Pass" ? "✅ PASS" : "❌ FAIL" }} (Score: `{{ run-coremark-default::coremark_score }}`)        |
| CoreMark (Vector) Compilation (`build-coremark-vector`)   | Successfully compiled                                 | {{ build-coremark-vector::status.assertion == "Pass" ? "✅ PASS" : "❌ FAIL" }}                                  |
| CoreMark (Vector) Execution (`run-coremark-vector`)       | Successfully runs, "CoreMark 1.0" flag present      | {{ run-coremark-vector::status.assertion == "Pass" ? "✅ PASS" : "❌ FAIL" }} (Score: `{{ run-coremark-vector::coremark_vector_score }}`) |
