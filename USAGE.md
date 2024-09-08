# 使用说明
## 配置测试
首先在总配置文件 `./config.toml` 中指定需要测试的发行版和软件包，例如：

```toml
distros = ["debian"]
packages = ["apache", "clang", "cmake", "docker", "erlang", "gcc", "gdb", "golang", "haproxy", "libmemcached", "lighttpd", "llvm", "mariadb", "nginx", "nodejs", "numpy", "ocaml"]
```

对于每个发行版，`./<distro>/config.toml` 是它的发行版配置文件，示例如下：
  
```toml
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

发行版目录下，每个软件包对应一个子目录，其中至少各存放一个 `test.sh` 作为测试脚本（一个子目录中可附带多个脚本）。脚本编写可参考 `debian` 目录下的现有示例。

如果有需要在每个测试脚本前全局执行的命令（如 Debian 下为避免交互式安装造成的干扰需添加 `export DEBIAN_FRONTEND=noninteractive` 环境变量），可在发行版目录下的 `prerequisite.sh` 中指定。

## 运行
附加任意命令行参数，将按附加的参数增量执行对应功能。

`--test` 参数将执行全部发行版的测试。

`--aggr` 参数将使复数个 report.json 聚合为 reports.json。

`--summ` 参数将执行生成结果操作。

可使用 RUST_LOG=(debug, warn, info, error) 环境变量指定日志输出等级（包括ssh连接日志），默认为 `info`。

### 例：运行全部测试并生成结果汇总

参考上文配置好测试后运行

```bash
cargo run -- --test --aggr --summ
```

将在发行版目录下的每个软件包子目录中各生成一个 report.json 作为该软件包的测试结果，并在当前目录生成聚合后的总报告 reports.json 和 summary.md。

## 全部命令行参数

```sh
cargo run -- --help
```

```sh
Usage: lintestor [OPTIONS]

Options:
      --test                       Run tests for all distributions
      --aggr                       Aggregate multiple report.json files into a single reports.json
      --summ                       Generate a summary report
      --config <Config file name>  Specify a different base configuration file
  -h, --help                       Print help
  -V, --version                    Print version
```