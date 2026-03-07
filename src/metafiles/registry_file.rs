use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct RegistryFile {
    #[serde(default)]
    pub projects: Vec<RegistryProject>,
}

impl RegistryFile {
    pub fn print_projects(&self) {
        let mut projects = self.projects.clone();
        projects.sort_by(|a, b| a.name.cmp(&b.name));
        for project in &projects {
            project.print();
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct RegistryProject {
    pub name: String,
    pub git: String,
    #[serde(default)]
    pub description: String,
}

impl RegistryProject {
    pub fn print(&self) {
        println!("- {} ({})", self.name.bright_cyan(), self.git);
        if !self.description.is_empty() {
            println!("    - {}", self.description);
        }
    }
}
