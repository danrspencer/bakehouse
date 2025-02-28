use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};

#[derive(Debug, Clone)]
pub struct DockerfileTemplate {
    template: Tera,
    pub context: Context,
}

impl DockerfileTemplate {
    pub fn new(template_path: &PathBuf) -> Result<Self> {
        let template_content = fs::read_to_string(&template_path)?;

        let mut tera = Tera::default();
        tera.add_raw_template("dockerfile", &template_content)?;

        Ok(Self {
            template: tera,
            context: Context::new(),
        })
    }

    // pub fn generate_dockerfile(&self) -> Result<String> {
    //

    //     let mut context = Context::new();
    //     for (key, value) in self.context_items.iter() {
    //         context.insert(key, value);
    //     }

    //     Ok(tera.render("dockerfile", &context)?)
    // }

    pub fn render(&self) -> Result<String> {
        Ok(self.template.render("dockerfile", &self.context)?)
    }
}
