# 使用说明 (v0.2.0+ 草案)

**注意:** 本文档描述的是 `lintestor` v0.2.0 及之后版本的预期工作方式，引入了基于 Markdown 测试模板的全新测试定义和执行流程。旧的基于 `.sh` 脚本的机制已被弃用。

---

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

---

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
```

---

### 2. 编写测试模板 (`.test.md`)

为每个 **Unit** 在每个 **Target** 上的测试创建一个 `.test.md` 文件。建议的存放结构是 `tests/<unit_name>/<target_name>.test.md`。

模板示例 (`tests/my_package/my_qemu_vm.test.md`):

```markdown
---
title: "My Package 功能测试 (在 My QEMU VM 上)"
target_config: "targets/my_qemu_vm/config.toml"
unit_name: "my_package"
unit_version_command: "my_package --version"
tags: ["core", "regression"]
---

# {{ title }}

*   **测试日期:** `{{ execution_date }}`
*   **目标信息:** `{{ target_info }}`
*   **单元版本:** `{{ unit_version }}`

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
| 安装依赖        | {{ install-deps::status.execution }} |
| 运行核心测试    | {{ run-core::status.execution }}     |
| 运行性能测试    | {{ run-perf::status.execution }}     |
```

---

### 4.2 关键语法

- **YAML Front Matter:**
    - `title`: 报告标题。
    - `target_config`: **必需**，指向此模板关联的 Target 配置文件路径。
    - `unit_name`: 被测单元的名称。
    - `unit_version_command`: (可选) 在 Target 上执行以获取 Unit 版本的命令。
    - `tags`: (可选) 用于分类和筛选测试的标签列表。
    - `references`: (可选) 引用其他测试模板文件。
- **Markdown 块属性 (`{...}`):**
    - `id="unique-id"`: 块的唯一标识符，用于依赖、引用等。
    - `exec=true`: 标记代码块为可执行。
    - `description="简短描述"`: 用于摘要表格。
    - `assert.exit_code=0`: 断言命令退出码为 0。
    - `assert.stdout_contains="文本"`: 断言标准输出包含指定文本。
    - `extract.变量名=/regex/`: 从标准输出提取内容到变量。
    - `depends_on=[...]`: 声明依赖。
    - **多断言支持:** 可以使用多个相同类型的断言，所有条件必须同时满足。
- **依赖关系 (`depends_on`)**:
    - 支持标题节ID和代码块ID依赖。
    - 支持跨模板依赖（`namespace::step_id`）。
- **占位符:**
    - `output {ref="command-id"}`: 插入命令输出。
    - `{{ variable_name }}`: 插入变量。
    - `{{ step-id::status.execution }}`: 步骤状态。
- **特殊变量:**
    - `{{ execution_date }}`、`{{ target_info }}`、`{{ unit_version }}` 等。
- **自动摘要:**
    - `{generate_summary=true}` 区域会自动填充详细表格。

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

---

## 依赖管理与自动机匹配机制

### ContentBlock 之间的依赖

Lintestor 采用**代码块级别的依赖管理**，每个 ContentBlock（无论是代码块还是标题节）都可以通过 `depends_on` 属性声明依赖于其他块。依赖可以是：
- 同一模板内的其他代码块或标题节（通过 id 指定）
- 跨模板的步骤（通过 `namespace::step_id` 指定）

依赖关系会被解析为有向图，Lintestor 会自动进行拓扑排序，确保所有依赖被满足后再执行当前块。这样可以灵活地表达复杂的测试流程和前置条件。

#### 依赖继承
- 如果标题节声明了依赖，节下所有代码块会自动继承该依赖。
- 代码块自身声明的依赖优先生效。

#### 执行顺序与并行
- 拓扑排序后，所有无依赖的块可并行执行。
- 已执行过的块不会重复执行。

### 多种 FSM（有限状态机）与自动机匹配

Lintestor 在解析和执行过程中大量使用了 FSM（有限状态机）来处理：
- **变量替换**：采用 Unicode-safe 的 FSM 替换算法，支持 `{{step-id::status.execution}}` 这类多级变量名，避免正则表达式的边界问题。
- **Markdown 块解析**：用 FSM 识别代码块、输出块、YAML front matter 等，保证对嵌套和边界的准确识别。
- **属性提取**：解析 `{id=..., exec=..., ...}` 这类属性时，FSM 能正确处理引号、转义、嵌套等复杂情况。

#### 自动机匹配的优势
- **健壮性**：不会因特殊字符或多字节字符（如中文）导致解析错误。
- **可维护性**：FSM 状态机逻辑清晰，便于扩展和调试。
- **多级变量支持**：变量名可带点（如 `status.assertion.0`），FSM 能完整识别整个变量名，不会误拆分。

#### 典型流程
1. **依赖解析**：构建依赖图，拓扑排序。
2. **块解析**：FSM 识别所有 ContentBlock 及其属性。
3. **变量替换**：FSM 扫描文本，遇到 `{{...}}` 时自动识别变量名并替换。
4. **执行与并行**：根据依赖关系自动调度执行，支持并行。

---

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

- 变量注册和查找均严格区分模板ID、步骤ID和变量名，支持多级变量名（如 status.assertion.0）。
- 变量管理器（VariableManager）负责所有变量的注册、查找和替换，支持 Unicode 安全和多命名空间。
- 变量引用时，命名空间（namespace）会被映射为模板ID，确保跨模板引用的唯一性。

这种标识方式确保了每个变量在全局变量表中的唯一性，避免了命名冲突。

#### 变量引用方式

用户可以使用以下三种方式在模板中引用变量：

1. **完全限定引用**: `{{ namespace::step_id::variable_name }}`
2. **模板级别引用**: `{{ namespace::variable_name }}`
3. **无前缀引用**: `{{ variable_name }}`

- 点表示法（`namespace.step_id.variable_name`）和双冒号表示法（`namespace::step_id::variable_name`）等价，系统自动统一处理。
- 推荐在跨模板或有歧义时使用完全限定引用。

#### 示例

```markdown
## 使用引用模板中的变量

- 基础变量 (模板级别引用): {{ base::version }} 或 {{ base.version }}
- 完全限定引用: {{ base::system-info::kernel_version }} 或 {{ base.system-info.kernel_version }}
- 当前模板变量 (无前缀): {{ score }}
```

---

## 运行测试

使用 `lintestor` 命令执行测试、聚合报告和生成摘要。

```bash
# 运行所有发现的 .test.md 模板定义的测试
./lintestor --test
# 仅运行特定目标的测试
./lintestor --test --target my_qemu_vm
# 仅运行特定单元的测试
./lintestor --test --unit my_package
# 仅运行包含特定标签的测试
./lintestor --test --tag core
# 聚合报告
./lintestor --aggr
# 生成汇总
./lintestor --summ
# 一步到位
./lintestor -tas
```

**执行流程:**

1.  `--test`:
    *   扫描工作目录下的 `tests/` 目录，查找 `.test.md` 文件。
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
4.  `--block-level`: (v0.2.0新增)
    *   启用代码块级别的依赖管理和执行系统。
    *   使用拓扑排序确定最优执行顺序。
    *   可能并行执行没有依赖关系的代码块。
    *   避免重复执行已执行过的代码块。

---

### 测试筛选与环境选择

Lintestor提供了灵活的测试筛选和环境选择机制：

1. **单元筛选 (`--unit`)**: 
   * 只运行指定单元名称的测试
2. **标签筛选 (`--tag`)**:
   * 只运行包含指定标签的测试
3. **环境类型覆盖**:
   * 使用 `--local`, `--remote`, `--qemu`, `--boardtest` 可覆盖测试模板中指定的环境类型
4. **复合筛选**:
   * 可组合使用多个筛选条件进行精确筛选

---

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
  --local/--remote/--qemu/--boardtest  指定环境
  --unit/--tag/--target                筛选
  --block-level                        代码块级依赖
  -h, --help                         Print help
  -V, --version                      Print version
```

---

如需更多帮助，请查阅项目 README 或联系开发者。