use std::path::Path;
use anyhow::Result;

pub struct DockerfileGenerator {
    pub base_image: String,
    pub pnpm_version: String,
}

impl Default for DockerfileGenerator {
    fn default() -> Self {
        Self {
            base_image: "node:20-slim".to_string(),
            pnpm_version: "8.15.1".to_string(),
        }
    }
}

impl DockerfileGenerator {
    pub fn generate(&self, package_path: &Path) -> Result<String> {
        let dockerfile = format!(
            r#"FROM {base_image} as builder

# Install pnpm
RUN corepack enable && corepack prepare pnpm@{pnpm_version} --activate

WORKDIR /app

# Copy root package.json and pnpm workspace files
COPY pnpm-lock.yaml ./
COPY pnpm-workspace.yaml ./
COPY package.json ./

# Copy all package.json files to ensure proper dependency installation
COPY packages/*/package.json ./packages/

# Install dependencies
RUN pnpm install --frozen-lockfile

# Copy source files
COPY . .

# Build the package
RUN pnpm --filter ./packages/{package_name} build

FROM {base_image}

WORKDIR /app

# Copy built assets from builder
COPY --from=builder /app/packages/{package_name}/dist ./dist
COPY --from=builder /app/packages/{package_name}/package.json ./

# Install production dependencies
RUN corepack enable && \
    corepack prepare pnpm@{pnpm_version} --activate && \
    pnpm install --prod

CMD ["node", "dist/index.js"]
"#,
            base_image = self.base_image,
            pnpm_version = self.pnpm_version,
            package_name = package_path
                .file_name()
                .unwrap()
                .to_string_lossy()
        );

        Ok(dockerfile)
    }
} 