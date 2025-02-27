use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use hcl::Value;
use indexmap::IndexMap;
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