use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::resolvers::pnpm::PackageJson;

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub path: PathBuf,
    pub package_json: PackageJson
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

    pub fn discover_packages(&mut self, root: &Path, globs: &[String]) -> Result<()> {
        println!("\nSearching for packages in: {}", root.display());
        println!("Using globs: {:?}", globs);

        for entry in WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
        {
            let entry = entry?;
            if entry.file_name() == "package.json" {
                let package_dir = entry.path().parent().unwrap();
                println!("Found package.json in: {}", package_dir.display());
                
                // Check if the package matches any of our globs
                let relative_path = package_dir.strip_prefix(root).unwrap();
                let matches = globs.iter().any(|glob| {
                    let result = glob::Pattern::new(glob)
                        .unwrap()
                        .matches_path(relative_path);
                    println!("  Checking glob '{}' against '{}': {}", 
                        glob, relative_path.display(), result);
                    result
                });

                if !matches {
                    println!("  Skipping - doesn't match any glob pattern");
                    continue;
                }

                let package_json = crate::resolvers::pnpm::load_package_json(entry.path())?;
                println!("  Adding package: {}", package_json.name);

                let package = Package {
                    name: package_json.name.clone(),
                    path: package_dir.to_path_buf(),
                    package_json,
                };

                self.packages.insert(package.name.clone(), package);
            }
        }
        Ok(())
    }

    pub fn get_packages(&self) -> &HashMap<String, Package> {
        &self.packages
    }

    // Get dependencies directly from package.json
    pub fn get_dependencies(&self, package_name: &str) -> Vec<String> {
        if let Some(package) = self.packages.get(package_name) {
            let mut deps = Vec::new();
            
            if let Some(ref dependencies) = package.package_json.dependencies {
                deps.extend(dependencies.keys().cloned());
            }
            if let Some(ref dev_dependencies) = package.package_json.dev_dependencies {
                deps.extend(dev_dependencies.keys().cloned());
            }
            
            // Only return dependencies that exist in our workspace
            deps.into_iter()
                .filter(|dep| self.packages.contains_key(dep))
                .collect()
        } else {
            vec![]
        }
    }
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
} 