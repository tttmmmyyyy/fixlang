use std::{
    fs::File,
    io::{ErrorKind, Read},
};

use serde::Deserialize;

use crate::{error::Errors, PROJECT_FILE_PATH};

#[derive(Deserialize, Default)]
pub struct ProjectFile {
    pub files: Vec<String>,
}

impl ProjectFile {
    pub fn read_file() -> Result<Self, Errors> {
        // Open a file exists at the path `PROJECT_FILE_PATH`.
        let res = File::open(PROJECT_FILE_PATH);
        if res.is_err() {
            let err = res.err().unwrap();
            match err.kind() {
                ErrorKind::NotFound => {
                    // If the file does not exist, return the empty `ProjectFile`.
                    return Ok(Self::default());
                }
                _ => {
                    // If the file exists but cannot be opened, raise error.
                    return Err(Errors::from_msg(&format!(
                        "Failed to open file \"{}\": {:?}",
                        PROJECT_FILE_PATH, err
                    )));
                }
            }
        }
        let mut file = res.unwrap();

        // Read the content of the file.
        let mut content = String::new();
        if let Err(e) = file.read_to_string(&mut content) {
            return Err(Errors::from_msg(&format!(
                "Failed to read file \"{}\": {:?}",
                PROJECT_FILE_PATH, e
            )));
        }

        // Parse the content as a toml file and return the `ProjectFile`.
        match toml::from_str(&content) {
            Ok(v) => Ok(v),
            Err(e) => {
                return Err(Errors::from_msg(&format!(
                    "Failed to parse file \"{}\": {:?}",
                    PROJECT_FILE_PATH, e
                )))
            }
        }
    }
}
