use anyhow::{Context, Result};
use std::path::Path;
pub mod model;
pub use model::*; // Re-export the types from model

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