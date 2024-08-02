# Lintestor

Lintestor 是一个基于 Rust 的自动化测试系统，支持多发行版（只是设计上）和多个软件包的自动化测试。

## 功能

- 管理不同发行版测试环境的启停
- 调度测试任务并执行
- 汇总测试报告
- 生成 Markdown 格式的测试结果总结

## 使用方法

如不附加任何命令行参数，则默认执行全部测试过程。

如附加任意命令行参数，则按附加的参数增量执行对应功能。

`--test` 参数将执行全部发行版的测试。

`--aggr` 参数将使复数个 report.json 聚合为 reports.json。

`--summ` 参数将执行生成结果操作。

### 运行测试

```bash
cargo run
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