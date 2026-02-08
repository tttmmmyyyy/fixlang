use core::panic;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use git2::{build::CheckoutBuilder, Repository};
use semver::Version;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

use crate::{
    configuration::{BuildMode, Configuration},
    dependency_resolver::{self, Dependency, Package, PackageName},
    error::Errors,
    misc::{to_absolute_path, warn_msg},
    project_file::{ProjectFile, ProjectFileDependency, ProjectName},
    EXTERNAL_PROJ_INSTALL_PATH, LOCK_FILE_PATH, LOCK_FILE_TEST_PATH, PROJECT_FILE_PATH,
};

// Get the lock file path based on the dependency mode.
pub fn get_lock_file_path(mode: BuildMode) -> &'static str {
    match mode {
        BuildMode::Test => LOCK_FILE_TEST_PATH,
        BuildMode::Build => LOCK_FILE_PATH,
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DependecyLockFile {
    pub proj_file_hash: String,
    pub dependencies: Vec<DependencyLockFileEntry>,
}

impl DependecyLockFile {
    // Update configuration by adding source files, linking libraries, ... as required by dependencies.
    pub fn set_config(&self, config: &mut Configuration) -> Result<(), Errors> {
        for dep in &self.dependencies {
            let proj_file = dep.project_file()?;
            proj_file.set_config(config, true)?;
        }
        Ok(())
    }

    // Create the lock file (on memory, not on file) to satisfy the dependencies of the given project file.
    pub fn create(proj_file: &ProjectFile, mode: BuildMode) -> Result<DependecyLockFile, Errors> {
        // Resolve the dependency.
        let prjs_info = ProjectsInfo {
            projects: Arc::new(Mutex::new(vec![ProjectInfo::from_project_file(proj_file)])),
        };
        let packages_retriever = create_package_retriever(prjs_info.clone());
        let versions_retriever = create_version_retriever(prjs_info.clone());
        println!("Resolving dependency for \"{}\"...", proj_file.general.name);
        let res = dependency_resolver::resolve_dependency(
            proj_file,
            packages_retriever.as_ref(),
            versions_retriever.as_ref(),
            mode,
        )?;
        if res.is_none() {
            return Err(Errors::from_msg(
                "Failed to resolve dependencies.".to_string(),
            ));
        }
        let prjs = res.unwrap();

        // Create a new lock file following the resolved dependencies.
        let mut lock_file = DependecyLockFile {
            proj_file_hash: proj_file.calculate_dependencies_hash(mode),
            dependencies: Vec::new(),
        };
        for prj in prjs {
            // Exclude the root project.
            if prj.name == proj_file.general.name {
                continue;
            }

            // Get the information of the dependent project.
            let prjs_info = prjs_info.projects.as_ref().lock().unwrap();
            let prj_info = &prjs_info
                .iter()
                .find(|info| &info.name == &prj.name)
                .expect(format!("\"{}\" not found in `projs_info`", prj.name).as_str());
            let ver_info = prj_info
                .versions
                .as_ref()
                .unwrap()
                .iter()
                .find(|info| &info.version == &prj.version)
                .unwrap();

            // If the version is not tagged, then warn the user.
            if prj_info.is_git() && !ver_info.tagged {
                let short_commit = format!("{}", ver_info.rev)
                    .chars()
                    .take(7)
                    .collect::<String>();
                warn_msg(&format!(
                    "No version tag is found for \"{}\", and using untagged version \"{}\" (commit {}).",
                    prj.name, prj.version, short_commit
                ));
            }

            // Create a new entry for the lock file.
            let dep = DependencyLockFileEntry {
                name: prj.name.clone(),
                version: prj.version.to_string(),
                path: match &prj_info.source {
                    ProjectSource::Local(path_buf) => path_buf.clone(),
                    ProjectSource::Git(_, _) => {
                        let dir = PathBuf::from(EXTERNAL_PROJ_INSTALL_PATH);
                        dir.join(format!("{}_{}", prj.name, prj.version.to_string()))
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

        // Sort dependencies by name to ensure consistent order in lock file.
        // This prevents unnecessary changes in the lock file when dependency resolution order changes.
        lock_file.dependencies.sort_by(|a, b| a.name.cmp(&b.name));

        println!("Dependencies resolved successfully.");
        Ok(lock_file)
    }

    // Install the dependencies.
    pub fn install(&self) -> Result<(), Errors> {
        for dep in &self.dependencies {
            if let Some(git_info) = &dep.git {
                let target_rev = git2::Oid::from_str(&git_info.rev).unwrap();
                // In case the source is a git repository,
                if dep.path.exists() {
                    // If the path exists, check that the revision is correct.
                    // If the revision is incorrect, or any git error occurs, remove the directory and clone the repository again.
                    let rev_match = Repository::open(&dep.path).and_then(|repo| {
                        let head = repo.head()?.target().unwrap_or(git2::Oid::zero());
                        return Ok(head == target_rev);
                    });
                    if let Ok(v) = rev_match {
                        if v {
                            // If the revision is ok, load the project file and validate whether it satisfies the dependency.
                            dep.check_name_version_match_proj_file()?;
                            continue; // This dependency is already installed and ok.
                        }
                    }
                    // If something is wrong, remove the directory.
                    std::fs::remove_dir_all(&dep.path).map_err(|e| {
                        Errors::from_msg(format!(
                            "Failed to remove the directory \"{}\": {:?}",
                            dep.path.to_string_lossy().to_string(),
                            e
                        ))
                    })?;
                }
                // Create the directory.
                std::fs::create_dir_all(&dep.path).map_err(|e| {
                    Errors::from_msg(format!(
                        "Failed to create the directory \"{}\": {:?}",
                        dep.path.to_string_lossy().to_string(),
                        e
                    ))
                })?;

                // Clone the repository.
                let repo = Repository::clone(&git_info.repo, &dep.path).map_err(|e| {
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

                println!(
                    "Dependency \"{}@{}\" installed successfully at \"{}\".",
                    dep.name,
                    dep.version,
                    dep.path.to_string_lossy().to_string()
                );
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

    // Update the lock file and install the dependencies.
    pub fn update_and_install(mode: BuildMode) -> Result<(), Errors> {
        // Remove lock file.
        let lock_file_path = Path::new(get_lock_file_path(mode));
        if lock_file_path.exists() {
            std::fs::remove_file(lock_file_path).expect("Failed to remove the lock file.");
        }
        ProjectFile::read_root_file()?.open_or_create_lock_file_and_install(mode)
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DependencyLockFileEntry {
    name: String,
    version: String,
    path: PathBuf,
    git: Option<DependencyLockGit>,
}

impl DependencyLockFileEntry {
    pub fn project_file(&self) -> Result<ProjectFile, Errors> {
        let proj_file_path = self.path.join(PROJECT_FILE_PATH);
        ProjectFile::read_file(&proj_file_path)
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
        if proj_file.general.version() != Version::parse(&self.version).unwrap() {
            return Err(Errors::from_msg(format!(
                "The version of the dependency \"{}\" installed at \"{}\" does not match the one specified in \"{}\". Try to run `fix deps update`.",
                self.name, self.path.to_string_lossy().to_string(), LOCK_FILE_PATH,
            )));
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
struct DependencyLockGit {
    repo: String,
    rev: String,
}

// Convert a `ProjectFileDependency` to a `dependency_resolver::Dependency`.
fn project_file_dep_to_dependency(dep: &ProjectFileDependency) -> Dependency {
    let name = dep.name.clone();
    let requirement = dep.version();
    Dependency { name, requirement }
}

// Following structures are used to cache project information retrieved by file IO or network IO.

#[derive(Clone, Default)]
struct ProjectsInfo {
    projects: Arc<Mutex<Vec<ProjectInfo>>>,
}

struct ProjectInfo {
    name: ProjectName,
    source: ProjectSource,
    versions: Option<Vec<VersionInfo>>, // Available versions. None if not retrieved yet.
    proj_files: Vec<ProjectFile>,       // Project files at different versions.
}

impl ProjectInfo {
    pub fn is_git(&self) -> bool {
        matches!(self.source, ProjectSource::Git(_, _))
    }

    fn from_project_file(proj_file: &ProjectFile) -> Self {
        ProjectInfo {
            name: proj_file.general.name.clone(),
            source: ProjectSource::Local(proj_file.path.parent().unwrap().to_path_buf()),
            versions: Some(vec![VersionInfo {
                version: proj_file.general.version(),
                rev: git2::Oid::zero(),
                tagged: false,
            }]),
            proj_files: vec![proj_file.clone()],
        }
    }

    // See into `self.source` and retrieve versions available for this package.
    fn retrieve_versions(&mut self) -> Result<(), Errors> {
        // If the versions are already retrieved, do nothing.
        if self.versions.is_some() {
            return Ok(());
        }
        self.source.prepre_git_repository()?;
        match &mut self.source {
            ProjectSource::Local(proj_path) => {
                // Read the project file.
                let proj_file_path = proj_path.clone().join(PROJECT_FILE_PATH);
                let proj_file = ProjectFile::read_file(&proj_file_path)?;
                let ver = proj_file.general.version();
                self.versions = Some(vec![VersionInfo {
                    version: ver,
                    rev: git2::Oid::zero(),
                    tagged: false,
                }]);
            }
            ProjectSource::Git(_url, repo) => {
                let repo = &repo.as_mut().unwrap().1;
                self.versions = Some(get_versions_from_repo(repo)?);
            }
        }
        Ok(())
    }

    // Get the project file at the given version.
    fn get_project_file(&mut self, version: &Version) -> Result<ProjectFile, Errors> {
        // See into the cache field `self.proj_files`.
        for proj_file in &self.proj_files {
            if &proj_file.general.version() == version {
                return Ok(proj_file.clone());
            }
        }

        // Retrieve the versions if not retrieved yet.
        self.retrieve_versions()?;

        // Check if the version exists, and get the revision.
        let rev = self
            .versions
            .as_ref()
            .unwrap()
            .iter()
            .find(|info| &info.version == version)
            .ok_or_else(|| {
                Errors::from_msg(format!(
                    "Version \"{}\" of project \"{}\" is not found.",
                    version, self.name
                ))
            })?
            .rev;

        // If the source is a project directory, open the project file and return it.
        match &self.source {
            ProjectSource::Local(proj_path) => {
                let proj_file = ProjectFile::read_file(&proj_path.join(PROJECT_FILE_PATH))?;
                self.proj_files.push(proj_file.clone());
                return Ok(proj_file);
            }
            _ => (),
        }

        // If the source is a git repository, checkout the given revision and read the project file.
        self.source.prepre_git_repository()?;
        let repo = self.source.get_git_repository();
        let commit = repo
            .find_commit(rev)
            .map_err(|e| Errors::from_msg_err("Failed to find commit", e))?;
        let mut checkout_opts = CheckoutBuilder::default();
        checkout_opts.force();
        repo.checkout_tree(&commit.into_object(), Some(&mut checkout_opts))
            .map_err(|e| Errors::from_msg_err("Failed to checkout commit", e))?;
        let proj_file = ProjectFile::read_file(&repo.workdir().unwrap().join(PROJECT_FILE_PATH))?;
        self.proj_files.push(proj_file.clone());

        Ok(proj_file)
    }
}

// Get versions from a git repository.
pub fn get_versions_from_repo(repo: &Repository) -> Result<Vec<VersionInfo>, Errors> {
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
        versions.push(VersionInfo {
            version,
            rev: oid,
            tagged: true,
        });

        return true;
    })
    .map_err(|e| Errors::from_msg_err("Failed to iterate over tags", e))?;

    // If any version tag is found, return them.
    if !versions.is_empty() {
        return Ok(versions);
    }

    // If no version tag is found, look for "fixproj.toml" file and use the version from it.
    let head = repo
        .head()
        .map_err(|e| Errors::from_msg_err("Failed to get HEAD", e))?;
    let head_oid = head
        .target()
        .ok_or_else(|| Errors::from_msg("HEAD does not point to any commit.".to_string()))?;
    let work_dir = repo.workdir().ok_or_else(|| {
        Errors::from_msg("Repository does not have a working directory.".to_string())
    })?;
    let proj_file_path = work_dir.join(PROJECT_FILE_PATH);
    let proj_file = ProjectFile::read_file(&proj_file_path)?;
    versions.push(VersionInfo {
        version: proj_file.general.version(),
        rev: head_oid,
        tagged: false,
    });

    Ok(versions)
}

pub struct VersionInfo {
    pub version: Version,
    rev: git2::Oid,   // Empty if source is `ProjectDir`.
    pub tagged: bool, // True if the version is tagged.
}

pub enum ProjectSource {
    // Just a fix project created at the given path.
    Local(PathBuf),
    // Remote git repository. The second field is a temporary directory where the repository is cloned.
    Git(String, Option<(TempDir, Repository)>),
}

impl ProjectSource {
    // Stringify the source for display.
    fn to_string(&self) -> Result<String, Errors> {
        Ok(match self {
            ProjectSource::Local(path_buf) => {
                to_absolute_path(path_buf)?.to_string_lossy().to_string()
            }
            ProjectSource::Git(url, _repo) => url.clone(),
        })
    }

    fn equivalent(&self, other: &Self) -> Result<bool, Errors> {
        match (self, other) {
            (ProjectSource::Local(path1), ProjectSource::Local(path2)) => {
                Ok(to_absolute_path(path1)? == to_absolute_path(path2)?)
            }
            (ProjectSource::Git(url1, _), ProjectSource::Git(url2, _)) => Ok(url1 == url2),
            _ => Ok(false),
        }
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
    fn prepre_git_repository(&mut self) -> Result<(), Errors> {
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

// Clones a git repository to a temporary directory.
pub fn clone_git_repo(url: &str) -> Result<(TempDir, Repository), Errors> {
    // Create a temporary directory to clone the repository.
    let temp_dir = tempfile::tempdir()
        .map_err(|e| Errors::from_msg_err("Failed to create a temporary directory", e))?;

    // Clone the repository.
    let repo = Repository::clone(url, temp_dir.path()).map_err(|e| {
        Errors::from_msg_err(&format!("Failed to clone the repository `{}`", url), e)
    })?;

    Ok((temp_dir, repo))
}

fn project_file_to_package(proj_file: &ProjectFile, mode: BuildMode) -> Package {
    let deps_list = proj_file.get_dependencies(mode);
    let mut deps = Vec::new();
    for dep in &deps_list {
        let dep = project_file_dep_to_dependency(dep);
        deps.push(dep);
    }
    Package {
        name: proj_file.general.name.clone(),
        version: proj_file.general.version(),
        deps,
    }
}

// Create package retriever which will be passed to `package_resolver::resolve_dependency`.
fn create_package_retriever(
    projs: ProjectsInfo,
) -> Box<dyn Fn(&PackageName, &Version, BuildMode) -> Result<Package, Errors>> {
    Box::new(move |prj_name, ver, mode| {
        let mut projs = projs.projects.as_ref().lock().unwrap();

        // Find the project.
        let prj = projs
            .iter_mut()
            .find(|pkg_data| &pkg_data.name == prj_name)
            .ok_or_else(|| {
                Errors::from_msg(format!("Source for \"{}\" is not found.", prj_name))
            })?;

        // Get the project file of the package at the given version.
        let proj_file = prj.get_project_file(ver)?;

        // Check that the project name is correct.
        if &proj_file.general.name != prj_name {
            return Err(Errors::from_msg(format!(
                "\"{}\" is found, but a different project name \"{}\" is specified in its project file.",
                prj_name,
                proj_file.general.name
            )));
        }

        // Check that the project version is correct.
        if proj_file.general.version() != *ver {
            return Err(Errors::from_msg(format!(
                "\"{}@{}\" is found, but a different version \"{}\" is specified in its project file.",
                prj_name,
                ver,
                proj_file.general.version()
            )));
        }

        // Register new dependent projects to the packages cache.
        let deps_list = proj_file.get_dependencies(mode);
        for dep in &deps_list {
            let dep_src = proj_file.get_dependency_source(&dep.name);
            if let Some(prj) = projs.iter().find(|pkg| &pkg.name == &dep.name) {
                // If the project is already in the cache, then check that the sources are the same between `pkg` and `dep`.
                if prj.source.equivalent(&dep_src)? {
                    continue;
                }
                return Err(Errors::from_msg(format!(
                    "\"{}\" is required twice with different sources: \"{}\" and \"{}\".",
                    dep.name,
                    prj.source.to_string()?,
                    dep_src.to_string()?
                )));
            }
            let prj = ProjectInfo {
                name: dep.name.clone(),
                source: dep_src,
                versions: None,         // Filled on demand.
                proj_files: Vec::new(), // Filled on demand.
            };
            projs.push(prj);
        }

        // Use the mode parameter to get dependencies.
        // For root project, mode will be Test or Build as specified.
        // For dependent projects, mode will always be Build (set by dependency resolver).
        Ok(project_file_to_package(&proj_file, mode))
    })
}

// Create version retriever which will be passed to `package_resolver::resolve_dependency`.
fn create_version_retriever(
    pkgs: ProjectsInfo,
) -> Box<dyn Fn(&PackageName) -> Result<Vec<Version>, Errors>> {
    Box::new(move |pkg_name| {
        let mut prjs = pkgs.projects.as_ref().lock().unwrap();

        // Find the package.
        let prj = prjs
            .iter_mut()
            .find(|pkg_data| &pkg_data.name == pkg_name)
            .ok_or_else(|| {
                Errors::from_msg(format!("Source for \"{}\" is not found.", pkg_name))
            })?;

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
