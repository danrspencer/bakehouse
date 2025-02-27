use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub path: PathBuf,
    pub version: String,
    pub dependencies: std::collections::HashSet<String>,
}

pub struct Workspace {
    packages: HashMap<String, Package>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    pub fn add_package(&mut self, 
        name: String, 
        path: PathBuf, 
        version: String,
        dependencies: std::collections::HashSet<String>
    ) {
        let package = Package {
            name: name.clone(),
            path,
            version,
            dependencies,
        };
        self.packages.insert(name, package);
    }

    pub fn get_packages(&self) -> &HashMap<String, Package> {
        &self.packages
    }

    pub fn get_dependencies(&self, package_name: &str) -> Vec<String> {
        if let Some(package) = self.packages.get(package_name) {
            package.dependencies.iter()
                .filter(|dep| self.packages.contains_key(*dep))
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }
} 