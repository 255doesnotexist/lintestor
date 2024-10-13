//! Manages test scripts for a specific distribution and package.
///
/// This struct is responsible for discovering and storing paths to test scripts
/// located in a specific directory structure.
pub struct TestScriptManager {
    test_scripts: Vec<String>,
    metadata_script: Option<String>,
    skipped_scripts: Vec<String>,
}

/// The name of the metadata script, if it exists.
const METADATA_SCRIPT_NAME: &str = "metadata.sh";

impl TestScriptManager {
    /// Creates a new `TestScriptManager` instance.
    ///
    /// This method scans the directory `./{distro}/{package}` for `.sh` files
    /// and stores their paths. Note that `metadata.sh` files does not count as test scripts
    /// and would be treated specially, as these scripts are for storing the metadata variables
    /// of the package rather than for testing purposes.
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
    /// let skip_scripts = vec!["test1.sh".to_string(), "test2.sh".to_string()];
    /// let manager = TestScriptManager::new("ubuntu", "nginx", skip_scripts).expect("Failed to create TestScriptManager");
    /// ```
    pub fn new(
        distro: &str,
        package: &str,
        skip_scripts: Option<Vec<String>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = format!("./{}/{}", distro, package);
        let mut test_scripts = Vec::new();
        let mut metadata_script = None;
        let skipped_scripts = skip_scripts.unwrap_or_default();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "sh" {
                let final_path = path.to_str().unwrap_or_default().to_string();
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();

                if skipped_scripts.contains(&file_name.to_string()) {
                    log::debug!("skiped {}", &file_name.to_string());
                    continue;
                }

                if file_name == METADATA_SCRIPT_NAME {
                    metadata_script = Some(final_path.clone());
                } else {
                    test_scripts.push(final_path);
                }
            }
        }

        if metadata_script.is_none() {
            log::warn!(
                "Missing metadata.sh for {}/{}, its metadata will not be recorded",
                distro,
                package
            );
        }
        Ok(TestScriptManager {
            test_scripts,
            metadata_script,
            skipped_scripts,
        })
    }

    /// Returns a vector containing the paths of all discovered test scripts, excluding the skipped ones.
    ///
    /// # Returns
    ///
    /// A `Vec<String>`, where each `String` is the path to a discovered test script that is not in the list of scripts to skip.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate::TestScriptManager;
    ///
    /// let skip_scripts = vec!["test1.sh".to_string(), "test2.sh".to_string()];
    /// let manager = TestScriptManager::new("ubuntu", "nginx", skip_scripts).expect("Failed to create TestScriptManager");
    /// let scripts = manager.get_test_scripts();
    /// for script in scripts {
    ///     println!("Test script: {}", script);
    /// }
    /// ```
    pub fn get_test_scripts(&self) -> Vec<String> {
        self.test_scripts
            .iter()
            .filter(|script| !self.skipped_scripts.contains(script))
            .cloned()
            .collect()
    }

    /// Returns a slice containing the names of all discovered test scripts.
    ///
    /// # Returns
    ///
    /// A slice of `String`s, where each `String` is the name of a discovered test script.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate::TestScriptManager;
    ///
    /// let manager = TestScriptManager::new("ubuntu", "nginx").expect("Failed to create TestScriptManager");
    /// let scripts = manager.get_test_script_names();
    /// for script in scripts {
    ///     println!("Test script: {}", script);
    /// }
    /// ```
    pub fn get_test_script_names(&self) -> Vec<String> {
        self.test_scripts
            .iter()
            .map(|path| path.rsplit('/').next().unwrap().to_string())
            .collect()
    }

    /// Returns the local path to the `metadata.sh` script, if it exists.
    ///
    /// # Returns
    ///
    /// A `Some` containing the path to the `metadata.sh` script if it exists, or `None` if it doesn't.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate::TestScriptManager;
    ///
    /// let manager = TestScriptManager::new("ubuntu", "nginx").expect("Failed to create TestScriptManager");
    /// let metadata_script = manager.get_metadata_script();
    /// if let Some(script) = metadata_script {
    ///     println!("Metadata script: {}", script);
    /// }
    /// ```
    pub fn get_metadata_script(&self) -> Option<String> {
        self.metadata_script.clone() // Is there a way not to clone it?
    }

    /// Returns the name of the `metadata.sh` script, if it exists.
    ///
    /// # Returns
    ///
    /// A `Some` containing the name of the `metadata.sh` script if it exists, or `None` if it doesn't.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate::TestScriptManager;
    ///
    /// let manager = TestScriptManager::new("ubuntu", "nginx").expect("Failed to create TestScriptManager");
    /// let metadata_script = manager.get_metadata_script_name();
    /// if let Some(script) = metadata_script {
    ///     println!("Metadata script: {}", script);
    /// }
    /// ```
    pub fn get_metadata_script_name(&self) -> Option<String> {
        self.metadata_script
            .as_ref()
            .map(|path: &String| path.rsplit('/').next().unwrap().to_string())
    }
}
