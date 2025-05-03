//! Manages test templates for a specific distribution and unit.

use std::path::Path;
///
/// This struct is responsible for discovering and storing paths to test templates
/// located in a specific directory structure.
pub struct TestTemplateManager {
    test_templates: Vec<String>,
    metadata_template: Option<String>,
}

/// The name of the metadata template, if it exists.
const METADATA_SCRIPT_NAME: &str = "metadata.sh";

impl TestTemplateManager {
    /// Creates a new `TestTemplateManager` instance.
    ///
    /// This method scans the directory `./{target}/{unit}` for `.sh` files
    /// and stores their paths. Note that `metadata.sh` files does not count as test templates
    /// and would be treated specially, as these templates are for storing the metadata variables
    /// of the unit rather than for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `target` - The name of the distribution.
    /// * `unit` - The name of the unit.
    /// * `dir` - Working directory which contains the test folders and files, defaults to env::current_dir()
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `TestTemplateManager` instance if successful,
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
    /// use your_crate::TestTemplateManager;
    ///
    /// let skip_templates = vec!["test1.sh".to_string(), "test2.sh".to_string()];
    /// let manager = TestTemplateManager::new("ubuntu", "nginx", skip_templates).expect("Failed to create TestTemplateManager");
    /// ```
    pub fn new(
        target: &str,
        unit: &str,
        skip_templates: Vec<String>,
        working_dir: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let directory = working_dir.join(format!("{}/{}", target, unit));
        let mut test_templates = Vec::new();
        let mut metadata_template = None;

        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "sh" {
                let final_path = path.to_str().unwrap_or_default().to_string();
                let file_name = path.file_name();

                if skip_templates
                    .iter()
                    .any(|s| s.as_str() == file_name.unwrap())
                {
                    log::debug!("skipped {}", file_name.unwrap().to_string_lossy());
                    continue;
                }

                match file_name {
                    Some(name) if name == METADATA_SCRIPT_NAME => {
                        metadata_template = Some(final_path)
                    }
                    _ => test_templates.push(final_path),
                }
            }
        }
        if metadata_template.is_none() {
            log::warn!(
                "Missing metadata.sh for {}/{}, its metadata will not be recorded",
                target,
                unit
            );
        }
        Ok(TestTemplateManager {
            test_templates,
            metadata_template,
        })
    }
    /// Returns a slice containing the paths of all discovered test templates.
    ///
    /// # Returns
    ///
    /// A slice of `String`s, where each `String` is the path to a discovered test template.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate::TestTemplateManager;
    ///
    /// let manager = TestTemplateManager::new("ubuntu", "nginx").expect("Failed to create TestTemplateManager");
    /// let templates = manager.get_test_templates();
    /// for template in templates {
    ///     println!("Test template: {}", template);
    /// }
    /// ```
    /// Returns a slice containing the paths of all discovered test templates.
    pub fn get_test_templates(&self) -> &[String] {
        &self.test_templates
    }

    /// Returns a slice containing the names of all discovered test templates.
    ///
    /// # Returns
    ///
    /// A slice of `String`s, where each `String` is the name of a discovered test template.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate::TestTemplateManager;
    ///
    /// let manager = TestTemplateManager::new("ubuntu", "nginx").expect("Failed to create TestTemplateManager");
    /// let templates = manager.get_test_template_names();
    /// for template in templates {
    ///     println!("Test template: {}", template);
    /// }
    /// ```
    pub fn get_test_template_names(&self) -> Vec<String> {
        self.test_templates
            .iter()
            .map(|path| path.rsplit('/').next().unwrap().to_string())
            .collect()
    }

    /// Returns the local path to the `metadata.sh` template, if it exists.
    ///
    /// # Returns
    ///
    /// A `Some` containing the path to the `metadata.sh` template if it exists, or `None` if it doesn't.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate::TestTemplateManager;
    ///
    /// let manager = TestTemplateManager::new("ubuntu", "nginx").expect("Failed to create TestTemplateManager");
    /// let metadata_template = manager.get_metadata_template();
    /// if let Some(template) = metadata_template {
    ///     println!("Metadata template: {}", template);
    /// }
    /// ```
    pub fn get_metadata_template(&self) -> Option<String> {
        self.metadata_template.clone() // Is there a way not to clone it?
    }

    /// Returns the name of the `metadata.sh` template, if it exists.
    ///
    /// # Returns
    ///
    /// A `Some` containing the name of the `metadata.sh` template if it exists, or `None` if it doesn't.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate::TestTemplateManager;
    ///
    /// let manager = TestTemplateManager::new("ubuntu", "nginx").expect("Failed to create TestTemplateManager");
    /// let metadata_template = manager.get_metadata_template_name();
    /// if let Some(template) = metadata_template {
    ///     println!("Metadata template: {}", template);
    /// }
    /// ```
    pub fn get_metadata_template_name(&self) -> Option<String> {
        self.metadata_template
            .as_ref()
            .map(|path: &String| path.rsplit('/').next().unwrap().to_string())
    }
}
