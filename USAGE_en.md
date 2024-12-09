# Usage Guide
## Setup Tests
### Distro setup
Each distribution should have all their test files stored in separate subdirectories (`./<distro>`) under the working directory (defaults to the program's CWD; set with the `-D`/`--directory` flag). Specify distro-specifc options in their respective config files (`./<distro>/config.toml`) as follows:
  
```toml
enabled = true # Enable tests for this distribution. Its folder will not be discovered if set to false
testing_type = "qemu-based-remote" # or "locally"、"remote"

startup_script = "./debian/start_qemu.sh" # path to QEMU startup script. IGNORED when testing_type is set to "locally" or "remote"
stop_script = "./debian/stop_qemu.sh" # path to QEMU stop script. IGNORED when testing_type is set to "locally" or "remote"
skip_packages = ["docker"] # Skip testing for these packages

[connection] # Valid for "remote"/"qemu-based-remote". Only SSH is supported at the moment
method = "ssh"
ip = "localhost"
port = 2222
username = "root"
password = "root"

[boardtest] # Valid for "boardtest"
token = "your_boardtest_server_token" # Auth credentials
board_config = "boards/bpif3.toml" # Path to board config TOML file (on the boardtest server)
serial = "sdwirec_alpha" # Serial number for SD Mux device
mi_sdk_enabled = false # Optional: Enable Mi SDK controller
api_url = "http://yourserver:23333/" # API server URL
timeout_secs = 300 # Test timeout in seconds
```

### Package setup
Each subdirectory corresponding to a package should contain at least one `metadata.sh` script for storing the package's metadata to be used in the generated reports. Please define the following variables in the script:
```
PACKAGE_VERSION="3.30.3" # Package version. Either specify manually or fetch with commands
# e.g. get version of a package installed through `dpkg/apt` on Debian-based distros
# PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | awk '{print $3}')
PACKAGE_PRETTY_NAME="CMake" # A "pretty name" for the package (otherwise the name of the subdirectory would be used as package name)
PACKAGE_TYPE="Build System" # Package type
PACKAGE_DESCRIPTION="Cross-platform make" # Brief description of the package (variable currently unused)
```
Any other `.sh` scripts (except `metadata.sh`) in the subdirectory would be run as tests. Each script represents an individual "test case" for the package. For writing the respective test scripts, refer to existing ones under the `debian` folder to get you started.

If certain commands need to be run globally prior each test script (eg. `export DEBIAN_FRONTEND=noninteractive` may be used on Debian-based systems to prevent apt interactive prompts), put them in `prerequisite.sh` under the distro directory. 

## Run tests

Configure the tests following the steps above and run

```bash
./lintestor --test --aggr --summ
```
or simply
```bash
./lintestor -tas
```

A `report.json` report would be generated for each package under their respective subfolders under the program's working directory. Now that the tests are done, check out the aggregated `reports.json` and the Markdown result matrix `summary.md` in the current directory.

To toggle logging levels, set the `RUST_LOG` environment variable to one of the following: debug, warn, info, error. `info` is the default logging level.

### Specify distros or packages to test

Append the `--distro` and the `--package` flag respectively, e.g.:
```bash
--distro debian --package apache
--distro debian,bianbu,openkylin --package apache,clang,cmake
```
This is optional and will override the settings defined in the main config file.

## Full CLI parameters

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