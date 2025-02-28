# Bakehouse 🍞

A CLI tool that leverages Docker BuildKit and Bake to create highly optimized, cache-efficient build systems for monorepos. Currently focused on PNPM workspaces, with plans to expand to other package managers and languages.

## Why Bakehouse?

Building monorepos in Docker can be challenging:
- Complex dependency graphs need careful handling
- Build caching is critical for performance
- Traditional Dockerfile approaches often rebuild more than necessary

Bakehouse solves these problems by:
- Automatically analyzing your workspace dependency graph
- Generating optimized Docker Bake configurations that maximize cache usage
- Creating BuildKit-optimized Dockerfiles for each package
- Ensuring dependencies are built only when necessary

## Features

- **Intelligent Dependency Analysis**: Automatically maps your monorepo's dependency graph
- **BuildKit Optimization**: Generates Dockerfiles that leverage BuildKit's advanced caching features
- **Bake Configuration**: Creates HCL-based Docker Bake files for sophisticated multi-stage builds
- **Cache Efficiency**: Ensures each package's build cache can be reused by its dependents
- **PNPM Support**: Currently optimized for PNPM workspaces (more package managers coming soon)

## Prerequisites

- Docker with BuildKit support enabled
- PNPM for package management
- Rust (for building from source)
- Just command runner (optional, for convenience commands)

## Installation

```bash
git clone https://github.com/yourusername/bakehouse.git
cd bakehouse
cargo build --release
```

The binary will be available at `target/release/bakehouse`.

## Usage

Basic usage:

```bash
bakehouse --workspace ./path/to/your/workspace
```

This will:
1. Analyze your workspace dependency graph
2. Generate BuildKit-optimized Dockerfiles for each package
3. Create a `docker-bake.hcl` file with the optimal build configuration

### Building Your Project

Once Bakehouse has generated the configuration:

```bash
# Build all packages
docker buildx bake

# Build a specific package (will automatically build dependencies)
docker buildx bake package-name

# Build with maximum parallelization
docker buildx bake --progress=plain
```

### Development Commands

The project includes several convenience commands via Just:

```bash
# List all available commands
just

# Generate bake files for the sample project
just generate-sample

# Clean up generated files
just clean-sample

# Generate and build the sample project
just bake-sample
```

## How It Works

1. **Workspace Analysis**: Bakehouse scans your monorepo to understand the dependency relationships between packages
2. **Dockerfile Generation**: Creates optimized Dockerfiles that properly handle dependencies and maximize cache usage
3. **Bake Configuration**: Generates a `docker-bake.hcl` file that:
   - Defines build targets for each package
   - Sets up proper dependency ordering
   - Configures BuildKit's advanced caching features
   - Enables parallel building where possible

## Project Structure

```
.
├── src/           # Source code
├── sample/        # Sample project for testing
├── Cargo.toml     # Rust dependencies and project configuration
├── Justfile       # Convenience commands
└── README.md      # This file
```

## Future Plans

- Support for additional package managers (Yarn, npm, pnpm)
- Language-specific optimizations
- Custom build stage templates
- Remote cache configuration
- Build matrix support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[MIT License](LICENSE) 