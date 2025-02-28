use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct BakehouseConfig {
    /// The default output format for docker-bake files (hcl or json)
    #[serde(default = "default_output_format")]
    pub output_format: String,

    /// Custom Dockerfile template mappings
    /// The key is a glob pattern that matches package paths
    /// The value is the path to the Dockerfile template to use
    #[serde(default)]
    pub templates: HashMap<String, PathBuf>,
}

fn default_output_format() -> String {
    "hcl".to_string()
}

impl Default for BakehouseConfig {
    fn default() -> Self {
        Self {
            output_format: default_output_format(),
            templates: HashMap::new(),
        }
    }
}

impl BakehouseConfig {
    /// Load the configuration from a .bakehouse file in the given directory
    pub fn load(workspace_root: &PathBuf) -> Result<Self> {
        let config_path = workspace_root.join(".bakehouse");
        
        // If the file doesn't exist, return default config
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(config_path)?;
        let config: BakehouseConfig = serde_yaml::from_str(&content)?;

        Ok(config)
    }

    /// Find a matching template for a given package path
    pub fn find_template(&self, package_path: &PathBuf) -> Option<&PathBuf> {
        for (glob, template) in &self.templates {
            if glob::Pattern::new(glob)
                .ok()?
                .matches_path(package_path)
            {
                return Some(template);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = BakehouseConfig::default();
        assert_eq!(config.output_format, "hcl");
        assert!(config.templates.is_empty());
    }

    #[test]
    fn test_load_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join(".bakehouse");

        let config_content = r#"
output_format: json
templates:
  "apps/*": "./templates/app.dockerfile"
  "packages/*": "./templates/lib.dockerfile"
"#;
        fs::write(&config_path, config_content)?;

        let config = BakehouseConfig::load(&temp_dir.path().to_path_buf())?;
        assert_eq!(config.output_format, "json");
        assert_eq!(config.templates.len(), 2);
        assert_eq!(
            config.templates.get("apps/*").unwrap(),
            &PathBuf::from("./templates/app.dockerfile")
        );

        Ok(())
    }

    #[test]
    fn test_find_template() -> Result<()> {
        let mut config = BakehouseConfig::default();
        config.templates.insert(
            "apps/*".to_string(),
            PathBuf::from("./templates/app.dockerfile"),
        );
        config.templates.insert(
            "packages/*".to_string(),
            PathBuf::from("./templates/lib.dockerfile"),
        );

        let app_path = PathBuf::from("apps/my-app");
        let lib_path = PathBuf::from("packages/my-lib");
        let other_path = PathBuf::from("other/thing");

        assert_eq!(
            config.find_template(&app_path).unwrap(),
            &PathBuf::from("./templates/app.dockerfile")
        );
        assert_eq!(
            config.find_template(&lib_path).unwrap(),
            &PathBuf::from("./templates/lib.dockerfile")
        );
        assert!(config.find_template(&other_path).is_none());

        Ok(())
    }
} 