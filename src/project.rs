use std::{
    fs::File,
    io::{ErrorKind, Read},
    path::PathBuf,
};

use serde::Deserialize;

use crate::{error::Errors, Configuration, SourceFile, Span, PROJECT_FILE_PATH};

#[derive(Deserialize, Default)]
pub struct ProjectFile {
    pub files: Vec<String>,
}

impl ProjectFile {
    // Read the project file at `PROJECT_FILE_PATH` and return the `ProjectFile`.
    // - err_if_not_found: If true, raise error if the file does not exist. Otherwise, return the empty `ProjectFile` in that case.
    pub fn read_file(err_if_not_found: bool) -> Result<Self, Errors> {
        // Open a file exists at the path `PROJECT_FILE_PATH`.
        let res = File::open(PROJECT_FILE_PATH);
        if res.is_err() {
            let err = res.err().unwrap();
            match err.kind() {
                ErrorKind::NotFound => {
                    // If the file does not exist, return the empty `ProjectFile`.
                    if err_if_not_found {
                        return Err(Errors::from_msg(&format!(
                            "File \"{}\" not found.",
                            PROJECT_FILE_PATH
                        )));
                    } else {
                        return Ok(Self::default());
                    }
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
                let input = SourceFile::from_file_path(PathBuf::from(PROJECT_FILE_PATH));
                let (start, end) = e.span().map(|r| (r.start, r.end)).unwrap_or((0, 0));
                let span = Span { start, end, input };
                return Err(Errors::from_msg_srcs(
                    &format!(
                        "Failed to parse file \"{}\": {}",
                        PROJECT_FILE_PATH,
                        e.message()
                    ),
                    &[&Some(span)],
                ));
            }
        }
    }

    // Update a configuration from a project file.
    pub fn set_config_from_proj_file(config: &mut Configuration, proj_file: &ProjectFile) {
        let mut files = proj_file.files.iter().map(|f| PathBuf::from(f)).collect();
        config.source_files.append(&mut files);
    }
}
