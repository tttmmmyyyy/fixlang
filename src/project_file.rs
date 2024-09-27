use crate::{
    dependency_lockfile::{DependecyLockFile, ProjectSource},
    error::Errors,
    registry_file::RegistryFile,
    Configuration, ExtraCommand, FixOptimizationLevel, LinkType, SourceFile, Span, SubCommand,
    LOCK_FILE_PATH, PROJECT_FILE_PATH, TRY_FIX_RESOLVE,
};
use semver::{Version, VersionReq};
use serde::Deserialize;
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

// The name of a project.
pub type ProjectName = String;

// The `general` section of the project file.
#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileGeneral {
    // The name of the project.
    pub name: ProjectName,
    // The version of the project.
    // Use `version` method to get the value validated as semver.
    pub version: String,
    // The description of the project.
    #[allow(unused)]
    pub description: Option<String>,
    // `registries` section
    #[serde(default)]
    pub registries: Vec<String>,
    // The authors of the project.
    #[allow(unused)]
    pub authors: Option<Vec<String>>,
    // The license of the project.
    #[allow(unused)]
    pub license: Option<String>,
}

impl ProjectFileGeneral {
    // Get the version.
    pub fn version(&self) -> Version {
        Version::parse(&self.version).unwrap()
    }
}

// The `build` section of the project file.
#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileBuild {
    files: Vec<PathBuf>,
    #[serde(default)]
    objects: Vec<PathBuf>,
    static_links: Option<Vec<String>>,
    dynamic_links: Option<Vec<String>>,
    library_paths: Option<Vec<PathBuf>>,
    threaded: Option<bool>,
    debug: Option<bool>,
    opt_level: Option<String>,
    output: Option<PathBuf>,
    #[serde(default)]
    preliminary_commands: Vec<Vec<String>>,
    test: Option<ProjectFileBuildTest>,
}

// The `build.test` section of the project file.
#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileBuildTest {
    files: Vec<PathBuf>,
    #[serde(default)]
    objects: Vec<PathBuf>,
    static_links: Option<Vec<String>>,
    dynamic_links: Option<Vec<String>>,
    library_paths: Option<Vec<PathBuf>>,
    threaded: Option<bool>,
    debug: Option<bool>,
    opt_level: Option<String>,
    #[serde(default)]
    preliminary_commands: Vec<Vec<String>>,
    memcheck: Option<bool>,
}

// The entry of `dependencies` section of the project file.
#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileDependency {
    // Name of the project.
    pub name: ProjectName,
    // Path to directory.
    pub path: Option<PathBuf>,
    // Git repository.
    pub git: Option<ProjectFileDependencyGit>,
    // Version requirement for the dependent project.
    // If None, the latest version is used.
    pub version: Option<String>,
}

impl ProjectFileDependency {
    // Get the version requirement.
    pub fn version(&self) -> VersionReq {
        match &self.version {
            Some(v) => VersionReq::parse(v).unwrap(),
            None => VersionReq::STAR,
        }
    }
}

// The `git` field of the dependency.
#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileDependencyGit {
    // The URL of the git repository.
    pub url: String,
}

// The project file.
#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectFile {
    // `general` section
    pub general: ProjectFileGeneral,
    // `build` section
    pub build: ProjectFileBuild,
    // `dependencies` section
    #[serde(default)]
    pub dependencies: Vec<ProjectFileDependency>,
    // The hash value of the project file.
    #[serde(skip)]
    pub hash: String,
    // The path to the project file.
    #[serde(skip)]
    pub path: PathBuf,
}

impl ProjectFile {
    // Read the project file at `PROJECT_FILE_PATH`.
    pub fn read_root_file() -> Result<ProjectFile, Errors> {
        let proj_file_path = Path::new(PROJECT_FILE_PATH);
        ProjectFile::read_file(&proj_file_path)
    }

    // Read the project file at `PROJECT_FILE_PATH` and return the `ProjectFile`.
    pub fn read_file(path: &Path) -> Result<Self, Errors> {
        let mut file = File::open(path).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to open file \"{}\". {:?}",
                path.to_string_lossy().to_string(),
                e
            ))
        })?;

        // Read the content of the file.
        let mut content = String::new();
        if let Err(e) = file.read_to_string(&mut content) {
            return Err(Errors::from_msg(format!(
                "Failed to read file \"{}\": {:?}",
                path.to_string_lossy().to_string(),
                e
            )));
        }

        // Parse the content as a toml file and return the `ProjectFile`.
        let mut proj_file: ProjectFile = match toml::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                let (start, end) = e.span().map(|r| (r.start, r.end)).unwrap_or((0, 0));
                let span = Span {
                    start,
                    end,
                    input: SourceFile::from_file_path(path.to_path_buf()),
                };
                return Err(Errors::from_msg_srcs(
                    format!(
                        "Failed to parse file \"{}\": {}",
                        path.to_string_lossy().to_string(),
                        e.message()
                    ),
                    &[&Some(span)],
                ));
            }
        };

        // Set `hash` field.
        let content_hash = format!("{:x}", md5::compute(content.as_bytes()));
        proj_file.hash = content_hash;

        // Set `path` field.
        proj_file.path = path.to_path_buf();

        // Perform validation.
        proj_file.validate()?;

        Ok(proj_file)
    }

    fn validate_project_name(name: &ProjectName, span: Option<Span>) -> Result<(), Errors> {
        // The project name should be non-empty, and can only contain alphanumeric characters, hyphens.
        if name.is_empty() {
            return Err(Errors::from_msg_srcs(
                "Project name should not be empty.".to_string(),
                &[&span],
            ));
        }
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(Errors::from_msg_srcs(
                "Project name should only contain alphanumeric characters and hyphens.".to_string(),
                &[&span],
            ));
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Errors> {
        // Validate the general section.

        // Validate the project name.
        Self::validate_project_name(&self.general.name, Some(self.project_file_span(0, 0)))?;

        // Validate the version.
        Version::parse(&self.general.version).map_err(|e| {
            Errors::from_msg_srcs(
                format!("Failed to parse version: {}", e),
                &[&Some(self.project_file_span(0, 0))],
            )
        })?;

        // Validate the dependencies section.
        let mut dep_names = vec![];
        for dep in &self.dependencies {
            // Cannot have duplicate dependencies.
            if dep_names.contains(&dep.name) {
                return Err(Errors::from_msg_srcs(
                    format!("Duplicate dependency on \"{}\"", dep.name),
                    &[&Some(self.project_file_span(0, 0))],
                ));
            }
            dep_names.push(dep.name.clone());

            // Validate the project name.
            Self::validate_project_name(&dep.name, Some(self.project_file_span(0, 0)))?;

            // Either of `path` or `git` should be specified.
            if (dep.path.is_none() && dep.git.is_none())
                || (dep.path.is_some() && dep.git.is_some())
            {
                return Err(Errors::from_msg_srcs(
                    "Either of `path` or `git` should be specified in a dependency.".to_string(),
                    &[&Some(self.project_file_span(0, 0))],
                ));
            }

            // Validate the version.
            if let Some(version) = &dep.version {
                VersionReq::parse(version).map_err(|e| {
                    Errors::from_msg_srcs(
                        format!("Failed to parse version: {}", e),
                        &[&Some(self.project_file_span(0, 0))],
                    )
                })?;
            }
        }

        Ok(())
    }

    // Update a configuration from a project file.
    // - `dependent_proj`: If true, self is the project file of a dependent project. In this case, append the source files, libraries, library search paths, threaded mode to the configuration but ignore other fields such as debug mode, optimization level, output file, etc.
    pub fn set_config(
        self: &ProjectFile,
        config: &mut Configuration,
        dependent_proj: bool,
    ) -> Result<(), Errors> {
        // Should we consider `[build.test]` section?
        let use_build_test = !dependent_proj
            && (config.subcommand == SubCommand::Test
                || config.subcommand == SubCommand::Diagnostics);

        // Append source files.
        config.source_files.append(
            &mut self
                .build
                .files
                .iter()
                .map(|p| self.join_to_project_dir(p))
                .collect(),
        );
        if use_build_test {
            config
                .source_files
                .append(&mut self.build.test.as_ref().map_or(vec![], |test| {
                    test.files
                        .iter()
                        .map(|p| self.join_to_project_dir(p))
                        .collect()
                }));
        }

        // Append object files.
        config.object_files.append(
            &mut self
                .build
                .objects
                .iter()
                .map(|p| self.join_to_project_dir(p))
                .collect(),
        );
        if use_build_test {
            config
                .object_files
                .append(&mut self.build.test.as_ref().map_or(vec![], |test| {
                    test.objects
                        .iter()
                        .map(|p| self.join_to_project_dir(p))
                        .collect()
                }));
        }

        // Append static libraries.
        if let Some(static_libs) = self.build.static_links.as_ref() {
            config.linked_libraries.append(
                &mut static_libs
                    .iter()
                    .map(|lib_name| (lib_name.clone(), LinkType::Static))
                    .collect(),
            );
        }
        if use_build_test {
            if let Some(static_libs) = self
                .build
                .test
                .as_ref()
                .and_then(|test| test.static_links.as_ref())
            {
                config.linked_libraries.append(
                    &mut static_libs
                        .iter()
                        .map(|lib_name| (lib_name.clone(), LinkType::Static))
                        .collect(),
                );
            }
        }

        // Append dynamic libraries.
        if let Some(dynamic_libs) = self.build.dynamic_links.as_ref() {
            config.linked_libraries.append(
                &mut dynamic_libs
                    .iter()
                    .map(|lib_name| (lib_name.clone(), LinkType::Dynamic))
                    .collect(),
            );
        }
        if use_build_test {
            if let Some(dynamic_libs) = self
                .build
                .test
                .as_ref()
                .and_then(|test| test.dynamic_links.as_ref())
            {
                config.linked_libraries.append(
                    &mut dynamic_libs
                        .iter()
                        .map(|lib_name| (lib_name.clone(), LinkType::Dynamic))
                        .collect(),
                );
            }
        }

        // Append library search paths.
        if let Some(lib_paths) = self.build.library_paths.as_ref() {
            config.library_search_paths.append(
                &mut lib_paths
                    .iter()
                    .map(|p| self.join_to_project_dir(p))
                    .collect(),
            );
        }
        if use_build_test {
            if let Some(lib_paths) = self
                .build
                .test
                .as_ref()
                .and_then(|test| test.library_paths.as_ref())
            {
                config.library_search_paths.append(
                    &mut lib_paths
                        .iter()
                        .map(|p| self.join_to_project_dir(p))
                        .collect(),
                );
            }
        }

        // Set threaded-mode.
        if let Some(threaded) = self.build.threaded {
            if threaded {
                config.set_threaded();
            }
        }
        if use_build_test {
            if let Some(threaded) = self.build.test.as_ref().and_then(|test| test.threaded) {
                if threaded {
                    config.set_threaded();
                }
            }
        }

        // Set extra commands.
        for command in &self.build.preliminary_commands {
            config.extra_commands.push(ExtraCommand {
                work_dir: self.path.parent().unwrap().to_path_buf(),
                command: command.clone(),
            });
        }
        if use_build_test {
            for command in &self
                .build
                .test
                .as_ref()
                .map_or(vec![], |test| test.preliminary_commands.clone())
            {
                config.extra_commands.push(ExtraCommand {
                    work_dir: self.path.parent().unwrap().to_path_buf(),
                    command: command.clone(),
                });
            }
        }

        // Set the memory check mode.
        if use_build_test {
            if let Some(memcheck) = self.build.test.as_ref().and_then(|test| test.memcheck) {
                if memcheck {
                    config.set_valgrind(crate::ValgrindTool::MemCheck);
                }
            }
        }

        if dependent_proj {
            return Ok(());
        }

        // Set debug mode.
        if let Some(debug) = self.build.debug {
            if debug {
                config.set_debug_info();
            }
        }
        if use_build_test {
            if let Some(debug) = self.build.test.as_ref().and_then(|test| test.debug) {
                if debug {
                    config.set_debug_info();
                }
            }
        }

        // Set optimization level.
        if let Some(opt_level) = self.build.opt_level.as_ref() {
            if let Some(opt_level) = FixOptimizationLevel::from_str(opt_level) {
                config.fix_opt_level = opt_level;
            } else {
                return Err(Errors::from_msg_srcs(
                    format!("Unknown optimization level: \"{}\"", opt_level),
                    &[&Some(self.project_file_span(0, 0))],
                ));
            }
        }
        if use_build_test {
            if let Some(opt_level) = self
                .build
                .test
                .as_ref()
                .and_then(|test| test.opt_level.as_ref())
            {
                if let Some(opt_level) = FixOptimizationLevel::from_str(opt_level) {
                    config.fix_opt_level = opt_level;
                } else {
                    return Err(Errors::from_msg_srcs(
                        format!("Unknown optimization level: \"{}\"", opt_level),
                        &[&Some(self.project_file_span(0, 0))],
                    ));
                }
            }
        }

        // Set output file.
        if let Some(output) = self.build.output.as_ref() {
            config.out_file_path = Some(PathBuf::from(output));
        }

        Ok(())
    }

    // Open the lock file.
    // If the project has no dependencies, return an empty lock file.
    pub fn open_lock_file(&self) -> Result<DependecyLockFile, Errors> {
        // If there are no dependencies, the lock file is not necessary.
        if self.dependencies.is_empty() {
            return Ok(DependecyLockFile::default());
        }

        // Try to open the valid dependency lock file.
        // If the project file hash is different from the one in the lock file, the lock file is invalid.
        let content = std::fs::read_to_string(LOCK_FILE_PATH).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to read the lock file: {:?}. {}",
                e, TRY_FIX_RESOLVE
            ))
        })?;
        let lock_file = toml::from_str::<DependecyLockFile>(&content).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to parse the lock file: {:?}. {}",
                e, TRY_FIX_RESOLVE
            ))
        })?;
        if lock_file.proj_file_hash != self.hash {
            return Err(Errors::from_msg(format!(
                "The lock file is not up to date. {}",
                TRY_FIX_RESOLVE
            )));
        }
        Ok(lock_file)
    }

    // Open the lock file or create a new one if it does not exist.
    pub fn open_or_create_lock_file(&self) -> Result<DependecyLockFile, Errors> {
        Ok(match self.open_lock_file() {
            Ok(lock_file) => lock_file,
            Err(_) => {
                let lock_file = DependecyLockFile::create(self)?;
                let content = toml::to_string(&lock_file).map_err(|e| {
                    Errors::from_msg(format!("Failed to serialize the lock file: {:?}", e))
                })?;
                std::fs::write(LOCK_FILE_PATH, content).map_err(|e| {
                    Errors::from_msg(format!("Failed to write the lock file: {:?}", e))
                })?;
                lock_file
            }
        })
    }

    // Open the lock file, create a new one if it does not exist, and install the dependencies.
    pub fn open_or_create_lock_file_and_isntall(&self) -> Result<(), Errors> {
        self.open_or_create_lock_file().and_then(|lf| lf.install())
    }

    // Update configuration by adding source files, linking libraries, ... as required by dependencies.
    pub fn install_dependencies(
        self: &ProjectFile,
        config: &mut Configuration,
    ) -> Result<(), Errors> {
        // Update the lock file if necessary.
        let lock_file = self.open_or_create_lock_file()?;

        // Install the dependencies.
        lock_file.install()?;

        // See the dependencies and update the configuration.
        lock_file.set_config(config)?;

        Ok(())
    }

    // Create span for a range in the project file.
    fn project_file_span(&self, start: usize, end: usize) -> Span {
        let input = SourceFile::from_file_path(self.path.clone());
        Span { start, end, input }
    }

    // Convert a relative path to an absolute path by joining it with the directory of the project file.
    fn join_to_project_dir(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            return path.to_path_buf();
        } else {
            return self.path.parent().unwrap().join(path);
        }
    }

    // Get the source of a dependent project.
    pub fn get_dependency_source(&self, name: &ProjectName) -> ProjectSource {
        for dep in &self.dependencies {
            if &dep.name != name {
                continue;
            }
            if let Some(path) = &dep.path {
                return ProjectSource::Local(self.join_to_project_dir(path));
            }
            if let Some(git) = &dep.git {
                return ProjectSource::Git(git.url.clone(), None);
            }
            panic!("No source specified for dependency `{}`.", name);
        }
        panic!("Project `{}` not found in dependencies.", name);
    }

    // Creates an example project file in the current directory.
    pub fn create_example_file() -> Result<(), Errors> {
        let content = include_str!("docs/project_template.toml");
        std::fs::write(PROJECT_FILE_PATH, content).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to create file \"{}\": {:?}.",
                PROJECT_FILE_PATH, e
            ))
        })?;
        Ok(())
    }

    // Add dependencies to Fix projects to the project file.
    pub fn add_dependencies(&self, proj_vers: &Vec<String>) -> Result<(), Errors> {
        let mut added = "".to_string();

        // Parse each element of `proj_vars` as the form `proj-name@ver_req`.
        let mut projs: Vec<Option<(String, String)>> = vec![]; // (proj_name, version string)
        for proj_ver in proj_vers {
            let proj_ver_split = proj_ver.split('@').collect::<Vec<&str>>();
            if proj_ver_split.len() == 0 || proj_ver_split.len() > 2 {
                return Err(Errors::from_msg(format!(
                    "Invalid project specification: \"{}\". It should be in the form \"proj-name\" or \"proj-name@ver_req\"",
                    proj_ver
                )));
            }
            let proj_name = proj_ver_split[0];
            ProjectFile::validate_project_name(&proj_name.to_string(), None)?;
            let version = if proj_ver_split.len() == 2 {
                let _ = VersionReq::parse(proj_ver_split[1]).map_err(|e| {
                    Errors::from_msg(format!(
                        "Failed to parse version requirement in \"{}\": {:?}",
                        proj_ver, e
                    ))
                })?;
                proj_ver_split[1].to_string()
            } else {
                "*".to_string()
            };
            projs.push(Some((proj_name.to_string(), version)));
        }

        // Check if dependencies to the same project are specified multiple times.
        for i in 0..projs.len() {
            for j in i + 1..projs.len() {
                if projs[i].as_ref().unwrap().0 == projs[j].as_ref().unwrap().0 {
                    return Err(Errors::from_msg(format!(
                        "The project \"{}\" is specified multiple times.",
                        projs[i].as_ref().unwrap().0
                    )));
                }
            }
        }

        // Check if the project file already contains the dependencies.
        for prj_ver in &projs {
            let proj_name = &prj_ver.as_ref().unwrap().0;
            if self.dependencies.iter().any(|dep| &dep.name == proj_name) {
                return Err(Errors::from_msg(format!(
                    "The project file already contains a dependency on \"{}\".",
                    proj_name
                )));
            }
        }

        // Fetch the registry files.
        for reg_url in &self.general.registries {
            let reg_res = reqwest::blocking::get(reg_url).map_err(|e| {
                Errors::from_msg(format!(
                    "Failed to fetch registry file \"{}\": {:?}",
                    reg_url, e
                ))
            })?;
            let reg_file = reg_res.text().map_err(|e| {
                Errors::from_msg(format!(
                    "Failed to fetch registry file \"{}\": {:?}",
                    reg_url, e
                ))
            })?;
            let reg_file = toml::from_str::<RegistryFile>(&reg_file).map_err(|e| {
                Errors::from_msg(format!(
                    "Failed to parse registry file \"{}\": {:?}",
                    reg_url, e
                ))
            })?;

            // For each project to be added, search it in the registry file.
            let mut remove_index = vec![]; // The indices of the projects to be removed from `projs`.
            for (i, proj_var) in projs.iter().enumerate() {
                if proj_var.is_none() {
                    // This project has already been removed.
                    continue;
                }
                let (proj_name, version) = proj_var.as_ref().unwrap();
                if let Some(proj_info) = reg_file
                    .projects
                    .iter()
                    .find(|prj_info| &prj_info.name == proj_name)
                {
                    added += "\n\n[[dependencies]]";
                    added += &format!("\nname = \"{}\"", proj_name);
                    added += &format!("\nversion = \"{}\"", version);
                    added += &format!("\ngit = {{ url = \"{}\" }}", proj_info.git);

                    remove_index.push(i);
                }
            }

            // Remove the projects that have been added.
            for i in remove_index.iter().rev() {
                projs[*i] = None;
            }
        }

        // Check if all the projects have been added.
        for proj_var in projs {
            if let Some(proj_var) = proj_var {
                return Err(Errors::from_msg(format!(
                    "The project \"{}\" is not found in the registries.",
                    proj_var.0
                )));
            }
        }

        // Write the added dependencies to the project file.
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.path)
            .map_err(|e| {
                Errors::from_msg(format!(
                    "Failed to open file \"{}\": {:?}",
                    self.path.to_string_lossy().to_string(),
                    e
                ))
            })?;
        file.write_all(added.as_bytes()).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to write to file \"{}\": {:?}",
                self.path.to_string_lossy().to_string(),
                e
            ))
        })?;
        Ok(())
    }
}
