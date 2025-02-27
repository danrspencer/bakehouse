use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use hcl::Value;
use indexmap::IndexMap;
use std::io;
use anyhow::Result;
use crate::dockerfile::DockerfileGenerator;
use std::fs;

#[derive(Debug, Serialize)]
pub struct Target {
    pub context: String,
    pub dockerfile: String,
    pub tags: Vec<String>,
    pub depends_on: Vec<String>,
    pub dockerfile_contents: Option<String>,
    pub contexts: Option<HashMap<String, String>>,
}

impl Target {
    pub fn new(package_path: &Path, workspace_root: &Path, dockerfile: String, tags: Vec<String>, depends_on: Vec<String>) -> Self {
        // First get the package path relative to the workspace root
        let relative_path = package_path
            .strip_prefix(workspace_root)
            .unwrap_or(package_path)
            .to_string_lossy()
            .into_owned();

        // Remove any leading "./" and ensure paths are relative to workspace root
        let context = relative_path.trim_start_matches("./").to_string();

        // For non-root targets, add the root context
        let contexts = if context != "." {
            let mut contexts = HashMap::new();
            contexts.insert("root".to_string(), "target:root".to_string());
            Some(contexts)
        } else {
            None
        };

        Self {
            context,
            dockerfile,
            tags,
            depends_on,
            dockerfile_contents: None,
            contexts,
        }
    }

    #[cfg(test)]
    pub fn with_context(context: String) -> Self {
        Self {
            context,
            dockerfile: "Dockerfile.bake".to_string(),
            tags: vec![],
            depends_on: vec![],
            dockerfile_contents: None,
            contexts: None,
        }
    }

    fn to_hcl(&self) -> Value {
        let mut map = IndexMap::new();
        map.insert("context".to_string(), Value::String(self.context.clone()));
        map.insert("dockerfile".to_string(), Value::String(self.dockerfile.clone()));
        map.insert("tags".to_string(), Value::Array(
            self.tags.iter().map(|t| Value::String(t.clone())).collect()
        ));
        map.insert("depends_on".to_string(), Value::Array(
            self.depends_on.iter().map(|d| Value::String(d.clone())).collect()
        ));

        // Add contexts if present
        if let Some(contexts) = &self.contexts {
            let contexts_map = contexts.iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect();
            map.insert("contexts".to_string(), Value::Object(contexts_map));
        }

        Value::Object(map)
    }
}

impl Default for Target {
    fn default() -> Self {
        Self {
            context: ".".to_string(),
            dockerfile: "Dockerfile.bake".to_string(),
            tags: vec!["workspace-root:latest".to_string()],
            depends_on: vec![],
            dockerfile_contents: None,
            contexts: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BakeFile {
    pub group: HashMap<String, Group>,
    pub target: HashMap<String, Target>,
}

#[derive(Debug, Serialize)]
pub struct Group {
    pub targets: Vec<String>,
}

impl BakeFile {
    pub fn new() -> Self {
        BakeFile {
            group: HashMap::new(),
            target: HashMap::new(),
        }
    }

    pub fn add_target(&mut self, name: String, mut target: Target) {
        // If we have dockerfile contents, write them to a file
        if let Some(contents) = target.dockerfile_contents.take() {
            let dockerfile_path = PathBuf::from(&target.context).join(&target.dockerfile);
            fs::write(dockerfile_path, contents).expect("Failed to write Dockerfile");
        }
        
        self.target.insert(name, target);
    }

    pub fn add_group(&mut self, name: String, targets: Vec<String>) {
        self.group.insert(name, Group { targets });
    }

    pub fn add_root_target(&mut self, workspace_root: &Path, node_version: String) -> Result<()> {
        let dockerfile_generator = DockerfileGenerator::default();
        let dockerfile = dockerfile_generator.generate(workspace_root)?;

        let target = Target {
            context: ".".into(),
            dockerfile: "Dockerfile.bake".into(),
            dockerfile_contents: Some(dockerfile.clone()),
            ..Default::default()
        };

        // Write the Dockerfile contents
        fs::write(workspace_root.join("Dockerfile.bake"), dockerfile)?;
        
        self.target.insert("root".into(), target);
        Ok(())
    }

    pub fn to_hcl(&self) -> String {
        let mut output = String::new();

        // Add groups
        for (name, group) in &self.group {
            output.push_str(&format!("group \"{}\" {{\n", name));
            output.push_str("  targets = [");
            output.push_str(&group.targets.iter()
                .map(|t| format!("\"{}\"", t))
                .collect::<Vec<_>>()
                .join(", "));
            output.push_str("]\n}\n\n");
        }

        // Add targets
        for (name, target) in &self.target {
            output.push_str(&format!("target \"{}\" {{\n", name));
            output.push_str(&format!("  context = \"{}\"\n", target.context));
            output.push_str(&format!("  dockerfile = \"{}\"\n", target.dockerfile));
            
            output.push_str("  tags = [");
            output.push_str(&target.tags.iter()
                .map(|t| format!("\"{}\"", t))
                .collect::<Vec<_>>()
                .join(", "));
            output.push_str("]\n");

            if !target.depends_on.is_empty() {
                output.push_str("  depends_on = [");
                output.push_str(&target.depends_on.iter()
                    .map(|d| format!("\"{}\"", d))
                    .collect::<Vec<_>>()
                    .join(", "));
                output.push_str("]\n");
            }

            // Add contexts if present
            if let Some(contexts) = &target.contexts {
                output.push_str("  contexts = {\n");
                for (key, value) in contexts {
                    output.push_str(&format!("    {} = \"{}\"\n", key, value));
                }
                output.push_str("  }\n");
            }
            
            output.push_str("}\n\n");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_target_new_with_workspace_root() {
        let workspace_root = PathBuf::from("/workspace");
        let package_path = PathBuf::from("/workspace/apps/api");
        
        let target = Target::new(
            &package_path,
            &workspace_root,
            "Dockerfile.bake".to_string(),
            vec!["sample-api:1.0.0".to_string()],
            vec![],
        );

        assert_eq!(target.context, "apps/api");
    }

    #[test]
    fn test_target_new_with_nested_workspace() {
        let workspace_root = PathBuf::from("/projects/myrepo/workspace");
        let package_path = PathBuf::from("/projects/myrepo/workspace/packages/logger");
        
        let target = Target::new(
            &package_path,
            &workspace_root,
            "Dockerfile.bake".to_string(),
            vec!["sample-logger:1.0.0".to_string()],
            vec![],
        );

        assert_eq!(target.context, "packages/logger");
    }

    #[test]
    fn test_target_new_with_relative_paths() {
        let workspace_root = PathBuf::from("./sample/monorepo");
        let package_path = PathBuf::from("./sample/monorepo/apps/web");
        
        let target = Target::new(
            &package_path,
            &workspace_root,
            "Dockerfile.bake".to_string(),
            vec!["sample-web:1.0.0".to_string()],
            vec![],
        );

        assert_eq!(target.context, "apps/web");
    }

    #[test]
    fn test_target_new_with_current_directory() {
        let workspace_root = PathBuf::from(".");
        let package_path = PathBuf::from("./apps/api");
        
        let target = Target::new(
            &package_path,
            &workspace_root,
            "Dockerfile.bake".to_string(),
            vec!["sample-api:1.0.0".to_string()],
            vec![],
        );

        assert_eq!(target.context, "apps/api");
    }

    #[test]
    fn test_target_new_with_absolute_paths() {
        let current_dir = std::env::current_dir().unwrap();
        let workspace_root = current_dir.join("sample/monorepo");
        let package_path = workspace_root.join("apps/api");
        
        let target = Target::new(
            &package_path,
            &workspace_root,
            "Dockerfile.bake".to_string(),
            vec!["sample-api:1.0.0".to_string()],
            vec![],
        );

        assert_eq!(target.context, "apps/api");
    }

    #[test]
    fn test_target_new_when_running_from_different_directory() {
        // Simulate running CLI from /some/other/path
        // but workspace is at /workspace/myapp
        let workspace_root = PathBuf::from("/workspace/myapp");
        let package_paths = vec![
            (
                PathBuf::from("/workspace/myapp/apps/api"),
                "apps/api"  // Expected context path
            ),
            (
                PathBuf::from("/workspace/myapp/packages/logger"),
                "packages/logger"  // Expected context path
            ),
        ];

        for (package_path, expected_context) in package_paths {
            let target = Target::new(
                &package_path,
                &workspace_root,
                "Dockerfile.bake".to_string(),
                vec!["test:1.0.0".to_string()],
                vec![],
            );

            assert_eq!(
                target.context, 
                expected_context,
                "Context path should be relative to workspace root, not current directory"
            );
        }
    }

    #[test]
    fn test_target_new_with_nested_workspace_from_different_directory() {
        // Simulate running CLI from /home/user
        // but workspace is at /var/projects/myapp/services
        let workspace_root = PathBuf::from("/var/projects/myapp/services");
        let package_path = PathBuf::from("/var/projects/myapp/services/backend/api");
        
        let target = Target::new(
            &package_path,
            &workspace_root,
            "Dockerfile.bake".to_string(),
            vec!["api:1.0.0".to_string()],
            vec![],
        );

        assert_eq!(
            target.context, 
            "backend/api",
            "Context should be relative to workspace root regardless of where CLI is run from"
        );
    }

    #[test]
    fn test_target_new_with_absolute_workspace_path() {
        // This simulates running:
        // cd /some/random/dir
        // cargo run -- --workspace /absolute/path/to/workspace
        let workspace_root = PathBuf::from("/absolute/path/to/workspace");
        let package_path = workspace_root.join("services/web");
        
        let target = Target::new(
            &package_path,
            &workspace_root,
            "Dockerfile.bake".to_string(),
            vec!["web:1.0.0".to_string()],
            vec![],
        );

        assert_eq!(
            target.context, 
            "services/web",
            "Context should always be relative to workspace root"
        );
    }

    #[test]
    fn test_target_to_hcl() {
        let target = Target {
            context: "apps/api".to_string(),
            dockerfile: "Dockerfile.bake".to_string(),
            tags: vec!["sample-api:1.0.0".to_string()],
            depends_on: vec!["sample-logger".to_string()],
            dockerfile_contents: None,
            contexts: None,
        };

        let hcl = target.to_hcl();
        
        // Verify the HCL structure
        if let Value::Object(map) = hcl {
            assert_eq!(map.get("context").unwrap(), &Value::String("apps/api".to_string()));
            assert_eq!(map.get("dockerfile").unwrap(), &Value::String("Dockerfile.bake".to_string()));
            
            if let Value::Array(tags) = map.get("tags").unwrap() {
                assert_eq!(tags[0], Value::String("sample-api:1.0.0".to_string()));
            } else {
                panic!("tags should be an array");
            }

            if let Value::Array(deps) = map.get("depends_on").unwrap() {
                assert_eq!(deps[0], Value::String("sample-logger".to_string()));
            } else {
                panic!("depends_on should be an array");
            }
        } else {
            panic!("target.to_hcl() should return an Object");
        }
    }

    #[test]
    fn test_bakefile_to_hcl() {
        let mut bake_file = BakeFile::new();
        
        // Add a target
        let target = Target {
            context: "apps/api".to_string(),
            dockerfile: "Dockerfile.bake".to_string(),
            tags: vec!["sample-api:1.0.0".to_string()],
            depends_on: vec!["sample-logger".to_string()],
            dockerfile_contents: None,
            contexts: None,
        };
        bake_file.add_target("sample-api".to_string(), target);

        // Add a group
        bake_file.add_group("default".to_string(), vec!["sample-api".to_string()]);

        let hcl = bake_file.to_hcl();

        // Expected HCL output
        let expected = r#"group "default" {
  targets = ["sample-api"]
}

target "sample-api" {
  context = "apps/api"
  dockerfile = "Dockerfile.bake"
  tags = ["sample-api:1.0.0"]
  depends_on = ["sample-logger"]
}

"#;

        assert_eq!(hcl, expected);
    }

    #[test]
    fn test_bakefile_to_hcl_multiple_targets() {
        let mut bake_file = BakeFile::new();
        
        // Add API target
        let api_target = Target {
            context: "apps/api".to_string(),
            dockerfile: "Dockerfile.bake".to_string(),
            tags: vec!["sample-api:1.0.0".to_string()],
            depends_on: vec!["sample-logger".to_string()],
            dockerfile_contents: None,
            contexts: None,
        };
        bake_file.add_target("sample-api".to_string(), api_target);

        // Add Admin target
        let admin_target = Target {
            context: "apps/admin".to_string(),
            dockerfile: "Dockerfile.bake".to_string(),
            tags: vec!["sample-admin:1.0.0".to_string()],
            depends_on: vec!["sample-types".to_string()],
            dockerfile_contents: None,
            contexts: None,
        };
        bake_file.add_target("sample-admin".to_string(), admin_target);

        // Add default group with both targets
        bake_file.add_group(
            "default".to_string(), 
            vec!["sample-api".to_string(), "sample-admin".to_string()]
        );

        let hcl = bake_file.to_hcl();

        // Verify the HCL contains both targets and the group
        assert!(hcl.contains(r#"group "default" {"#));
        assert!(hcl.contains(r#"targets = ["sample-api", "sample-admin"]"#));
        assert!(hcl.contains(r#"target "sample-api" {"#));
        assert!(hcl.contains(r#"target "sample-admin" {"#));
        assert!(hcl.contains(r#"context = "apps/api""#));
        assert!(hcl.contains(r#"context = "apps/admin""#));
    }

    #[test]
    fn test_target_new_handles_sanitized_names_and_paths() {
        let workspace_root = PathBuf::from("/workspace");
        let package_path = PathBuf::from("/workspace/apps/api");
        
        let target = Target::new(
            &package_path,
            &workspace_root,
            "Dockerfile.bake".to_string(),
            vec!["sample-api:1.0.0".to_string()], // Sanitized name
            vec!["sample-logger".to_string()],     // Sanitized dependency
        );

        assert_eq!(target.context, "apps/api");
        
        // Test that the generated HCL is valid
        let hcl = target.to_hcl();
        if let Value::Object(map) = hcl {
            assert_eq!(map.get("context").unwrap(), &Value::String("apps/api".to_string()));
            assert!(map.get("tags").unwrap().to_string().contains("sample-api:1.0.0"));
            assert!(map.get("depends_on").unwrap().to_string().contains("sample-logger"));
        }
    }

    #[test]
    fn test_bakefile_with_root_target() {
        let mut bake_file = BakeFile::new();
        
        // Add root target
        bake_file.add_root_target(Path::new("/workspace"), "18".to_string()).unwrap();

        // Add API target
        let api_target = Target {
            context: "apps/api".to_string(),
            dockerfile: "Dockerfile.bake".to_string(),
            tags: vec!["sample-api:1.0.0".to_string()],
            depends_on: vec!["root".to_string(), "sample-logger".to_string()],
            dockerfile_contents: None,
            contexts: None,
        };
        bake_file.add_target("sample-api".to_string(), api_target);

        let hcl = bake_file.to_hcl();

        // Verify root target exists
        assert!(hcl.contains(r#"target "root" {"#));
        assert!(hcl.contains(r#"context = ".""#));
        assert!(hcl.contains(r#"tags = ["workspace-root:latest"]"#));

        // Verify API target depends on root
        assert!(hcl.contains(r#"depends_on = ["root", "sample-logger"]"#));
    }
} 