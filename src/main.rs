use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
mod bake;
mod workspace;
mod dockerfile;

use workspace::Workspace;
use dockerfile::DockerfileGenerator;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to the PNPM workspace root
    #[arg(short, long, default_value = ".")]
    workspace: PathBuf,

    /// Output path for the Docker Bake file (relative to workspace root)
    #[arg(short, long, default_value = "docker-bake.hcl")]
    output: PathBuf,

    /// Output format (hcl or json)
    #[arg(short, long, default_value = "hcl")]
    format: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PnpmWorkspace {
    packages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PackageJson {
    name: String,
    version: String,
    dependencies: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<std::collections::HashMap<String, String>>,
    engines: Option<Engines>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Engines {
    node: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Recipe {
    name: String,
    ingredients: Vec<Ingredient>,
    preparation_time: u32, // in minutes
}

#[derive(Debug, Serialize, Deserialize)]
struct Ingredient {
    name: String,
    quantity: f32,
    unit: String,
}

// Update the sanitize function to handle both target names and image tags
fn sanitize_docker_name(name: &str) -> String {
    name.replace('@', "")
        .replace('/', "-")
        .to_lowercase()
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Get absolute paths for both workspace and output
    let workspace_root = std::fs::canonicalize(&args.workspace)?;
    let output_path = workspace_root.join(&args.output);

    // Read root package.json
    let root_package_json: PackageJson = serde_json::from_str(
        &std::fs::read_to_string(workspace_root.join("package.json"))
            .context("Failed to read root package.json")?
    )?;

    // Read pnpm-workspace.yaml - use workspace_root instead of args.workspace
    let workspace_file = workspace_root.join("pnpm-workspace.yaml");
    let workspace_content = std::fs::read_to_string(&workspace_file)
        .context("Failed to read pnpm-workspace.yaml")?;
    
    let workspace_config: PnpmWorkspace = serde_yaml::from_str(&workspace_content)
        .context("Failed to parse pnpm-workspace.yaml")?;

    println!("Found workspace configuration:");
    for package_glob in &workspace_config.packages {
        println!("- {}", package_glob);
    }

    // Discover packages and build dependency graph - use workspace_root
    let mut workspace = Workspace::new();
    workspace.discover_packages(&workspace_root, &workspace_config.packages)?;
    
    // Debug: Print discovered packages
    println!("\nDiscovered packages:");
    for (name, package) in workspace.get_packages() {
        println!("- {} at {}", name, package.path.display());
    }

    workspace.build_dependency_graph();

    // Debug: Print dependencies
    println!("\nPackage dependencies:");
    for (name, _) in workspace.get_packages() {
        let deps = workspace.get_dependencies(name);
        println!("- {} depends on: {:?}", name, deps);
    }

    let dockerfile_generator = DockerfileGenerator::new(root_package_json.clone());
    
    // Create bake file
    let mut bake_file = bake::BakeFile::new();

    // Add root target first
    let node_version = dockerfile_generator.get_node_version();
    bake_file.add_root_target(&workspace_root, node_version);
    
    // Add targets for each package
    for (name, package) in workspace.get_packages() {
        let dockerfile_path = package.path.join("Dockerfile.bake");
        let sanitized_name = sanitize_docker_name(&name);
        
        // Generate Dockerfile if it doesn't exist
        if !dockerfile_path.exists() {
            let dockerfile_content = dockerfile_generator.generate(&package.path)?;
            std::fs::write(&dockerfile_path, dockerfile_content)?;
            println!("Generated Dockerfile.bake for package {}", name);
        }

        let dependencies = workspace.get_dependencies(name)
            .into_iter()
            .map(|dep| sanitize_docker_name(&dep))
            .collect::<Vec<_>>();
        
        // Add root as a dependency
        let mut all_deps = vec!["root".to_string()];
        all_deps.extend(dependencies);

        let target = bake::Target::new(
            &package.path,
            &workspace_root,
            "Dockerfile.bake".to_string(),
            vec![format!("{}:{}", sanitized_name, package.package_json.version)],
            all_deps,
        );

        bake_file.add_target(sanitized_name, target);
    }

    // Add a default group with all targets
    bake_file.add_group(
        "default".to_string(),
        workspace.get_packages()
            .keys()
            .map(|name| sanitize_docker_name(name))
            .collect(),
    );

    // Write the bake file to the workspace root
    let bake_content = match args.format.as_str() {
        "json" => serde_json::to_string_pretty(&bake_file)?,
        "hcl" => bake_file.to_hcl(),
        _ => return Err(anyhow::anyhow!("Unsupported format: {}", args.format)),
    };
    
    std::fs::write(&output_path, bake_content)?;
    
    println!("Generated Docker Bake file at: {}", output_path.display());

    Ok(())
} 