//! 定义了统一的 TestEnvironment trait，用于与不同测试执行环境交互。

pub mod local;
pub mod remote;
pub mod qemu;

use crate::utils::CommandOutput;
use std::error::Error;
use std::path::Path;

/// 一个用于与测试环境（本地、远程SSH、boardtest等）交互的 trait。
/// 提供了常见操作方法，如运行命令、传输文件等。
pub trait TestEnvironment {
    /// 在测试环境中运行命令。
    ///
    /// # 参数
    /// * `command` - 要执行的命令字符串
    ///
    /// # 返回值
    /// 返回一个 `Result` 包含 `CommandOutput` 或错误
    fn run_command(&self, command: &str) -> Result<CommandOutput, Box<dyn Error>>;

    /// 上传本地文件到测试环境中的指定路径
    ///
    /// # 参数
    /// * `local_path` - 本地文件路径
    /// * `remote_path` - 测试环境中的目标路径
    /// * `mode` - 文件权限模式（如 0o644）
    ///
    /// # 返回值
    /// 成功或失败的 `Result`
    fn upload_file(
        &self,
        local_path: &Path,
        remote_path: &str,
        mode: i32,
    ) -> Result<(), Box<dyn Error>>;

    /// 从测试环境下载文件到本地路径
    ///
    /// # 参数
    /// * `remote_path` - 测试环境中的文件路径
    /// * `local_path` - 本地目标路径
    ///
    /// # 返回值
    /// 成功或失败的 `Result`
    fn download_file(&self, remote_path: &str, local_path: &Path) -> Result<(), Box<dyn Error>>;

    /// 读取测试环境中文件的内容
    ///
    /// # 参数
    /// * `remote_path` - 测试环境中的文件路径
    ///
    /// # 返回值
    /// 返回文件内容的字符串或错误
    fn read_remote_file(&self, remote_path: &str) -> Result<String, Box<dyn Error>>;

    /// 在测试环境中创建目录（包括父目录）
    ///
    /// # 参数
    /// * `remote_path` - 要创建的目录路径
    ///
    /// # 返回值
    /// 成功或失败的 `Result`
    fn mkdir(&self, remote_path: &str) -> Result<(), Box<dyn Error>>;

    /// 在测试环境中递归删除文件或目录
    ///
    /// # 参数
    /// * `remote_path` - 要删除的路径
    ///
    /// # 返回值
    /// 成功或失败的 `Result`
    fn rm_rf(&self, remote_path: &str) -> Result<(), Box<dyn Error>>;

    /// 获取测试环境的操作系统和内核版本信息
    ///
    /// # 返回值
    /// 返回包含 `(os_version, kernel_version)` 元组的 `Result`，或错误
    fn get_os_info(&self) -> Result<(String, String), Box<dyn Error>>;

    /// 执行环境的设置或连接建立
    /// 可能涉及SSH握手、API认证等
    /// 应该是幂等的
    fn setup(&mut self) -> Result<(), Box<dyn Error>>;

    /// 执行环境的清理或断开连接
    fn teardown(&mut self) -> Result<(), Box<dyn Error>>;
}