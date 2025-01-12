use crate::{
    config_file::ConfigFile,
    dependency_lockfile::{self, DependecyLockFile, ProjectSource},
    error::Errors,
    misc::{info_msg, warn_msg, Set},
    registry_file::RegistryFile,
    Configuration, ExtraCommand, FixOptimizationLevel, LinkType, OutputFileType, SourceFile, Span,
    LOCK_FILE_PATH, PROJECT_FILE_PATH, TRY_FIX_RESOLVE,
};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    hash::Hash,
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
    output_type: Option<String>,
    #[serde(default)]
    ld_flags: Vec<String>,
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
    ld_flags: Vec<String>,
    #[serde(default)]
    preliminary_commands: Vec<Vec<String>>,
    memcheck: Option<bool>,
}

// The entry of `dependencies` section of the project file.
#[derive(Deserialize, Serialize, Default, Clone)]
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
#[derive(Deserialize, Serialize, Default, Clone, Hash)]
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

        // Set `path` field.
        proj_file.path = path.to_path_buf();

        // Perform validation.
        proj_file.validate()?;

        Ok(proj_file)
    }

    // Calculate the hash value of the `dependencies` section.
    pub fn calculate_dependencies_hash(&self) -> String {
        // Sort the dependencies by name.
        let mut deps = self.dependencies.clone();
        deps.sort_by(|a, b| a.name.cmp(&b.name));

        let mut data = String::new();
        for dep in deps {
            data += serde_json::to_string(&dep).unwrap().as_str();
        }
        // Calculate the hash value.
        format!("{:x}", md5::compute(data))
    }

    pub fn validate_project_name(name: &ProjectName, span: Option<Span>) -> Result<(), Errors> {
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

    // Get source files of this project. Does not include files of dependent projects.
    // - `use_build_test`: If true, include files in the `[build.test]` section.
    pub fn get_files(&self, use_build_test: bool) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = self
            .build
            .files
            .iter()
            .map(|p| self.join_to_project_dir(p))
            .collect();
        if use_build_test {
            files.append(&mut self.build.test.as_ref().map_or(vec![], |test| {
                test.files
                    .iter()
                    .map(|p| self.join_to_project_dir(p))
                    .collect()
            }));
        }
        files
    }

    // Update a configuration from a project file.
    // - `is_dependent_proj`: If true, self is the project file of a dependent project.
    //   In this case, append the source files, libraries, library search paths, threaded mode to the configuration,
    //   but ignore other fields such as debug mode, optimization level, output file, etc.
    pub fn set_config(
        &self,
        config: &mut Configuration,
        is_dependent_proj: bool,
    ) -> Result<(), Errors> {
        // Should we consider `[build.test]` section?
        // If the project is a dependent project, we do not consider the `[build.test]` section.
        let use_build_test = !is_dependent_proj && config.subcommand.use_test_files();

        // Set the output file type.
        if let Some(output_file_type) = self.build.output_type.as_ref() {
            config.output_file_type = OutputFileType::from_str(output_file_type)?;
        }

        // Append source files.
        config
            .source_files
            .append(&mut self.get_files(use_build_test));

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

        // Add ld_flags.
        config.ld_flags.append(&mut self.build.ld_flags.clone());
        if use_build_test {
            config.ld_flags.append(
                &mut self
                    .build
                    .test
                    .as_ref()
                    .map_or(vec![], |test| test.ld_flags.clone()),
            );
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

        if is_dependent_proj {
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
        if lock_file.proj_file_hash != self.calculate_dependencies_hash() {
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
    pub fn create_example_file(proj_name: String) -> Result<(), Errors> {
        // If the project file already exists, do not overwrite it.
        if Path::new(PROJECT_FILE_PATH).exists() {
            return Err(Errors::from_msg(format!(
                "The file \"{}\" already exists.",
                PROJECT_FILE_PATH
            )));
        }

        let content = include_str!("docs/project_template.toml");
        // Replace `{PLACEHOLDER_PROJECT_NAME}` to `proj_name`.
        let content = content.replace("{PLACEHOLDER_PROJECT_NAME}", &proj_name);
        std::fs::write(PROJECT_FILE_PATH, content).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to create file \"{}\": {:?}.",
                PROJECT_FILE_PATH, e
            ))
        })?;
        Ok(())
    }

    // Add dependencies to Fix projects to the project file.
    pub fn add_dependencies(
        &self,
        proj_vers: &Vec<String>,
        fix_config: &ConfigFile,
    ) -> Result<(), Errors> {
        let mut added = "".to_string();

        // Parse each element of `proj_vars` as the form `proj-name@ver_req`.
        let mut projs: Vec<(String, Option<String>)> = vec![]; // (proj_name, ver_str)
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
                Some(proj_ver_split[1].to_string())
            } else {
                None
            };
            projs.push((proj_name.to_string(), version));
        }

        // Check if dependencies to the same project are specified multiple times.
        for i in 0..projs.len() {
            for j in i + 1..projs.len() {
                if projs[i].0 == projs[j].0 {
                    return Err(Errors::from_msg(format!(
                        "The project \"{}\" is specified multiple times.",
                        projs[i].0
                    )));
                }
            }
        }

        // Check if the project file already has the dependencies.
        for prj_ver in &projs {
            let proj_name = &prj_ver.0;
            if self.dependencies.iter().any(|dep| &dep.name == proj_name) {
                return Err(Errors::from_msg(format!(
                    "The project file already has a dependency on \"{}\".",
                    proj_name
                )));
            }
        }

        // Fetch the registry files.
        for reg_url in &fix_config.registries {
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
            let mut added_indices = Set::default();
            for (i, proj_var) in projs.iter().enumerate() {
                let (proj_name, version) = proj_var;
                if let Some(proj_info) = reg_file
                    .projects
                    .iter()
                    .find(|prj_info| &prj_info.name == proj_name)
                {
                    // If the project is found in the registry, add it to the project file.
                    println!(
                        "The project \"{}\" was found at \"{}\".",
                        proj_name, reg_url
                    );

                    // When the version requirement is empty, try to use the latest tagged version.
                    let version = match version {
                        Some(v) => v.clone(),
                        None => {
                            let (_tmp_dir, repo) =
                                dependency_lockfile::clone_git_repo(&proj_info.git)?;
                            let vers = dependency_lockfile::get_versions_from_repo(&repo)?;
                            let mut tagged_vers = vers
                                .iter()
                                .filter_map(|vi| {
                                    if vi.tagged {
                                        Some(vi.version.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>();
                            tagged_vers.sort();
                            if tagged_vers.is_empty() {
                                warn_msg(&format!(
                                    "Adding version requirement \"*\" for \"{}\" since there are no tagged versions. \
                                    This means that updating the lock file (which is done by `fix deps add` or `fix deps update`) may introduce breaking changes.",
                                    proj_name, 
                                ));
                                "*".to_string()
                            } else {
                                let latest = tagged_vers.pop().unwrap();
                                let latest =
                                    format!("{}.{}.{}", latest.major, latest.minor, latest.patch);
                                info_msg(&format!(
                                    "Adding version requirement \"{}\" for \"{}\" which is the latest tagged version.",
                                    latest, proj_name
                                ));
                                latest
                            }
                        }
                    };

                    added += "\n\n[[dependencies]]";
                    added += &format!("\nname = \"{}\"", proj_name);
                    added += &format!("\nversion = \"{}\"", version);
                    added += &format!("\ngit = {{ url = \"{}\" }}", proj_info.git);

                    added_indices.insert(i);
                }
            }

            // Remove the projects that have been added.
            projs = projs
                .into_iter()
                .enumerate()
                .filter_map(|(i, v)| {
                    if added_indices.contains(&i) {
                        None
                    } else {
                        Some(v)
                    }
                })
                .collect();
        }

        // Check if all the projects have been added.
        for proj_var in projs {
            return Err(Errors::from_msg(format!(
                "The project \"{}\" is not found in the registries.",
                proj_var.0
            )));
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
