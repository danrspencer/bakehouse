use serde::{Deserialize, Serialize};



#[derive(Debug, Serialize, Deserialize)]
pub struct PnpmWorkspace {
    pub packages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageJson {
    pub name: String,
    pub version: String,
    pub dependencies: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<std::collections::HashMap<String, String>>,
   pub engines: Option<Engines>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Engines {
    pub node: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub ingredients: Vec<Ingredient>,
    pub preparation_time: u32, // in minutes
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub quantity: f32,
    pub unit: String,
}