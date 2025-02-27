use std::path::Path;
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

    pub fn generate(&self, package_path: &Path) -> Result<String> {
        let node_version = self.get_node_version();
        let dockerfile = format!(
            r#"FROM node:{node_version}-alpine as builder

# Install pnpm
RUN corepack enable && corepack prepare pnpm@{pnpm_version} --activate

WORKDIR /app

# Copy root package.json and pnpm workspace files
COPY pnpm-lock.yaml ./
COPY pnpm-workspace.yaml ./
COPY package.json ./

# Copy all package.json files
COPY packages/*/package.json ./packages/
COPY apps/*/package.json ./apps/

# Install dependencies
RUN pnpm install --frozen-lockfile

# Copy source files
COPY . .

# Build the package
RUN pnpm --filter {package_name} build

FROM node:{node_version}-alpine

WORKDIR /app

# Copy built assets from builder
COPY --from=builder /app/{package_name}/dist ./dist
COPY --from=builder /app/{package_name}/package.json ./

# Install production dependencies
RUN corepack enable && \
    corepack prepare pnpm@{pnpm_version} --activate && \
    pnpm install --prod

CMD ["node", "dist/index.js"]
"#,
            node_version = node_version,
            pnpm_version = self.pnpm_version,
            package_name = package_path
                .strip_prefix(package_path.parent().unwrap().parent().unwrap())
                .unwrap()
                .to_string_lossy()
        );

        Ok(dockerfile)
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