use std::{collections::HashMap, path::PathBuf};
use tera::{Tera, Context};
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct DockerfileTemplate {
    pub template_path: PathBuf,
    pub context_items: HashMap<String, String>
}

impl DockerfileTemplate {
    pub fn generate_dockerfile(&self) -> Result<String> {
        let template_content = fs::read_to_string(&self.template_path)?;
        
        let mut tera = Tera::default();
        tera.add_raw_template("dockerfile", &template_content)?;
        
        let mut context = Context::new();
        for (key, value) in self.context_items.iter() {
            context.insert(key, value);
        }
    
    
        Ok(tera.render("dockerfile", &context)?)
    }
}