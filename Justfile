# List available commands
default:
    @just --list

# Clean up all files generated by the CLI in the sample folder
clean-sample:
    rm -f sample/docker-bake.hcl
    rm -f sample/apps/*/Dockerfile.bake
    rm -f sample/packages/*/Dockerfile.bake
    rm -rf sample/**/dist

# Generate bake files for sample app
generate-sample: clean-sample
    cargo run -- --workspace ./sample

# Generate and build the sample app
bake-sample: generate-sample
    # Build using docker buildx bake
    cd sample && docker buildx bake --load