use crate::{
    constants::{
        OPTIMIZATION_LEVEL_DEFAULT, OPTIMIZATION_LEVEL_MINIMUM, OPTIMIZATION_LEVEL_NONE,
        OPTIMIZATION_LEVEL_SEPARATED,
    },
    error::Errors,
    Configuration, FixOptimizationLevel, LinkType, SourceFile, Span, PROJECT_FILE_PATH,
};
use serde::Deserialize;
use std::{
    fs::File,
    io::{ErrorKind, Read},
    path::PathBuf,
};

#[derive(Deserialize, Default)]
pub struct ProjectFile {
    pub build: ProjectFileBuild,
    pub dependencies: Vec<ProjectFileDependency>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileBuild {
    files: Vec<String>,
    static_links: Option<Vec<String>>,
    dynamic_links: Option<Vec<String>>,
    library_paths: Option<Vec<String>>,
    debug: Option<bool>,
    opt_level: Option<String>,
    output: Option<String>,
    threaded: Option<bool>,
}

pub struct ProjectFileDependency {
    pub name: String,
    pub path: Option<String>,
    pub git: Option<ProjectFileDependencyGit>,
    pub version: String,
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
                        return Err(Errors::from_msg(format!(
                            "File \"{}\" not found.",
                            PROJECT_FILE_PATH
                        )));
                    } else {
                        return Ok(Self::default());
                    }
                }
                _ => {
                    // If the file exists but cannot be opened, raise error.
                    return Err(Errors::from_msg(format!(
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
            return Err(Errors::from_msg(format!(
                "Failed to read file \"{}\": {:?}",
                PROJECT_FILE_PATH, e
            )));
        }

        // Parse the content as a toml file and return the `ProjectFile`.
        match toml::from_str(&content) {
            Ok(v) => Ok(v),
            Err(e) => {
                let (start, end) = e.span().map(|r| (r.start, r.end)).unwrap_or((0, 0));
                let span = ProjectFile::project_file_span(start, end);
                return Err(Errors::from_msg_srcs(
                    format!(
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
    pub fn set_config_from_proj_file(
        config: &mut Configuration,
        proj_file: &ProjectFile,
    ) -> Result<(), Errors> {
        // Append source files.
        let mut files = proj_file
            .build
            .files
            .iter()
            .map(|f| PathBuf::from(f))
            .collect();
        config.source_files.append(&mut files);

        // Append static libraries.
        if let Some(static_libs) = proj_file.build.static_links.as_ref() {
            config.linked_libraries.append(
                &mut static_libs
                    .iter()
                    .map(|lib_name| (lib_name.clone(), LinkType::Static))
                    .collect(),
            );
        }

        // Append dynamic libraries.
        if let Some(dynamic_libs) = proj_file.build.dynamic_links.as_ref() {
            config.linked_libraries.append(
                &mut dynamic_libs
                    .iter()
                    .map(|lib_name| (lib_name.clone(), LinkType::Dynamic))
                    .collect(),
            );
        }

        // Append library search paths.
        if let Some(lib_paths) = proj_file.build.library_paths.as_ref() {
            config
                .library_search_paths
                .append(&mut lib_paths.iter().map(|p| PathBuf::from(p)).collect());
        }

        // Set debug mode.
        if let Some(debug) = proj_file.build.debug {
            config.debug_info = debug;
        }

        // Set optimization level.
        if let Some(opt_level) = proj_file.build.opt_level.as_ref() {
            match opt_level.as_str() {
                OPTIMIZATION_LEVEL_NONE => {
                    config.fix_opt_level = FixOptimizationLevel::None;
                }
                OPTIMIZATION_LEVEL_MINIMUM => {
                    config.fix_opt_level = FixOptimizationLevel::Minimum;
                }
                OPTIMIZATION_LEVEL_SEPARATED => {
                    config.fix_opt_level = FixOptimizationLevel::Separated;
                }
                OPTIMIZATION_LEVEL_DEFAULT => {
                    config.fix_opt_level = FixOptimizationLevel::Default;
                }
                _ => {
                    return Err(Errors::from_msg_srcs(
                        format!("Unknown optimization level: \"{}\"", opt_level),
                        &[&Some(ProjectFile::project_file_span(0, 0))],
                    ));
                }
            }
        }

        // Set output file.
        if let Some(output) = proj_file.build.output.as_ref() {
            config.out_file_path = Some(PathBuf::from(output));
        }

        // Set is threaded.
        if let Some(threaded) = proj_file.build.threaded {
            config.threaded = config.threaded || threaded;
        }
        Ok(())
    }

    // Create span for a range in the project file.
    fn project_file_span(start: usize, end: usize) -> Span {
        let input = SourceFile::from_file_path(PathBuf::from(PROJECT_FILE_PATH));
        Span { start, end, input }
    }
}
