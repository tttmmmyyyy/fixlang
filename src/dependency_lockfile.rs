use core::panic;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use git2::{build::CheckoutBuilder, Repository};
use semver::Version;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

use crate::{
    configuration::Configuration,
    dependency_resolver::{self, Dependency, Package, PackageName},
    error::Errors,
    project_file::{ProjectFile, ProjectFileDependency, ProjectName},
    EXTERNAL_PROJ_INSTALL_PATH, LOCK_FILE_PATH, PROJECT_FILE_PATH,
};

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
    pub fn create(proj_file: &ProjectFile) -> Result<DependecyLockFile, Errors> {
        // Resolve the dependency.
        let prjs_info = ProjectsInfo {
            projects: Arc::new(Mutex::new(vec![ProjectInfo::from_project_file(proj_file)])),
        };
        let packages_retriever = create_package_retriever(prjs_info.clone());
        let versions_retriever = create_version_retriever(prjs_info.clone());
        let logger = create_stdout_logger();
        println!(
            "Resolving dependency for project \"{}\"...",
            proj_file.general.name
        );
        let res = dependency_resolver::resolve_dependency(
            proj_file,
            packages_retriever.as_ref(),
            versions_retriever.as_ref(),
            logger.as_ref(),
        )?;
        if res.is_none() {
            return Err(Errors::from_msg(
                "Failed to resolve dependencies.".to_string(),
            ));
        }
        let prjs = res.unwrap();

        // Create a new lock file following the resolved dependencies.
        let mut lock_file = DependecyLockFile {
            proj_file_hash: proj_file.hash.clone(),
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
                .unwrap();
            let ver_info = prj_info
                .versions
                .as_ref()
                .unwrap()
                .iter()
                .find(|info| &info.version == &prj.version)
                .unwrap();

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

        println!("Dependency resolved successfully.");
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

                // Load the project file and validate whether it satisfies the dependency.
                dep.check_name_version_match_proj_file()?;

                println!(
                    "Dependent project \"{}\" v{} installed successfully at \"{}\".",
                    dep.name,
                    dep.version,
                    dep.path.to_string_lossy().to_string()
                );
            } else {
                // In case the source is a project directory,
                // Check the path exists.
                if !dep.path.exists() {
                    return Err(Errors::from_msg(format!(
                        "Dependent project \"{}\" is not found at \"{}\" as required in \"{}\".",
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
                "Dependent project \"{}\" installed at \"{}\" is not named \"{}\" as required in \"{}\".",
                self.name, self.path.to_string_lossy().to_string(), self.name, LOCK_FILE_PATH,
            )));
        }
        if proj_file.general.version() != Version::parse(&self.version).unwrap() {
            return Err(Errors::from_msg(format!(
                "Dependent project \"{}\" installed at \"{}\" is not at version \"{}\" as required in \"{}\".",
                self.name, self.path.to_string_lossy().to_string(), self.version, LOCK_FILE_PATH,
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
    fn from_project_file(proj_file: &ProjectFile) -> Self {
        ProjectInfo {
            name: proj_file.general.name.clone(),
            source: ProjectSource::Local(proj_file.path.parent().unwrap().to_path_buf()),
            versions: Some(vec![VersionInfo {
                version: proj_file.general.version(),
                rev: git2::Oid::zero(),
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
fn get_versions_from_repo(repo: &Repository) -> Result<Vec<VersionInfo>, Errors> {
    let mut versions: Vec<VersionInfo> = vec![];

    // First, look for tags.
    repo.tag_foreach(|oid, name| {
        let name = String::from_utf8_lossy(name).to_string();

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
        versions.push(VersionInfo { version, rev: oid });

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
    });

    Ok(versions)
}

struct VersionInfo {
    version: Version,
    rev: git2::Oid, // Empty if source is `ProjectDir`.
}

pub enum ProjectSource {
    // Just a fix project created at the given path.
    Local(PathBuf),
    // Remote git repository. The second field is a temporary directory where the repository is cloned.
    Git(String, Option<(TempDir, Repository)>),
}

impl ProjectSource {
    fn equivalent(&self, other: &Self) -> bool {
        match (self, other) {
            (ProjectSource::Local(path1), ProjectSource::Local(path2)) => path1 == path2,
            (ProjectSource::Git(url1, _), ProjectSource::Git(url2, _)) => url1 == url2,
            _ => false,
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

                // Create a temporary directory to clone the repository.
                let temp_dir = tempfile::tempdir().map_err(|e| {
                    Errors::from_msg_err("Failed to create a temporary directory", e)
                })?;

                // Clone the repository.
                let cloned = Repository::clone(url, temp_dir.path()).map_err(|e| {
                    Errors::from_msg_err(&format!("Failed to clone repository `{}`", url), e)
                })?;

                // Cache the repository.
                *repo = Some((temp_dir, cloned));

                Ok(())
            }
        }
    }
}

fn project_file_to_package(proj_file: &ProjectFile) -> Package {
    let mut deps = Vec::new();
    for dep in &proj_file.dependencies {
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
) -> Box<dyn Fn(&PackageName, &Version) -> Result<Package, Errors>> {
    Box::new(move |prj_name, ver| {
        let mut projs = projs.projects.as_ref().lock().unwrap();

        // Find the project.
        let prj = projs
            .iter_mut()
            .find(|pkg_data| &pkg_data.name == prj_name)
            .ok_or_else(|| {
                Errors::from_msg(format!("Source for project \"{}\" is not found.", prj_name))
            })?;

        // Get the project file of the package at the given version.
        let proj_file = prj.get_project_file(ver)?;

        // Register new dependent projects to the packages cache.
        for dep in &proj_file.dependencies {
            let dep_src = proj_file.get_dependency_source(&dep.name);
            if let Some(prj) = projs.iter().find(|pkg| &pkg.name == &dep.name) {
                // If the project is already in the cache, then check that the sources are the same between `pkg` and `dep`.
                if prj.source.equivalent(&dep_src) {
                    continue;
                }
                return Err(Errors::from_msg(format!(
                    "Project \"{}\" is required twice with different sources.",
                    dep.name
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

        Ok(project_file_to_package(&proj_file))
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
                Errors::from_msg(format!("Source for project \"{}\" is not found.", pkg_name))
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

#[derive(Clone, Default)]
struct LogsBuffer {
    lines: Arc<Mutex<Vec<String>>>,
}

// Creates a logger function which writes logs to the given LogBuffer.
#[allow(dead_code)]
fn create_logger(log_buf: LogsBuffer) -> Box<dyn Fn(&str)> {
    Box::new(move |msg: &str| {
        log_buf.lines.lock().unwrap().push(msg.to_string());
    })
}

fn create_stdout_logger() -> Box<dyn Fn(&str)> {
    Box::new(move |msg: &str| {
        println!("{}", msg);
    })
}
