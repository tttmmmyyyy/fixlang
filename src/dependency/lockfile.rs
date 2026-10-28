use crate::{
    configuration::{BuildConfigType, Configuration},
    constants::{
        EXTERNAL_PROJ_INSTALL_PATH, LOCK_FILE_LSP_PATH, LOCK_FILE_PATH, LOCK_FILE_TEST_PATH,
        PROJECT_FILE_PATH,
    },
    dependency::resolver::{
        self, Dependency, Package, PackageName, RetrievalError, Source, SourcedVersion,
    },
    error::Errors,
    metafiles::project_file::{
        compiler_version, ProjectFile, ProjectFileDependencyGit, ProjectFileRole, ProjectName,
        ProjectOrigin,
    },
    misc::{info_msg, to_absolute_path, warn_msg, Map},
    project_fs::{
        create_dir_guarding_project_links, remove_inside_dot_fixlang, DOT_FIXLANG_LINK_HINT,
    },
};
use git2::{build::CheckoutBuilder, ErrorClass, ErrorCode, Oid, Repository};
use rand::{thread_rng, Rng};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tempfile::TempDir;

#[derive(Clone, Copy)]
pub enum LockFileType {
    Build,
    Test,
    Lsp,
}

impl LockFileType {
    pub fn to_build_config_type(&self) -> BuildConfigType {
        match self {
            LockFileType::Build => BuildConfigType::Build,
            LockFileType::Test => BuildConfigType::Test,
            LockFileType::Lsp => BuildConfigType::Test, // For LSP, dependencies are resolved in the same way as Test mode.
        }
    }

    pub fn from_build_config_type(config_type: BuildConfigType) -> Self {
        match config_type {
            BuildConfigType::Build => LockFileType::Build,
            BuildConfigType::Test => LockFileType::Test,
        }
    }
}

// Get the lock file path based on the dependency mode.
pub fn get_lock_file_path(mode: LockFileType) -> &'static str {
    match mode {
        LockFileType::Test => LOCK_FILE_TEST_PATH,
        LockFileType::Build => LOCK_FILE_PATH,
        LockFileType::Lsp => LOCK_FILE_LSP_PATH,
    }
}

/// A lock file: the dependencies, direct and indirect, that a project's dependency entries resolved
/// to, each pinned to one version.
#[derive(Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DependencyLockFile {
    /// The hash of the dependency entries of the project file the lock file was resolved from. A
    /// lock file whose hash differs from that of the project file is out of date.
    pub proj_file_hash: String,
    /// The dependencies, sorted by name.
    pub dependencies: Vec<DependencyLockFileEntry>,
}

impl DependencyLockFile {
    // Update configuration by adding source files, linking libraries, ... as required by dependencies.
    pub fn set_config(&self, config: &mut Configuration) -> Result<(), Errors> {
        for dep in &self.dependencies {
            let proj_file = dep.project_file()?;
            proj_file.set_config(config)?;
        }
        Ok(())
    }

    // Create the lock file (on memory, not on file) to satisfy the dependencies of the given project file.
    pub fn create(
        proj_file: &ProjectFile,
        mode: BuildConfigType,
    ) -> Result<DependencyLockFile, Errors> {
        // Resolve the dependency.
        let root_info = ProjectInfo::from_project_file(proj_file)?;
        let root_version = SourcedVersion {
            version: proj_file.general.version(),
            source: root_info.source_id.clone(),
        };
        let prjs_info = ProjectsInfo {
            projects: Arc::new(Mutex::new(vec![root_info])),
        };

        // Pre-register pinned dependencies (those with rev or tag) in ProjectsInfo.
        // By pre-populating a ProjectInfo with `versions` set to a single pinned version
        // before the resolver runs, we force the resolver to use exactly that version:
        // `version_retriever` returns only the pinned version (since `retrieve_versions`
        // early-returns when `versions` is already set).
        let all_deps = proj_file.get_dependencies(mode);
        for dep in all_deps
            .iter()
            .filter(|d| d.git.as_ref().map_or(false, |g| g.has_ref()))
        {
            let git = dep.git.as_ref().unwrap();

            // Create a ProjectInfo and resolve the pinned ref.
            let mut proj_info =
                ProjectInfo::new(dep.name.clone(), ProjectSource::Git(git.url.clone(), None))?;
            let (version, _oid) = proj_info.resolve_pinned_ref(git)?;

            // Verify name matches.
            let pinned_name = &proj_info.proj_files[&version].general.name;
            if *pinned_name != dep.name {
                return Err(Errors::from_msg(format!(
                    "Dependency \"{}\" pinned to {} resolves to project \"{}\", which has a different name.",
                    dep.name,
                    git.ref_description(),
                    pinned_name
                )));
            }

            // Verify version constraint (if version is specified).
            let requirement = dep.version();
            if !requirement.matches(&version) {
                return Err(Errors::from_msg(format!(
                    "Dependency \"{}\" is pinned to {} (version {}), but the version requirement \"{}\" is not satisfied.",
                    dep.name,
                    git.ref_description(),
                    version,
                    requirement
                )));
            }

            // Register in ProjectsInfo.
            prjs_info.projects.lock().unwrap().push(proj_info);
        }

        let packages_retriever = create_package_retriever(prjs_info.clone());
        let versions_retriever = create_version_retriever(prjs_info.clone());
        info_msg(&format!(
            "Resolving dependency for \"{}\"...",
            proj_file.general.name
        ));
        let chosen = resolver::resolve_dependency(
            (&proj_file.general.name, &root_version),
            mode,
            &compiler_version(),
            packages_retriever.as_ref(),
            versions_retriever.as_ref(),
        )?;

        // Create a new lock file following the resolved dependencies.
        let mut lock_file = DependencyLockFile {
            proj_file_hash: proj_file.calculate_dependencies_hash(mode),
            dependencies: Vec::new(),
        };
        for (
            prj_name,
            SourcedVersion {
                version: prj_version,
                source,
            },
        ) in chosen
        {
            // Get the information of the dependent project.
            let mut projects = prjs_info.projects.as_ref().lock().unwrap();
            let prj_info = find_project(&mut projects, &prj_name, &source).unwrap_or_else(|| {
                panic!("\"{}\" from {} is not in `projs_info`", prj_name, source)
            });
            let ver_info = prj_info
                .versions
                .as_ref()
                .unwrap()
                .iter()
                .find(|info| &info.version == &prj_version)
                .unwrap();

            // If the version is not tagged, then warn the user.
            if prj_info.is_git() && !ver_info.tagged {
                let short_commit = format!("{}", ver_info.rev)
                    .chars()
                    .take(7)
                    .collect::<String>();
                warn_msg(&format!(
                    "No version tag is found for \"{}\", and using untagged version \"{}\" (commit {}).",
                    prj_name, prj_version, short_commit
                ));
            }

            // Create a new entry for the lock file.
            let dep = DependencyLockFileEntry {
                name: prj_name.clone(),
                version: prj_version.to_string(),
                path: match &prj_info.source {
                    ProjectSource::Local(path_buf) => path_buf.clone(),
                    ProjectSource::Git(_, _) => {
                        git_dependency_install_path(&prj_name, &prj_version.to_string())
                    }
                },
                git: match &prj_info.source {
                    ProjectSource::Local(_) => None,
                    ProjectSource::Git(url, _) => Some(DependencyLockGit {
                        repo: url.clone(),
                        rev: ver_info.rev.to_string(),
                    }),
                },
            };

            // Add the entry to the lock file.
            lock_file.dependencies.push(dep);
        }

        info_msg("Dependencies resolved successfully.");
        Ok(lock_file)
    }

    /// Checks that each entry follows the rules of the lock files fix writes: an entry names a
    /// project and a version, and a git entry pins a commit and lies at
    /// `git_dependency_install_path` of its name and version.
    ///
    /// # Returns
    ///
    /// A description of the first entry that breaks a rule.
    pub fn validate(&self) -> Result<(), String> {
        for dep in &self.dependencies {
            dep.validate()?;
        }
        Ok(())
    }

    /// Installs the dependencies: clones each git dependency into its install directory and checks
    /// out its commit, and checks that each dependency found there has the name and the version the
    /// lock file gives. A git dependency whose directory holds another commit, or no git repository,
    /// is removed and cloned again.
    pub fn install(&self) -> Result<(), Errors> {
        for dep in &self.dependencies {
            if let Some(git_info) = &dep.git {
                let target_rev = Oid::from_str(&git_info.rev)
                    .expect("a lock file holds a commit hash in each `rev`");
                // In case the source is a git repository,
                if dep.path.exists() {
                    // If the path exists, check that the revision is correct.
                    // If the revision is incorrect, or any git error occurs, remove the directory and clone the repository again.
                    // A HEAD that resolves to no commit holds no revision, and the directory is
                    // cloned again.
                    let rev_match = Repository::open(&dep.path)
                        .and_then(|repo| Ok(repo.head()?.target() == Some(target_rev)));
                    if rev_match == Ok(true) {
                        // If the revision is ok, load the project file and validate whether it satisfies the dependency.
                        dep.check_name_version_match_proj_file()?;
                        continue; // This dependency is already installed and ok.
                    }
                    // If something is wrong, remove the directory.
                    remove_inside_dot_fixlang(&dep.path)?;
                }
                // Create the directory, and clone the repository into it.
                let clone_dir =
                    create_dir_guarding_project_links(&dep.path, DOT_FIXLANG_LINK_HINT)?;
                let repo =
                    clone_repository_with_retry(&git_info.repo, &clone_dir).map_err(|e| {
                        Errors::from_msg(format!(
                            "Failed to clone the repository \"{}\" to \"{}\": {:?}",
                            git_info.repo,
                            dep.path.to_string_lossy().to_string(),
                            e
                        ))
                    })?;

                // Checkout the specified revision.
                let commit = repo
                    .find_commit(target_rev)
                    .map_err(|e| Errors::from_msg(format!("Failed to find commit: {:?}", e)))?;
                let mut checkout_opts = CheckoutBuilder::default();
                checkout_opts.force();
                repo.checkout_tree(&commit.into_object(), Some(&mut checkout_opts))
                    .map_err(|e| Errors::from_msg(format!("Failed to checkout commit: {:?}", e)))?;

                // Set HEAD to the target revision.
                repo.set_head_detached(target_rev)
                    .map_err(|e| Errors::from_msg(format!("Failed to set head: {:?}", e)))?;

                // Load the project file and validate whether it satisfies the dependency.
                dep.check_name_version_match_proj_file()?;

                info_msg(&format!(
                    "Dependency \"{}@{}\" installed at \"{}\".",
                    dep.name,
                    dep.version,
                    dep.path.to_string_lossy().to_string()
                ));
            } else {
                // In case the source is a project directory,
                // Check the path exists.
                if !dep.path.exists() {
                    return Err(Errors::from_msg(format!(
                        "Dependency \"{}\" is not found at \"{}\" as required in \"{}\".",
                        dep.name,
                        dep.path.to_string_lossy().to_string(),
                        LOCK_FILE_PATH,
                    )));
                }

                // Load the project file and validate whether it satisfies the dependency.
                dep.check_name_version_match_proj_file()?;
                // This dependency is already installed and ok.
            }
        }
        Ok(())
    }

    /// Writes the lock file of the given mode afresh from the project file, whatever lock file is
    /// on disk, and installs the dependencies it pins.
    pub fn update_and_install(mode: BuildConfigType) -> Result<(), Errors> {
        ProjectFile::read_root_file()?
            .create_lock_file(LockFileType::from_build_config_type(mode))?
            .install()
    }
}

/// The directory a git dependency of the given name and version is cloned into.
fn git_dependency_install_path(name: &str, version: &str) -> PathBuf {
    PathBuf::from(EXTERNAL_PROJ_INSTALL_PATH).join(format!("{}_{}", name, version))
}

/// One dependency of a lock file.
#[derive(Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DependencyLockFileEntry {
    /// The name of the project.
    name: String,
    /// The version of the project, as `1.0.1`.
    version: String,
    /// The directory holding the project: `git_dependency_install_path` of the name and the version
    /// for a git dependency, and the directory the project file names for a local one.
    path: PathBuf,
    /// Where a git dependency is cloned from. A local dependency has none.
    git: Option<DependencyLockGit>,
}

impl DependencyLockFileEntry {
    /// Checks this entry against the rules `DependencyLockFile::validate` lists.
    fn validate(&self) -> Result<(), String> {
        if !ProjectFile::is_project_name(&self.name) {
            return Err(format!("\"{}\" is not a project name", self.name));
        }
        if let Err(e) = Version::parse(&self.version) {
            return Err(format!(
                "the version \"{}\" of \"{}\" is not a version: {}",
                self.version, self.name, e
            ));
        }
        if let Some(git) = &self.git {
            if let Err(e) = Oid::from_str(&git.rev) {
                return Err(format!(
                    "the revision \"{}\" of \"{}\" is not a commit hash: {}",
                    git.rev,
                    self.name,
                    e.message()
                ));
            }
            let install_path = git_dependency_install_path(&self.name, &self.version);
            if self.path != install_path {
                return Err(format!(
                    "the git dependency \"{}\" is at \"{}\", where it belongs at \"{}\"",
                    self.name,
                    self.path.to_string_lossy(),
                    install_path.to_string_lossy()
                ));
            }
        }
        Ok(())
    }

    /// This dependency's `fixproj.toml`, with `role = Dependent` and its `source` (Git when the
    /// entry carries git info, Local otherwise) filled in, ready to be passed into `set_config`.
    pub fn project_file(&self) -> Result<ProjectFile, Errors> {
        let proj_file_path = self.path.join(PROJECT_FILE_PATH);
        let mut pf = ProjectFile::read_file(&proj_file_path)?;
        pf.is_fix_version_compatible()?;
        pf.role = ProjectFileRole::Dependent;
        pf.source = Some(match &self.git {
            Some(g) => ProjectOrigin::Git {
                url: g.repo.clone(),
                commit: g.rev.clone(),
            },
            None => ProjectOrigin::Local(to_absolute_path(&self.path)?),
        });
        Ok(pf)
    }

    pub fn check_name_version_match_proj_file(&self) -> Result<(), Errors> {
        let proj_file = self.project_file()?;
        if proj_file.general.name != self.name {
            return Err(Errors::from_msg(format!(
                "The name of the dependency installed at \"{}\" does not match the one specified in \"{}\". Try to run `fix deps update`.",
                self.path.to_string_lossy().to_string(),
                LOCK_FILE_PATH,
            )));
        }
        if proj_file.general.version()
            != Version::parse(&self.version).expect("a lock file holds a version in each `version`")
        {
            return Err(Errors::from_msg(format!(
                "The version of the dependency \"{}\" installed at \"{}\" does not match the one specified in \"{}\". Try to run `fix deps update`.",
                self.name, self.path.to_string_lossy().to_string(), LOCK_FILE_PATH,
            )));
        }
        Ok(())
    }
}

/// The repository and the commit a git dependency of a lock file is pinned to.
#[derive(Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
struct DependencyLockGit {
    /// The URL of the repository.
    repo: String,
    /// The hash of the commit.
    rev: String,
}

// Following structures are used to cache project information retrieved by file IO or network IO.

/// The projects the resolution has learned of, each at one source.
#[derive(Clone, Default)]
struct ProjectsInfo {
    /// The projects, the root project first.
    projects: Arc<Mutex<Vec<ProjectInfo>>>,
}

struct ProjectInfo {
    name: ProjectName,
    source: ProjectSource,
    /// `source` as the resolver identifies it, with a directory made absolute.
    source_id: Source,
    versions: Option<Vec<VersionInfo>>, // Available versions. None if not retrieved yet.
    /// The project files read so far, each under the version listed for the commit or the
    /// directory it was read from. A file can name a different version from the one it is listed
    /// under, so the key is the listed version.
    proj_files: Map<Version, ProjectFile>,
}

impl ProjectInfo {
    pub fn is_git(&self) -> bool {
        matches!(self.source, ProjectSource::Git(_, _))
    }

    /// The project `name` at `source`, whose versions are retrieved on demand.
    fn new(name: ProjectName, source: ProjectSource) -> Result<Self, Errors> {
        Ok(ProjectInfo {
            name,
            source_id: source.to_source()?,
            source,
            versions: None,
            proj_files: Map::default(),
        })
    }

    /// The project of `proj_file`, at the directory of the file, in its one version.
    fn from_project_file(proj_file: &ProjectFile) -> Result<Self, Errors> {
        let mut project = ProjectInfo::new(
            proj_file.general.name.clone(),
            ProjectSource::Local(proj_file.path.parent().unwrap().to_path_buf()),
        )?;
        project.versions = Some(vec![VersionInfo {
            version: proj_file.general.version(),
            rev: Oid::zero(),
            tagged: false,
        }]);
        project
            .proj_files
            .insert(proj_file.general.version(), proj_file.clone());
        Ok(project)
    }

    /// Lists the versions of the project at its source, once. A project directory whose project
    /// file cannot be read is unusable; a repository that cannot be read fails the retrieval.
    fn retrieve_versions(&mut self) -> Result<(), RetrievalError> {
        // If the versions are already retrieved, do nothing.
        if self.versions.is_some() {
            return Ok(());
        }
        self.source
            .prepare_git_repository()
            .map_err(RetrievalError::Fatal)?;
        match &mut self.source {
            ProjectSource::Local(proj_path) => {
                let proj_file = ProjectFile::read_file(&proj_path.join(PROJECT_FILE_PATH))
                    .map_err(RetrievalError::Unusable)?;
                let version = proj_file.general.version();
                self.versions = Some(vec![VersionInfo {
                    version: version.clone(),
                    rev: Oid::zero(),
                    tagged: false,
                }]);
                self.proj_files.insert(version, proj_file);
            }
            ProjectSource::Git(_url, repo) => {
                let repo = &repo.as_mut().unwrap().1;
                self.versions = Some(get_versions_from_repo(repo)?);
            }
        }
        Ok(())
    }

    /// The project file at the given version. A project file that cannot be read makes the version
    /// unusable; a repository that cannot be read fails the retrieval.
    fn get_project_file(&mut self, version: &Version) -> Result<ProjectFile, RetrievalError> {
        self.retrieve_versions()?;
        if let Some(proj_file) = self.proj_files.get(version) {
            return Ok(proj_file.clone());
        }

        // The resolver asks only for versions the versions retriever listed.
        let rev = self
            .versions
            .as_ref()
            .unwrap()
            .iter()
            .find(|info| &info.version == version)
            .unwrap_or_else(|| panic!("version {} of \"{}\" was not listed", version, self.name))
            .rev;
        assert!(
            self.is_git(),
            "the project file of \"{}\" in a project directory is read as its version is listed",
            self.name
        );

        // Check out the commit of the version and read the project file.
        self.source
            .prepare_git_repository()
            .map_err(RetrievalError::Fatal)?;
        let failure = |what: &str, e| {
            RetrievalError::Fatal(Errors::from_msg_err(
                &format!("Failed to {} of \"{}\" {}", what, self.name, version),
                e,
            ))
        };
        let repo = self.source.get_git_repository();
        let commit = repo
            .find_commit(rev)
            .map_err(|e| failure("find the commit", e))?;
        let mut checkout_opts = CheckoutBuilder::default();
        checkout_opts.force();
        repo.checkout_tree(&commit.into_object(), Some(&mut checkout_opts))
            .map_err(|e| failure("check out the commit", e))?;
        let proj_file = ProjectFile::read_file(&repo.workdir().unwrap().join(PROJECT_FILE_PATH))
            .map_err(RetrievalError::Unusable)?;
        self.proj_files.insert(version.clone(), proj_file.clone());

        Ok(proj_file)
    }

    /// The project file at the given version, checked to name this project at that version. A
    /// version whose project file cannot be read, or names another project or version, is
    /// unusable.
    fn usable_project_file(&mut self, version: &Version) -> Result<ProjectFile, RetrievalError> {
        let proj_file = self.get_project_file(version)?;
        if proj_file.general.name != self.name {
            return Err(RetrievalError::Unusable(Errors::from_msg(format!(
                "\"{}\" is found, but a different project name \"{}\" is specified in its project file.",
                self.name, proj_file.general.name
            ))));
        }
        if proj_file.general.version() != *version {
            return Err(RetrievalError::Unusable(Errors::from_msg(format!(
                "\"{}@{}\" is found, but a different version \"{}\" is specified in its project file.",
                self.name,
                version,
                proj_file.general.version()
            ))));
        }
        Ok(proj_file)
    }

    /// Resolve a pinned git ref (rev or tag) and pre-populate versions and proj_files.
    fn resolve_pinned_ref(
        &mut self,
        git_ref: &ProjectFileDependencyGit,
    ) -> Result<(Version, Oid), Errors> {
        // Clone the repository.
        self.source.prepare_git_repository()?;
        let repo = self.source.get_git_repository();

        // Resolve rev/tag to OID.
        let oid = resolve_git_ref(repo, git_ref)?;

        // Checkout and read fixproj.toml (same pattern as get_project_file).
        let commit = repo
            .find_commit(oid)
            .map_err(|e| Errors::from_msg_err("Failed to find commit", e))?;
        let mut checkout_opts = CheckoutBuilder::default();
        checkout_opts.force();
        repo.checkout_tree(&commit.into_object(), Some(&mut checkout_opts))
            .map_err(|e| Errors::from_msg_err("Failed to checkout commit", e))?;
        let proj_file = ProjectFile::read_file(&repo.workdir().unwrap().join(PROJECT_FILE_PATH))?;

        let version = proj_file.general.version();

        // Pre-populate versions (single entry) and proj_files cache.
        self.versions = Some(vec![VersionInfo {
            version: version.clone(),
            rev: oid,
            tagged: true,
        }]);
        self.proj_files.insert(version.clone(), proj_file);

        Ok((version, oid))
    }
}

// Resolve a rev or tag specification to a git OID.
fn resolve_git_ref(repo: &Repository, git: &ProjectFileDependencyGit) -> Result<Oid, Errors> {
    if let Some(rev) = &git.rev {
        let obj = repo
            .revparse_single(rev)
            .map_err(|e| Errors::from_msg(format!("Failed to find rev \"{}\": {}", rev, e)))?;
        Ok(obj.id())
    } else if let Some(tag) = &git.tag {
        let refname = format!("refs/tags/{}", tag);
        let reference = repo
            .find_reference(&refname)
            .map_err(|e| Errors::from_msg(format!("Failed to find tag \"{}\": {}", tag, e)))?;
        let obj = reference.peel_to_commit().map_err(|e| {
            Errors::from_msg(format!("Failed to peel tag \"{}\" to commit: {}", tag, e))
        })?;
        Ok(obj.id())
    } else {
        unreachable!("resolve_git_ref called without rev or tag")
    }
}

/// The version of a project that `fix deps add` writes a requirement for.
pub enum VersionToAdd {
    /// This tagged version.
    Tagged(Version),
    /// The repository has no tagged version, and its newest commit is usable.
    Untagged,
}

/// Chooses the version of the project `name` in the git repository at `url` that `fix deps add`
/// writes a requirement for: the newest tagged version the resolution can use, that is, one whose
/// project file can be read, names this project at that version, and accepts the running
/// compiler. A repository with no tag is judged by its newest commit. When no version is usable,
/// the error says why each was left out.
pub fn version_to_add(name: &ProjectName, url: &str) -> Result<VersionToAdd, Errors> {
    let mut project = ProjectInfo::new(name.clone(), ProjectSource::Git(url.to_string(), None))?;
    project.retrieve_versions().map_err(|err| match err {
        RetrievalError::Fatal(errs) => errs,
        RetrievalError::Unusable(errs) => Errors::from_msg(format!(
            "No version of \"{}\" can be used: {}",
            name,
            errs.messages()
        )),
    })?;
    let listed = project.versions.as_ref().unwrap();
    let any_tagged = listed.iter().any(|info| info.tagged);
    let mut candidates = listed
        .iter()
        .filter(|info| info.tagged == any_tagged)
        .map(|info| info.version.clone())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.reverse();

    let mut passed_over = vec![];
    for version in &candidates {
        let reason = match project.usable_project_file(version) {
            Ok(proj_file) if proj_file.fix_version().matches(&compiler_version()) => {
                if !passed_over.is_empty() {
                    info_msg(&format!(
                        "Passing over newer versions of \"{}\": {}.",
                        name,
                        passed_over.join("; ")
                    ));
                }
                return Ok(if any_tagged {
                    VersionToAdd::Tagged(version.clone())
                } else {
                    VersionToAdd::Untagged
                });
            }
            Ok(proj_file) => format!(
                "{} requires Fix {}, and this is Fix {}",
                version,
                proj_file.fix_version(),
                compiler_version()
            ),
            Err(RetrievalError::Unusable(errs)) => format!("{}: {}", version, errs.messages()),
            Err(RetrievalError::Fatal(errs)) => return Err(errs),
        };
        passed_over.push(reason);
    }
    Err(Errors::from_msg(format!(
        "No version of \"{}\" can be used: {}.",
        name,
        passed_over.join("; ")
    )))
}

/// The versions of the project in the git repository `repo`, one for each version tag.
///
/// A repository with no version tag lists the version its newest commit's project file gives; when
/// that file cannot be read, the repository is unusable.
fn get_versions_from_repo(repo: &Repository) -> Result<Vec<VersionInfo>, RetrievalError> {
    let mut versions: Vec<VersionInfo> = vec![];

    // First, look for tags.
    repo.tag_foreach(|oid, name| {
        let name = String::from_utf8_lossy(name).to_string();

        // `name` is in the format "refs/tags/v0.1.0".
        // Split the name by "/" and get the last part.
        let name = name.split('/').last().unwrap_or(&name);

        // Remove `v` prefix if exists.
        let name = if name.starts_with("v") {
            &name[1..]
        } else {
            &name
        };

        // Parse the version.
        let parsed = Version::parse(name);
        if parsed.is_err() {
            return true; // Continue.
        }
        let version = parsed.unwrap();

        // Peel the OID to a commit. For annotated tags `oid` points to a tag
        // object, not the commit, so a later `find_commit` would fail; for
        // lightweight tags `oid` is already a commit OID and peel returns it
        // unchanged.
        let commit_oid = match repo
            .find_object(oid, None)
            .and_then(|obj| obj.peel_to_commit())
        {
            Ok(commit) => commit.id(),
            Err(_) => return true, // Skip tags that don't resolve to a commit.
        };

        versions.push(VersionInfo {
            version,
            rev: commit_oid,
            tagged: true,
        });

        return true;
    })
    .map_err(|e| RetrievalError::Fatal(Errors::from_msg_err("Failed to iterate over tags", e)))?;

    // If any version tag is found, return them.
    if !versions.is_empty() {
        return Ok(versions);
    }

    // If no version tag is found, look for "fixproj.toml" file and use the version from it.
    let fatal = RetrievalError::Fatal;
    let head = repo
        .head()
        .map_err(|e| fatal(Errors::from_msg_err("Failed to get HEAD", e)))?;
    let head_oid = head.target().ok_or_else(|| {
        fatal(Errors::from_msg(
            "HEAD does not point to any commit.".to_string(),
        ))
    })?;
    let work_dir = repo.workdir().ok_or_else(|| {
        fatal(Errors::from_msg(
            "Repository does not have a working directory.".to_string(),
        ))
    })?;
    let proj_file_path = work_dir.join(PROJECT_FILE_PATH);
    let proj_file = ProjectFile::read_file(&proj_file_path).map_err(RetrievalError::Unusable)?;
    versions.push(VersionInfo {
        version: proj_file.general.version(),
        rev: head_oid,
        tagged: false,
    });

    Ok(versions)
}

pub struct VersionInfo {
    pub version: Version,
    rev: Oid,         // Empty if source is `ProjectDir`.
    pub tagged: bool, // True if the version is tagged.
}

pub enum ProjectSource {
    // Just a fix project created at the given path.
    Local(PathBuf),
    // Remote git repository. The second field is a temporary directory where the repository is cloned.
    Git(String, Option<(TempDir, Repository)>),
}

impl ProjectSource {
    /// The source as the resolver identifies it: two sources are the same when they name the same
    /// directory or the same URL.
    fn to_source(&self) -> Result<Source, Errors> {
        Ok(match self {
            ProjectSource::Local(path) => Source::Local(to_absolute_path(path)?),
            ProjectSource::Git(url, _repo) => Source::Git(url.clone()),
        })
    }

    // Get the git repository, assuming it is already prepared.
    fn get_git_repository(&mut self) -> &mut Repository {
        match self {
            ProjectSource::Local(_path_buf) => {
                panic!("Called `get_git_repository` for `ProjectDir`")
            }
            ProjectSource::Git(_url, repo) => &mut repo.as_mut().unwrap().1,
        }
    }

    // Open the git repository and return it.
    fn prepare_git_repository(&mut self) -> Result<(), Errors> {
        match self {
            ProjectSource::Local(_path_buf) => {
                // Nothing to do.
                Ok(())
            }
            ProjectSource::Git(url, repo) => {
                // If the repository is already opened, nothing to do.
                if let Some((_, _)) = repo {
                    return Ok(());
                }

                // Clone the repository.
                *repo = Some(clone_git_repo(url)?);

                Ok(())
            }
        }
    }
}

/// Clones a git repository to a temporary directory.
pub fn clone_git_repo(url: &str) -> Result<(TempDir, Repository), Errors> {
    // Create a temporary directory to clone the repository.
    let temp_dir = tempfile::tempdir()
        .map_err(|e| Errors::from_msg_err("Failed to create a temporary directory", e))?;

    // Clone the repository.
    let repo = clone_repository_with_retry(url, temp_dir.path()).map_err(|e| {
        Errors::from_msg_err(&format!("Failed to clone the repository `{}`", url), e)
    })?;

    Ok((temp_dir, repo))
}

/// The waits before the second and later attempts of a clone. A clone is attempted once more than
/// there are waits. Each wait is scaled by a random factor between 0.5 and 1.5 (see
/// `clone_repository_with_retry`).
const CLONE_RETRY_WAITS: [Duration; 4] = [
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(4),
    Duration::from_secs(8),
];

/// Clones the repository at `url` into the directory `path`, and tries again after a failure of the
/// connection to the remote, which the next attempt may not meet.
///
/// libgit2 empties `path` when a clone fails, so each attempt finds `path` as the first attempt
/// found it.
///
/// The waits are randomized so that the clients a single outage cut off at once spread their next
/// attempts over time, and the recovering remote meets them a few at a time.
fn clone_repository_with_retry(url: &str, path: &Path) -> Result<Repository, git2::Error> {
    for wait in CLONE_RETRY_WAITS {
        match Repository::clone(url, path) {
            Err(error) if is_transient_git_failure(&error) => {
                let wait = wait.mul_f64(thread_rng().gen_range(0.5..1.5));
                info_msg(&format!(
                    "Failed to clone the repository `{}` ({}); trying again in {:.1} s.",
                    url,
                    error.message(),
                    wait.as_secs_f64()
                ));
                thread::sleep(wait);
            }
            result => return result,
        }
    }
    Repository::clone(url, path)
}

/// Whether `error` is a failure of the connection to a remote, which the next attempt may not
/// meet: a connection cut off, a TLS handshake that broke, or an address that did not resolve.
///
/// libgit2 reports a connection refused or reset by the remote, over TLS among them, as an error of
/// the operating system (`ErrorClass::Os`). That class also holds the errors of local files, which
/// are then attempted again as well.
///
/// Two answers from a remote are excluded, since a second attempt gets the same answer: a request
/// for credentials, which is how GitHub answers for a repository that does not exist, and a
/// certificate that does not verify.
fn is_transient_git_failure(error: &git2::Error) -> bool {
    matches!(
        error.class(),
        ErrorClass::Net | ErrorClass::Http | ErrorClass::Ssl | ErrorClass::Os
    ) && !matches!(error.code(), ErrorCode::Auth | ErrorCode::Certificate)
}

/// The project `name` at `source` among `projects`, if it is there.
fn find_project<'a>(
    projects: &'a mut [ProjectInfo],
    name: &PackageName,
    source: &Source,
) -> Option<&'a mut ProjectInfo> {
    projects
        .iter_mut()
        .find(|project| &project.name == name && &project.source_id == source)
}

/// The project `name` at `source` among `projects`, which the resolver has learned of from a
/// requirement on it.
fn known_project<'a>(
    projects: &'a mut [ProjectInfo],
    name: &PackageName,
    source: &Source,
) -> &'a mut ProjectInfo {
    find_project(projects, name, source).unwrap_or_else(|| {
        panic!(
            "no requirement on \"{}\" from {} has been read",
            name, source
        )
    })
}

/// The package retriever for `resolver::resolve_dependency`. It reads project files through the
/// cache `projs`, and adds to it the projects their dependencies name.
fn create_package_retriever(
    projs: ProjectsInfo,
) -> Box<dyn Fn(&PackageName, &SourcedVersion, BuildConfigType) -> Result<Package, RetrievalError>>
{
    Box::new(move |prj_name, ver, mode| {
        let mut projs = projs.projects.as_ref().lock().unwrap();
        let prj = known_project(&mut projs, prj_name, &ver.source);

        let proj_file = prj.usable_project_file(&ver.version)?;

        // Read the dependencies, and add the projects they name to the cache, so that the versions
        // retriever finds them. The dependencies of the root follow `mode`; the resolver passes
        // `Build` for every other project.
        let mut deps = vec![];
        for dep in &proj_file.get_dependencies(mode) {
            // Warn if a transitive dependency specifies rev/tag (it will be ignored). The root
            // project is the first of `projs`, and its own pins are applied.
            if let Some(git) = &dep.git {
                let is_root = prj_name == &projs[0].name && ver.source == projs[0].source_id;
                if git.has_ref() && !is_root {
                    warn_msg(&format!(
                        "Dependency \"{}\" in \"{}\" specifies {}, but git ref pinning is only applied to direct dependencies. The specification will be ignored.",
                        dep.name,
                        prj_name,
                        git.ref_description()
                    ));
                }
            }
            let dep_project =
                ProjectInfo::new(dep.name.clone(), proj_file.get_dependency_source(&dep.name))
                    .map_err(RetrievalError::Unusable)?;
            let source = dep_project.source_id.clone();
            if find_project(&mut projs, &dep.name, &source).is_none() {
                projs.push(dep_project);
            }
            deps.push(Dependency {
                name: dep.name.clone(),
                requirement: dep.version(),
                source,
            });
        }
        Ok(Package {
            deps,
            fix_version: proj_file.fix_version(),
        })
    })
}

/// The versions retriever for `resolver::resolve_dependency`, which lists versions through the
/// cache `pkgs`.
fn create_version_retriever(
    pkgs: ProjectsInfo,
) -> Box<dyn Fn(&PackageName, &Source) -> Result<Vec<Version>, RetrievalError>> {
    Box::new(move |pkg_name, source| {
        let mut prjs = pkgs.projects.as_ref().lock().unwrap();
        let prj = known_project(&mut prjs, pkg_name, source);
        prj.retrieve_versions()?;
        Ok(prj
            .versions
            .as_ref()
            .unwrap()
            .iter()
            .map(|info| info.version.clone())
            .collect())
    })
}

#[cfg(test)]
mod tests {
    use super::is_transient_git_failure;
    use git2::{ErrorClass, ErrorCode, Repository};
    use std::fs;
    use tempfile::TempDir;

    /// A connection the remote cut off in the middle of a clone, and a TLS handshake that broke, are
    /// failures after which the clone is attempted again.
    #[test]
    fn a_connection_cut_off_or_a_broken_handshake_is_transient() {
        let cut_off = git2::Error::new(ErrorCode::GenericError, ErrorClass::Http, "unexpected EOF");
        let broken_handshake =
            git2::Error::new(ErrorCode::GenericError, ErrorClass::Ssl, "unknown error");
        assert!(is_transient_git_failure(&cut_off));
        assert!(is_transient_git_failure(&broken_handshake));
    }

    /// The answer GitHub gives for a repository that does not exist is reported at once.
    #[test]
    fn a_missing_repository_on_github_is_not_transient() {
        let missing = git2::Error::new(
            ErrorCode::Auth,
            ErrorClass::Http,
            "remote authentication required but no callback set",
        );
        assert!(!is_transient_git_failure(&missing));
    }

    /// A certificate that does not verify is reported at once.
    #[test]
    fn a_certificate_that_does_not_verify_is_not_transient() {
        let unverified = git2::Error::new(
            ErrorCode::Certificate,
            ErrorClass::Ssl,
            "the SSL certificate is invalid",
        );
        assert!(!is_transient_git_failure(&unverified));
    }

    /// A connection the remote refused is a failure after which the clone is attempted again.
    #[test]
    fn a_refused_connection_is_transient() {
        let dir = TempDir::new().unwrap();
        let error = Repository::clone("http://127.0.0.1:1/dep.git", dir.path())
            .err()
            .unwrap();
        assert_eq!(error.class(), ErrorClass::Os);
        assert!(is_transient_git_failure(&error));
    }

    /// An address that did not resolve is a failure after which the clone is attempted again.
    #[test]
    fn an_address_that_did_not_resolve_is_transient() {
        let unresolved = git2::Error::new(
            ErrorCode::GenericError,
            ErrorClass::Net,
            "failed to resolve address for github.com: Temporary failure in name resolution",
        );
        assert!(is_transient_git_failure(&unresolved));
    }

    /// A failure outside the connection to a remote is reported at once: here, the one libgit2
    /// gives for a clone from a local directory that holds no repository.
    #[test]
    fn a_failure_outside_the_connection_is_not_transient() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("not_a_repository");
        fs::create_dir(&source).unwrap();
        let error = Repository::clone(source.to_str().unwrap(), &dir.path().join("clone"))
            .err()
            .unwrap();
        assert!(!is_transient_git_failure(&error));
    }
}
