use std::path::Path;
use tera::{Tera, Context};
use std::fs;
use anyhow::Result;
use crate::PackageJson;

pub struct DockerfileGenerator {
    pub pnpm_version: String,
    root_package_json: Option<PackageJson>,
}

impl Default for DockerfileGenerator {
    fn default() -> Self {
        Self {
            pnpm_version: "8.15.1".to_string(),
            root_package_json: None,
        }
    }
}

impl DockerfileGenerator {
    pub fn new(root_package_json: PackageJson) -> Self {
        Self {
            pnpm_version: "8.15.1".to_string(),
            root_package_json: Some(root_package_json),
        }
    }

    pub fn get_node_version(&self) -> String {
        self.root_package_json
            .as_ref()
            .and_then(|pkg| pkg.engines.as_ref())
            .and_then(|engines| engines.node.as_ref())
            .map(|version| version.trim_start_matches('v').to_string())
            .unwrap_or_else(|| "20".to_string())
    }

    fn is_root_package(&self, package_path: &Path) -> bool {
        package_path.join("pnpm-workspace.yaml").exists()
    }

    pub fn generate(&self, package_path: &Path) -> Result<String> {
        let node_version = self.get_node_version();
        let template_name = if self.is_root_package(package_path) {
            "Dockerfile.root.tera"
        } else {
            "Dockerfile.tera"
        };
        
        let template_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/templates/");
        let template_content = fs::read_to_string(template_path.to_owned() + template_name)?;
        
        let mut tera = Tera::default();
        tera.add_raw_template("dockerfile", &template_content)?;
        
        let mut context = Context::new();
        context.insert("node_version", &node_version);
        context.insert("pnpm_version", &self.pnpm_version);
        context.insert("workdir", "/app");
        
        if !self.is_root_package(package_path) {
            let package_name = package_path
                .strip_prefix(package_path.parent().unwrap().parent().unwrap())?
                .to_string_lossy();
                
            context.insert("package_name", &package_name);
            
            let copy_files = vec![
                "pnpm-lock.yaml",
                "pnpm-workspace.yaml",
                "package.json",
                "packages/*/package.json",
                "apps/*/package.json",
                ".",
            ];
            context.insert("copy_files", &copy_files);
            
            let run_commands = vec![
                format!("corepack enable && corepack prepare pnpm@{} --activate", self.pnpm_version),
                "pnpm install --frozen-lockfile".to_string(),
                format!("pnpm --filter {} build", package_name),
            ];
            context.insert("run_commands", &run_commands);
        }

        Ok(tera.render("dockerfile", &context)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PackageJson, Engines};

    #[test]
    fn test_dockerfile_uses_engines_node_version() {
        let root_package = PackageJson {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            dependencies: None,
            dev_dependencies: None,
            engines: Some(Engines {
                node: Some("18.19.0".to_string()),
            }),
        };

        let generator = DockerfileGenerator::new(root_package);
        let dockerfile = generator.generate(Path::new("apps/api")).unwrap();

        assert!(dockerfile.contains("FROM node:18.19.0-alpine as builder"));
        assert!(dockerfile.contains("FROM node:18.19.0-alpine"));
    }

    #[test]
    fn test_dockerfile_uses_default_node_version() {
        let generator = DockerfileGenerator::default();
        let dockerfile = generator.generate(Path::new("apps/api")).unwrap();

        assert!(dockerfile.contains("FROM node:20-alpine as builder"));
        assert!(dockerfile.contains("FROM node:20-alpine"));
    }

    #[test]
    fn test_dockerfile_strips_v_prefix() {
        let root_package = PackageJson {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            dependencies: None,
            dev_dependencies: None,
            engines: Some(Engines {
                node: Some("v16.14.0".to_string()),
            }),
        };

        let generator = DockerfileGenerator::new(root_package);
        let dockerfile = generator.generate(Path::new("apps/api")).unwrap();

        assert!(dockerfile.contains("FROM node:16.14.0-alpine as builder"));
        assert!(dockerfile.contains("FROM node:16.14.0-alpine"));
    }
} 