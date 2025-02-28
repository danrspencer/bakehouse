use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tera::{self, Map};

use crate::dockerfile::DockerfileTemplate;

// Define traits for package information
pub trait PackageInfo {
    fn name(&self) -> &str;
    fn path(&self) -> &PathBuf;
    fn version(&self) -> &str;
    fn dependencies(&self) -> &HashSet<String>;
    fn dockerfile_template(&self) -> &DockerfileTemplate;

    fn sanitized_name(&self) -> String {
        sanitized_name(self.name())
    }
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
    pub dependencies: HashMap<String, PathBuf>,
    pub dockerfile_template: DockerfileTemplate,
}

pub struct Workspace {
    pub name: String,
    pub path: PathBuf,
    pub version: String,
    pub dockerfile_template: DockerfileTemplate,
    pub packages: HashMap<String, Package>,
}

fn sanitized_name(name: &str) -> String {
    name.replace('@', "").replace('/', "-").to_lowercase()
}

impl Workspace {
    pub fn new<W: WorkspaceInfo>(workspace_info: W) -> Self {
        let root = workspace_info.root_package();

        let mut workspace = Self {
            name: root.sanitized_name(),
            path: root.path().clone(),
            version: root.version().to_string(),
            dockerfile_template: root.dockerfile_template().clone(),
            packages: HashMap::new(),
        };

        let mut package_paths: HashMap<String, PathBuf> = HashMap::new();

        for package_info in workspace_info.packages() {
            package_paths.insert(package_info.sanitized_name(), package_info.path().clone());
        }

        for package_info in workspace_info.packages() {
            let mut deps = HashMap::new();

            for dep_name in package_info.dependencies() {
                let sanitized_dep_name = sanitized_name(dep_name);
                if let Some(dep_path) = package_paths.get(&sanitized_dep_name) {
                    deps.insert(sanitized_dep_name, dep_path.clone());
                }
            }

            let mut dockerfile_template = package_info.dockerfile_template().clone();

            dockerfile_template.context.insert("path", &package_info.path().strip_prefix(&workspace.path)
            .unwrap_or(package_info.path())
            .to_string_lossy()
            .to_string());
            
            // Convert dependencies to a format that Tera can iterate over directly
            let deps_vec: Vec<_> = deps.iter().map(|(name, path)| {
                let relative_path = path.strip_prefix(&workspace.path)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                
                tera::Value::Object({
                    let mut m = Map::new();
                    m.insert("name".to_string(), tera::Value::String(name.clone()));
                    m.insert("path".to_string(), tera::Value::String(relative_path));
                    m
                })
            }).collect();
            
            dockerfile_template.context.insert("dependencies", &deps_vec);

            // TODO - I don't think the workspace should capture this dependency here, we should do it as part of
            // of the Tera file generation
            deps.insert(root.sanitized_name(), root.path().clone());

            workspace.add_package(
                package_info.sanitized_name(),
                package_info.path().clone(),
                package_info.version().to_string(),
                deps,
                dockerfile_template
            )
        }

        workspace
    }

    fn add_package(
        &mut self,
        name: String,
        path: PathBuf,
        version: String,
        dependencies: HashMap<String, PathBuf>,
        dockerfile_template: DockerfileTemplate,
    ) {
        let package = Package {
            name: name.clone(),
            path,
            version,
            dependencies,
            dockerfile_template,
        };
        self.packages.insert(name, package);
    }

    pub fn get_dependencies(&self, package_name: &str) -> Vec<(String, PathBuf)> {
        if let Some(package) = self.packages.get(package_name) {
            package
                .dependencies
                .iter()
                .map(|(name, path)| (name.clone(), path.clone()))
                .collect()
        } else {
            vec![]
        }
    }
}
