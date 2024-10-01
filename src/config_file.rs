use serde::Deserialize;

use crate::{error::Errors, DEFAULT_REGISTRY, FIX_CONFIG_FILE_NAME};

// `.fixconfig.toml` file structure
#[derive(Deserialize)]
pub struct ConfigFile {
    // URLs to registries.
    // The former element is higher priority than the latter.
    #[serde(default)]
    pub registries: Vec<String>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            registries: vec![DEFAULT_REGISTRY.to_string()],
        }
    }
}

impl ConfigFile {
    // Read the content of file `FIX_CONFIG_FILE_NAME` at the home directory, and parse it as `ConfigFile`.
    // TODO: Should we consider the same file in the current directory?
    pub fn load() -> Result<ConfigFile, Errors> {
        let home_dir = dirs::home_dir();
        if home_dir.is_none() {
            return Err(Errors::from_msg("Cannot get home directory.".to_string()));
        }
        let home_dir = home_dir.unwrap();
        let config_file_path = home_dir.join(FIX_CONFIG_FILE_NAME);

        // If the config file does not exist, return a default `ConfigFile`.
        if !config_file_path.exists() {
            return Ok(ConfigFile::default());
        }

        let config_file_content = std::fs::read_to_string(&config_file_path).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to read \"{}\": {:?}",
                config_file_path.to_string_lossy(),
                e
            ))
        })?;
        let mut config_file: ConfigFile = toml::from_str(&config_file_content).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to parse \"{}\": {:?}",
                config_file_path.to_string_lossy(),
                e
            ))
        })?;

        // Merge with the default config file.
        config_file.merge(ConfigFile::default());
        Ok(config_file)
    }

    // Merge two config files.
    // Fields of `self` have higher priority than `other`.
    pub fn merge(&mut self, other: ConfigFile) {
        self.registries.extend(other.registries);
    }
}
