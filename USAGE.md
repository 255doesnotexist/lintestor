# 使用说明
## 配置测试

对于每个发行版，在工作目录（默认为程序所在目录， 可使用 `-D`/`--directory` 参数指定）下分别为其创建一个 `./<distro>` 目录，`./<distro>/config.toml` 是它的发行版配置文件，示例如下：
  
```toml
enabled = true # 启用该发行版的测试；为 false 则该目录将不会被检测到
testing_type = "qemu-based-remote" # 或 "locally"、"remote"
# 指定测试环境类型。在参数为 locally、remote 时不需求 qemu 启动脚本。在 locally 时不需求连接信息。
startup_script = "./debian/start_qemu.sh" # qemu 启动脚本；如果测试环境类型为 locally 或 remote 则无需此项
stop_script = "./debian/stop_qemu.sh" # qemu 停止脚本；如果测试环境类型为 locally 或 remote 则无需此项
skip_packages = ["docker"] # 应跳过测试的包

[connection] # 如果测试环境类型为 locally 则无需此项；目前仅支持 SSH
method = "ssh"
ip = "localhost"
port = 2222
username = "root"
password = "root"
```

发行版目录下，每个软件包对应一个子目录，其中至少各存放一个 `metadata.sh` 存放该软件包对应的元数据。请在其中定义好以下变量：
```
PACKAGE_VERSION="3.30.3" # 软件包版本，可手动指定也可使用命令获取。
# 例：Debian 下获取通过 `dpkg/apt` 安装的软件包的版本
# PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | awk '{print $3}')
PACKAGE_PRETTY_NAME="CMake" # 软件包别名 (pretty 包名；普通包名即子目录名)
PACKAGE_TYPE="Build System" # 软件包类型
PACKAGE_DESCRIPTION="Cross-platform make" # 软件包的简要说明（此项暂时未使用）
```
一个子目录中可附带多个脚本，除上述 `metadata.sh` 外的其他 `.sh` 脚本均将视为测试脚本并运行（一个脚本即一个子测试）。脚本编写可参考 `debian` 目录下的现有示例。

如果有需要在每个测试脚本前全局执行的命令（如 Debian 下为避免交互式安装造成的干扰需添加 `export DEBIAN_FRONTEND=noninteractive` 环境变量），可在发行版目录下的 `prerequisite.sh` 中指定。

## 运行
附加任意命令行参数，将按附加的参数增量执行对应功能。

`--test` 参数将执行全部发行版的测试。

`--aggr` 参数将使复数个 report.json 聚合为 reports.json。

`--summ` 参数将执行生成结果操作。

`--directory` 参数指定测试文件所在的工作目录（如上文所述）。

`--distro` 参数指定要测试的发行版（可选，将覆盖掉总配置文件）；语法形如 `--distro debian` 或 `--distro debian,bianbu,openkylin`。

`--package` 参数指定要测试的软件包（可选，将覆盖掉总配置文件）；语法形如 `--package apache` 或 `--package apache,clang,cmake`。

可使用 `RUST_LOG=(debug, warn, info, error)` 环境变量指定日志输出等级（包括ssh连接日志），默认为 `info`。

### 运行全部测试并生成结果汇总

参考上文配置好测试后运行

```bash
./lintestor --test --aggr --summ
```

或者

```bash
./lintestor -tas
```
将在发行版目录下的每个软件包子目录中各生成一个 report.json 作为该软件包的测试结果，并在当前**工作目录下**生成聚合后的总报告 reports.json 和 summary.md。

## 全部命令行参数

```bash
./lintestor --help
```

```bash
Usage: lintestor [OPTIONS]

Options:
  -t, --test                           Run tests (for all distributions by default)
  -a, --aggr                           Aggregate multiple report.json files into a single reports.json
  -s, --summ                           Generate a summary report
  -D, --directory <working_directory>  Specify working directory with preconfigured test files
  -d, --distro <distro>                Specify distributions to test
  -p, --package <package>              Specify packages to test
      --skip-successful                Skip previous successful tests (instead of overwriting their results)
  -h, --help                           Print help
  -V, --version                        Print version
```