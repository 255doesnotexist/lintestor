/// Manages test scripts for a specific distribution and package.
///
/// This struct is responsible for discovering and storing paths to test scripts
/// located in a specific directory structure.
pub struct TestScriptManager {
    test_scripts: Vec<String>,
}

impl TestScriptManager {
    /// Creates a new `TestScriptManager` instance.
    ///
    /// This method scans the directory `./{distro}/{package}` for `.sh` files
    /// and stores their paths.
    ///
    /// # Arguments
    ///
    /// * `distro` - The name of the distribution.
    /// * `package` - The name of the package.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `TestScriptManager` instance if successful,
    /// or an error if the directory couldn't be read or if there were issues accessing file information.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    /// - If the specified directory doesn't exist or can't be read.
    /// - If there are permission issues when accessing files in the directory.
    /// - If there are issues converting file paths to strings.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate::TestScriptManager;
    ///
    /// let manager = TestScriptManager::new("ubuntu", "nginx").expect("Failed to create TestScriptManager");
    /// ```
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
        Ok(TestScriptManager { test_scripts })
    }

    /// Returns a slice containing the paths of all discovered test scripts.
    ///
    /// # Returns
    ///
    /// A slice of `String`s, where each `String` is the path to a discovered test script.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate::TestScriptManager;
    ///
    /// let manager = TestScriptManager::new("ubuntu", "nginx").expect("Failed to create TestScriptManager");
    /// let scripts = manager.get_test_scripts();
    /// for script in scripts {
    ///     println!("Test script: {}", script);
    /// }
    /// ```
    pub fn get_test_scripts(&self) -> &[String] {
        &self.test_scripts
    }
}
