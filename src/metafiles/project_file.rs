use crate::{
    configuration::{
        BuildConfigType, Configuration, FixOptimizationLevel, LinkType, OutputFileType,
        ProjectSources, Sanitizer, ValgrindTool,
    },
    constants::{
        PROJECT_FILE_PATH, SAMPLE_MAIN_FILE_PATH, SAMPLE_TEST_FILE_PATH, TRY_FIX_DEPS_UPDATE,
        TRY_FIX_DEPS_UPDATE_TEST, WHOLE_PROGRAM_IN_ONE_UNIT, WHOLE_PROGRAM_IN_ONE_UNIT_STR,
    },
    dependency::lockfile::{
        clone_git_repo, get_lock_file_path, get_versions_from_repo, DependecyLockFile,
        LockFileType, ProjectSource,
    },
    error::Errors,
    hash::HashSource,
    metafiles::{config_file::ConfigFile, registry_file::RegistryFile},
    misc::{info_msg, to_absolute_path, warn_msg, Set},
    parse::sourcefile::{SourceFile, Span},
    preliminary_command::{PreliminaryCommand, PreliminaryCommandMode},
};
use regex::Regex;
use reqwest::Url;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    hash::Hash,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use toml::Spanned;

/// The name of a project, as the `[general]` section of its project file gives it and the
/// dependency entries of other projects refer to it.
pub type ProjectName = String;

/// The `general` section of the project file, holding the project's identity: its name, its
/// version, and the compiler versions it builds with.
#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileGeneral {
    /// The name of the project, which the dependency entries of other projects refer to it by.
    pub name: ProjectName,
    /// The version of the project, written as semver. `ProjectFile::validate` rejects a string that
    /// is not one.
    pub version: String,
    /// The compiler versions this project builds with, written as a semver requirement. Left out,
    /// the project builds with every version.
    pub fix_version: Option<String>,
    /// A sentence describing the project, shown to whoever reads the project file.
    #[allow(unused)]
    pub description: Option<String>,
    /// The people credited with the project.
    #[allow(unused)]
    pub authors: Option<Vec<String>>,
    /// The name of the license the project is distributed under.
    #[allow(unused)]
    pub license: Option<String>,
}

impl ProjectFileGeneral {
    /// The `version` field parsed as semver. Panics unless `ProjectFile::validate` has accepted
    /// the field.
    pub fn version(&self) -> Version {
        Version::parse(&self.version).unwrap()
    }
}

/// The `build` section of the project file, holding the settings a build of the project reads.
#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileBuild {
    /// The Fix source files the build compiles, each resolved against the project directory and
    /// paired with its byte range in the project file.
    files: Vec<Spanned<PathBuf>>,
    /// The object files the build links, each resolved against the project directory.
    #[serde(default)]
    objects: Vec<PathBuf>,
    /// The libraries the build links statically, each named as the linker's `-l` names it: `"abc"`
    /// links `libabc.a`.
    static_links: Option<Vec<String>>,
    /// The libraries the build links dynamically, each named as the linker's `-l` names it: `"xyz"`
    /// links `libxyz.so`.
    dynamic_links: Option<Vec<String>>,
    /// The directories the linker searches for libraries, each resolved against the project
    /// directory.
    library_paths: Option<Vec<PathBuf>>,
    /// Further flags handed to the linker as they are written.
    #[serde(default)]
    ld_flags: Vec<String>,
    /// The commands run in the project directory before the Fix program is compiled, each written
    /// as a program followed by its arguments. They run once the user has approved them.
    #[serde(default)]
    preliminary_commands: Vec<Vec<String>>,

    /// Whether to build the program with thread-safe reference counting, which lets a value cross
    /// threads. Unset builds it with single-threaded reference counting.
    threaded: Option<bool>,
    /// Name of the sanitizer to instrument the built program with, from the set
    /// `Sanitizer::from_str` accepts.
    sanitize: Option<String>,
    /// Whether to put debugging information into the built program. Unset leaves it out.
    debug: Option<bool>,
    /// Name of the optimization level to build at, from the set `FixOptimizationLevel::from_str`
    /// accepts.
    opt_level: Option<String>,
    /// The path `fix build` writes its output file to. `fix run` and `fix test` build into a
    /// temporary place of their own.
    output: Option<PathBuf>,
    /// Name of the kind of file `fix build` produces, from the set `OutputFileType::from_str`
    /// accepts.
    output_type: Option<String>,
    /// Whether the program prints a backtrace when a run-time error ends it. Unset ends it with the
    /// error message alone.
    backtrace: Option<bool>,
    /// Regex patterns of the CPU features to turn off, so that `"avx512.*"` keeps the program off
    /// every feature whose name that pattern matches.
    #[serde(default)]
    disable_cpu_features: Vec<String>,
    /// Whether to leave the run-time checks, such as the array bounds check, out of the program.
    /// Unset keeps them.
    no_runtime_check: Option<bool>,
    /// Whether to compile `eval {side}; {main}` as `{main}`, leaving the effect of `{side}` out of
    /// the program. Unset evaluates `{side}`.
    skip_eval: Option<bool>,
    /// The average number of entries — the top-level functions and the global values whose code is
    /// generated — one compilation unit holds, or `"inf"` for the whole program in one unit. A
    /// `fix test` build reads the same value, and `--cu-size` on the command line overrides it.
    cu_size: Option<CompilationUnitSize>,

    /// The `build.test` sub-section, which supplies the settings a `fix test` build reads beside
    /// the ones this `build` section gives.
    test: Option<ProjectFileBuildTest>,
}

/// What a `cu_size` field of the project file is written as: a number of entries, or the word that
/// asks for the whole program in one unit.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum CompilationUnitSize {
    /// How many entries one compilation unit holds on average.
    Entries(usize),
    /// `constants::WHOLE_PROGRAM_IN_ONE_UNIT_STR`, which asks for one unit for the whole program.
    Named(String),
}

/// The `build.test` section of the project file, holding the settings a `fix test` build reads.
/// The doc of each field states how its value combines with the one the `build` section gives.
#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileBuildTest {
    /// The Fix source files a test build compiles, added to the ones the `build` section gives.
    /// Each is resolved against the project directory and paired with its byte range in the project
    /// file.
    files: Vec<Spanned<PathBuf>>,
    /// The object files a test build links, added to the ones the `build` section gives.
    #[serde(default)]
    objects: Vec<PathBuf>,
    /// The libraries a test build links statically, added to the ones the `build` section gives.
    static_links: Option<Vec<String>>,
    /// The libraries a test build links dynamically, added to the ones the `build` section gives.
    dynamic_links: Option<Vec<String>>,
    /// The directories the linker searches for libraries in a test build, added to the ones the
    /// `build` section gives.
    library_paths: Option<Vec<PathBuf>>,
    /// Further flags handed to the linker in a test build, added to the ones the `build` section
    /// gives.
    #[serde(default)]
    ld_flags: Vec<String>,
    /// The commands run before a test build compiles, added to the ones the `build` section gives.
    #[serde(default)]
    preliminary_commands: Vec<Vec<String>>,

    /// Whether to build a test with thread-safe reference counting. Unset leaves the value the
    /// `build` section gives in force.
    threaded: Option<bool>,
    /// Name of the sanitizer to instrument the built test program with, from the set
    /// `Sanitizer::from_str` accepts.
    sanitize: Option<String>,
    /// Whether to put debugging information into a test build. Unset leaves the value the `build`
    /// section gives in force.
    debug: Option<bool>,
    /// Name of the optimization level to build a test at, from the set
    /// `FixOptimizationLevel::from_str` accepts. Unset leaves the value the `build` section gives in
    /// force.
    opt_level: Option<String>,
    /// Whether a test prints a backtrace when a run-time error ends it. Unset leaves the value the
    /// `build` section gives in force.
    backtrace: Option<bool>,
    /// Regex patterns of the CPU features to turn off in a test build, added to the ones the
    /// `build` section gives.
    #[serde(default)]
    disable_cpu_features: Vec<String>,
    /// Whether to leave the run-time checks, such as the array bounds check, out of a test build.
    /// Unset keeps them, and the value the `build` section gives covers the program alone.
    no_runtime_check: Option<bool>,
    /// Whether to compile `eval {side}; {main}` as `{main}` in a test build, leaving the effect of
    /// `{side}` out of it. Unset evaluates `{side}`, and the value the `build` section gives covers
    /// the program alone.
    skip_eval: Option<bool>,

    /// Whether `fix test` runs the built test program under valgrind's memcheck tool.
    memcheck: Option<bool>,
}

/// One entry of the `dependencies` or `test_dependencies` section: a project this project builds
/// against, and where that project is fetched from.
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileDependency {
    /// The name of the project depended on, as that project's own `[general]` section gives it.
    pub name: ProjectName,
    /// The directory the project sits in, resolved against the directory of the project file that
    /// declares it. Exactly one of `path` and `git` is written.
    pub path: Option<PathBuf>,
    /// The git repository the project is cloned from. Exactly one of `path` and `git` is written.
    pub git: Option<ProjectFileDependencyGit>,
    /// The versions of the project this entry accepts, written as a semver requirement in Cargo's
    /// syntax. Left out, the latest version is taken.
    pub version: Option<String>,
}

impl ProjectFileDependency {
    /// The `version` field parsed as a semver requirement. A field left out accepts every version.
    /// Panics unless `ProjectFile::validate` has accepted the field.
    pub fn version(&self) -> VersionReq {
        match &self.version {
            Some(v) => VersionReq::parse(v).unwrap(),
            None => VersionReq::STAR,
        }
    }
}

/// The `git` field of a dependency entry: the repository the project is cloned from, and the ref
/// the clone is pinned to.
#[derive(Deserialize, Serialize, Default, Clone, Hash)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileDependencyGit {
    /// The URL of the git repository.
    pub url: String,
    /// The commit hash the dependency is pinned to. At most one of `rev` and `tag` is written; with
    /// neither, the version requirement of the entry picks a tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    /// The tag name the dependency is pinned to. At most one of `rev` and `tag` is written; with
    /// neither, the version requirement of the entry picks a tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

impl ProjectFileDependencyGit {
    /// Returns true if `rev` or `tag` is specified.
    pub fn has_ref(&self) -> bool {
        self.rev.is_some() || self.tag.is_some()
    }

    /// Returns a human-readable description of the pinned ref.
    pub fn ref_description(&self) -> String {
        if let Some(rev) = &self.rev {
            format!("rev \"{}\"", rev)
        } else if let Some(tag) = &self.tag {
            format!("tag \"{}\"", tag)
        } else {
            "no ref".to_string()
        }
    }
}

/// The place a loaded project file has in the current build, set by the loader that read the
/// file.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub enum ProjectFileRole {
    /// The project the build was started in, whose project file alone contributes the settings that
    /// take one value across the whole build.
    #[default]
    Root,
    /// A project another project of the build declares as a dependency.
    Dependent,
}

/// Where a project comes from.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProjectOrigin {
    /// A project in the local file system, at the absolute path this variant holds: the root
    /// project, or one a `path` dependency entry names.
    Local(PathBuf),
    /// A project cloned from a git repository, identified by the repository URL and the commit hash
    /// the lock file pins the clone to.
    Git { url: String, commit: String },
}

impl ProjectOrigin {
    /// A string naming where the project comes from: the project directory for a local project, and
    /// `git+` followed by the repository URL for a git one, so that every commit of one repository
    /// shares a key.
    pub fn to_trust_key(&self) -> String {
        match self {
            ProjectOrigin::Local(p) => p.to_string_lossy().to_string(),
            ProjectOrigin::Git { url, .. } => format!("git+{}", url),
        }
    }

    /// The commit hash a git project is pinned to.
    pub fn commit_hash(&self) -> Option<&str> {
        match self {
            ProjectOrigin::Local(_) => None,
            ProjectOrigin::Git { commit, .. } => Some(commit),
        }
    }
}

/// A project file (`fixproj.toml`): what the project is, how it is built, what it depends on, and
/// what the loader that read it knows about where it came from.
#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectFile {
    /// The `general` section, holding the project's name and version.
    pub general: ProjectFileGeneral,
    /// The `build` section, holding the settings a build reads.
    pub build: ProjectFileBuild,
    /// The `dependencies` entries: the projects every build of this one may use.
    #[serde(default)]
    pub dependencies: Vec<ProjectFileDependency>,
    /// The `test_dependencies` entries: the projects the test sources of this one may use, beside
    /// the ones `dependencies` names.
    #[serde(default)]
    pub test_dependencies: Vec<ProjectFileDependency>,
    /// The path this file was read from. Every relative path the file writes is resolved against
    /// the directory holding it.
    #[serde(skip)]
    pub path: PathBuf,
    /// The place this project has in the current build, set by the loader that read the file.
    #[serde(skip)]
    pub role: ProjectFileRole,
    /// Where this project came from. A freshly deserialized `ProjectFile` carries `None`, and the
    /// loader that read the file fills it in before the file configures a build.
    #[serde(skip)]
    pub source: Option<ProjectOrigin>,
}

impl ProjectFile {
    /// The projects this project declares as dependencies for a build of the given mode. Their
    /// names are distinct, since `validate` rejects a project file that declares one project twice.
    ///
    /// # Arguments
    ///
    /// * `mode` - `Test` also takes the entries of the `test_dependencies` section.
    pub fn get_dependencies(&self, mode: BuildConfigType) -> Vec<ProjectFileDependency> {
        match mode {
            BuildConfigType::Test => {
                let mut all_deps = self.dependencies.clone();
                all_deps.extend(self.test_dependencies.clone());
                all_deps
            }
            BuildConfigType::Build => self.dependencies.clone(),
        }
    }

    /// Reads the project file of the current directory as the root project of the build, with its
    /// `role` and `source` populated.
    pub fn read_root_file() -> Result<ProjectFile, Errors> {
        let proj_file_path = Path::new(PROJECT_FILE_PATH);
        let mut proj_file = ProjectFile::read_file(&proj_file_path)?;
        proj_file.role = ProjectFileRole::Root;
        proj_file.source = Some(ProjectOrigin::Local(to_absolute_path(
            proj_file
                .path
                .parent()
                .expect("ProjectFile::path always points to fixproj.toml inside a directory"),
        )?));
        Ok(proj_file)
    }

    /// Reads the project file at `path`, checks its fields, and checks that the project accepts the
    /// running compiler's version. The returned file carries the default `role` and `source`, which
    /// the loader sets to match where the file came from.
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

    /// The hash that decides when the lock file has to be built again: it covers the dependency
    /// entries this project declares, and the whole project file of each path dependency, so that a
    /// change to what a local dependency itself depends on reaches the hash.
    ///
    /// # Arguments
    ///
    /// * `mode` - `Test` also covers the entries of the `test_dependencies` section.
    pub fn calculate_dependencies_hash(&self, mode: BuildConfigType) -> String {
        let mut deps = self.get_dependencies(mode);

        // Sort the dependencies by name.
        deps.sort_by(|a, b| a.name.cmp(&b.name));

        let mut hash_source = HashSource::default();
        hash_source.push_list(deps.iter().map(|dep| serde_json::to_string(dep).unwrap()));

        // Also include the content of path-based dependencies' project files in the hash.
        // This ensures that when a local path dependency changes its own dependencies,
        // the lock file is invalidated and re-created.
        hash_source.push_list(deps.iter().map(|dep| match &dep.path {
            Some(dep_dir) => {
                let proj_file_path = self.join_to_project_dir(dep_dir).join(PROJECT_FILE_PATH);
                fs::read_to_string(&proj_file_path).unwrap_or_default()
            }
            None => String::new(),
        }));

        hash_source.finish()
    }

    /// Checks that `name` is a non-empty string of alphanumeric characters and hyphens.
    ///
    /// # Arguments
    ///
    /// * `span` - The place in the project file the error points at, if the name was read from one.
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

    /// Checks one dependency entry: its project name, that it names exactly one of `path` and
    /// `git`, that its version requirement parses, and that a git entry pins at most one of `rev`
    /// and `tag`.
    ///
    /// # Arguments
    ///
    /// * `span` - The place in the project file each error points at.
    fn validate_dependency_entry(dep: &ProjectFileDependency, span: Span) -> Result<(), Errors> {
        // Validate the project name.
        Self::validate_project_name(&dep.name, Some(span.clone()))?;

        // Either of `path` or `git` should be specified.
        if (dep.path.is_none() && dep.git.is_none()) || (dep.path.is_some() && dep.git.is_some()) {
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
                    &[&Some(span.clone())],
                )
            })?;
        }

        // Validate git ref: rev and tag are mutually exclusive.
        if let Some(git) = &dep.git {
            if git.rev.is_some() && git.tag.is_some() {
                return Err(Errors::from_msg_srcs(
                    "Only one of `rev` or `tag` can be specified in a git dependency.".to_string(),
                    &[&Some(span)],
                ));
            }
        }

        Ok(())
    }

    /// Checks the fields the build reads from the project file: the project name, `version` and
    /// `fix_version`, the dependency entries and the uniqueness of their names, and
    /// `disable_cpu_features`. Each error points at the project file.
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

    /// Checks that every entry of `disable_cpu_features` is a valid regular expression.
    pub fn validate_disable_cpu_features(disable_cpu_features: &[String]) -> Result<(), Errors> {
        for feature in disable_cpu_features {
            // Check if each feature is a valid regex.
            if let Err(e) = Regex::new(feature) {
                return Err(Errors::from_msg(format!(
                    "Invalid regex in `disable-cpu-feature`: {}",
                    e
                )));
            }
        }
        Ok(())
    }

    /// The source-file entries listed in this project's own project file, each paired (via
    /// `Spanned`) with its byte range in that file.
    ///
    /// # Arguments
    ///
    /// * `mode` - `Test` also takes the files listed in the `[build.test]` section.
    fn source_file_entries(&self, mode: BuildConfigType) -> Vec<&Spanned<PathBuf>> {
        let mut entries: Vec<&Spanned<PathBuf>> = self.build.files.iter().collect();
        if mode == BuildConfigType::Test {
            entries.extend(self.test_only_file_entries());
        }
        entries
    }

    /// The source-file entries listed in the `[build.test]` section, which a test build compiles
    /// beside the ones an ordinary build compiles. Each is paired with its byte range in the
    /// project file.
    fn test_only_file_entries(&self) -> Vec<&Spanned<PathBuf>> {
        self.build
            .test
            .as_ref()
            .map_or(vec![], |test| test.files.iter().collect())
    }

    /// The names of the projects this project declares as dependencies for a build of the given
    /// mode. A test build declares the test dependencies beside the ordinary ones.
    fn declared_dependency_names(&self, mode: BuildConfigType) -> Set<ProjectName> {
        self.get_dependencies(mode)
            .iter()
            .map(|dep| dep.name.clone())
            .collect()
    }

    /// The paths of this project's own source files, resolved against the project directory.
    ///
    /// # Arguments
    ///
    /// * `mode` - `Test` also takes the files listed in the `[build.test]` section.
    pub fn get_files(&self, mode: BuildConfigType) -> Vec<PathBuf> {
        self.source_file_entries(mode)
            .iter()
            .map(|entry| self.join_to_project_dir(entry.get_ref()))
            .collect()
    }

    /// The paths of the source files a test build compiles beside the ones an ordinary build
    /// compiles, resolved against the project directory. A file the `[build.test]` section repeats
    /// from the `build` section is one of the ordinary sources, so it stays out.
    fn get_test_only_files(&self) -> Vec<PathBuf> {
        let build_files: Set<PathBuf> =
            self.get_files(BuildConfigType::Build).into_iter().collect();
        self.test_only_file_entries()
            .iter()
            .map(|entry| self.join_to_project_dir(entry.get_ref()))
            .filter(|path| !build_files.contains(path))
            .collect()
    }

    /// Checks that every source file listed in the project file exists on disk. Each error points at
    /// the offending entry, so an editor attaches the problem to the project file, which is where
    /// its cause is.
    ///
    /// # Arguments
    ///
    /// * `mode` - `Test` also checks the files listed in the `[build.test]` section.
    fn check_source_files_exist(&self, mode: BuildConfigType) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for entry in self.source_file_entries(mode) {
            if self.join_to_project_dir(entry.get_ref()).exists() {
                continue;
            }
            let span = entry.span();
            errors.append(Errors::from_msg_srcs(
                format!(
                    "Source file \"{}\" does not exist.",
                    entry.get_ref().to_string_lossy()
                ),
                &[&Some(self.project_file_span(span.start, span.end))],
            ));
        }
        errors.to_result()
    }

    /// The compiler versions this project builds with, as its `fix_version` field requires. A
    /// project file naming none builds with every version. Panics unless `validate` has accepted
    /// the field.
    pub fn fix_version(&self) -> VersionReq {
        match &self.general.fix_version {
            Some(v) => VersionReq::parse(v).unwrap(),
            None => VersionReq::STAR,
        }
    }

    /// Checks that the running compiler's version satisfies what this project requires of it.
    pub fn is_fix_version_compatible(&self) -> Result<(), Errors> {
        if self
            .fix_version()
            .matches(&Version::parse(env!("CARGO_PKG_VERSION")).unwrap())
        {
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

    /// The `Configuration::cu_size` a `cu_size` field of this project file asks for, or an error
    /// pointing at the project file when it asks for neither a positive number of entries nor the
    /// whole program in one unit.
    fn read_cu_size(&self, cu_size: &CompilationUnitSize) -> Result<usize, Errors> {
        let refused = |written: String| {
            Errors::from_msg_srcs(
                format!(
                    "A `cu_size` is a positive number or \"{}\", and this one is {}",
                    WHOLE_PROGRAM_IN_ONE_UNIT_STR, written
                ),
                &[&Some(self.project_file_span(0, 0))],
            )
        };
        match cu_size {
            CompilationUnitSize::Entries(0) => Err(refused("0".to_string())),
            CompilationUnitSize::Entries(entries) => Ok(*entries),
            CompilationUnitSize::Named(name) if name == WHOLE_PROGRAM_IN_ONE_UNIT_STR => {
                Ok(WHOLE_PROGRAM_IN_ONE_UNIT)
            }
            CompilationUnitSize::Named(name) => Err(refused(format!("\"{}\"", name))),
        }
    }

    /// The optimization level an `opt_level` field of this project file names, or an error pointing
    /// at the project file when it names no level the compiler has.
    fn read_opt_level(&self, opt_level: &str) -> Result<FixOptimizationLevel, Errors> {
        FixOptimizationLevel::from_str(opt_level).ok_or_else(|| {
            Errors::from_msg_srcs(
                format!("Unknown optimization level: \"{}\"", opt_level),
                &[&Some(self.project_file_span(0, 0))],
            )
        })
    }

    /// Links the libraries a `static_links` or `dynamic_links` field names, each bound to the
    /// program in the way `link_type` describes.
    fn link_libraries(
        config: &mut Configuration,
        libraries: Option<&[String]>,
        link_type: LinkType,
    ) {
        let Some(libraries) = libraries else {
            return;
        };
        config
            .linked_libraries
            .extend(libraries.iter().map(|name| (name.clone(), link_type)));
    }

    /// Updates a configuration from a project file.
    ///
    /// `self.role` decides whether the fields that only the root project contributes are skipped,
    /// and `self.source` tags `preliminary_commands` for trust-store lookup. Both must be populated
    /// before this is called.
    pub fn set_config(&self, config: &mut Configuration) -> Result<(), Errors> {
        let is_dependent_proj = self.role == ProjectFileRole::Dependent;
        let project_origin = self
            .source
            .clone()
            .expect("ProjectFile::source must be set by the loader before set_config");

        // Determine the build mode. A dependent project contributes its `build` section alone.
        let mut mode = config.subcommand.build_mode();
        if is_dependent_proj {
            mode = BuildConfigType::Build;
        }

        // Reject missing source files up front, so that the error points at the project-file entry
        // naming the file and an editor attaches it there.
        self.check_source_files_exist(mode)?;

        // Record what this project provides and what it declares, so that an import reaching past
        // the projects it declares can be told from one that stays within them. The sources of a
        // test build carry declarations of their own, since the test dependencies are the ones the
        // test sources may use, so they are recorded as a contribution beside the ordinary sources.
        config.project_sources.push(ProjectSources {
            name: self.general.name.clone(),
            version: self.general.version.clone(),
            origin: project_origin.clone(),
            declared_dependencies: self.declared_dependency_names(BuildConfigType::Build),
            files: self.get_files(BuildConfigType::Build),
        });
        if mode == BuildConfigType::Test {
            config.project_sources.push(ProjectSources {
                name: self.general.name.clone(),
                version: self.general.version.clone(),
                origin: project_origin.clone(),
                declared_dependencies: self.declared_dependency_names(BuildConfigType::Test),
                files: self.get_test_only_files(),
            });
        }

        // The records above are what the build compiles. The root project's files are the user's
        // own as well, which scopes diagnostics to the code they can edit: a deprecated use inside
        // a dependency is the dependency's problem.
        if !is_dependent_proj {
            config.root_source_files.extend(self.get_files(mode));
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
        if mode == BuildConfigType::Test {
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
        Self::link_libraries(config, self.build.static_links.as_deref(), LinkType::Static);
        if mode == BuildConfigType::Test {
            Self::link_libraries(
                config,
                self.build
                    .test
                    .as_ref()
                    .and_then(|test| test.static_links.as_deref()),
                LinkType::Static,
            );
        }

        // Append dynamic libraries.
        Self::link_libraries(
            config,
            self.build.dynamic_links.as_deref(),
            LinkType::Dynamic,
        );
        if mode == BuildConfigType::Test {
            Self::link_libraries(
                config,
                self.build
                    .test
                    .as_ref()
                    .and_then(|test| test.dynamic_links.as_deref()),
                LinkType::Dynamic,
            );
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
        if mode == BuildConfigType::Test {
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
        if mode == BuildConfigType::Test {
            config.ld_flags.append(
                &mut self
                    .build
                    .test
                    .as_ref()
                    .map_or(vec![], |test| test.ld_flags.clone()),
            );
        }

        // Set preliminary commands.
        let work_dir = to_absolute_path(
            self.path
                .parent()
                .expect("ProjectFile::path always points to fixproj.toml inside a directory"),
        )?;
        let project_name = self.general.name.clone();
        for command in &self.build.preliminary_commands {
            config.preliminary_commands.push(PreliminaryCommand {
                work_dir: work_dir.clone(),
                command: command.clone(),
                project_name: project_name.clone(),
                mode: PreliminaryCommandMode::Build,
                source: project_origin.clone(),
            });
        }
        if mode == BuildConfigType::Test {
            for command in &self
                .build
                .test
                .as_ref()
                .map_or(vec![], |test| test.preliminary_commands.clone())
            {
                config.preliminary_commands.push(PreliminaryCommand {
                    work_dir: work_dir.clone(),
                    command: command.clone(),
                    project_name: project_name.clone(),
                    mode: PreliminaryCommandMode::Test,
                    source: project_origin.clone(),
                });
            }
        }

        // Set the memory check mode.
        if mode == BuildConfigType::Test {
            if let Some(memcheck) = self.build.test.as_ref().and_then(|test| test.memcheck) {
                if memcheck {
                    config.set_valgrind(ValgrindTool::MemCheck);
                }
            }
        }

        // From here on, only the settings in the project file of the root project are reflected.
        if is_dependent_proj {
            return Ok(());
        }

        // Set threaded-mode.
        if let Some(threaded) = self.build.threaded {
            if threaded {
                config.set_threaded();
            }
        }
        if mode == BuildConfigType::Test {
            if let Some(threaded) = self.build.test.as_ref().and_then(|test| test.threaded) {
                if threaded {
                    config.set_threaded();
                }
            }
        }

        // Set the sanitizer.
        if let Some(sanitizer) = &self.build.sanitize {
            config.set_sanitizer(Sanitizer::from_str(sanitizer)?)?;
        }
        if mode == BuildConfigType::Test {
            if let Some(sanitizer) = self
                .build
                .test
                .as_ref()
                .and_then(|test| test.sanitize.as_ref())
            {
                config.set_sanitizer(Sanitizer::from_str(sanitizer)?)?;
            }
        }

        // Set debug mode.
        if let Some(debug) = self.build.debug {
            if debug {
                config.set_debug_info();
            }
        }
        if mode == BuildConfigType::Test {
            if let Some(debug) = self.build.test.as_ref().and_then(|test| test.debug) {
                if debug {
                    config.set_debug_info();
                }
            }
        }

        // Set the size of a compilation unit. A test build reads the value the `build` section
        // gives, since dividing a program is about how it is built rather than about what is built.
        if let Some(cu_size) = self.build.cu_size.as_ref() {
            config.cu_size = self.read_cu_size(cu_size)?;
        }

        // Set optimization level.
        if let Some(opt_level) = self.build.opt_level.as_ref() {
            config.set_fix_opt_level(self.read_opt_level(opt_level)?);
        }
        if mode == BuildConfigType::Test {
            if let Some(opt_level) = self
                .build
                .test
                .as_ref()
                .and_then(|test| test.opt_level.as_ref())
            {
                config.set_fix_opt_level(self.read_opt_level(opt_level)?);
            }
        }

        // The kind of file a build produces is read whatever the invocation is, so that a project
        // file naming a kind that does not exist is reported by every command that reads it.
        let output_file_type = self
            .build
            .output_type
            .as_ref()
            .map(|output_type| OutputFileType::from_str(output_type))
            .transpose()?;

        // Set the output file and its kind. The two describe what `fix build` produces; `fix run`
        // and `fix test` build an executable in a temporary place, run it, and remove it, so a
        // project file asking a build for a dynamic library still gets a program it can run, and a
        // test run leaves the output file of a build where it is.
        if config.subcommand.produces_output_file() {
            if let Some(output) = self.build.output.as_ref() {
                config.out_file_path = Some(PathBuf::from(output));
            }
            if let Some(output_file_type) = output_file_type {
                config.output_file_type = output_file_type;
            }
        }

        // Set backtrace mode.
        if let Some(backtrace) = self.build.backtrace {
            if backtrace {
                config.set_backtrace();
            }
        }
        if mode == BuildConfigType::Test {
            if let Some(backtrace) = self.build.test.as_ref().and_then(|test| test.backtrace) {
                if backtrace {
                    config.set_backtrace();
                }
            }
        }

        // Set disable_cpu_features.
        config
            .disable_cpu_features_regex
            .append(&mut self.build.disable_cpu_features.clone());
        if mode == BuildConfigType::Test {
            config.disable_cpu_features_regex.append(
                &mut self
                    .build
                    .test
                    .as_ref()
                    .map_or(vec![], |test| test.disable_cpu_features.clone()),
            );
        }

        // Set no_runtime_check and skip_eval. A test build reads these from the `build.test`
        // section alone, so that a project which drops the run-time checks or the `eval`
        // expressions from its program keeps them in its tests.
        let (no_runtime_check, skip_eval) = if mode == BuildConfigType::Test {
            let test = self.build.test.as_ref();
            (
                test.and_then(|test| test.no_runtime_check),
                test.and_then(|test| test.skip_eval),
            )
        } else {
            (self.build.no_runtime_check, self.build.skip_eval)
        };
        config.no_runtime_check = no_runtime_check.unwrap_or(false);
        config.skip_eval = skip_eval.unwrap_or(false);

        Ok(())
    }

    /// Open the lock file.
    /// If the project has no dependencies, return an empty lock file.
    pub fn open_lock_file(&self, mode: LockFileType) -> Result<DependecyLockFile, Errors> {
        // If there are no dependencies, the lock file is not necessary.
        if self
            .get_dependencies(mode.to_build_config_type())
            .is_empty()
        {
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
        let content = fs::read_to_string(lock_file_path).map_err(|e| {
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
        if lock_file.proj_file_hash != self.calculate_dependencies_hash(mode.to_build_config_type())
        {
            return Err(Errors::from_msg(format!(
                "The lock file is not up to date. {}",
                msg_try_fix_deps_update
            )));
        }
        Ok(lock_file)
    }

    /// Writes `lock_file` to the path a lock file of the given kind is read from.
    fn save_lock_file(lock_file: &DependecyLockFile, mode: LockFileType) -> Result<(), Errors> {
        let content = toml::to_string(lock_file)
            .map_err(|e| Errors::from_msg(format!("Failed to serialize lock file: {:?}", e)))?;
        let lock_file_path = get_lock_file_path(mode);
        fs::write(lock_file_path, content)
            .map_err(|e| Errors::from_msg(format!("Failed to write lock file: {:?}", e)))?;
        Ok(())
    }

    /// The lock file of the given kind. One that is missing or out of date is built afresh from
    /// this project's dependency entries and written out.
    pub fn open_or_create_lock_file(
        &self,
        mode: LockFileType,
    ) -> Result<DependecyLockFile, Errors> {
        Ok(match self.open_lock_file(mode) {
            Ok(lock_file) => lock_file,
            Err(_) => {
                let lock_file = DependecyLockFile::create(self, mode.to_build_config_type())?;
                Self::save_lock_file(&lock_file, mode)?;
                lock_file
            }
        })
    }

    /// The lock file of the given kind, with its dependencies installed. One that is missing or out
    /// of date is built afresh and written out, creating the directory that holds it where that is
    /// missing too, so that a project which has never been built reaches a lock file as well.
    pub fn open_or_auto_update_lock_file(
        &self,
        mode: LockFileType,
    ) -> Result<DependecyLockFile, Errors> {
        // Try to open existing lock file.
        match self.open_lock_file(mode) {
            Ok(lock_file) => Ok(lock_file),
            Err(_) => {
                // If the lock file does not exist or is invalid, automatically create/update it.

                // Ensure the parent directory exists (e.g., .fixlang/ for LSP lock file).
                let lock_file_path = get_lock_file_path(mode);
                if let Some(parent) = Path::new(lock_file_path).parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        Errors::from_msg(format!("Failed to create directory: {:?}", e))
                    })?;
                }

                // Create the lock file.
                let lock_file = DependecyLockFile::create(self, mode.to_build_config_type())?;

                // Save the lock file.
                Self::save_lock_file(&lock_file, mode)?;

                // Install the dependencies.
                lock_file.install()?;

                Ok(lock_file)
            }
        }
    }

    /// Installs the dependencies the lock file of the given kind pins, writing that lock file first
    /// where the one on disk is missing or out of date.
    pub fn open_or_create_lock_file_and_install(&self, mode: LockFileType) -> Result<(), Errors> {
        self.open_or_create_lock_file(mode)
            .and_then(|lf| lf.install())
    }

    /// Adds what this project's dependencies contribute to `config`: their source files, the
    /// libraries they link, and the rest of the settings their project files give. The dependencies
    /// are installed first, and the lock file written where the one on disk is missing or out of
    /// date.
    pub fn install_dependencies(
        self: &ProjectFile,
        config: &mut Configuration,
        mode: BuildConfigType,
    ) -> Result<(), Errors> {
        // Update the lock file if necessary.
        let lock_file =
            self.open_or_create_lock_file(LockFileType::from_build_config_type(mode))?;

        // Install the dependencies.
        lock_file.install()?;

        // See the dependencies and update the configuration.
        lock_file.set_config(config)?;

        Ok(())
    }

    /// The place in this project file a diagnostic points at.
    ///
    /// # Arguments
    ///
    /// * `start`, `end` - Byte offsets into the project file. `0, 0` points at its head, which is
    ///   where a diagnostic with no finer place to point goes.
    fn project_file_span(&self, start: usize, end: usize) -> Span {
        let input = SourceFile::from_file_path(self.path.clone());
        Span { start, end, input }
    }

    /// `path` resolved against the directory holding this project file. An absolute `path` is
    /// returned as it is.
    fn join_to_project_dir(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            return path.to_path_buf();
        } else {
            return self
                .path
                .parent()
                .expect("ProjectFile::path always points to fixproj.toml inside a directory")
                .join(path);
        }
    }

    /// Where the project named `name` is fetched from, as the `dependencies` and
    /// `test_dependencies` entries of this project file say. Panics when no entry names the
    /// project, or when the entry that does names no source.
    pub fn get_dependency_source(&self, name: &ProjectName) -> ProjectSource {
        for dep in self
            .dependencies
            .iter()
            .chain(self.test_dependencies.iter())
        {
            if &dep.name != name {
                continue;
            }
            if let Some(dep_dir) = &dep.path {
                return ProjectSource::Local(self.join_to_project_dir(dep_dir));
            }
            if let Some(git) = &dep.git {
                return ProjectSource::Git(git.url.clone(), None);
            }
            panic!("No source specified for dependency `{}`.", name);
        }
        panic!("Project `{}` not found in dependencies.", name);
    }

    /// Creates a project file named after `proj_name`, a sample "main.fix" and a sample "test.fix"
    /// in the current directory. An error is raised as soon as one of the three names is taken, and
    /// the file that carries it is left as it is.
    pub fn create_example_file(proj_name: String) -> Result<(), Errors> {
        if Path::new(PROJECT_FILE_PATH).exists() {
            return Err(Errors::from_msg(format!(
                "The file \"{}\" already exists.",
                PROJECT_FILE_PATH
            )));
        }

        let content = include_str!("../docs/project_template.toml");

        // Replace `{PLACEHOLDER_PROJECT_NAME}` with `proj_name`.
        let content = content.replace("{PLACEHOLDER_PROJECT_NAME}", &proj_name);

        // Replace `{PLACEHOLDER_FIX_VERSION}` with the current version of Fix.
        let content = content.replace("{PLACEHOLDER_FIX_VERSION}", env!("CARGO_PKG_VERSION"));

        fs::write(PROJECT_FILE_PATH, content).map_err(|e| {
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
        let main_fix_content = include_str!("../docs/main_template.fix");
        fs::write(SAMPLE_MAIN_FILE_PATH, main_fix_content).map_err(|e| {
            Errors::from_msg(format!("Failed to create file \"main.fix\": {:?}.", e))
        })?;

        // Create sample "test.fix" file in the current directory.
        if Path::new(SAMPLE_TEST_FILE_PATH).exists() {
            return Err(Errors::from_msg(format!(
                "The file \"test.fix\" already exists."
            )));
        }
        let test_fix_content = include_str!("../docs/test_template.fix");
        fs::write(SAMPLE_TEST_FILE_PATH, test_fix_content).map_err(|e| {
            Errors::from_msg(format!("Failed to create file \"test.fix\": {:?}.", e))
        })?;

        Ok(())
    }

    /// Appends a dependency entry for each project of `proj_specs` to this project file, taking the
    /// repository of each from the registries `fix_config` names.
    ///
    /// # Arguments
    ///
    /// * `proj_specs` - Each is `proj-name` or `proj-name@ver_req`. A name written without a
    ///   version requirement takes the latest tagged version of the project.
    /// * `mode` - `Build` writes `[[dependencies]]` entries, `Test` writes `[[test_dependencies]]`
    ///   ones.
    pub fn add_dependencies(
        &self,
        proj_specs: &Vec<String>,
        fix_config: &ConfigFile,
        mode: BuildConfigType,
    ) -> Result<(), Errors> {
        let mut added_toml = "".to_string();

        // Parse each element of `proj_specs` as the form `proj-name@ver_req`.
        let mut proj_ver_reqs: Vec<(String, Option<String>)> = vec![]; // (proj_name, ver_req)
        for proj_spec in proj_specs {
            let proj_spec_parts = proj_spec.split('@').collect::<Vec<&str>>();
            if proj_spec_parts.len() == 0 || proj_spec_parts.len() > 2 {
                return Err(Errors::from_msg(format!(
                    "Invalid project specification: \"{}\". It should be in the form \"proj-name\" or \"proj-name@ver_req\"",
                    proj_spec
                )));
            }
            let proj_name = proj_spec_parts[0];
            ProjectFile::validate_project_name(&proj_name.to_string(), None)?;
            let version = if proj_spec_parts.len() == 2 {
                let _ = VersionReq::parse(proj_spec_parts[1]).map_err(|e| {
                    Errors::from_msg(format!(
                        "Failed to parse version requirement in \"{}\": {:?}",
                        proj_spec, e
                    ))
                })?;
                Some(proj_spec_parts[1].to_string())
            } else {
                None
            };
            proj_ver_reqs.push((proj_name.to_string(), version));
        }

        // Check if dependencies to the same project are specified multiple times.
        for i in 0..proj_ver_reqs.len() {
            for j in i + 1..proj_ver_reqs.len() {
                if proj_ver_reqs[i].0 == proj_ver_reqs[j].0 {
                    return Err(Errors::from_msg(format!(
                        "The project \"{}\" is specified multiple times.",
                        proj_ver_reqs[i].0
                    )));
                }
            }
        }

        // Check if the project file already has the dependencies.
        let existing_deps = self.get_dependencies(mode);
        for proj_ver_req in &proj_ver_reqs {
            let proj_name = &proj_ver_req.0;
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
            for (proj_idx, proj_ver_req) in proj_ver_reqs.iter().enumerate() {
                let (proj_name, version) = proj_ver_req;
                if let Some(proj_info) = reg_file
                    .projects
                    .iter()
                    .find(|proj_info| &proj_info.name == proj_name)
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
                            let (_tmp_dir, repo) = clone_git_repo(&proj_info.git)?;
                            let version_infos = get_versions_from_repo(&repo)?;
                            let mut tagged_versions = version_infos
                                .iter()
                                .filter_map(|version_info| {
                                    if version_info.tagged {
                                        Some(version_info.version.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>();
                            tagged_versions.sort();
                            if tagged_versions.is_empty() {
                                warn_msg(&format!(
                                    "Adding version requirement \"*\" for \"{}\" since there are no tagged versions. \
                                    This means that updating the lock file (which is done by `fix deps add` or `fix deps update`) may introduce breaking changes.",
                                    proj_name,
                                ));
                                "*".to_string()
                            } else {
                                let latest = tagged_versions.pop().unwrap();
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
                        BuildConfigType::Build => "[[dependencies]]",
                        BuildConfigType::Test => "[[test_dependencies]]",
                    };
                    added_toml += "\n\n";
                    added_toml += section_name;
                    added_toml += &format!("\nname = \"{}\"", proj_name);
                    added_toml += &format!("\nversion = \"{}\"", version);
                    added_toml += &format!("\ngit = {{ url = \"{}\" }}", proj_info.git);

                    added_indices.insert(proj_idx);
                }
            }

            // Remove the projects that have been added.
            proj_ver_reqs = proj_ver_reqs
                .into_iter()
                .enumerate()
                .filter_map(|(proj_idx, proj_ver_req)| {
                    if added_indices.contains(&proj_idx) {
                        None
                    } else {
                        Some(proj_ver_req)
                    }
                })
                .collect();
        }

        // Check if all the projects have been added.
        for proj_ver_req in proj_ver_reqs {
            return Err(Errors::from_msg(format!(
                "The project \"{}\" is not found in the registries.",
                proj_ver_req.0
            )));
        }

        // Write the added dependencies to the project file.
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&self.path)
            .map_err(|e| {
                Errors::from_msg(format!(
                    "Failed to open file \"{}\": {:?}",
                    self.path.to_string_lossy().to_string(),
                    e
                ))
            })?;
        file.write_all(added_toml.as_bytes()).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to write to file \"{}\": {:?}",
                self.path.to_string_lossy().to_string(),
                e
            ))
        })?;
        Ok(())
    }

    /// Retrieves the registry file at `loc` and parses it.
    ///
    /// # Arguments
    ///
    /// * `loc` - A URL the file is fetched over HTTP from, or a path it is read from. A location
    ///   that parses as a URL is treated as one.
    pub fn retrieve_registry_file(loc: &str) -> Result<RegistryFile, Errors> {
        let reg_file_content = if Url::parse(loc).is_ok() {
            // The location is a URL.
            let response = reqwest::blocking::get(loc).map_err(|e| {
                Errors::from_msg(format!(
                    "Failed to fetch registry file \"{}\": {:?}",
                    loc, e
                ))
            })?;
            response.text().map_err(|e| {
                Errors::from_msg(format!(
                    "Failed to fetch registry file \"{}\": {:?}",
                    loc, e
                ))
            })?
        } else {
            // The location is a file path.
            fs::read_to_string(loc).map_err(|e| {
                Errors::from_msg(format!("Failed to read registry file \"{}\": {:?}", loc, e))
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

#[cfg(test)]
mod tests {
    use super::ProjectFile;
    use crate::configuration::BuildConfigType;
    use std::fs;
    use tempfile::TempDir;

    /// The hash deciding when the lock file is built again covers the whole project file of each
    /// path dependency, so that a change to what a local dependency itself depends on reaches it. A
    /// lock file surviving such a change would name the dependencies of a project that has since
    /// asked for others.
    #[test]
    fn test_the_dependencies_hash_follows_a_path_dependencys_own_project_file() {
        let temp = TempDir::new().expect("Failed to create temp directory");
        let root = temp.path().join("root");
        let dep = temp.path().join("dep");
        fs::create_dir_all(&root).expect("Failed to create the root project's directory");
        fs::create_dir_all(&dep).expect("Failed to create the dependency's directory");
        fs::write(
            root.join("fixproj.toml"),
            "[general]\nname = \"root\"\nversion = \"0.1.0\"\n\n\
             [build]\nfiles = [\"main.fix\"]\n\n\
             [[dependencies]]\nname = \"dep\"\nversion = \"*\"\npath = \"../dep\"\n",
        )
        .expect("Failed to write the root project file");

        let hash_of = |dependencies_of_the_dependency: &str| {
            fs::write(
                dep.join("fixproj.toml"),
                format!(
                    "[general]\nname = \"dep\"\nversion = \"0.1.0\"\n\n\
                     [build]\nfiles = [\"lib.fix\"]\n{}",
                    dependencies_of_the_dependency
                ),
            )
            .expect("Failed to write the dependency's project file");
            ProjectFile::read_file(&root.join("fixproj.toml"))
                .unwrap_or_else(|errs| panic!("Failed to read the root project file: {}", errs))
                .calculate_dependencies_hash(BuildConfigType::Build)
        };

        assert_ne!(
            hash_of(""),
            hash_of("\n[[dependencies]]\nname = \"other\"\nversion = \"*\"\npath = \"../other\"\n"),
            "a dependency the path dependency itself declares reaches the hash the lock file is \
             checked against"
        );
    }
}
