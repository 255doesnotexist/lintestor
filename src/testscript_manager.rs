pub struct TestScriptManager {
    test_scripts: Vec<String>,
}

impl TestScriptManager {
    pub fn new(distro: &str, package: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = format!("./{}/{}", distro, package);
        let mut test_scripts = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "sh" {
                test_scripts.push(path.to_str().unwrap_or_default().to_string());
            }
        }
        Ok(TestScriptManager {
            test_scripts,
        })
    }

    pub fn get_test_scripts(&self) -> &[String] {
        &self.test_scripts
    }
}

