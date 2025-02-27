use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::dockerfile::DockerfileTemplate;

// Define traits for package information
pub trait PackageInfo {
    fn name(&self) -> &str;
    fn path(&self) -> &PathBuf;
    fn version(&self) -> &str;
    fn dependencies(&self) -> &HashSet<String>;
    fn dockerfile_template(&self) -> &DockerfileTemplate;
}

pub trait WorkspaceInfo {    
    fn root_package(&self) -> &dyn PackageInfo;
    fn packages(&self) -> Vec<&dyn PackageInfo>;
}

#[derive(Clone, Debug)]
pub struct Package {
    pub name: String,
    pub path: PathBuf,
    pub version: String,
    pub dependencies: HashSet<String>,
    pub dockerfile_template: DockerfileTemplate
}

pub struct Workspace {
    pub name: String,
    pub path: PathBuf,
    pub version: String,
    pub dockerfile_template: DockerfileTemplate,

    pub packages: HashMap<String, Package>,
}

impl Workspace {
    pub fn new<W: WorkspaceInfo>(workspace_info: W) -> Self {
        let root = workspace_info.root_package();

        let mut workspace = Self {
            name: root.name().to_string(),
            path: root.path().clone(),
            version: root.version().to_string(),
            dockerfile_template: root.dockerfile_template().clone(),
            packages: HashMap::new(),
        };

        // Add workspace packages with root as dependency
        for package_info in workspace_info.packages() {
            let mut deps = package_info.dependencies().clone();
            deps.insert(root.name().to_string());
            workspace.add_package(
                package_info.name().to_string(),
                package_info.path().clone(),
                package_info.version().to_string(),
                deps,
                package_info.dockerfile_template().clone()
            )
        }

        workspace
    }

    // Keep this as an internal method
    fn add_package(&mut self, 
        name: String, 
        path: PathBuf, 
        version: String,
        dependencies: HashSet<String>,
        dockerfile_template: DockerfileTemplate,
    ) {
        let package = Package {
            name: name.clone(),
            path,
            version,
            dependencies,
            dockerfile_template
        };
        self.packages.insert(name, package);
    }

    // TODO - I'm not sure we need this function
    pub fn get_dependencies(&self, package_name: &str) -> Vec<String> {
        if let Some(package) = self.packages.get(package_name) {
            package.dependencies.iter()
                .filter(|dep| self.packages.contains_key(*dep) || &self.name == *dep)
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }
} 