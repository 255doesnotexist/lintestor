pub struct RunnerContext {
    pub is_remote: bool,
    pub remote_ip: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub use_boardtest: bool,
    pub boardtest_config: Option<BoardtestConfig>,
}

pub struct RunnerManager;

impl RunnerManager {
    pub fn get_runner(
        context: RunnerContext,
    ) -> Result<Box<dyn TestRunner>, Box<dyn std::error::Error>> {
        if context.use_boardtest {
            Ok(Box::new(BoardtestRunner::new(
                context
                    .boardtest_config
                    .ok_or("需要提供 Boardtest 配置")?,
            )))
        } else if context.is_remote {
            Ok(Box::new(RemoteTestRunner::new(
                context.remote_ip.ok_or("需要提供远程 IP 地址")?,
                context.port.unwrap_or(22),
                context.username.ok_or("需要提供用户名")?,
                context.password,
                context.private_key_path,
            )))
        } else {
            Ok(Box::new(LocalTestRunner::new()))
        }
    }
}