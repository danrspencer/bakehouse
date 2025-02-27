use std::{collections::{hash_map, HashMap}, path::PathBuf};

use anyhow::Result;
use bake::Target;
use clap::Parser;
mod bake;
mod workspace;
mod dockerfile;
mod resolvers;

use workspace::Workspace;

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

    // Load the PNPM workspace
    let pnpm_workspace = resolvers::pnpm::load_workspace(&workspace_root)?;

    // In main(), get the root package before moving pnpm_workspace into Workspace
    let workspace = Workspace::new(pnpm_workspace);

    // Debug: Print discovered packages
    println!("\nDiscovered packages:");
    for (name, package) in &workspace.packages {
        println!("- {} at {}", name, package.path.display());
    }

    // Debug: Print dependencies
    println!("\nPackage dependencies:");
    for (name, _) in &workspace.packages {
        let deps = workspace.get_dependencies(&name);
        println!("- {} depends on: {:?}", name, deps);
    }
    
    // Create bake file
    let mut bake_file = bake::BakeFile::new();

    let dockerfile_path = workspace.path.join("Dockerfile.bake");
    let workspace_sanitized_name = sanitize_docker_name(&workspace.name);

    // TODO - reduce this duplication!!
    // Generate Dockerfile if it doesn't exist
    if !dockerfile_path.exists() {
        // TODO - fix this clone
        let dockerfile_content = workspace.dockerfile_template.generate_dockerfile();
        std::fs::write(&dockerfile_path, dockerfile_content.unwrap())?;
        println!("Generated Dockerfile.bake for package {}", workspace.name);
    }

    bake_file.add_target(workspace_sanitized_name.clone(), Target::new(&workspace.path, &workspace_root, "Dockerfile.bake".to_string(),
            vec![format!("{}:{}", &workspace_sanitized_name, workspace.version)],
            vec![], HashMap::new()));

    // Add targets for each package
    for (name, package) in &workspace.packages {
        let dockerfile_path = package.path.join("Dockerfile.bake");
        let sanitized_name = sanitize_docker_name(&name);
        
        // Generate Dockerfile if it doesn't exist
        if !dockerfile_path.exists() {
            // TODO - fix this clone
            let dockerfile_content = package.dockerfile_template.generate_dockerfile();
            std::fs::write(&dockerfile_path, dockerfile_content.unwrap())?;
            println!("Generated Dockerfile.bake for package {}", name);
        }

        let dependencies = workspace.get_dependencies(&name)
            .into_iter()
            .map(|dep| sanitize_docker_name(&dep))
            .collect::<Vec<_>>();

        let target = Target::new(
            &package.path,
            &workspace_root,
            "Dockerfile.bake".to_string(),
            vec![format!("{}:{}", sanitized_name, package.version)],
            dependencies,
            { 
                let mut map = HashMap::new();
                map.insert(workspace_sanitized_name.clone(), format!("target:{}", workspace_sanitized_name));
                map
             }
        );

        bake_file.add_target(sanitized_name, target);
    }

    // Add a default group with all targets
    bake_file.add_group(
        "default".to_string(),
        workspace.packages.clone()
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