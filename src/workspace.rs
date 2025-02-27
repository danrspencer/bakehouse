use std::collections::HashMap;
use std::path::PathBuf;

// Define a trait for package information
pub trait PackageInfo {
    fn name(&self) -> &str;
    fn path(&self) -> &PathBuf;
    fn version(&self) -> &str;
    fn dependencies(&self) -> &std::collections::HashSet<String>;
}

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub path: PathBuf,
    pub version: String,
    pub dependencies: std::collections::HashSet<String>,
}

pub struct Workspace {
    packages: HashMap<String, Package>,
}

impl Workspace {
    pub fn new<I, T>(packages: I) -> Self 
    where 
        I: IntoIterator<Item = T>,
        T: PackageInfo,
    {
        let mut workspace = Self {
            packages: HashMap::new(),
        };

        for package_info in packages {
            workspace.add_package(
                package_info.name().to_string(),
                package_info.path().clone(),
                package_info.version().to_string(),
                package_info.dependencies().clone(),
            );
        }

        workspace
    }

    // Keep this as an internal method
    fn add_package(&mut self, 
        name: String, 
        path: PathBuf, 
        version: String,
        dependencies: std::collections::HashSet<String>
    ) {
        let package = Package {
            name: name.clone(),
            path,
            version,
            dependencies,
        };
        self.packages.insert(name, package);
    }

    pub fn get_packages(&self) -> &HashMap<String, Package> {
        &self.packages
    }

    pub fn get_dependencies(&self, package_name: &str) -> Vec<String> {
        if let Some(package) = self.packages.get(package_name) {
            package.dependencies.iter()
                .filter(|dep| self.packages.contains_key(*dep))
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }
} 