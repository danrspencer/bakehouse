use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct Target {
    pub context: String,
    pub dockerfile: String,
    pub tags: Vec<String>,
    pub depends_on: Vec<String>,
}

impl Target {
    pub fn new(package_path: &Path, workspace_root: &Path, dockerfile: String, tags: Vec<String>, depends_on: Vec<String>) -> Self {
        // First get the package path relative to the workspace root
        let relative_path = package_path
            .strip_prefix(workspace_root)
            .unwrap_or(package_path);

        // Since docker-bake.json will be in the workspace root, we can use the relative path directly
        let context = relative_path
            .to_string_lossy()
            .into_owned();

        Self {
            context,
            dockerfile,
            tags,
            depends_on,
        }
    }

    #[cfg(test)]
    pub fn with_context(context: String) -> Self {
        Self {
            context,
            dockerfile: "Dockerfile.bake".to_string(),
            tags: vec![],
            depends_on: vec![],
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

    pub fn add_target(&mut self, name: String, target: Target) {
        self.target.insert(name, target);
    }

    pub fn add_group(&mut self, name: String, targets: Vec<String>) {
        self.group.insert(name, Group { targets });
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
} 