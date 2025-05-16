//! Boardtest 测试环境的实现

use crate::config::boardtest_config::BoardtestConfig;
use crate::test_environment::{CommandOutput, TestEnvironment};
use anyhow::Context as _;
use log::debug;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

// API 响应结构体
#[derive(Debug, Deserialize)]
struct CreateTestResponse {
    test_id: String,
}

#[derive(Debug, Deserialize)]
struct TestStatusResponse {
    status: String,
}

#[derive(Debug, Serialize)]
struct TestConfig {
    tests: Vec<TestCase>,
}

#[derive(Debug, Serialize)]
struct TestCase {
    name: String,
    command: String,
    expected_output: String,
    method: String,
    timeout: u64,
}

/// 通过 Boardtest API 实现的测试环境
pub struct BoardtestEnvironment {
    config: BoardtestConfig,
    client: Option<Client>,
    current_test_id: Option<String>,
    command_results: HashMap<String, CommandOutput>,
}

impl BoardtestEnvironment {
    pub fn new(config: BoardtestConfig) -> Self {
        BoardtestEnvironment {
            config,
            client: None,
            current_test_id: None,
            command_results: HashMap::new(),
        }
    }

    fn ensure_client(&mut self) -> Result<&Client, Box<dyn Error>> {
        if self.client.is_none() {
            let timeout = Duration::from_secs(self.config.timeout_secs);
            let client = Client::builder()
                .timeout(timeout)
                .build()
                .context("创建 HTTP 客户端失败")?;
            self.client = Some(client);
        }
        
        Ok(self.client.as_ref().unwrap())
    }

    // 创建测试
    fn create_test(
        &mut self,
        base64_content: &str,
        command: &str,
    ) -> Result<String, Box<dyn Error>> {
        // Clone all needed config values before mutable borrowing
        let api_url = self.config.api_url.clone();
        let token = self.config.token.clone();
        let board_config = self.config.board_config.clone();
        let serial = self.config.serial.clone();
        let mi_sdk_enabled = self.config.mi_sdk_enabled;
        let timeout_secs = self.config.timeout_secs;
        
        let client = self.ensure_client()?;
        
        // 先写入测试配置
        let test_config = TestConfig {
            tests: vec![TestCase {
                name: "execute_command".to_string(),
                command: command.to_string(),
                expected_output: "0".to_string(), // 期望返回值为0
                method: "exit_code".to_string(),
                timeout: timeout_secs,
            }],
        };

        let write_test_resp = client
            .post(format!("{api_url}/write_test"))
            .json(&json!({
                "token": token,
                "test_name": "unit_test",
                "test_content": test_config
            }))
            .send()
            .context("写入测试配置失败")?;

        if !write_test_resp.status().is_success() {
            return Err(format!(
                "写入测试配置失败: {}",
                write_test_resp.text()?
            ).into());
        }

        // 然后创建测试
        let create_test_args = format!(
            "-f -t -s -b {} -S {}{}",
            board_config,
            serial,
            if mi_sdk_enabled {
                " -M"
            } else {
                ""
            }
        );

        let create_resp = client
            .post(format!("{api_url}/create_test"))
            .json(&json!({
                "token": token,
                "args": create_test_args
            }))
            .send()
            .context("创建测试失败")?;

        if !create_resp.status().is_success() {
            return Err(format!("创建测试失败: {}", create_resp.text()?).into());
        }

        let create_test_response: CreateTestResponse = create_resp.json()?;
        let test_id = create_test_response.test_id;
        self.current_test_id = Some(test_id.clone());
        Ok(test_id)
    }

    // 启动测试
    fn start_test(&self, test_id: &str) -> Result<(), Box<dyn Error>> {
        let client = self.client.as_ref().ok_or("HTTP 客户端未初始化")?;
        let api_url = &self.config.api_url;
        let token = &self.config.token;
        
        let resp = client
            .post(format!("{api_url}/start_test/{test_id}"))
            .json(&json!({
                "token": token
            }))
            .send()
            .context("启动测试失败")?;

        if !resp.status().is_success() {
            return Err(format!("启动测试失败: {}", resp.text()?).into());
        }
        Ok(())
    }

    // 等待测试完成
    fn wait_for_test_completion(&self, test_id: &str) -> Result<bool, Box<dyn Error>> {
        let client = self.client.as_ref().ok_or("HTTP 客户端未初始化")?;
        let start_time = Instant::now();
        let timeout_secs = self.config.timeout_secs;
        let api_url = &self.config.api_url;

        while start_time.elapsed() < Duration::from_secs(timeout_secs) {
            let resp = client
                .get(format!("{api_url}/test_status/{test_id}"))
                .send()
                .context("获取测试状态失败")?;

            if !resp.status().is_success() {
                return Err(format!("获取测试状态失败: {}", resp.text()?).into());
            }

            let status: TestStatusResponse = resp.json()?;
            match status.status.as_str() {
                "completed" => return Ok(true),
                "failed" => return Ok(false),
                "stopped" => return Ok(false),
                _ => {
                    thread::sleep(Duration::from_secs(5)); // 每5秒检查一次
                    continue;
                }
            }
        }

        Err(format!(
            "测试超时 {timeout_secs} 秒后未完成"
        ).into())
    }

    // 获取测试输出
    fn get_test_output(&self, test_id: &str) -> Result<String, Box<dyn Error>> {
        let client = self.client.as_ref().ok_or("HTTP 客户端未初始化")?;
        let api_url = &self.config.api_url;
        
        let resp = client
            .get(format!("{api_url}/test_output/{test_id}"))
            .send()
            .context("获取测试输出失败")?;

        if !resp.status().is_success() {
            return Err(format!("获取测试输出失败: {}", resp.text()?).into());
        }

        Ok(resp.text()?)
    }

    // 执行命令并获取结果的完整流程
    fn execute_command_on_board(&mut self, command: &str, base64_content: Option<&str>) -> Result<CommandOutput, Box<dyn Error>> {
        let cmd_with_extract = if let Some(content) = base64_content {
            format!(
                "echo '{content}' | base64 -d | tar xz -C /tmp && {command}"
            )
        } else {
            command.to_string()
        };

        // 创建测试（包括已有的 base64 内容，如果有的话）
        let test_id = self.create_test(base64_content.unwrap_or(""), &cmd_with_extract)?;
        
        // 启动测试
        self.start_test(&test_id)?;
        
        // 等待测试完成
        let test_passed = self.wait_for_test_completion(&test_id)?;
        
        // 获取测试输出
        let test_output = self.get_test_output(&test_id)?;
        
        let result = CommandOutput {
            command: command.to_string(),
            exit_status: if test_passed { 0 } else { 1 },
            output: format!("stdout:\n{test_output}\nstderr:\n"), // 在 boardtest 中没有分开的 stderr
        };
        
        // 存储命令结果以备将来使用
        self.command_results.insert(command.to_string(), result.clone());
        
        Ok(result)
    }
}

impl TestEnvironment for BoardtestEnvironment {
    fn setup(&mut self) -> Result<(), Box<dyn Error>> {
        debug!("BoardtestEnvironment setup called.");
        // 确保我们有一个 HTTP 客户端
        self.ensure_client()?;
        Ok(())
    }

    fn teardown(&mut self) -> Result<(), Box<dyn Error>> {
        debug!("BoardtestEnvironment teardown called.");
        // 清理当前测试
        self.current_test_id = None;
        self.command_results.clear();
        Ok(())
    }

    fn run_command(&self, command: &str) -> Result<CommandOutput, Box<dyn Error>> {
        // 注意：我们在 board 上实际上是建立新的测试，这里需要 &mut self
        // 但由于 trait 定义为 &self，我们需要使用另一种方式实现
        debug!("在 BoardtestEnvironment 上运行命令: {command}");
        
        // 检查我们是否已经有该命令的结果
        if let Some(result) = self.command_results.get(command) {
            return Ok(result.clone());
        }
        
        // 理论上 run_command 应该是非可变的，但 boardtest 需要创建新的测试
        // 这里我们返回一个错误提示函数调用者应该使用哪种方法
        Err("BoardtestEnvironment 不支持直接调用 run_command。请使用 execute_command 方法，该方法需要可变引用。".into())
    }

    fn upload_file(&self, local_path: &Path, _remote_path: &str, _mode: i32) -> Result<(), Box<dyn Error>> {
        // Boardtest 不支持直接上传文件，必须通过测试包含的 base64 内容
        Err(format!("BoardtestEnvironment 不支持直接上传单个文件: {local_path:?}，请使用 execute_command 并提供整个测试目录的 base64 内容").into())
    }

    fn download_file(&self, remote_path: &str, _local_path: &Path) -> Result<(), Box<dyn Error>> {
        // Boardtest 不支持直接下载文件
        Err(format!("BoardtestEnvironment 不支持下载文件: {remote_path}").into())
    }

    fn read_remote_file(&self, remote_path: &str) -> Result<String, Box<dyn Error>> {
        // 我们可以通过运行 cat 命令来模拟
        // 但由于 run_command 的限制，我们只能返回错误
        Err(format!(
            "BoardtestEnvironment 不支持直接读取远程文件: {remote_path}。请使用 execute_command 运行 'cat {remote_path}'"
        ).into())
    }

    fn mkdir(&self, remote_path: &str) -> Result<(), Box<dyn Error>> {
        // 同样，不能直接在 board 上创建目录
        Err(format!(
            "BoardtestEnvironment 不支持直接创建远程目录: {remote_path}。请使用 execute_command 运行 'mkdir -p {remote_path}'"
        ).into())
    }

    fn rm_rf(&self, remote_path: &str) -> Result<(), Box<dyn Error>> {
        // 同样，不能直接在 board 上删除目录
        Err(format!(
            "BoardtestEnvironment 不支持直接删除远程目录: {remote_path}。请使用 execute_command 运行 'rm -rf {remote_path}'"
        ).into())
    }

    fn get_os_info(&self) -> Result<(String, String), Box<dyn Error>> {
        Err("BoardtestEnvironment 不支持直接获取操作系统信息，必须通过测试脚本中的命令获取".into())
    }
}

// 为 BoardtestEnvironment 添加执行命令的额外方法
impl BoardtestEnvironment {
    /// 在 boardtest 上执行命令（请注意这需要可变引用）
    pub fn execute_command(&mut self, command: &str) -> Result<CommandOutput, Box<dyn Error>> {
        self.execute_command_on_board(command, None)
    }
    
    /// 上传 tar.gz base64 内容并在 boardtest 上执行命令
    pub fn execute_command_with_content(&mut self, command: &str, base64_content: &str) -> Result<CommandOutput, Box<dyn Error>> {
        self.execute_command_on_board(command, Some(base64_content))
    }
}