# Usage Guide (v0.2.0+)

Lintestor v0.2.0+ uses Markdown (`.test.md`) files as test templates, replacing the old Shell script model.

## Configuring Tests

### 1. Target Configuration

Each test target environment (e.g., QEMU VM, remote server, local environment) requires a TOML configuration file. It is recommended to store these files in a `targets/` subdirectory within the working directory, for example, `targets/my_qemu_vm/config.toml`.

**Example `targets/<target_name>/config.toml`:**
```toml
# testing_type: Defines the type of testing environment.
# Possible values: "locally", "remote", "qemu-based-remote", "serial"
testing_type = "remote"

# [connection]: Required when testing_type is "remote", "qemu-based-remote", or "serial".
[connection]
# For "remote" and "qemu-based-remote":
method = "ssh"
ip = "localhost"
port = 2222
username = "tester"
# private_key_path = "~/.ssh/id_rsa_tester" # Path to SSH private key
password = "your_password"               # Or use a password

# For "serial":
# device = "/dev/ttyUSB0" # Serial device path
# baud_rate = 115200      # Baud rate

# [executor]: Optional, for controlling command execution behavior.
[executor]
command_timeout = 300  # Command timeout in seconds, default 300
retry_count = 1        # Number of retries on command failure (initial execution not counted), default 1
retry_interval = 5     # Retry interval in seconds, default 5
maintain_session = true # Whether to maintain the session for consecutive steps on the same target (mainly for SSH), default true
continue_on_error = false # Whether to continue executing other independent steps in the template after a step fails, default false
```

### 2. Test Template Configuration (`.test.md`)

Create a `.test.md` file for each test of a Unit on a specific Target. The recommended structure is `tests/<unit_name>/<target_name>.test.md`, or place it directly in a scanned path (like the project root, `tests/`, or `templates/`).

**Example Test Template (`example.test.md`):**
```markdown
---
# YAML Front Matter: Defines test metadata
title: "Example Unit Functional Test"
target_config: "targets/my_qemu_vm/config.toml" # **Required**, points to the target configuration file
unit_name: "example_unit"
tags: ["core", "smoke"]
# references: # Optional, reference other templates
#   - template_path: "common/setup.test.md"
#     namespace: "common_setup"
---

# {{ title }}

*   **Test Date:** `{{ execution_date }}`
*   **Target Info:** `{{ target_info }}`
*   **Unit Version:** `{{ unit_version }}`

## 1. Installation Step {id="install_step"}

```bash {id="install_cmd" exec=true description="Install dependencies" assert.exit_code=0}
echo "Installing dependencies..."
# sudo apt-get install -y some-package
echo "Dependencies installed."
```
**Result:**
```output {ref="install_cmd"}
# The output of install_cmd will be automatically inserted here
```

## 2. Functional Test {id="functional_test" depends_on=["install_cmd"]}

```bash {id="run_test_cmd" exec=true description="Run functional test" assert.stdout_contains="Test Passed" extract.value=/Result: (\w+)/}
echo "Running test..."
echo "Test Passed"
echo "Result: Success"
```
**Result:**
```output {ref="run_test_cmd"}
```
Extracted value: `{{ value }}`

## 3. Summary {id="summary_section" generate_summary=true}
<!-- A heading section with generate_summary=true will automatically generate a step summary table -->
<!-- Alternatively, use <!-- LINTESOR_SUMMARY_TABLE --> to manually specify the location -->
```

**Key Template Syntax:**
-   **YAML Front Matter:**
    -   `target_config`: (Required) Path to the target configuration file.
    -   Other optional fields like `title`, `unit_name`, `tags`, `unit_version_command`, `references`.
-   **Markdown Code Block Attributes (`{...}`):**
    -   `id="unique-id"`: Unique ID for the step.
    -   `exec=true`: Marks the block as executable.
    -   `description="Description"`: Used in reports.
    -   `assert.exit_code=0`: Asserts the exit code.
    -   `assert.stdout_contains="text"`: Asserts that standard output contains the given text.
    -   `extract.variable_name=/regex/`: Extracts data from output into a variable.
    -   `depends_on=["id1", "namespace::id2"]`: Declares dependencies.
-   **Variable Reference:** Use `{{ variable_name }}` or `{{ step_id::variable_name }}`.
-   **Output Block:** `output {ref="command_id"}` is used to display command output.

## Running Tests

Use the `--test` (or `-t`) argument to execute tests. Lintestor will automatically discover and execute test templates, and generate reports.

```bash
# Run all tests in the current directory and its subdirectories (tests/, templates/)
./lintestor --test

# Run a specific template file
./lintestor --test --template ./path/to/specific.test.md

# Run test templates in a specific directory
./lintestor --test --test-dir ./path/to/tests

# Output reports to a specific directory (defaults to ./reports)
./lintestor --test --reports-dir ./my_custom_reports

# Keep the original directory structure of templates in the report directory
./lintestor --test --keep-report-structure
```

**Output:**
-   Each successfully executed test template will generate a corresponding `.report.md` file in the reports directory.
-   After all tests are completed, a `summary.test.md.report.md` file will be generated at the root of the reports directory, summarizing all test results.

**Filtering Tests:**
-   `--target <TARGET_NAME>`: Filter by target name (from the `target_config` filename or its internal `name` field).
-   `--unit <UNIT_NAME>`: Filter by unit name (from the `unit_name` in template metadata).
-   `--tag <TAG_NAME>`: Filter by tag (from the `tags` in template metadata).

**Log Level:**
Use `--verbose` to increase log verbosity, or `--quiet` to reduce log output. The default log level is `info`.

## Main Command-line Arguments

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