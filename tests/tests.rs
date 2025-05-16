use assert_cmd::Command;
use std::{
    env, fs,
    io::{self, Write},
    path::Path,
};
mod tests {
    use super::*;

    #[test]
    fn integration_test() {
        // TODO: Port integration test to 0.2.0+
        // 运行命令
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let output = cmd
            .env("RUST_LOG", "debug")
            .output()
            .expect("failed to execute process");

        // 输出命令执行结果
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    }
}
