# Usage Guide
## Setup Tests
To start, define the distribution name and packages of interest in the main config file `.config.toml`, eg.:

```toml
distros = ["debian"]
packages = ["apache", "clang", "cmake", "docker", "erlang", "gcc", "gdb", "golang", "haproxy", "libmemcached", "lighttpd", "llvm", "mariadb", "nginx", "nodejs", "numpy", "ocaml"]
```

Then specify distro-specifc options in the target distribution's config file `./<distro>/config.toml`:
  
```toml
testing_type = "qemu-based-remote" # or "locally"、"remote"

startup_script = "./debian/start_qemu.sh" # path to QEMU startup script. IGNORED when testing_type is set to "locally" or "remote"
stop_script = "./debian/stop_qemu.sh" # path to QEMU stop script. IGNORED when testing_type is set to "locally" or "remote"
skip_packages = ["docker"] # Skip testing for these packages

[connection] # IGNORED when testing_type is set to "locally". Only SSH is supported at the moment
method = "ssh"
ip = "localhost"
port = 2222
username = "root"
password = "root"
```
Each subdirectory corresponding to a package should contain at least one test script (multiple scripts would be treated as multiple test cases). For writing the respective test scripts, refer to existing ones under the `debian` folder to get you started.

If certain commands need to be run globally prior each test script (eg. `export DEBIAN_FRONTEND=noninteractive` may be used on Debian-based systems to prevent apt interactive prompts), put them in `prerequisite.sh` under the distro directory. 

### Run tests

Configure the tests following the steps above and run

```bash
cargo run -- --test --aggr --summ
```

A `report.json` report would be generated for each package under their respective subfolders. Now that the tests are done, check out the aggregated `reports.json` and the Markdown result matrix `summary.md` in the current directory.

To toggle logging levels, set the `RUST_LOG` environment variable to one of the following: debug, warn, info, error. `info` is the default logging level.
## Full CLI parameters

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