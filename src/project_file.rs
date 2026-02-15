use crate::{
    Configuration, ExtraCommand, FixOptimizationLevel, LinkType, OutputFileType, PROJECT_FILE_PATH, SourceFile, Span, TRY_FIX_DEPS_UPDATE, config_file::ConfigFile, configuration::LockFileType, constants::{SAMPLE_MAIN_FILE_PATH, SAMPLE_TEST_FILE_PATH, TRY_FIX_DEPS_UPDATE_TEST}, dependency_lockfile::{self, DependecyLockFile, ProjectSource, get_lock_file_path}, error::Errors, misc::{Set, info_msg, warn_msg}, registry_file::RegistryFile
};
use reqwest::Url;
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
    // The Fix compiler version.
    // Defaults to "*".
    pub fix_version: Option<String>,
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
    #[serde(default)]
    ld_flags: Vec<String>,
    #[serde(default)]
    preliminary_commands: Vec<Vec<String>>,

    threaded: Option<bool>,
    debug: Option<bool>,
    opt_level: Option<String>,
    output: Option<PathBuf>,
    output_type: Option<String>,
    backtrace: Option<bool>,
    #[serde(default)]
    disable_cpu_features: Vec<String>,
    #[serde(default)]
    no_runtime_check: bool,

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
    #[serde(default)]
    ld_flags: Vec<String>,    
    #[serde(default)]
    preliminary_commands: Vec<Vec<String>>,

    threaded: Option<bool>,
    debug: Option<bool>,
    opt_level: Option<String>,
    backtrace: Option<bool>,
    #[serde(default)]
    disable_cpu_features: Vec<String>,

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
    // `test_dependencies` section
    #[serde(default)]
    pub test_dependencies: Vec<ProjectFileDependency>,
    // The path to the project file.
    #[serde(skip)]
    pub path: PathBuf,
}

impl ProjectFile {
    // Get dependencies based on mode.
    pub fn get_dependencies(&self, mode: LockFileType) -> Vec<ProjectFileDependency> {
        match mode {
            LockFileType::Test | LockFileType::Lsp => {
                // Merge dependencies and test_dependencies
                // Note: Duplicate check is already performed in validate()
                let mut all_deps = self.dependencies.clone();
                all_deps.extend(self.test_dependencies.clone());
                all_deps
            }
            LockFileType::Build => self.dependencies.clone(),
        }
    }

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

        // Check if the project file is compatible with the current version of Fix.
        proj_file.is_fix_version_compatible()?;

        Ok(proj_file)
    }

    // Calculate the hash value of the `dependencies` section.
    pub fn calculate_dependencies_hash(&self, mode: LockFileType) -> String {
        // Get dependencies based on mode.
        let mut deps = match mode {
            LockFileType::Test | LockFileType::Lsp => {
                // Merge dependencies and test_dependencies
                let mut all_deps = self.dependencies.clone();
                all_deps.extend(self.test_dependencies.clone());
                all_deps
            }
            LockFileType::Build => self.dependencies.clone(),
        };
        
        // Sort the dependencies by name.
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

    // Validate a single dependency entry.
    fn validate_dependency_entry(dep: &ProjectFileDependency, span: Span) -> Result<(), Errors> {
        // Validate the project name.
        Self::validate_project_name(&dep.name, Some(span.clone()))?;

        // Either of `path` or `git` should be specified.
        if (dep.path.is_none() && dep.git.is_none())
            || (dep.path.is_some() && dep.git.is_some())
        {
            return Err(Errors::from_msg_srcs(
                "Either of `path` or `git` should be specified in a dependency.".to_string(),
                &[&Some(span.clone())],
            ));
        }

        // Validate the version.
        if let Some(version) = &dep.version {
            VersionReq::parse(version).map_err(|e| {
                Errors::from_msg_srcs(
                    format!("Failed to parse version: {}", e),
                    &[&Some(span)],
                )
            })?;
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
                format!("Failed to parse `version`: {}", e),
                &[&Some(self.project_file_span(0, 0))],
            )
        })?;

        // Validate `fix_version`.
        if let Some(fix_version) = &self.general.fix_version {
            VersionReq::parse(fix_version).map_err(|e| {
                Errors::from_msg_srcs(
                    format!("Failed to parse `fix_version`: {}", e),
                    &[&Some(self.project_file_span(0, 0))],
                )
            })?;
        }

        // Validate the dependencies section and check for duplicates.
        let mut dep_names = Set::default();
        for dep in &self.dependencies {
            if !dep_names.insert(&dep.name) {
                return Err(Errors::from_msg_srcs(
                    format!("Duplicate dependency on \"{}\"", dep.name),
                    &[&Some(self.project_file_span(0, 0))],
                ));
            }
            Self::validate_dependency_entry(dep, self.project_file_span(0, 0))?;
        }

        // Validate the test_dependencies section and check for duplicates.
        for dep in &self.test_dependencies {
            if !dep_names.insert(&dep.name) {
                return Err(Errors::from_msg_srcs(
                    format!("Duplicate dependency on \"{}\"", dep.name),
                    &[&Some(self.project_file_span(0, 0))],
                ));
            }
            Self::validate_dependency_entry(dep, self.project_file_span(0, 0))?;
        }

        // Validate disable_cpu_features.
        Self::validate_disable_cpu_features(&self.build.disable_cpu_features)?;

        Ok(())
    }

    // Validate `disable_cpu_features`.
    pub fn validate_disable_cpu_features(dcfs: &[String]) -> Result<(), Errors> {
        for feature in dcfs {
            // Check if each feature is a valid regex.
            if let Err(e) = regex::Regex::new(feature) {
                return Err(Errors::from_msg(format!(
                    "Invalid regex in `disable-cpu-feature`: {}",
                    e
                )));
            }
        }
        Ok(())
    }

    // Get source files of this project. Does not include files of dependent projects.
    // - `mode`: The build mode (Build or Test). If Test, include files in the `[build.test]` section.
    pub fn get_files(&self, mode: LockFileType) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = self
            .build
            .files
            .iter()
            .map(|p| self.join_to_project_dir(p))
            .collect();
        if mode == LockFileType::Test {
            files.append(&mut self.build.test.as_ref().map_or(vec![], |test| {
                test.files
                    .iter()
                    .map(|p| self.join_to_project_dir(p))
                    .collect()
            }));
        }
        files
    }
    
    // Get the version requirement for the Fix compiler.
    pub fn fix_version(&self) -> VersionReq {
        match &self.general.fix_version {
            Some(v) => VersionReq::parse(v).unwrap(),
            None => VersionReq::STAR,
        }
    }

    // Check if the project file is compatible with the current version of Fix.
    pub fn is_fix_version_compatible(&self) -> Result<(), Errors> {
        if self.fix_version().matches(&Version::parse(env!("CARGO_PKG_VERSION")).unwrap()) {
            Ok(())
        } else {
            Err(Errors::from_msg(format!(
                "The project \"{}\" requires Fix version \"{}\", but the current version of Fix is \"{}\".",
                self.general.name,
                self.fix_version(),
                env!("CARGO_PKG_VERSION"),
            )))
        }
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
        // Determine the build mode.
        // If the project is a dependent project, we do not consider the `[build.test]` section.
        let mut mode = config.subcommand.build_mode();
        if is_dependent_proj {
            mode = LockFileType::Build;
        }

        // Append source files.
        config
            .source_files
            .append(&mut self.get_files(mode));

        // Append object files.
        config.object_files.append(
            &mut self
                .build
                .objects
                .iter()
                .map(|p| self.join_to_project_dir(p))
                .collect(),
        );
        if mode == LockFileType::Test {
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
        if mode == LockFileType::Test {
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
        if mode == LockFileType::Test {
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
        if mode == LockFileType::Test {
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
        if mode == LockFileType::Test {
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
        if mode == LockFileType::Test {
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
        if mode == LockFileType::Test {
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
        if mode == LockFileType::Test {
            if let Some(memcheck) = self.build.test.as_ref().and_then(|test| test.memcheck) {
                if memcheck {
                    config.set_valgrind(crate::ValgrindTool::MemCheck);
                }
            }
        }

        // From here on, only the settings in the project file of the root project are reflected.
        if is_dependent_proj {
            return Ok(());
        }

        // Set debug mode.
        if let Some(debug) = self.build.debug {
            if debug {
                config.set_debug_info();
            }
        }
        if mode == LockFileType::Test {
            if let Some(debug) = self.build.test.as_ref().and_then(|test| test.debug) {
                if debug {
                    config.set_debug_info();
                }
            }
        }

        // Set optimization level.
        if let Some(opt_level) = self.build.opt_level.as_ref() {
            if let Some(opt_level) = FixOptimizationLevel::from_str(opt_level) {
                config.set_fix_opt_level(opt_level);
            } else {
                return Err(Errors::from_msg_srcs(
                    format!("Unknown optimization level: \"{}\"", opt_level),
                    &[&Some(self.project_file_span(0, 0))],
                ));
            }
        }
        if mode == LockFileType::Test {
            if let Some(opt_level) = self
                .build
                .test
                .as_ref()
                .and_then(|test| test.opt_level.as_ref())
            {
                if let Some(opt_level) = FixOptimizationLevel::from_str(opt_level) {
                    config.set_fix_opt_level(opt_level);
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

        // Set the output file type.
        if let Some(output_file_type) = self.build.output_type.as_ref() {
            config.output_file_type = OutputFileType::from_str(output_file_type)?;
        }

        // Set backtrace mode.
        if let Some(backtrace) = self.build.backtrace {
            if backtrace {
                config.set_backtrace();
            }
        }
        if mode == LockFileType::Test {
            if let Some(backtrace) = self.build.test.as_ref().and_then(|test| test.backtrace) {
                if backtrace {
                    config.set_backtrace();
                }
            }
        }

        // Set disable_cpu_features.
        config.disable_cpu_features_regex.append(&mut self.build.disable_cpu_features.clone());
        if mode == LockFileType::Test {
            config.disable_cpu_features_regex.append(
                &mut self
                    .build
                    .test
                    .as_ref()
                    .map_or(vec![], |test| test.disable_cpu_features.clone()),
            );
        }

        // Set no_runtime_check.
        config.no_runtime_check = self.build.no_runtime_check;
        if mode == LockFileType::Test {
            config.no_runtime_check = false;
        }

        Ok(())
    }

    // Open the lock file.
    // If the project has no dependencies, return an empty lock file.
    pub fn open_lock_file(&self, mode: LockFileType) -> Result<DependecyLockFile, Errors> {
        // If there are no dependencies, the lock file is not necessary.
        if self.get_dependencies(mode).is_empty() {
            return Ok(DependecyLockFile::default());
        }

        // Try to open the valid dependency lock file.
        // If the project file hash is different from the one in the lock file, the lock file is invalid.
        let lock_file_path = get_lock_file_path(mode);
        let msg_try_fix_deps_update = match mode {
            LockFileType::Build => TRY_FIX_DEPS_UPDATE,
            LockFileType::Test => TRY_FIX_DEPS_UPDATE_TEST,
            LockFileType::Lsp => TRY_FIX_DEPS_UPDATE_TEST, // LSP uses auto-update, so this message is rarely shown
        };
        let content = std::fs::read_to_string(lock_file_path).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to read the lock file: {:?}. {}",
                e, msg_try_fix_deps_update
            ))
        })?;
        let lock_file = toml::from_str::<DependecyLockFile>(&content).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to parse the lock file: {:?}. {}",
                e, msg_try_fix_deps_update
            ))
        })?;
        if lock_file.proj_file_hash != self.calculate_dependencies_hash(mode) {
            return Err(Errors::from_msg(format!(
                "The lock file is not up to date. {}",
                msg_try_fix_deps_update
            )));
        }
        Ok(lock_file)
    }

    // Helper method to save lock file to disk.
    fn save_lock_file(lock_file: &DependecyLockFile, mode: LockFileType) -> Result<(), Errors> {
        let content = toml::to_string(lock_file).map_err(|e| {
            Errors::from_msg(format!("Failed to serialize lock file: {:?}", e))
        })?;
        let lock_file_path = get_lock_file_path(mode);
        std::fs::write(lock_file_path, content).map_err(|e| {
            Errors::from_msg(format!("Failed to write lock file: {:?}", e))
        })?;
        Ok(())
    }

    // Open the lock file or create a new one if it does not exist.
    pub fn open_or_create_lock_file(&self, mode: LockFileType) -> Result<DependecyLockFile, Errors> {
        Ok(match self.open_lock_file(mode) {
            Ok(lock_file) => lock_file,
            Err(_) => {
                let lock_file = DependecyLockFile::create(self, mode)?;
                Self::save_lock_file(&lock_file, mode)?;
                lock_file
            }
        })
    }

    // Open the lock file, or automatically create/update it if it does not exist or is invalid, and install the dependencies.
    // This method is designed for LSP to automatically manage lock files without user intervention.
    // Returns error without panicking, allowing LSP to report diagnostics to the user.
    pub fn open_or_auto_update_lock_file(&self, mode: LockFileType) -> Result<DependecyLockFile, Errors> {
        // Try to open existing lock file.
        match self.open_lock_file(mode) {
            Ok(lock_file) => Ok(lock_file),
            Err(_) => {
                // If the lock file does not exist or is invalid, automatically create/update it.
                
                // Ensure the parent directory exists (e.g., .fixlang/ for LSP lock file).
                let lock_file_path = get_lock_file_path(mode);
                if let Some(parent) = Path::new(lock_file_path).parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        Errors::from_msg(format!("Failed to create directory: {:?}", e))
                    })?;
                }
                
                // Create the lock file.
                let lock_file = DependecyLockFile::create(self, mode)?;
                
                // Save the lock file.
                Self::save_lock_file(&lock_file, mode)?;
                
                // Install the dependencies.
                lock_file.install()?;
                
                Ok(lock_file)
            }
        }
    }

    // Open the lock file, create a new one if it does not exist, and install the dependencies.
    pub fn open_or_create_lock_file_and_install(&self, mode: LockFileType) -> Result<(), Errors> {
        self.open_or_create_lock_file(mode).and_then(|lf| lf.install())
    }

    // Update configuration by adding source files, linking libraries, ... as required by dependencies.
    pub fn install_dependencies(
        self: &ProjectFile,
        config: &mut Configuration,
        mode: LockFileType,
    ) -> Result<(), Errors> {
        // Update the lock file if necessary.
        let lock_file = self.open_or_create_lock_file(mode)?;

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
        // Search in both dependencies and test_dependencies
        for dep in self.dependencies.iter().chain(self.test_dependencies.iter()) {
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
        // Create sample "fixproj.toml" file in the current directory.

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
        
        // Replace `{PLACEHOLDER_FIX_VERSION}` to the current version of Fix.
        let content = content.replace("{PLACEHOLDER_FIX_VERSION}", env!("CARGO_PKG_VERSION"));

        std::fs::write(PROJECT_FILE_PATH, content).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to create file \"{}\": {:?}.",
                PROJECT_FILE_PATH, e
            ))
        })?;

        // Create sample "main.fix" file in the current directory.
        if Path::new(SAMPLE_MAIN_FILE_PATH).exists() {
            return Err(Errors::from_msg(format!(
                "The file \"main.fix\" already exists."
            )));
        }
        let main_fix_content = include_str!("docs/main_template.fix");
        std::fs::write(SAMPLE_MAIN_FILE_PATH, main_fix_content).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to create file \"main.fix\": {:?}.",
                e
            ))
        })?;

        // Create sample "test.fix" file in the current directory.
        if Path::new(SAMPLE_TEST_FILE_PATH).exists() {
            return Err(Errors::from_msg(format!(
                "The file \"test.fix\" already exists."
            )));
        }
        let test_fix_content = include_str!("docs/test_template.fix");
        std::fs::write(SAMPLE_TEST_FILE_PATH, test_fix_content).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to create file \"test.fix\": {:?}.",
                e
            ))
        })?;

        Ok(())
    }

    // Add dependencies to Fix projects to the project file.
    pub fn add_dependencies(
        &self,
        proj_vers: &Vec<String>,
        fix_config: &ConfigFile,
        mode: LockFileType,
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
        let existing_deps = self.get_dependencies(mode);
        for prj_ver in &projs {
            let proj_name = &prj_ver.0;
            if existing_deps.iter().any(|dep| &dep.name == proj_name) {
                return Err(Errors::from_msg(format!(
                    "The project file already has a dependency on \"{}\".",
                    proj_name
                )));
            }
        }

        // Fetch the registry files.
        for reg_loc in &fix_config.registries {
            let reg_file = ProjectFile::retrieve_registry_file(reg_loc)?;

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
                    info_msg(&format!(
                        "The project \"{}\" was found in \"{}\".",
                        proj_name, reg_loc
                    ));

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

                    let section_name = match mode {
                        LockFileType::Build => "[[dependencies]]",
                        LockFileType::Test => "[[test_dependencies]]",
                        LockFileType::Lsp => unreachable!("add_dependencies should not be called with LockFileType::Lsp"),
                    };
                    added += "\n\n";
                    added += section_name;
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

    // Retrieve the registry file at the specified location.
    // 
    // - `loc`: The location of the registry file, which is a url or a file path.
    pub fn retrieve_registry_file(loc: &str) -> Result<RegistryFile, Errors> {
        let reg_file_content = if Url::parse(loc).is_ok() {
            // The location is a URL.
            let reg_res = reqwest::blocking::get(loc).map_err(|e| {
                Errors::from_msg(format!(
                    "Failed to fetch registry file \"{}\": {:?}",
                    loc, e
                ))
            })?;
            reg_res.text().map_err(|e| {
                Errors::from_msg(format!(
                    "Failed to fetch registry file \"{}\": {:?}",
                    loc, e
                ))
            })?
        } else {
            // The location is a file path.
            std::fs::read_to_string(loc).map_err(|e| {
                Errors::from_msg(format!(
                    "Failed to read registry file \"{}\": {:?}",
                    loc, e
                ))
            })?
        };
        let reg_file = toml::from_str::<RegistryFile>(&reg_file_content).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to parse registry file \"{}\": {:?}",
                loc, e
            ))
        })?;
        Ok(reg_file)
    }
}
