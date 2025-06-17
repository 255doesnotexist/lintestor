# Lintestor 使用指南 (v0.2.0+)

Lintestor 使用 Markdown (`.test.md`) 文件作为测试模板。本文档说明了如何配置测试环境和编写测试模板。

## 1. 目标配置 (Target Configuration)

在编写测试模板之前，需要先为测试目标（如QEMU虚拟机、远程服务器等）创建一个 TOML 配置文件。建议将这些文件存放于工作目录下的 `targets/` 子目录中。

**`targets/my_qemu_vm/config.toml` 示例:**
```toml
# testing_type: 定义测试环境类型。
# 可选值: "locally", "remote", "qemu-based-remote", "serial"
testing_type = "remote"

# [connection]: 当 testing_type 为 "remote", "qemu-based-remote", "serial" 时需要。
[connection]
# 对于 "remote" 和 "qemu-based-remote":
method = "ssh"
ip = "localhost"
port = 2222
username = "tester"
# private_key_path = "~/.ssh/id_rsa_tester" # SSH 私钥路径
password = "your_password"               # 或使用密码

# 对于 "serial":
# device = "/dev/ttyUSB0" # 串口设备路径
# baud_rate = 115200      # 波特率

# [executor]: 可选，用于控制命令执行行为。
[executor]
command_timeout = 300  # 命令超时时间（秒），默认 300
retry_count = 1        # 命令失败重试次数（首次执行不计入），默认 1
retry_interval = 5     # 重试间隔（秒），默认 5
maintain_session = true # 是否为同一目标上的连续步骤保持会话（主要用于SSH），默认 true
continue_on_error = false # 步骤失败后是否继续执行模板中其他独立步骤，默认 false
```

---

## 2. 模板结构与元数据 (YAML Front Matter)

一个 `.test.md` 文件由文件顶部的 **YAML Front Matter** (元数据) 和下方的 **Markdown 正文**组成。

元数据区域位于 `---` 分隔符之间，用于定义测试的基本信息。

```yaml
---
# 报告中显示的主标题
title: "示例单元功能测试"

# [必需] 指向目标环境的配置文件，路径相对于工作目录
target_config: "targets/my_qemu_vm/config.toml"

# 被测单元的名称，用于测试筛选
unit_name: "example_unit"

# 标签，用于测试筛选
tags: ["core", "smoke"]

# [可选] 引用其他模板，用于跨文件依赖
references:
  - template: "common/setup.test.md"
    as: "common_setup" # 定义一个命名空间，用于引用该模板中的步骤

# [可选] 自定义字段，可以在模板中作为变量引用
custom_field: "some_value"
---
```

~~不建议使用 YAML 的非引号包裹字符串功能，不然非预期的爆炸是可能发生的。~~

---

## 3. 模板正文 (Markdown Body)

正文由一系列 Markdown 块组成，用于定义测试的具体步骤和报告的最终样式。

### 标题 (Headings)

标题用于组织报告结构，它本身也是依赖管理中的一个步骤。

```markdown
## 1. 安装步骤 {id="install_step"}
```

-   `id`: 为标题步骤提供一个唯一的局部ID，以便其他步骤可以依赖它。
-   `generate_summary=true`: 如果给一个标题添加此属性，Lintestor 会在该标题下方自动生成整个模板的步骤执行摘要表。

### 文本 (Text)

标准 Markdown 文本会直接呈现在报告中。可以在文本中使用 `{{ variable_name }}` 来引用变量。

```markdown
这是一个描述性文本。测试将在目标 `{{ target_info.name }}` 上运行。
```

### 代码块 (Code Blocks)

代码块用于定义要执行的命令、断言和数据提取规则。

````markdown
```bash {id="install_cmd", exec=true, description="安装依赖", assert.exit_code=0}
echo "正在安装依赖..."
# sudo apt-get install -y some-package
echo "依赖安装完成。"
```
````

**关键属性:**

-   `id`: **必需**，步骤的唯一局部ID。
-   `exec=true`: **必需**，标记此代码块为可执行。
-   `description`: [可选] 步骤的描述，将用于报告中。
-   `depends_on`: [可选] 声明依赖关系，值为一个步骤ID的数组。例如: `depends_on=["step1", "common_setup::step2"]`。
-   `visible`: [可选] 默认为 `true`。设为 `false` 可在最终报告中隐藏此代码块本身（但命令仍会执行）。

可以在代码块中使用其他代码块执行后产生的变量，但这个支持是实验性的。建议手动加一下 depends_on，因为代码块中的隐式依赖还有一些问题。

**断言 (Assertions):**

用于验证命令的执行结果是否符合预期。

-   `assert.exit_code=0`: 断言命令的退出码。
-   `assert.stdout_contains="some text"`: 断言标准输出包含指定文本。
-   `assert.stderr_contains="error message"`: 断言标准错误输出包含指定文本。
-   `assert.stdout_not_contains="forbidden text"`: 断言标准输出**不**包含指定文本。
-   `assert.stderr_not_contains="unexpected error"`: 断言标准错误输出**不**包含指定文本。
-   `assert.stderr_not_matches=/pattern/`: 断言标准错误输出**不**匹配指定的正则表达式。
-   `assert.stdout_not_matches=/pattern/`: 断言标准输出**不**匹配指定的正则表达式。


**数据提取 (Data Extraction):**

从命令输出中提取值并存入变量。

-   `extract.my_var=/Result: (\w+)/`: 使用正则表达式从标准输出中提取内容，并将第一个捕获组的值存入名为 `my_var` 的变量中。你可以在后续步骤中通过 `{{ my_var }}` 或 `{{ install_cmd::my_var }}` 来引用它。
- 默认提取的都是字符串因为内部并没有类型系统，只是用了简单的 `HashMap<key, value>` 来存提取的东西。

### 输出块 (Output Blocks)

用于在报告中显式地展示某个已执行命令的输出。

````markdown
**安装结果:**
```output {ref="install_cmd"}
# 此处将自动插入 install_cmd 的输出
```
````

-   `ref`: **必需**，指向要显示输出的步骤的 `id`。
-   `stream`: [可选] 指定要显示的输出流。
    -   `stdout` (默认): 只显示标准输出。
    -   `stderr`: 只显示标准错误。
    -   `both`: 同时显示标准输出和标准错误。目前实现方式是将两个流粗暴地拼接。（注意 serial 目标无法区分两种流因此本选项无效。）

### 摘要表 (Summary Table)

有两种方式在报告中插入所有步骤的执行摘要：

1.  在任意标题上添加 `{generate_summary=true}` 属性。
2.  在 Markdown 的任意位置使用 HTML 注释 `<!-- LINTESOR_SUMMARY_TABLE -->` 作为占位符。

---

## 4. 变量系统

-   **内部存储**: 所有变量都以 `模板ID::步骤ID::变量名` 的形式唯一存储，确保无冲突。
-   **变量引用**: 使用 `{{ variable_name }}` 或 `{{ step_id::variable_name }}`。当存在命名冲突时，建议使用后者以明确指定作用域。
-   **内置变量**:
    -   `execution_date`: 测试执行日期。
    -   `target_info`: 包含目标配置信息的对象 (例如 `{{ target_info.name }}` )。
    -   `unit_version`: 一个写死的测试单元版本号。

---

## 5. 依赖管理

Lintestor 会根据步骤间的依赖关系构建执行顺序。

-   **显式依赖**: 通过在代码块或标题上使用 `depends_on=["step_id"]` 属性来明确声明。
-   **结构依赖**: 父标题会自动依赖其下的所有子步骤（子标题、代码块等），这主要用于保证报告的结构完整性。
-   **隐式依赖**: 如果步骤A的命令中引用了步骤B提取的变量 (例如 `{{ B::my_var }}`), Lintestor 会自动推断出A依赖于B。（实验性的，不要过度信任这个。）

---

## 6. 运行测试

使用 `--test` (或 `-t`) 参数执行测试。

```bash
# 运行当前目录及子目录中的所有 .test.md 文件
./lintestor --test

# 运行指定的模板文件
./lintestor --test --template ./path/to/specific.test.md

# 运行指定目录下的所有模板
./lintestor --test --test-dir ./path/to/tests

# 将报告输出到自定义目录 (默认为 ./reports)
./lintestor --test --reports-dir ./my_custom_reports
```

**筛选测试:**

-   `--target <TARGET_NAME>`: 按目标名称筛选。
-   `--unit <UNIT_NAME>`: 按单元名称 (`unit_name` 元数据) 筛选。
-   `--tag <TAG_NAME>`: 按标签 (`tags` 元数据) 筛选。

```bash
./lintestor --help
```

```text
Execute and manage tests embedded in Markdown files

Usage: lintestor [OPTIONS] { --test | --parse-only }
       lintestor --test [TEST_OPTIONS]
       lintestor --parse-only [PARSE_OPTIONS]

Options:
  -t, --test
          Execute test templates
  -p, --parse-only
          Parse templates without execution
  -v, --verbose
          Enable verbose logging
  -q, --quiet
          Suppress non-essential output
      --local
          Execute in local environment
      --remote
          Execute on remote target via SSH
      --qemu
          Execute in QEMU virtual machine
      --serial
          Execute via serial connection
      --template <TEMPLATE>
          Path to test template file
  -D, --test-dir <TEST_DIR>
          Directory containing test templates
      --reports-dir <REPORTS_DIR>
          Output directory for test reports
  -o, --output <OUTPUT>
          Output file for aggregate report
      --unit <UNIT>
          Filter tests by unit name
      --tag <TAG>
          Filter tests by tag
      --target <TARGET>
          Target configuration file
      --continue-on-error <CONTINUE_ON_ERROR>
          Continue on test failures [default: false] [possible values: true, false]
      --timeout <TIMEOUT>
          Command timeout in seconds [default: 300]
      --retry <RETRY>
          Number of retries on failure [default: 3]
      --retry-interval <RETRY_INTERVAL>
          Retry interval in seconds [default: 5]
      --maintain-session <MAINTAIN_SESSION>
          Keep session alive between commands [default: true] [possible values: true, false]
  -k, --keep-template-directory-structure
          Preserve directory structure in reports
  -h, --help
          Print help
  -V, --version
          Print version

EXECUTION MODES:
  --test                 Execute test templates
  --parse-only           Parse templates without execution

ENVIRONMENT TYPES:
  --local                Execute in local environment
  --remote               Execute on remote target via SSH
  --qemu                 Execute in QEMU virtual machine
  --serial               Execute via serial connection

FILTER OPTIONS:
  --unit <NAME>          Filter tests by unit name
  --tag <TAG>            Filter tests by tag
  --target <FILE>        Use specific target configuration

EXAMPLES:
  lintestor --test --template T.test.md
  lintestor --test --test-dir tests/ --local
  lintestor --test --remote --target prod.toml --unit integration
  lintestor --parse-only --template test.md
  lintestor --test --qemu --continue-on-error --timeout 600
```