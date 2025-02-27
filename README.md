# Bakehouse 🍞

A CLI tool designed to generate Docker Bake files for PNPM monorepos. Bakehouse simplifies the containerization process for complex monorepo projects by automatically generating Docker Bake configurations and Dockerfiles.

## Features

- Automatic Docker Bake file generation for PNPM workspaces
- Smart dependency resolution for monorepo packages
- Template-based Dockerfile generation
- Support for custom build configurations
- Built with Rust for maximum performance and reliability

## Prerequisites

- Rust (latest stable version)
- Docker with Buildx support
- PNPM package manager
- Just command runner (optional, for convenience commands)

## Installation

Clone the repository and build from source:

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
1. Scan your PNPM workspace
2. Generate appropriate Dockerfile.bake files for each package
3. Create a docker-bake.hcl file with the build configuration

### Using Just Commands

The project includes several convenience commands via Just:

```bash
# List all available commands
just

# Generate bake files for the sample project
just generate-sample

# Clean up generated files in the sample folder
just clean-sample

# Generate and build the sample project
just bake-sample
```

## Project Structure

```
.
├── src/           # Source code
├── sample/        # Sample project for testing
├── Cargo.toml     # Rust dependencies and project configuration
├── Justfile       # Convenience commands
└── README.md      # This file
```

## Development

The project is built with Rust and uses several key dependencies:

- `tokio` - Async runtime
- `serde` - Serialization/deserialization
- `clap` - Command line argument parsing
- `tera` - Template rendering
- `hcl-rs` - HCL file parsing/generation

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Add your chosen license here]

## Support

[Add support information here] 