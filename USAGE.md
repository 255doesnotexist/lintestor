# 使用说明 (v0.2.0+ 草案)

**注意:** 本文档描述的是 `lintestor` v0.2.0 及之后版本的预期工作方式，引入了基于 Markdown 测试模板的全新测试定义和执行流程。旧的基于 `.sh` 脚本的机制已被弃用。

## 核心概念

`lintestor` 现在围绕 **测试模板 (Test Template)** 进行工作。每个模板是一个 Markdown 文件 (`.test.md`)，它完整地定义了针对特定 **目标 (Target)** 上的特定 **单元 (Unit)** 的测试流程。

*   **目标 (Target):** 指具体的测试执行环境，例如一个 QEMU 虚拟机、一个物理设备或容器。每个 Target 由一个配置文件 (`config.toml`) 定义。
*   **单元 (Unit):** 指被测试的逻辑实体，例如一个软件包、一个库、一项功能或一组 API。
*   **测试模板 (`.test.md`):** 测试的**唯一事实来源 (Single Source of Truth)**。它包含：
    *   **元数据 (Metadata):** 使用 YAML Front Matter 定义，如测试标题、关联的 Target 配置、Unit 名称等。
    *   **描述性文本:** 解释测试目的、步骤和预期结果。
    *   **可执行命令块:** 标记需要在 Target 上执行的 Shell 命令。
    *   **断言 (Assertions):** 定义检查点，用于判断命令执行是否成功（例如，检查退出码、输出内容）。
    *   **数据提取 (Data Extraction):** 从命令输出中提取特定信息（如版本号、性能指标）到变量中。
    *   **依赖关系 (Dependencies):** 定义执行顺序依赖，可以依赖于标题节ID或代码块ID。
    *   **报告占位符:** 指定命令输出、提取的变量和状态信息应插入到最终报告的哪个位置。
*   **测试报告 (`.report.md`):** 执行测试模板后生成的最终 Markdown 文件，包含了所有执行细节、输出和结果状态。

## 配置测试

### 1. 定义目标 (Target)

每个测试目标（如特定的 QEMU 镜像或物理设备）都需要一个配置文件。建议将这些文件存放在工作目录下的 `targets/` 子目录中，例如 `targets/my_qemu_vm/config.toml` 或 `targets/k1_device/config.toml`。

`config.toml` 文件结构与之前类似，但现在它定义的是一个 **Target** 而不是一个发行版：

```toml
# targets/my_qemu_vm/config.toml 示例

# [target_info] # (可选) 关于此目标的基本信息
# name = "My QEMU VM (Debian Sid)"
# description = "QEMU VM for testing on Debian Sid"
# info_command = "uname -a && lsb_release -a" # (可选) 在目标上运行以获取信息的命令

testing_type = "qemu-based-remote" # 或 "remote", "boardtest", "locally"

# QEMU 相关 (仅当 testing_type 为 qemu-* 时需要)
startup_script = "scripts/start_my_qemu.sh" # 启动脚本路径 (相对于工作目录)
stop_script = "scripts/stop_my_qemu.sh"   # 停止脚本路径

# 连接信息 (仅当 testing_type 为 remote, qemu-*, boardtest 时需要)
[connection]
method = "ssh"
ip = "localhost"
port = 2222
username = "tester"
private_key_path = "~/.ssh/id_rsa_tester" # 或使用 password
# ... 其他 SSH 相关选项

[boardtest] # (仅当 testing_type 为 boardtest 时需要)
token = "your_boardtest_token"
api_url = "http://boardtest.example.com:23333/"
board_config = "board_configs/my_board.toml" # Boardtest 服务器上的配置
serial = "device_serial_123"
# ... 其他 Boardtest 相关选项

# [environment] # (可选) 在目标上执行所有命令前设置的环境变量
# VAR1 = "value1"
# DEBIAN_FRONTEND = "noninteractive"
```

### 2. 编写测试模板 (`.test.md`)

为每个 **Unit** 在每个 **Target** 上的测试创建一个 `.test.md` 文件。建议的存放结构是 `tests/<unit_name>/<target_name>.test.md`。

模板示例 (`tests/my_package/my_qemu_vm.test.md`):

```markdown
    ---
    title: "My Package 功能测试 (在 My QEMU VM 上)"
    target_config: "targets/my_qemu_vm/config.toml" # 关联的目标配置文件路径
    unit_name: "my_package"
    unit_version_command: "my_package --version" # 获取 Unit 版本的命令
    tags: ["core", "regression"]
    ---

    # {{ title }}

    *   **测试日期:** `{{ execution_date }}`
    *   **目标信息:** `{{ target_info }}` {# 由 target_config 中的 info_command 获取 #}
    *   **单元版本:** `{{ unit_version }}` {# 由 unit_version_command 获取 #}

    ## 1. 安装依赖 {id="install"}

    安装必要的依赖包。

    ```bash {id="install-deps" exec=true description="安装依赖" assert.exit_code=0}
    sudo apt-get update
    sudo apt-get install -y libdependency1
    echo "依赖安装完成。"
    ```

    **结果:**
    ```output {ref="install-deps"}
    # lintestor 将在此处插入 install-deps 命令的输出
    ```

    ## 2. 执行核心功能测试 {id="core-test" depends_on=["install"]}

    运行核心功能测试脚本。

    ```bash {id="run-core" exec=true description="运行核心测试" assert.stdout_contains="All tests passed" extract.pass_rate=/Pass Rate: (\d+)%/}
    echo "正在运行核心测试..."
    sleep 2
    echo "Pass Rate: 100%"
    echo "All tests passed."
    ```

    **结果:**
    ```output {ref="run-core"}
    # lintestor 将在此处插入 run-core 命令的输出
    ```

    测试通过率: {{ pass_rate }}%

    ## 3. 性能测试 {id="perf-test" depends_on=["install"]}

    运行性能基准测试。

    ```bash {id="run-perf" exec=true description="运行性能测试" assert.exit_code=0 extract.score=/Score: (\d+\.\d+)/}
    echo "正在运行性能测试..."
    sleep 3
    echo "Score: 1234.56"
    ```

    **结果:**
    ```output {ref="run-perf"}
    # lintestor 将在此处插入 run-perf 命令的输出
    ```

    性能得分: {{ score }}

    ## 4. 测试总结 {id="summary" generate_summary=true}

    | 测试步骤描述     | 状态                     |
    |-----------------|--------------------------|
    | 安装依赖        | {{ status.install-deps }} |
    | 运行核心测试    | {{ status.run-core }}     |
    | 运行性能测试    | {{ status.run-perf }}     |

```

**关键语法解释:**

*   **YAML Front Matter:**
    *   `title`: 报告标题。
    *   `target_config`: **必需**，指向此模板关联的 Target 配置文件路径。
    *   `unit_name`: 被测单元的名称。
    *   `unit_version_command`: (可选) 在 Target 上执行以获取 Unit 版本的命令。
    *   `tags`: (可选) 用于分类和筛选测试的标签列表。
*   **Markdown 块属性 (`{...}`)**:
    *   `id="unique-id"`: 块的唯一标识符，用于依赖、引用等。
    *   `exec=true`: 标记代码块为可执行。
    *   `description="简短描述"`: 用于摘要表格。
    *   `assert.exit_code=0`: 断言命令退出码为 0。
    *   `assert.stdout_contains="文本"`: 断言标准输出包含指定文本。
    *   `assert.stdout_not_contains="文本"`: 断言标准输出不包含指定文本。
    *   `assert.stderr_contains="文本"`: 断言标准错误包含指定文本。 
    *   `assert.stderr_not_contains="文本"`: 断言标准错误不包含指定文本。
    *   `assert.stderr_matches=/regex/`: 断言标准错误匹配正则表达式。
    *   `extract.变量名=/regex/`: 从标准输出提取匹配正则表达式捕获组 1 的内容到 `变量名`。
    *   `depends_on=["id1", "id2"]`: 声明此块依赖于 ID 为 `id1` 和 `id2` 的块成功完成。
*   **依赖关系 (`depends_on`)**:
    *   在**标题节**上声明: `## 测试步骤 {id="step-id" depends_on=["other-step"]}` - 该部分下的所有代码块都将继承此依赖关系。
    *   在**代码块**上声明: ```bash {id="block-id" depends_on=["other-block"]}``` - 只影响该特定代码块。
    *   **依赖类型**:
        *   可以依赖于**标题节ID**: 意味着依赖该标题下的所有代码块执行成功。
        *   可以依赖于**代码块ID**: 只依赖特定代码块执行成功。
        *   **继承机制**: 如果代码块没有显式指定依赖关系，但所在标题节有依赖，则代码块会继承标题的依赖关系。
*   **占位符:**
    *   `output {ref="command-id"}`: 标记此处插入 ID 为 `command-id` 的命令的输出。
    *   `{{ variable_name }}`: 插入提取的变量或特殊变量的值。
    *   `{{ status.step-id }}`: 插入 ID 为 `step-id` 的步骤的最终状态图标 (✅/❌/⚠️/❓)。
*   **特殊变量:**
    *   `{{ execution_date }}`: 测试执行日期。
    *   `{{ target_info }}`: 由 `target_config.toml` 中 `info_command` 获取的目标信息。
    *   `{{ unit_version }}`: 由 `unit_version_command` 获取的单元版本。
*   **自动摘要:**
    *   `{generate_summary=true}`: 标记一个（通常是表格）区域，`lintestor` 将根据已执行步骤的状态自动填充。

## 运行测试

使用 `lintestor` 命令执行测试、聚合报告和生成摘要。

```bash
# 运行所有发现的 .test.md 模板定义的测试
./lintestor --test

# 仅运行特定目标的测试
./lintestor --test --target my_qemu_vm # 使用 target 配置文件名 (不含 .toml) 或路径

# 仅运行特定单元的测试
./lintestor --test --unit my_package

# 仅运行包含特定标签的测试
./lintestor --test --tag core

# 组合筛选
./lintestor --test --target k1_device --unit other_package --tag regression

# 运行测试后，聚合所有生成的 .report.md 文件
./lintestor --aggr

# 根据聚合后的 reports.json 生成最终的摘要矩阵 summary.md
./lintestor --summ

# 一步完成所有操作
./lintestor --test --aggr --summ
# 或简写
./lintestor -tas
```

**执行流程:**

1.  `--test`:
    *   扫描工作目录 (`-D` 指定，默认为当前目录) 下的 `tests/` 目录，查找 `.test.md` 文件。
    *   根据 `--target`, `--unit`, `--tag` 参数筛选模板。
    *   对每个选中的模板：
        *   解析模板，构建执行计划（考虑 `depends_on`）。
        *   读取模板元数据中 `target_config` 指向的配置文件，建立与目标的连接。
        *   按计划顺序执行标记为 `{exec=true}` 的命令块，处理断言、提取变量。
        *   执行完成后，在模板文件所在目录下生成对应的 `.report.md` 文件，包含所有执行细节。
2.  `--aggr`:
    *   扫描工作目录下所有生成的 `.report.md` 文件。
    *   提取关键信息（元数据、总体状态等）。
    *   在工作目录下生成（或覆盖）`reports.json` 文件。
3.  `--summ`:
    *   读取 `reports.json` 文件。
    *   在工作目录下生成（或覆盖）`summary.md` 文件，包含 Target x Unit 的测试结果矩阵。

## 命令行参数

```bash
# 获取最新帮助信息
./lintestor --help
```

```bash
Usage: lintestor [OPTIONS]

Options:
  -t, --test                           运行通过 .test.md 模板定义的测试
  -a, --aggr                           聚合所有 .report.md 文件到 reports.json
  -s, --summ                           根据 reports.json 生成 summary.md 摘要报告
  -D, --directory <working_directory>  指定包含 targets/ 和 tests/ 的工作目录
      --target <target>                指定要测试的目标 (可指定多个，逗号分隔)
      --unit <unit>                    指定要测试的单元 (可指定多个，逗号分隔)
      --tag <tag>                      指定要运行的测试标签 (可指定多个，逗号分隔)
      --skip-successful                (待定/可能移除) 跳过先前成功的测试
  -h, --help                           打印帮助信息
  -V, --version                        打印版本信息
```

**环境变量:**

*   `RUST_LOG=(debug, warn, info, error)`: 控制日志输出级别，默认为 `info`。`debug` 级别会显示详细的 SSH 通信等信息。