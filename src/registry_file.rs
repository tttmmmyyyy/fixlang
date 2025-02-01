use serde::Deserialize;

#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct RegistryFile {
    #[serde(default)]
    pub projects: Vec<RegistryProject>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct RegistryProject {
    pub name: String,
    pub git: String,
    #[serde(default)]
    pub description: String,
}
