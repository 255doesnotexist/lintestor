# Lintestor

[Docs](https://255doesnotexist.github.io/lintestor/) | [Release](about:blank) | [Summary](https://github.com/255doesnotexist/lintestor/blob/main/summary.md)

Lintestor 是一个基于 Rust 的自动化测试系统，支持多发行版（只是设计上）和多个软件包的自动化测试。

## 功能

- 管理不同发行版测试环境的启停
- 调度测试任务并执行
- 汇总测试报告
- 生成 Markdown 格式的测试结果总结

## 使用方法

附加任意命令行参数，将按附加的参数增量执行对应功能。

`--test` 参数将执行全部发行版的测试。

`--aggr` 参数将使复数个 report.json 聚合为 reports.json。

`--summ` 参数将执行生成结果操作。

~~`--locally` 参数将仅在本地运行测试，不启动 QEMU。~~
（此参数已弃用。）

### 运行全部测试

```bash
cargo run -- --test --aggr --summ
```

这会读取 config.toml，执行其中包含的发行版和包的测试。

示例配置文件如下：

```toml
distros = ["debian"]
packages = ["apache", "clang", "cmake", "docker", "erlang", "gcc", "gdb", "golang", "haproxy", "libmemcached", "lighttpd", "llvm", "mariadb", "nginx", "nodejs", "numpy", "ocaml"]
```

对于每个发行版，./\<distro\>/config.toml 是它的发行版配置文件。

其中存放着：

- 启停它测试环境的必要脚本。
- 这个发行版中需要被跳过测试的包。
- 连接到这个发行版测试环境的方式和参数。

示例发行版配置：

```toml
testing_type = "qemu-based-remote" # 或 "locally"、"remote"
# 在参数为 locally、remote 时不需求 qemu 启动脚本。在 locally 时不需求连接信息。
startup_script = "./debian/start_qemu.sh"
stop_script = "./debian/stop_qemu.sh"
skip_packages = ["docker"]

[connection]
method = "ssh" # 目前仅支持 SSH
ip = "localhost"
port = 2222
username = "root"
password = "root"
```

执行下面的命令，可查看全部命令行参数。

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