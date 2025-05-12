use clap::Parser;
use std::path::PathBuf;

/// Lintestor - 一个用于执行和管理嵌入在Markdown中的测试的工具
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct CliArgs {
    /// 运行测试 - 执行测试模板中的命令
    #[clap(short = 't', long = "test")]
    pub test: bool,

    /// 聚合报告 - 将多个测试报告合并为一个JSON文件
    #[clap(short = 'a', long = "aggregate")]
    pub aggregate: bool,

    /// 生成汇总 - 从聚合报告生成Markdown格式汇总报告
    #[clap(short = 's', long = "summarize")]
    pub summarize: bool,

    /// 仅解析 - 只解析测试模板但不执行命令
    #[clap(short = 'p', long = "parse-only")]
    pub parse_only: bool,

    /// 详细模式 - 显示更多日志信息
    #[clap(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// 安静模式 - 不显示提示和进度信息
    #[clap(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// 本地测试模式 - 在本地环境中执行测试
    #[clap(long = "local")]
    pub local: bool,

    /// 远程测试模式 - 在远程环境中执行测试
    #[clap(long = "remote")]
    pub remote: bool,

    /// QEMU测试模式 - 在QEMU环境中执行测试
    #[clap(long = "qemu")]
    pub qemu: bool,

    /// 板测试模式 - 在目标板上执行测试
    #[clap(long = "boardtest")]
    pub boardtest: bool,

    /// 测试模板路径 - 指定单一测试模板的路径
    #[clap(long = "template")]
    pub template: Option<PathBuf>,

    /// 测试目录 - 指定包含测试模板的目录
    #[clap(short = 'D', long = "test-dir")]
    pub test_dir: Option<PathBuf>,

    /// 报告目录 - 指定存放测试报告的目录
    #[clap(long = "reports-dir")]
    pub reports_dir: Option<PathBuf>,

    /// 聚合报告输出 - 指定聚合报告的输出文件路径
    #[clap(long = "output", short = 'o')]
    pub output: Option<PathBuf>,

    /// 报告JSON路径 - 指定包含报告数据的JSON文件路径
    #[clap(long = "reports-json")]
    pub reports_json: Option<PathBuf>,

    /// 汇总报告路径 - 指定汇总报告的输出路径
    #[clap(long = "summary-path")]
    pub summary_path: Option<PathBuf>,

    /// 报告路径 - 指定单一测试报告的输出路径
    #[clap(long = "report-path")]
    pub report_path: Option<PathBuf>,

    /// 单元名称 - 通过单元名称筛选测试
    #[clap(long = "unit")]
    pub unit: Option<String>,

    /// 标签 - 通过标签筛选测试
    #[clap(long = "tag")]
    pub tag: Option<String>,

    /// 目标配置文件 - 指定目标配置文件路径
    #[clap(long = "target")]
    pub target: Option<PathBuf>,

    /// 出错继续 - 即使测试失败也继续执行其余测试
    #[clap(long = "continue-on-error")]
    pub continue_on_error: bool,

    /// 执行命令超时时间（秒）
    #[clap(long = "timeout", default_value = "300")]
    pub timeout: u64,

    /// 命令失败后重试次数
    #[clap(long = "retry", default_value = "3")]
    pub retry: u32,
}

impl CliArgs {
    /// 解析命令行参数
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// 判断是否需要测试
    pub fn should_test(&self) -> bool {
        self.test
    }

    /// 判断是否需要聚合
    pub fn should_aggregate(&self) -> bool {
        self.aggregate
    }

    /// 判断是否需要汇总
    pub fn should_summarize(&self) -> bool {
        self.summarize
    }

    /// 判断是否为仅解析模式
    pub fn is_parse_only(&self) -> bool {
        self.parse_only
    }

    /// 获取测试环境类型
    pub fn get_environment_type(&self) -> Option<String> {
        if self.local {
            Some("local".to_string())
        } else if self.remote {
            Some("remote".to_string())
        } else if self.qemu {
            Some("qemu".to_string())
        } else if self.boardtest {
            Some("boardtest".to_string())
        } else {
            None
        }
    }

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

    /// 获取筛选条件
    pub fn get_filters(&self) -> (Option<&str>, Option<&str>, Option<&str>) {
        (self.unit.as_deref(), self.tag.as_deref(), self.target.as_deref().and_then(|p| p.to_str()))
    }
}