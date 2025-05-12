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

enabled = true
testing_type = "qemu-based-remote" # 或 "remote", "boardtest", "locally"

# 连接信息 (仅当 testing_type 为 remote, qemu-*, boardtest 时需要)
[connection]
method = "ssh"
ip = "localhost"
port = 2222
username = "tester"
private_key_path = "~/.ssh/id_rsa_tester" # 或使用 password
jump_hosts = ["jump1.example.com", "user@jump2.example.com:2222"] # 可选，SSH跳板机列表
max_retries = 3     # 可选，连接失败时的最大重试次数，默认为3
connect_timeout_secs = 15  # 可选，连接超时时间（秒），默认为15秒

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
    references:
    - template: "base_test.test.md"
        as: "base"
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

    ```bash {id="run-core" exec=true description="运行核心测试" assert.stdout_contains="All tests passed" extract.pass_rate=/Pass Rate: (\d+)%/ depends_on=["base::setup-env"]}
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
    *   `references`: (可选) 引用其他测试模板文件，格式为 `[{ template_path: "相对路径", namespace: "引用命名空间" }]`。
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
    *   **多断言支持:** 可以使用多个相同类型的断言（例如 `assert.stderr_not_contains="Error" assert.stderr_not_contains="Failed"`），系统将正确处理所有断言，所有条件必须同时满足才能通过测试。
*   **依赖关系 (`depends_on`)**:
    *   在**标题节**上声明: `## 测试步骤 {id="step-id" depends_on=["other-step"]}` - 该部分下的所有代码块都将继承此依赖关系。
    *   在**代码块**上声明: ```bash {id="block-id" depends_on=["other-block"]}``` - 只影响该特定代码块。
    *   **依赖类型**:
        *   可以依赖于**标题节ID**: 意味着依赖该标题下的所有代码块执行成功。
        *   可以依赖于**代码块ID**: 只依赖特定代码块执行成功。
        *   **继承机制**: 如果代码块没有显式指定依赖关系，但所在标题节有依赖，则代码块会继承标题的依赖关系。
        *   **跨模板依赖**: 使用 `namespace::step_id` 格式可以依赖其他模板中的步骤，例如 `depends_on=["base::setup-env"]`。
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
    *   摘要表格现在包含步骤ID、描述、状态、退出码、输出摘要、错误信息等丰富信息，提供更全面的测试执行概览。
    *   即使步骤没有描述，摘要表格也会显示步骤ID作为标识。

测试总结区域会被自动填充为包含更多详细信息的表格：

```markdown
| 步骤ID | 描述 | 状态 | 退出码 | 输出摘要 | 错误信息 |
|--------|------|------|--------|----------|----------|
| install-deps | 安装依赖 | ✅ Pass | 0 | 依赖安装完成。 | - |
| run-core | 运行核心测试 | ✅ Pass | 0 | Pass Rate: 100% | - |
| run-perf | 运行性能测试 | ✅ Pass | 0 | Score: 1234.56 | - |
```

注意：
1. **UTF-8安全处理:** 所有输出处理均支持中文等UTF-8字符，确保不会因字符边界问题发生错误。
2. **断言评估:** 测试步骤的通过与否完全取决于用户定义的断言，而不是stderr是否有内容。即使stderr有错误信息，只要断言全部通过，步骤仍被视为通过。

## 模板引用和变量命名空间

Lintestor v0.2.0 新增了强大的模板引用功能，允许在一个测试模板中引用和重用另一个测试模板中的步骤和变量，大大提高了测试代码的复用能力。

### 模板引用声明

在 YAML Front Matter 中使用 `references` 数组定义对其他模板的引用：

```yaml
---
title: "性能测试模板"
# ...其他元数据...
references: [
  { template_path: "common/base_test.test.md", namespace: "base" },
  { template_path: "common/setup_utils.test.md", namespace: "setup" }
]
---
```

每个引用包含两个关键信息：
- `template_path`: 被引用模板的路径（相对于工作目录）
- `namespace`: 为该引用分配的命名空间，用于在当前模板中引用被引用模板中的步骤和变量

### 代码块级别依赖管理

Lintestor v0.2.0 实现了基于代码块级别的依赖管理系统，相比之前基于文档级别的依赖管理，提供了更精确的控制和更高的执行效率：

1. **精确的依赖关系**：依赖关系精确到代码块级别，一个步骤可以只依赖另一个模板中的特定步骤，而不是整个模板
2. **灵活的执行顺序**：执行顺序完全由依赖关系图决定，而不是模板出现的顺序
3. **高效的并行执行**：没有依赖关系的代码块可以并行执行，提高执行效率
4. **避免重复执行**：已执行过的代码块不会重复执行，即使多个模板引用了相同的步骤

### 跨模板依赖声明

使用 `namespace::step_id` 格式在代码块的 `depends_on` 属性中声明跨模板依赖：

```markdown
```bash {id="performance-test" exec=true depends_on=["base::setup-env", "setup::prepare-tools"]}
# 这个代码块依赖于 base 命名空间中的 setup-env 步骤和 setup 命名空间中的 prepare-tools 步骤
echo "运行性能测试..."
```
```

### 变量命名空间和引用

在 Lintestor 中，变量管理采用了严格的命名空间隔离机制，确保不同模板、不同步骤之间的变量引用准确无误：

#### 变量的内部存储机制

所有变量在内部都使用全限定标识符（Fully Qualified Identifier）进行存储，格式为：

```
模板ID::步骤ID::变量名
```

- **模板ID**: 可以是模板文件名（不含扩展名）或模板唯一标识符
- **步骤ID**: 产生该变量的步骤ID
- **变量名**: 原始变量名称

这种标识方式确保了每个变量在全局变量表中的唯一性，避免了命名冲突。

#### 命名空间是逻辑概念

重要的是，命名空间（如 `base`, `perf` 等）在 Lintestor 中是**纯逻辑概念**，仅用于用户引用变量。内部系统会将命名空间映射到实际的模板ID，然后通过模板ID组织变量存储。

当一个模板被多次引用时（例如模板A以命名空间"base1"引用模板C，模板B以命名空间"base2"引用相同的模板C），变量只会在模板C执行时存储一次，但可以通过不同的命名空间引用访问。

#### 变量引用方式

用户可以使用以下三种方式在模板中引用变量：

1. **完全限定引用**: `{{ namespace::step_id::variable_name }}`
   - 例如：`{{ base::system-info::kernel_version }}`
   - 最精确的引用方式，唯一确定地指定了特定命名空间中特定步骤产生的特定变量

2. **模板级别引用**: `{{ namespace::variable_name }}`
   - 例如：`{{ base::version }}`
   - 引用指定命名空间中的变量，无论它在哪个步骤中产生
   - 系统会自动查找该命名空间中匹配的变量

3. **无前缀引用**: `{{ variable_name }}`
   - 例如：`{{ score }}`
   - 最简洁的引用方式
   - 只在当前上下文中无命名冲突时可用
   - 系统会优先查找当前模板中的变量，如果不存在且全局无命名冲突，则可能查找其他模板

#### 变量查找规则

变量查找遵循"最近优先"原则：

1. 首先在当前代码块内查找
2. 然后在当前步骤内查找
3. 然后在当前模板内查找
4. 最后在全局范围内查找（需确保无命名冲突）

使用命名空间引用时，系统会：
1. 将命名空间映射到对应的模板ID
2. 根据规则构造全限定标识符
3. 在全局变量表中查找该标识符

#### 点表示法与双冒号表示法

Lintestor 同时支持两种变量引用表示法：

- **点表示法**: `{{ namespace.variable_name }}` 或 `{{ namespace.step_id.variable_name }}`
- **双冒号表示法**: `{{ namespace::variable_name }}` 或 `{{ namespace::step_id::variable_name }}`

这两种表示法在系统内部会被统一处理，用户可以选择自己喜欢的方式。

#### 示例

```markdown
## 使用引用模板中的变量

- 基础变量 (模板级别引用): {{ base::version }} 或 {{ base.version }}
- 完全限定引用: {{ base::system-info::kernel_version }} 或 {{ base.system-info.kernel_version }}
- 当前模板变量 (无前缀): {{ score }}
```

这种命名机制确保了变量引用的准确性和隔离性，同时在简单情况下保持了使用的便利性。

## SSH 跳板机功能

`lintestor` 支持通过一系列跳板机（Jump Hosts）连接到远程测试目标。这在目标机器不直接暴露在公网，或者位于多层安全网络环境中时特别有用。

### 配置跳板机

在目标的 `config.toml` 文件中，使用 `jump_hosts` 数组指定跳板机序列：

```toml
[connection]
method = "ssh"
ip = "target-server"  # 最终目标主机
port = 22
username = "tester"   # 最终目标的用户名
private_key_path = "~/.ssh/id_rsa_tester"  # 最终目标的认证方式
jump_hosts = ["jumphost1", "user2@jumphost2:2222", "jump3.example.com"]  # 跳板机序列
max_retries = 3     # 可选，连接失败时的最大重试次数，默认为3
connect_timeout_secs = 15  # 可选，连接超时时间（秒），默认为15秒
```

### 跳板机认证

**重要说明：** 跳板机的认证信息（用户名、密码、密钥等）**不是**在 `config.toml` 中配置的，而是从您系统中的 SSH 客户端配置获取：

1. 系统级的 SSH 配置（`/etc/ssh/ssh_config`）
2. 用户级的 SSH 配置（`~/.ssh/config`）

这意味着：

- 跳板机的主机名、别名、用户名、端口等应该在您的 SSH 配置中预先定义
- 跳板机的密钥应该已经通过 `ssh-agent` 加载或在 SSH 配置中指定
- 系统将遵循您本地 SSH 客户端的所有设置，如 ControlMaster、连接超时等

### 使用别名

您可以在 `jump_hosts` 中使用 SSH 配置文件中定义的主机别名：

```
# 在 ~/.ssh/config 中:
Host jump1
    HostName jump1.example.com
    User jumpuser
    Port 2222
    IdentityFile ~/.ssh/jump_key

# 在 config.toml 中:
jump_hosts = ["jump1"]  # 使用别名即可，系统会解析
```

### 连接流程

当使用跳板机时，`lintestor` 执行以下步骤：

1. 启动系统 SSH 客户端，使用 `-J` 选项指定跳板机链
2. 创建一个从本地随机端口到最终目标的隧道
3. 通过该本地端口使用 SSH 库连接到目标，进行后续操作

### 故障排除

如果跳板机连接失败：

1. 确保所有跳板机在您的 `~/.ssh/config` 中配置正确
2. 使用 `ssh -J jumphost1,user@jumphost2 user@target` 测试您的跳板机链
3. 检查是否需要运行 `ssh-add` 将您的私钥添加到 SSH 代理
4. 将环境变量 `RUST_LOG=debug` 与 lintestor 一起使用，查看详细的连接调试信息

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

# 指定测试环境类型
./lintestor --test --local      # 强制使用本地环境
./lintestor --test --remote     # 强制使用远程SSH环境
./lintestor --test --qemu       # 强制使用QEMU环境
./lintestor --test --boardtest  # 强制使用Boardtest环境

# 组合环境类型与筛选条件
./lintestor --test --local --unit my_package --tag core

# 运行测试后，聚合所有生成的 .report.md 文件
./lintestor --aggr

# 根据聚合后的 reports.json 生成最终的摘要矩阵 summary.md
./lintestor --summ

# 一步完成所有操作
./lintestor --test --aggr --summ
# 或简写
./lintestor -tas

# 使用代码块级别执行模式（更高效）
./lintestor --test --block-level
```

**执行流程:**

1.  `--test`:
    *   扫描工作目录 (`-D` 指定，默认为当前目录) 下的 `tests/` 目录，查找 `.test.md` 文件。
    *   根据 `--target`, `--unit`, `--tag` 参数筛选模板。
    *   对每个选中的模板：
        *   解析模板，构建执行计划（考虑 `depends_on`）。
        *   读取模板元数据中 `target_config` 指向的配置文件，建立与目标的连接。
        *   如果指定了环境类型（`--local`, `--remote`, `--qemu`, `--boardtest`），将覆盖 `target_config` 中的设置。
        *   按计划顺序执行标记为 `{exec=true}` 的命令块，处理断言、提取变量。
        *   执行完成后，在模板文件所在目录下生成对应的 `.report.md` 文件，包含所有执行细节。
2.  `--aggr`:
    *   扫描工作目录下所有生成的 `.report.md` 文件。
    *   提取关键信息（元数据、总体状态等）。
    *   在工作目录下生成（或覆盖）`reports.json` 文件。
3.  `--summ`:
    *   读取 `reports.json` 文件。
    *   在工作目录下生成（或覆盖）`summary.md` 文件，包含 Target x Unit 的测试结果矩阵。
4.  `--block-level`: (v0.2.0新增)
    *   启用代码块级别的依赖管理和执行系统。
    *   使用拓扑排序确定最优执行顺序。
    *   可能并行执行没有依赖关系的代码块。
    *   避免重复执行已执行过的代码块。

### 测试筛选与环境选择

Lintestor提供了灵活的测试筛选和环境选择机制：

1. **单元筛选 (`--unit`)**: 
   * 只运行指定单元名称的测试
   * 单元名称来自测试模板的 `unit_name` 元数据字段
   * 例如: `./lintestor --test --unit libc` 将只运行 `unit_name: "libc"` 的测试模板

2. **标签筛选 (`--tag`)**:
   * 只运行包含指定标签的测试
   * 标签来自测试模板的 `tags` 元数据数组
   * 例如: `./lintestor --test --tag regression` 将只运行 `tags` 包含 "regression" 的测试模板

3. **环境类型覆盖**:
   * 使用 `--local`, `--remote`, `--qemu`, `--boardtest` 可覆盖测试模板中指定的环境类型
   * 这使您可以在不修改测试模板的情况下，在不同的环境中执行相同的测试
   * 例如: `./lintestor --test --local --unit kernel-modules` 将以本地模式运行所有内核模块测试，即便它们的 `target_config` 指定了其他环境

4. **复合筛选**:
   * 可组合使用多个筛选条件进行精确筛选
   * 例如: `./lintestor --test --local --unit core-utils --tag quick`

这些筛选机制在大型测试套件中尤为有用，可以帮助您快速运行特定的测试子集，而不是整个测试套件。

## 命令行参数

```bash
# 获取最新帮助信息
./lintestor --help
```

```bash
Options:
  -t, --test                         运行测试 - 执行测试模板中的命令
  -a, --aggregate                    聚合报告 - 将多个测试报告合并为一个JSON文件
  -s, --summarize                    生成汇总 - 从聚合报告生成Markdown格式汇总报告
  -p, --parse-only                   仅解析 - 只解析测试模板但不执行命令
  -v, --verbose                      详细模式 - 显示更多日志信息
  -q, --quiet                        安静模式 - 不显示提示和进度信息
      --local                        本地测试模式 - 在本地环境中执行测试
      --remote                       远程测试模式 - 在远程环境中执行测试
      --qemu                         QEMU测试模式 - 在QEMU环境中执行测试
      --boardtest                    板测试模式 - 在目标板上执行测试
      --template <TEMPLATE>          测试模板路径 - 指定单一测试模板的路径
  -D, --test-dir <TEST_DIR>          测试目录 - 指定包含测试模板的目录
      --reports-dir <REPORTS_DIR>    报告目录 - 指定存放测试报告的目录
  -o, --output <o>                   聚合报告输出 - 指定聚合报告的输出文件路径
      --reports-json <REPORTS_JSON>  报告JSON路径 - 指定包含报告数据的JSON文件路径
      --summary-path <SUMMARY_PATH>  汇总报告路径 - 指定汇总报告的输出路径
      --report-path <REPORT_PATH>    报告路径 - 指定单一测试报告的输出路径
      --unit <UNIT>                  单元名称 - 通过单元名称筛选测试
      --tag <TAG>                    标签 - 通过标签筛选测试
      --target <TARGET>              目标配置文件 - 指定目标配置文件路径
      --continue-on-error            出错继续 - 即使测试失败也继续执行其余测试
      --timeout <TIMEOUT>            执行命令超时时间（秒） [default: 300]
      --retry <RETRY>                命令失败后重试次数 [default: 3]
      --block-level                  代码块级别执行 - 使用代码块级别的依赖管理和执行系统
      --parallel <PARALLEL>          并行度 - 并行执行的最大代码块数量 [default: 1]
  -h, --help                         Print help
  -V, --version                      Print version
```

**环境变量:**

*   `RUST_LOG=(debug, warn, info, error)`: 控制日志输出级别，默认为 `info`。`debug` 级别会显示详细的 SSH 通信等信息。