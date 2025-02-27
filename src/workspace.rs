use anyhow::{Context, Result};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub path: PathBuf,
    pub package_json: crate::PackageJson,
}

pub struct Workspace {
    packages: HashMap<String, Package>,
    dependency_graph: DiGraph<String, ()>,
    node_indices: HashMap<String, NodeIndex>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            dependency_graph: DiGraph::new(),
            node_indices: HashMap::new(),
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

                let package_json: crate::PackageJson = serde_json::from_str(
                    &std::fs::read_to_string(entry.path())
                        .context("Failed to read package.json")?,
                )?;

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

    pub fn build_dependency_graph(&mut self) {
        // Clear existing graph
        self.dependency_graph = DiGraph::new();
        self.node_indices.clear();

        // Add all packages as nodes
        for package_name in self.packages.keys() {
            let idx = self.dependency_graph.add_node(package_name.clone());
            self.node_indices.insert(package_name.clone(), idx);
        }

        // Add dependency edges
        let packages: Vec<_> = self.packages.values().collect();
        for package in packages {
            let from_idx = self.node_indices[&package.name];

            // Add regular dependencies
            if let Some(deps) = &package.package_json.dependencies {
                for dep_name in deps.keys() {
                    if let Some(to_idx) = self.node_indices.get(dep_name) {
                        self.dependency_graph.add_edge(from_idx, *to_idx, ());
                    }
                }
            }

            // Add dev dependencies
            if let Some(deps) = &package.package_json.dev_dependencies {
                for dep_name in deps.keys() {
                    if let Some(to_idx) = self.node_indices.get(dep_name) {
                        self.dependency_graph.add_edge(from_idx, *to_idx, ());
                    }
                }
            }
        }
    }

    pub fn get_packages(&self) -> &HashMap<String, Package> {
        &self.packages
    }

    pub fn get_dependencies(&self, package_name: &str) -> Vec<String> {
        if let Some(idx) = self.node_indices.get(package_name) {
            self.dependency_graph
                .neighbors(*idx)
                .map(|i| self.dependency_graph[i].clone())
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