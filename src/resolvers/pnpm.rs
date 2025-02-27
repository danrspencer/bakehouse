use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::collections::HashSet;
pub mod model;
pub use model::*; // Re-export the types from model

#[derive(Debug)]
pub struct PnpmWorkspaceInfo {
    pub root_package: PackageJson,
    pub workspace_config: PnpmWorkspace,
    pub packages: Vec<PackageInfo>,
}

#[derive(Debug)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub dependencies: HashSet<String>,
}

pub fn load_workspace(workspace_root: &Path) -> Result<PnpmWorkspaceInfo> {
    // Load root package.json
    let root_package = load_package_json(
        &workspace_root.join("package.json")
    )?;

    // Load workspace configuration
    let workspace_file = workspace_root.join("pnpm-workspace.yaml");
    let workspace_config = load_workspace_config(&workspace_file)?;

    println!("Found workspace configuration:");
    for package_glob in &workspace_config.packages {
        println!("- {}", package_glob);
    }

    // Discover all packages
    let packages = discover_workspace_packages(workspace_root, &workspace_config.packages)?;

    Ok(PnpmWorkspaceInfo {
        root_package,
        workspace_config,
        packages,
    })
}

pub fn discover_workspace_packages(
    workspace_root: &Path,
    package_globs: &[String]
) -> Result<Vec<PackageInfo>> {
    let mut packages = Vec::new();
    
    println!("\nSearching for packages in: {}", workspace_root.display());
    println!("Using globs: {:?}", package_globs);

    for entry in WalkDir::new(workspace_root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
    {
        let entry = entry?;
        if entry.file_name() == "package.json" {
            let package_dir = entry.path().parent().unwrap();
            println!("Found package.json in: {}", package_dir.display());
            
            // Check if the package matches any of our globs
            let relative_path = package_dir.strip_prefix(workspace_root).unwrap();
            let matches = package_globs.iter().any(|glob| {
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

            let package_json = load_package_json(entry.path())?;
            println!("  Adding package: {}", package_json.name);

            // Collect all dependencies into a HashSet
            let mut dependencies = HashSet::new();
            if let Some(deps) = &package_json.dependencies {
                dependencies.extend(deps.keys().cloned());
            }
            if let Some(dev_deps) = &package_json.dev_dependencies {
                dependencies.extend(dev_deps.keys().cloned());
            }

            packages.push(PackageInfo {
                name: package_json.name,
                version: package_json.version,
                path: package_dir.to_path_buf(),
                dependencies,
            });
        }
    }
    
    Ok(packages)
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

pub fn load_package_json(path: &Path) -> Result<PackageJson> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read package.json")?;
    
    serde_json::from_str(&content)
        .context("Failed to parse package.json")
}

pub fn load_workspace_config(path: &Path) -> Result<PnpmWorkspace> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read pnpm-workspace.yaml")?;
    
    serde_yaml::from_str(&content)
        .context("Failed to parse pnpm-workspace.yaml")
}