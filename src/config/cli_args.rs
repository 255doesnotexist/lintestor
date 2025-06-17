use clap::Parser;
use std::path::PathBuf;

// Lintestor - 执行和管理嵌入在Markdown文件中的测试
#[derive(Parser, Debug)]
#[clap(
    name = "lintestor",
    version,
    about = "Execute and manage tests embedded in Markdown files",
    override_usage = "lintestor [OPTIONS] { --test | --parse-only }\n       lintestor --test [TEST_OPTIONS]\n       lintestor --parse-only [PARSE_OPTIONS]",
    after_help = "EXECUTION MODES:\n  --test                 Execute test templates\n  --parse-only           Parse templates without execution\n\nENVIRONMENT TYPES:\n  --local                Execute in local environment\n  --remote               Execute on remote target via SSH\n  --qemu                 Execute in QEMU virtual machine\n  --serial               Execute via serial connection\n\nFILTER OPTIONS:\n  --unit <NAME>          Filter tests by unit name\n  --tag <TAG>            Filter tests by tag\n  --target <FILE>        Use specific target configuration\n\nEXAMPLES:\n  lintestor --test --template T.test.md\n  lintestor --test --test-dir tests/ --local\n  lintestor --test --remote --target prod.toml --unit integration\n  lintestor --parse-only --template test.md\n  lintestor --test --qemu --continue-on-error --timeout 600"
)]
pub struct CliArgs {
    // Run tests - Execute commands in test templates
    // 运行测试 - 执行测试模板中的命令
    #[clap(short = 't', long = "test", help = "Execute test templates")]
    pub test: bool,

    // Parse only - Parse test templates without executing commands
    // 仅解析 - 只解析测试模板但不执行命令
    #[clap(short = 'p', long = "parse-only", help = "Parse templates without execution")]
    pub parse_only: bool,

    // Verbose mode - Show more log information
    // 详细模式 - 显示更多日志信息
    #[clap(short = 'v', long = "verbose", help = "Enable verbose logging")]
    pub verbose: bool,

    // Quiet mode - Suppress prompts and progress information
    // 安静模式 - 不显示提示和进度信息
    #[clap(short = 'q', long = "quiet", help = "Suppress non-essential output")]
    pub quiet: bool,

    // Local test mode - Execute tests in local environment
    // 本地测试模式 - 在本地环境中执行测试
    #[clap(long = "local", help = "Execute in local environment")]
    pub local: bool,

    // Remote test mode - Execute tests in remote environment
    // 远程测试模式 - 在远程环境中执行测试
    #[clap(long = "remote", help = "Execute on remote target via SSH")]
    pub remote: bool,

    // QEMU test mode - Execute tests in QEMU environment
    // QEMU测试模式 - 在QEMU环境中执行测试
    #[clap(long = "qemu", help = "Execute in QEMU virtual machine")]
    pub qemu: bool,

    // Serial test mode - Execute tests via serial connection
    // 串口测试模式 - 通过串口执行测试
    #[clap(long = "serial", help = "Execute via serial connection")]
    pub serial: bool,

    // Test template path - Specify path to a single test template
    // 测试模板路径 - 指定单一测试模板的路径
    #[clap(long = "template", help = "Path to test template file")]
    pub template: Option<PathBuf>,

    // Test directory - Specify directory containing test templates
    // 测试目录 - 指定包含测试模板的目录
    #[clap(short = 'D', long = "test-dir", help = "Directory containing test templates")]
    pub test_dir: Option<PathBuf>,

    // Reports directory - Specify directory for test reports
    // 报告目录 - 指定存放测试报告的目录
    #[clap(long = "reports-dir", help = "Output directory for test reports")]
    pub reports_dir: Option<PathBuf>,

    // Aggregate report output - Specify output file path for aggregate report
    // 聚合报告输出 - 指定聚合报告的输出文件路径
    #[clap(long = "output", short = 'o', help = "Output file for aggregate report")]
    pub output: Option<PathBuf>,

    // Unit name - Filter tests by unit name
    // 单元名称 - 通过单元名称筛选测试
    #[clap(long = "unit", help = "Filter tests by unit name")]
    pub unit: Option<String>,

    // Tag - Filter tests by tag
    // 标签 - 通过标签筛选测试
    #[clap(long = "tag", help = "Filter tests by tag")]
    pub tag: Option<String>,

    // Target config file - Specify target configuration file path
    // 目标配置文件 - 指定目标配置文件路径
    #[clap(long = "target", help = "Target configuration file")]
    pub target: Option<PathBuf>,

    // Continue on error - Continue executing remaining tests even if some fail
    // 出错继续 - 即使测试失败也继续执行其余测试
    #[clap(long = "continue-on-error", default_value = "false", help = "Continue on test failures")]
    pub continue_on_error: Option<bool>,

    // Command execution timeout (seconds)
    // 执行命令超时时间（秒）
    #[clap(long = "timeout", default_value = "300", help = "Command timeout in seconds")]
    pub timeout: Option<u64>,

    // Number of retries after command failure
    // 命令失败后重试次数
    #[clap(long = "retry", default_value = "3", help = "Number of retries on failure")]
    pub retry: Option<u32>,

    // Retry interval (seconds)
    // 重试间隔时间（秒）
    #[clap(long = "retry-interval", default_value = "5", help = "Retry interval in seconds")]
    pub retry_interval: Option<u64>,

    // Maintain session connection
    // 保持会话连接
    #[clap(long = "maintain-session", default_value = "true", help = "Keep session alive between commands")]
    pub maintain_session: Option<bool>,

    // Keep template directory structure when outputting reports
    // 输出报告时保持模板的原始目录结构
    #[clap(short = 'k', long, default_value = "true", help = "Preserve directory structure in reports")]
    pub keep_template_directory_structure: bool,
}

impl CliArgs {
    /// Parse command line arguments
    /// 解析命令行参数
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Get environment type
    /// 获取测试环境类型
    pub fn get_environment_type(&self) -> Option<String> {
        if self.serial {
            Some("serial".to_string())
        } else if self.local {
            Some("local".to_string())
        } else if self.remote {
            Some("remote".to_string())
        } else if self.qemu {
            Some("qemu".to_string())
        } else {
            None
        }
    }

    /// Get log level
    /// 获取日志级别
    pub fn get_log_level(&self) -> &str {
        if self.quiet {
            "error"
        } else if self.verbose {
            "debug"
        } else {
            "info"
        }
    }

    /// Get filter conditions
    /// 获取筛选条件
    pub fn get_filters(&self) -> (Option<&str>, Option<&str>, Option<&str>) {
        (
            self.unit.as_deref(),
            self.tag.as_deref(),
            self.target.as_deref().and_then(|p| p.to_str()),
        )
    }
}
