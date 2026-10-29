//! Dependency resolution: choosing a version of every project a root project depends on, directly
//! or transitively, so that every version requirement holds. The search is the PubGrub algorithm of
//! the `pubgrub` crate, which learns the cause of each conflict and so never retries a combination
//! of versions that the cause already rules out.
//!
//! A version of a project is identified together with the source it is fetched from, so the
//! requirement that every project come from one source is checked by the search like any other
//! requirement.

use crate::{
    configuration::BuildConfigType,
    error::Errors,
    misc::{Map, Set},
};
use pubgrub::{
    Dependencies, DependencyConstraints, DependencyProvider, PackageResolutionStatistics,
    PubGrubError, VersionSet,
};
use semver::{Version, VersionReq};
use std::{
    cell::{Ref, RefCell},
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

mod explain;

pub(crate) use explain::Unavailability;
use explain::{explain, merge_left_out_versions};

pub type PackageName = String;

/// Where the versions of a project are fetched from.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Source {
    /// The project directory at this absolute path, which holds one version.
    Local(PathBuf),
    /// The git repository at this URL, whose tags name the versions.
    Git(String),
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Source::Local(path) => write!(f, "{}", path.display()),
            Source::Git(url) => write!(f, "{}", url),
        }
    }
}

/// A version of a project, together with the source it is fetched from.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SourcedVersion {
    pub version: Version,
    pub source: Source,
}

/// Writes the version followed by the source it is fetched from.
///
/// # Examples
/// `1.0.0 from https://github.com/a/b.git`
impl Display for SourcedVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} from {}", self.version, self.source)
    }
}

/// What the resolver reads of a project at one version.
#[derive(Clone)]
pub struct Package {
    /// The version requirements this version has on other projects.
    pub deps: Vec<Dependency>,
    /// The compiler versions this version builds with, as its `fix_version` field requires.
    pub fix_version: VersionReq,
}

/// A version requirement on a project, which is to be fetched from `source`.
#[derive(Clone)]
pub struct Dependency {
    pub name: PackageName,
    pub requirement: VersionReq,
    pub source: Source,
}

/// Why a version of a project could not be retrieved.
pub enum RetrievalError {
    /// The version itself is broken, as a project file that cannot be read is: the resolution
    /// leaves the version out and goes on.
    Unusable(Errors),
    /// Retrieving failed for a reason outside the version, as a repository that cannot be read
    /// does: the resolution stops.
    Fatal(Errors),
}

/// Retrieves a project at a version. The build configuration selects the dependencies the project
/// file lists.
pub type PackageRetriever<'a> =
    &'a dyn Fn(&PackageName, &SourcedVersion, BuildConfigType) -> Result<Package, RetrievalError>;

/// Lists the versions a project exists in at a source.
pub type VersionRetriever<'a> =
    &'a dyn Fn(&PackageName, &Source) -> Result<Vec<Version>, RetrievalError>;

/// Chooses a version of every project the root project depends on, directly or transitively, such
/// that every version requirement holds, every project comes from the one source all requirements
/// on it name, and every chosen version accepts the running compiler. Among the choices, newer
/// versions are preferred.
///
/// A version that is unusable, or whose `fix_version` rejects `compiler_version`, is left out of
/// the choice. When no choice exists, the error explains which requirements conflict.
///
/// # Arguments
/// * `mode` — the build configuration whose dependencies the root project brings in. The
///   dependencies of other projects are read from their `build` section.
/// * `compiler_version` — the version of the running compiler.
///
/// # Returns
/// The chosen projects other than the root, sorted by name.
pub fn resolve_dependency(
    root: (&PackageName, &SourcedVersion),
    mode: BuildConfigType,
    compiler_version: &Version,
    package_retriever: PackageRetriever<'_>,
    versions_retriever: VersionRetriever<'_>,
) -> Result<Vec<(PackageName, SourcedVersion)>, Errors> {
    let provider = Provider {
        root_name: root.0.clone(),
        root_version: root.1.clone(),
        mode,
        compiler_version: compiler_version.clone(),
        package_retriever,
        versions_retriever,
        known_versions: RefCell::new(Map::from_iter([(
            root.0.clone(),
            BTreeSet::from([root.1.clone()]),
        )])),
        listed_sources: RefCell::default(),
        requirements: RefCell::default(),
    };
    match pubgrub::resolve(&provider, root.0.clone(), root.1.clone()) {
        Ok(selected) => {
            let mut chosen = selected
                .into_iter()
                .filter(|(name, _)| name != root.0)
                .collect::<Vec<_>>();
            chosen.sort();
            Ok(chosen)
        }
        Err(PubGrubError::NoSolution(mut tree)) => {
            tree.collapse_no_versions();
            merge_left_out_versions(&mut tree);
            Err(Errors::from_msg(explain(&provider, &tree)))
        }
        Err(PubGrubError::ErrorRetrievingDependencies { source, .. })
        | Err(PubGrubError::ErrorChoosingVersion { source, .. })
        | Err(PubGrubError::ErrorInShouldCancel(source)) => Err(source),
    }
}

/// Answers the PubGrub solver's questions about the projects: their versions, and the
/// requirements and `fix_version` of each version.
struct Provider<'a, 'b> {
    /// The name of the root project, whose dependencies are read under `mode`.
    root_name: PackageName,
    /// The version of the root project.
    root_version: SourcedVersion,
    /// The build configuration whose dependencies the root project brings in.
    mode: BuildConfigType,
    /// The version of the running compiler, which the `fix_version` of every chosen version must
    /// accept.
    compiler_version: Version,
    package_retriever: PackageRetriever<'a>,
    versions_retriever: VersionRetriever<'b>,
    /// The versions of the root project, and of each project at each source some retrieved version
    /// has a requirement on. The solver asks only about these projects.
    known_versions: RefCell<Map<PackageName, BTreeSet<SourcedVersion>>>,
    /// The projects and sources whose versions `known_versions` holds.
    listed_sources: RefCell<Set<(PackageName, Source)>>,
    /// The requirements of each retrieved version that is not left out, as its project file
    /// writes them.
    requirements: RefCell<Map<(PackageName, SourcedVersion), Vec<Dependency>>>,
}

impl Provider<'_, '_> {
    /// The versions of `name` the provider has listed, at every source.
    fn known_versions_of(&self, name: &PackageName) -> Ref<'_, BTreeSet<SourcedVersion>> {
        Ref::map(self.known_versions.borrow(), |known| {
            known
                .get(name)
                .unwrap_or_else(|| panic!("no requirement on \"{}\" has been read", name))
        })
    }

    /// The versions of `dependency.name` at `dependency.source` that satisfy
    /// `dependency.requirement`. The first requirement read on a project at a source lists its
    /// versions there.
    fn versions_satisfying(&self, dependency: &Dependency) -> Result<Versions, RetrievalError> {
        let key = (dependency.name.clone(), dependency.source.clone());
        if !self.listed_sources.borrow().contains(&key) {
            let versions = (self.versions_retriever)(&dependency.name, &dependency.source)?;
            self.known_versions
                .borrow_mut()
                .entry(dependency.name.clone())
                .or_default()
                .extend(versions.into_iter().map(|version| SourcedVersion {
                    version,
                    source: dependency.source.clone(),
                }));
            self.listed_sources.borrow_mut().insert(key);
        }
        Ok(Versions::Only(self.listed_versions_accepted_by(dependency)))
    }

    /// The versions of `dependency.name` the provider has listed at `dependency.source` that
    /// satisfy `dependency.requirement`.
    fn listed_versions_accepted_by(&self, dependency: &Dependency) -> BTreeSet<SourcedVersion> {
        self.known_versions_of(&dependency.name)
            .iter()
            .filter(|v| v.source == dependency.source && dependency.requirement.matches(&v.version))
            .cloned()
            .collect()
    }

    /// The versions of `name` at `source`, which the provider has listed.
    fn versions_at(&self, name: &PackageName, source: &Source) -> Vec<Version> {
        self.known_versions_of(name)
            .iter()
            .filter(|v| &v.source == source)
            .map(|v| v.version.clone())
            .collect()
    }
}

impl DependencyProvider for Provider<'_, '_> {
    /// A project, by its name.
    type P = PackageName;
    /// A version of a project, together with its source.
    type V = SourcedVersion;
    /// A set of versions of a project.
    type VS = Versions;
    /// Decides first the projects that took part in more conflicts, then those with fewer versions
    /// left to choose from.
    type Priority = (u32, Reverse<usize>);
    /// Why a version is left out of the choice.
    type M = Unavailability;
    /// What stops the resolution.
    type Err = Errors;

    /// Counts the conflicts `package` took part in, and its known versions in `range`.
    fn prioritize(
        &self,
        package: &PackageName,
        range: &Versions,
        package_conflicts_counts: &PackageResolutionStatistics,
    ) -> Self::Priority {
        let candidate_count = self
            .known_versions_of(package)
            .iter()
            .filter(|v| range.contains(v))
            .count();
        (
            package_conflicts_counts.conflict_count(),
            Reverse(candidate_count),
        )
    }

    /// The newest known version in `range`.
    fn choose_version(
        &self,
        package: &PackageName,
        range: &Versions,
    ) -> Result<Option<SourcedVersion>, Errors> {
        Ok(self
            .known_versions_of(package)
            .iter()
            .rev()
            .find(|v| range.contains(v))
            .cloned())
    }

    /// The requirements of `package` at `version`, read under `mode` for the root project and from
    /// the `build` section for any other. A version that cannot be read, or whose `fix_version`
    /// rejects the running compiler, is unavailable, with the reason. For the root, whose project
    /// file is the one being resolved, the reason is reported as an error.
    fn get_dependencies(
        &self,
        package: &PackageName,
        version: &SourcedVersion,
    ) -> Result<Dependencies<PackageName, Versions, Unavailability>, Errors> {
        let is_root = *package == self.root_name && *version == self.root_version;
        let mode = if is_root {
            self.mode
        } else {
            BuildConfigType::Build
        };
        let unavailable = |reason: Unavailability| {
            if is_root {
                Err(Errors::from_msg(format!("\"{}\" {}.", package, reason)))
            } else {
                Ok(Dependencies::Unavailable(reason))
            }
        };
        let retrieved = match (self.package_retriever)(package, version, mode) {
            Ok(retrieved) => retrieved,
            Err(RetrievalError::Unusable(errs)) if is_root => return Err(errs),
            Err(RetrievalError::Unusable(errs)) => {
                return unavailable(Unavailability::Unreadable(errs.messages()))
            }
            Err(RetrievalError::Fatal(errs)) => return Err(errs),
        };
        if !retrieved.fix_version.matches(&self.compiler_version) {
            return unavailable(Unavailability::RejectsCompiler {
                fix_versions: BTreeMap::from([(version.clone(), retrieved.fix_version)]),
                compiler: self.compiler_version.clone(),
            });
        }
        assert_eq!(
            retrieved
                .deps
                .iter()
                .map(|dependency| &dependency.name)
                .collect::<Set<_>>()
                .len(),
            retrieved.deps.len(),
            "\"{}\" {} requires one project twice",
            package,
            version
        );
        let mut constraints = vec![];
        for dependency in &retrieved.deps {
            let versions = match self.versions_satisfying(dependency) {
                Ok(versions) => versions,
                Err(RetrievalError::Unusable(errs)) => {
                    return unavailable(Unavailability::DependencyUnreadable {
                        name: dependency.name.clone(),
                        source: dependency.source.clone(),
                        message: errs.messages(),
                    })
                }
                Err(RetrievalError::Fatal(errs)) => return Err(errs),
            };
            if versions == Versions::empty() {
                return unavailable(Unavailability::NoVersionSatisfies {
                    name: dependency.name.clone(),
                    requirement: dependency.requirement.clone(),
                    source: dependency.source.clone(),
                    existing: self.versions_at(&dependency.name, &dependency.source),
                });
            }
            constraints.push((dependency.name.clone(), versions));
        }
        self.requirements
            .borrow_mut()
            .insert((package.clone(), version.clone()), retrieved.deps);
        Ok(Dependencies::Available(
            constraints
                .into_iter()
                .collect::<DependencyConstraints<_, _>>(),
        ))
    }
}

/// A set of versions of one project, kept as the versions it holds or as the versions it leaves
/// out. A version requirement becomes the set of the existing versions it accepts at its source,
/// so the set follows the requirement's own matching rules, those for pre-releases included.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Versions {
    /// Exactly these versions.
    Only(BTreeSet<SourcedVersion>),
    /// Every version but these.
    AllBut(BTreeSet<SourcedVersion>),
}

impl VersionSet for Versions {
    type V = SourcedVersion;

    fn empty() -> Self {
        Versions::Only(BTreeSet::new())
    }

    fn singleton(v: SourcedVersion) -> Self {
        Versions::Only(BTreeSet::from([v]))
    }

    fn complement(&self) -> Self {
        match self {
            Versions::Only(vs) => Versions::AllBut(vs.clone()),
            Versions::AllBut(vs) => Versions::Only(vs.clone()),
        }
    }

    fn intersection(&self, other: &Self) -> Self {
        match (self, other) {
            (Versions::Only(a), Versions::Only(b)) => {
                Versions::Only(a.intersection(b).cloned().collect())
            }
            (Versions::Only(a), Versions::AllBut(b)) | (Versions::AllBut(b), Versions::Only(a)) => {
                Versions::Only(a.difference(b).cloned().collect())
            }
            (Versions::AllBut(a), Versions::AllBut(b)) => {
                Versions::AllBut(a.union(b).cloned().collect())
            }
        }
    }

    fn contains(&self, v: &SourcedVersion) -> bool {
        match self {
            Versions::Only(vs) => vs.contains(v),
            Versions::AllBut(vs) => !vs.contains(v),
        }
    }
}

impl Versions {
    /// The versions the set holds or leaves out.
    fn listed(&self) -> &BTreeSet<SourcedVersion> {
        match self {
            Versions::Only(vs) | Versions::AllBut(vs) => vs,
        }
    }
}

/// Writes the versions of each source together, followed by the source.
///
/// # Examples
/// `Only({1.0.0 at A, 1.1.0 at A})` is written `1.0.0 | 1.1.0 from A`, and `AllBut({1.0.0 at A})`
/// is written `other than 1.0.0 from A`.
impl Display for Versions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let by_source = |vs: &BTreeSet<SourcedVersion>| {
            group_by_source(vs)
                .iter()
                .map(|(source, versions)| format!("{} from {}", versions.join(" | "), source))
                .collect::<Vec<_>>()
                .join(" or ")
        };
        match self {
            Versions::Only(vs) if vs.is_empty() => write!(f, "(no existing version)"),
            Versions::Only(vs) => write!(f, "{}", by_source(vs)),
            Versions::AllBut(vs) if vs.is_empty() => write!(f, "(any version)"),
            Versions::AllBut(vs) => write!(f, "other than {}", by_source(vs)),
        }
    }
}

/// The versions in `versions` as text, grouped by their source.
fn group_by_source(versions: &BTreeSet<SourcedVersion>) -> BTreeMap<&Source, Vec<String>> {
    let mut groups: BTreeMap<&Source, Vec<String>> = BTreeMap::new();
    for v in versions {
        groups
            .entry(&v.source)
            .or_default()
            .push(v.version.to_string());
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_dependency, Dependency, Package, PackageName, PackageRetriever, RetrievalError,
        Source, SourcedVersion, VersionRetriever,
    };
    use crate::configuration::BuildConfigType;
    use crate::error::Errors;
    use crate::misc::Map;
    use semver::{Version, VersionReq};
    use std::cell::Cell;

    /// The registry the tests resolve against: the packages, each at a version and a source.
    type Registry = Map<(PackageName, SourcedVersion), Package>;

    /// The source a test fetches the package `name` from, unless it names another.
    fn source_of(name: &str) -> Source {
        Source::Git(format!("https://example.com/{}.git", name))
    }

    /// The version `version` of a package at `source`.
    fn at(version: &str, source: Source) -> SourcedVersion {
        SourcedVersion {
            version: Version::parse(version).unwrap(),
            source,
        }
    }

    /// The requirement `req` on the package `name`, fetched from `source_of(name)`.
    fn dependency(name: &str, req: &str) -> Dependency {
        dependency_from(name, req, source_of(name))
    }

    /// The requirement `req` on the package `name`, fetched from `source`.
    fn dependency_from(name: &str, req: &str, source: Source) -> Dependency {
        Dependency {
            name: name.to_string(),
            requirement: VersionReq::parse(req).unwrap(),
            source,
        }
    }

    /// A registry of packages: each entry is a package of one version and the requirements it has,
    /// written as `("name", "version", &[("dep", "requirement")])`. Every package is fetched from
    /// `source_of` its name, and accepts every compiler.
    fn registry(entries: &[(&str, &str, &[(&str, &str)])]) -> Registry {
        let mut registry = Map::default();
        for (name, version, deps) in entries {
            add(
                &mut registry,
                name,
                at(version, source_of(name)),
                deps.iter().map(|(dep, req)| dependency(dep, req)).collect(),
            );
        }
        registry
    }

    /// Adds to `registry` the package `name` at `version`, with the requirements `deps`, accepting
    /// every compiler.
    fn add(registry: &mut Registry, name: &str, version: SourcedVersion, deps: Vec<Dependency>) {
        registry.insert(
            (name.to_string(), version),
            Package {
                deps,
                fix_version: VersionReq::STAR,
            },
        );
    }

    /// The versions of the package `name` at `source` in `registry`.
    fn versions_in(registry: &Registry, name: &PackageName, source: &Source) -> Vec<Version> {
        registry
            .keys()
            .filter(|(n, v)| n == name && v.source == *source)
            .map(|(_, v)| v.version.clone())
            .collect()
    }

    /// The name of the root package.
    const ROOT: &str = "root";

    /// Resolves the dependencies of the root package under the build configuration `mode`, with the
    /// compiler at version 1.6.1, and returns the chosen packages other than the root or the error
    /// message.
    fn resolve_root(
        mode: BuildConfigType,
        package_retriever: PackageRetriever<'_>,
        versions_retriever: VersionRetriever<'_>,
    ) -> Result<Vec<(PackageName, SourcedVersion)>, String> {
        resolve_dependency(
            (
                &ROOT.to_string(),
                &at("0.1.0", Source::Local("/root".into())),
            ),
            mode,
            &Version::new(1, 6, 1),
            package_retriever,
            versions_retriever,
        )
        .map_err(|errs| errs.to_string().trim_end().to_string())
    }

    /// The outcome of resolving against a registry.
    struct Resolution {
        /// The chosen packages other than the root as `name@version`, sorted, or the error message.
        chosen: Result<Vec<String>, String>,
        /// How many times the resolver retrieved a package at a version.
        retrievals: usize,
    }

    /// Resolves the requirements `root_deps` of a root package against `registry`, under the build
    /// configuration, with the compiler at version 1.6.1. Retrieving a package version listed in
    /// `unusable` fails as a broken version does.
    fn resolve_with(
        registry: &Registry,
        root_deps: &[Dependency],
        unusable: &[(&str, &str)],
    ) -> Resolution {
        let root = Package {
            deps: root_deps.to_vec(),
            fix_version: VersionReq::STAR,
        };
        let retrievals = Cell::new(0);
        let package_retriever =
            |name: &PackageName, version: &SourcedVersion, _: BuildConfigType| {
                retrievals.set(retrievals.get() + 1);
                if name == ROOT {
                    return Ok(root.clone());
                }
                if unusable
                    .iter()
                    .any(|(n, v)| n == name && Version::parse(v).unwrap() == version.version)
                {
                    return Err(RetrievalError::Unusable(Errors::from_msg(
                        "the tag is broken".to_string(),
                    )));
                }
                Ok(registry[&(name.clone(), version.clone())].clone())
            };
        let versions_retriever =
            |name: &PackageName, source: &Source| Ok(versions_in(registry, name, source));
        let chosen = resolve_root(
            BuildConfigType::Build,
            &package_retriever,
            &versions_retriever,
        )
        .map(|chosen| {
            chosen
                .iter()
                .map(|(name, version)| format!("{}@{}", name, version.version))
                .collect()
        });
        Resolution {
            chosen,
            retrievals: retrievals.get(),
        }
    }

    /// Resolves the requirements `root_deps`, each on the package of its name at `source_of` that
    /// name, against `registry`, and returns the chosen packages other than the root as
    /// `name@version`, sorted, or the error message.
    fn resolve(registry: &Registry, root_deps: &[(&str, &str)]) -> Result<Vec<String>, String> {
        let root_deps = root_deps
            .iter()
            .map(|(dep, req)| dependency(dep, req))
            .collect::<Vec<_>>();
        resolve_with(registry, &root_deps, &[]).chosen
    }

    /// When a package's dependency accepts several versions of a package and the root pins an older
    /// one, the pinned version is chosen.
    #[test]
    fn a_pinned_version_wins_over_a_newer_version_a_dependency_accepts() {
        let registry = registry(&[
            ("collection", "1.0.0", &[("common", "^1.0.0")]),
            ("common", "1.0.0", &[]),
            ("common", "1.1.0", &[]),
        ]);
        assert_eq!(
            resolve(&registry, &[("collection", "=1.0.0"), ("common", "=1.0.0")]),
            Ok(vec![
                "collection@1.0.0".to_string(),
                "common@1.0.0".to_string()
            ])
        );
    }

    /// When the conflict shows itself only after a version of another package has been chosen, the
    /// search returns to that choice and tries an older version.
    ///
    /// The newest `a` brings in `c` with any version, and the newest `c` is 2.0.0. Every version of
    /// `b` requires `c` 1.0.0, so the search has to go back to `c`.
    #[test]
    fn a_conflict_found_later_revisits_an_earlier_choice() {
        let registry = registry(&[
            ("a", "1.0.0", &[]),
            ("a", "2.0.0", &[("c", "*")]),
            ("b", "1.0.0", &[("c", "=1.0.0")]),
            ("b", "2.0.0", &[("c", "=1.0.0")]),
            ("b", "3.0.0", &[("c", "=1.0.0")]),
            ("c", "1.0.0", &[]),
            ("c", "2.0.0", &[]),
        ]);
        assert_eq!(
            resolve(&registry, &[("a", "*"), ("b", "*")]),
            Ok(vec![
                "a@2.0.0".to_string(),
                "b@3.0.0".to_string(),
                "c@1.0.0".to_string()
            ])
        );
    }

    /// When the newest version of a package has dependencies no version satisfies, an older version
    /// is chosen, and the requirements of the rejected version do not constrain it.
    ///
    /// `x` 2.0.0 requires `y` `^2.0.0`, which no version of `y` satisfies; `x` 1.0.0 requires `y`
    /// `^1.0.0`.
    #[test]
    fn the_requirements_of_a_rejected_version_are_dropped() {
        let registry = registry(&[
            ("x", "1.0.0", &[("y", "^1.0.0")]),
            ("x", "2.0.0", &[("y", "^2.0.0")]),
            ("y", "1.0.0", &[]),
        ]);
        assert_eq!(
            resolve(&registry, &[("x", "*")]),
            Ok(vec!["x@1.0.0".to_string(), "y@1.0.0".to_string()])
        );
    }

    /// A requirement of the root that conflicts with one it has through another package is
    /// explained along the chain of requirements, each written as the package writes it. A package
    /// met at one source is written without it.
    #[test]
    fn a_conflict_through_a_dependency_is_explained_along_the_chain() {
        let registry = registry(&[
            ("collection", "1.0.0", &[("common", "^1.1.0")]),
            ("common", "1.0.0", &[]),
            ("common", "1.1.0", &[]),
        ]);
        assert_eq!(
            resolve(&registry, &[("collection", "^1.0.0"), ("common", "=1.0.0")]).unwrap_err(),
            "error: Failed to resolve dependencies.\n\
             Because \"root\" requires \"collection\" ^1.0.0, which requires \"common\" ^1.1.0, \
             \"root\" indirectly requires \"common\" 1.1.0.\n\
             So, because \"root\" also requires \"common\" =1.0.0, the requirements of \"root\" \
             cannot be satisfied."
        );
    }

    /// Two packages whose every version requires conflicting versions of a third are explained as
    /// two packages that cannot be used together.
    #[test]
    fn two_packages_requiring_conflicting_versions_are_explained() {
        let registry = registry(&[
            ("a", "1.0.0", &[("c", "=1.0.0")]),
            ("a", "1.1.0", &[("c", "=1.0.0")]),
            ("b", "1.0.0", &[("c", "=2.0.0")]),
            ("b", "1.1.0", &[("c", "=2.0.0")]),
            ("c", "1.0.0", &[]),
            ("c", "2.0.0", &[]),
        ]);
        assert_eq!(
            resolve(&registry, &[("a", "^1.0.0"), ("b", "^1.0.0")]).unwrap_err(),
            "error: Failed to resolve dependencies.\n\
             Because every version of \"a\" requires \"c\" =1.0.0 and every version of \"b\" \
             requires \"c\" =2.0.0, \"a\" and \"b\" cannot be used together.\n\
             So, because \"root\" requires \"a\" ^1.0.0 and \"b\" ^1.0.0, the requirements of \
             \"root\" cannot be satisfied."
        );
    }

    /// A conflict reached along two branches is explained branch by branch, and the conclusion of
    /// the first is referred to by its number where the second uses it.
    #[test]
    fn a_conflict_along_two_branches_is_explained_by_branch() {
        let registry = registry(&[
            ("foo", "1.0.0", &[("a", "^1.0.0"), ("b", "^1.0.0")]),
            ("foo", "1.1.0", &[("x", "^1.0.0"), ("y", "^1.0.0")]),
            ("a", "1.0.0", &[("b", "^2.0.0")]),
            ("b", "1.0.0", &[]),
            ("b", "2.0.0", &[]),
            ("x", "1.0.0", &[("y", "^2.0.0")]),
            ("y", "1.0.0", &[]),
            ("y", "2.0.0", &[]),
        ]);
        assert_eq!(
            resolve(&registry, &[("foo", "^1.0.0")]).unwrap_err(),
            "error: Failed to resolve dependencies.\n\
             Because \"foo\" 1.0.0 requires \"a\" ^1.0.0, which requires \"b\" ^2.0.0, \"foo\" \
             1.0.0 indirectly requires \"b\" 2.0.0.\n\
             And because \"foo\" 1.0.0 requires \"b\" ^1.0.0, \"foo\" 1.0.0 cannot be used. (1)\n\
             \n\
             Because \"foo\" 1.1.0 requires \"x\" ^1.0.0, which requires \"y\" ^2.0.0, \"foo\" \
             1.1.0 indirectly requires \"y\" 2.0.0.\n\
             And because \"foo\" 1.1.0 requires \"y\" ^1.0.0, \"foo\" 1.1.0 cannot be used.\n\
             And because \"foo\" 1.0.0 cannot be used (1), no version of \"foo\" can be used.\n\
             So, because \"root\" requires \"foo\" ^1.0.0, the requirements of \"root\" cannot be \
             satisfied."
        );
    }

    /// The registry of a conflict that has nothing to do with `count` independent packages `a0`,
    /// `a1`, ..., each in four versions: `b` in three versions all requires `c` at `c_requirement`,
    /// and `c` exists in 1.0.0 and 2.0.0. Also returns the root's requirements: every `a`, `b` and
    /// `c`.
    fn registry_with_an_unrelated_conflict(
        count: usize,
        c_requirement: &str,
    ) -> (Registry, Vec<Dependency>) {
        let names = (0..count).map(|i| format!("a{}", i)).collect::<Vec<_>>();
        let mut entries: Vec<(&str, &str, &[(&str, &str)])> = vec![];
        for name in &names {
            for version in ["1.0.0", "2.0.0", "3.0.0", "4.0.0"] {
                entries.push((name, version, &[]));
            }
        }
        let b_deps = [("c", c_requirement)];
        for version in ["1.0.0", "2.0.0", "3.0.0"] {
            entries.push(("b", version, &b_deps));
        }
        entries.push(("c", "1.0.0", &[]));
        entries.push(("c", "2.0.0", &[]));
        let mut root_deps = names
            .iter()
            .map(|name| dependency(name, "*"))
            .collect::<Vec<_>>();
        root_deps.push(dependency("b", "*"));
        root_deps.push(dependency("c", "*"));
        (registry(&entries), root_deps)
    }

    /// A conflict between a few packages is resolved, or found unresolvable, without trying the
    /// combinations of the versions of packages that take no part in it. Trying them would retrieve
    /// packages more than a hundred million times here.
    #[test]
    fn a_conflict_does_not_make_the_search_try_unrelated_combinations() {
        for (c_requirement, expected_c) in [("=1.0.0", Some("c@1.0.0")), ("=3.0.0", None)] {
            let (registry, root_deps) = registry_with_an_unrelated_conflict(12, c_requirement);
            let resolution = resolve_with(&registry, &root_deps, &[]);
            match expected_c {
                Some(c) => assert!(resolution.chosen.unwrap().contains(&c.to_string())),
                None => assert!(resolution.chosen.is_err()),
            }
            assert!(
                resolution.retrievals <= 40,
                "retrieved {} times for `c` {}",
                resolution.retrievals,
                c_requirement
            );
        }
    }

    /// A version whose `fix_version` rejects the running compiler is left out, and an older version
    /// that accepts it is chosen. When no version accepts it, the resolution fails.
    #[test]
    fn a_version_rejecting_the_compiler_is_left_out() {
        let mut registry = registry(&[("x", "1.0.0", &[]), ("x", "2.0.0", &[])]);
        let requires = |registry: &mut Registry, version: &str, req: &str| {
            registry
                .get_mut(&("x".to_string(), at(version, source_of("x"))))
                .unwrap()
                .fix_version = VersionReq::parse(req).unwrap();
        };
        requires(&mut registry, "2.0.0", "^1.7.0");
        assert_eq!(
            resolve(&registry, &[("x", "*")]),
            Ok(vec!["x@1.0.0".to_string()])
        );

        requires(&mut registry, "1.0.0", "^1.7.0");
        assert!(resolve(&registry, &[("x", "*")]).is_err());
    }

    /// The versions of a package left out because they reject the running compiler are explained
    /// together, after the version of the running compiler, and followed by a hint.
    #[test]
    fn versions_rejecting_the_compiler_are_explained_together() {
        let mut registry = registry(&[
            ("json", "1.0.0", &[]),
            ("json", "1.1.0", &[]),
            ("web", "1.0.0", &[("json", "^1.0.0")]),
        ]);
        for (version, req) in [("1.0.0", "^1.7.0"), ("1.1.0", "^1.8.0")] {
            registry
                .get_mut(&("json".to_string(), at(version, source_of("json"))))
                .unwrap()
                .fix_version = VersionReq::parse(req).unwrap();
        }
        assert_eq!(
            resolve(&registry, &[("web", "^1.0.0")]).unwrap_err(),
            "error: Failed to resolve dependencies.\n\
             This is Fix 1.6.1.\n\
             \n\
             Because no version of \"json\" supports Fix 1.6.1 (1.0.0 requires Fix ^1.7.0, 1.1.0 \
             requires Fix ^1.8.0) and \"web\" 1.0.0 requires \"json\" ^1.0.0, \"web\" 1.0.0 cannot \
             be used.\n\
             So, because \"root\" requires \"web\" ^1.0.0, the requirements of \"root\" cannot be \
             satisfied.\n\
             HINT: Use a newer Fix, or require versions that support Fix 1.6.1."
        );
    }

    /// A version that is unusable is left out, and an older version is chosen.
    #[test]
    fn an_unusable_version_is_left_out() {
        let registry = registry(&[("x", "1.0.0", &[]), ("x", "2.0.0", &[])]);
        assert_eq!(
            resolve_with(&registry, &[dependency("x", "*")], &[("x", "2.0.0")]).chosen,
            Ok(vec!["x@1.0.0".to_string()])
        );
    }

    /// A version rejected for a dependency fetched from one source leaves nothing behind that stops
    /// an older version from fetching that dependency from another source.
    ///
    /// `x` 2.0.0 requires `d` `^2.0.0` from one repository, which holds only `d` 1.0.0; `x` 1.0.0
    /// requires `d` from another repository.
    #[test]
    fn a_rejected_version_leaves_no_source_behind() {
        let first = Source::Git("https://example.com/first/d.git".to_string());
        let second = Source::Git("https://example.com/second/d.git".to_string());
        let mut registry = registry(&[]);
        for (version, deps) in [
            ("1.0.0", vec![dependency_from("d", "*", second.clone())]),
            ("2.0.0", vec![dependency_from("d", "^2.0.0", first.clone())]),
        ] {
            add(&mut registry, "x", at(version, source_of("x")), deps);
        }
        for source in [first, second] {
            add(&mut registry, "d", at("1.0.0", source), vec![]);
        }
        assert_eq!(
            resolve(&registry, &[("x", "*")]),
            Ok(vec!["d@1.0.0".to_string(), "x@1.0.0".to_string()])
        );
    }

    /// Two requirements that fetch one package from two sources conflict, and the message names
    /// both sources, followed by a hint.
    #[test]
    fn two_sources_of_one_package_conflict() {
        let other = Source::Git("https://example.com/other/d.git".to_string());
        let mut registry = registry(&[("d", "1.0.0", &[])]);
        add(&mut registry, "d", at("1.0.0", other.clone()), vec![]);
        add(
            &mut registry,
            "x",
            at("1.0.0", source_of("x")),
            vec![dependency_from("d", "*", other)],
        );
        assert_eq!(
            resolve(&registry, &[("d", "*"), ("x", "*")]).unwrap_err(),
            "error: Failed to resolve dependencies.\n\
             Because \"root\" requires \"d\" from https://example.com/d.git and \"x\" 1.0.0 \
             requires \"d\" from https://example.com/other/d.git, \"root\" cannot use \"x\".\n\
             So, because \"root\" also requires \"x\", the requirements of \"root\" cannot be \
             satisfied.\n\
             HINT: Every requirement on one project must name the same source."
        );
    }

    /// Retrieving a version that fails for a reason outside the version stops the resolution with
    /// that error, also where an older version would avoid it.
    #[test]
    fn a_failure_to_retrieve_stops_the_resolution() {
        let registry = registry(&[("x", "1.0.0", &[]), ("x", "2.0.0", &[])]);
        let package_retriever =
            |name: &PackageName, version: &SourcedVersion, _: BuildConfigType| {
                if name == ROOT {
                    return Ok(Package {
                        deps: vec![dependency("x", "*")],
                        fix_version: VersionReq::STAR,
                    });
                }
                if version.version == Version::new(2, 0, 0) {
                    return Err(RetrievalError::Fatal(Errors::from_msg(
                        "the checkout failed".to_string(),
                    )));
                }
                Ok(registry[&(name.clone(), version.clone())].clone())
            };
        let versions_retriever =
            |name: &PackageName, source: &Source| Ok(versions_in(&registry, name, source));
        let message = resolve_root(
            BuildConfigType::Build,
            &package_retriever,
            &versions_retriever,
        )
        .unwrap_err();
        assert!(message.contains("the checkout failed"), "{}", message);
    }

    /// A package whose versions cannot be listed stops the resolution with the error of the
    /// listing, also where an older version of the package that requires it would avoid it.
    ///
    /// `x` 2.0.0 requires `y`, whose versions cannot be listed; `x` 1.0.0 requires nothing.
    #[test]
    fn a_failure_to_list_versions_stops_the_resolution() {
        let registry = registry(&[("x", "1.0.0", &[]), ("x", "2.0.0", &[("y", "*")])]);
        let package_retriever =
            |name: &PackageName, version: &SourcedVersion, _: BuildConfigType| {
                if name == ROOT {
                    return Ok(Package {
                        deps: vec![dependency("x", "*")],
                        fix_version: VersionReq::STAR,
                    });
                }
                Ok(registry[&(name.clone(), version.clone())].clone())
            };
        let versions_retriever = |name: &PackageName, source: &Source| {
            if name == "y" {
                return Err(RetrievalError::Fatal(Errors::from_msg(
                    "the repository is gone".to_string(),
                )));
            }
            Ok(versions_in(&registry, name, source))
        };
        let message = resolve_root(
            BuildConfigType::Build,
            &package_retriever,
            &versions_retriever,
        )
        .unwrap_err();
        assert!(message.contains("the repository is gone"), "{}", message);
    }

    /// A version requiring a package whose source cannot be read, such as a directory with no
    /// project file, is left out, and an older version is chosen. When the root itself requires
    /// such a package, the resolution stops with the error of reading it.
    ///
    /// `x` 2.0.0 requires `y`, whose versions cannot be read; `x` 1.0.0 requires nothing.
    #[test]
    fn a_version_requiring_an_unreadable_package_is_left_out() {
        let registry = registry(&[("x", "1.0.0", &[]), ("x", "2.0.0", &[("y", "*")])]);
        let package_retriever = |root_deps: &'static [(&'static str, &'static str)]| {
            let registry = &registry;
            move |name: &PackageName, version: &SourcedVersion, _: BuildConfigType| {
                if name == ROOT {
                    return Ok(Package {
                        deps: root_deps.iter().map(|(n, r)| dependency(n, r)).collect(),
                        fix_version: VersionReq::STAR,
                    });
                }
                Ok(registry[&(name.clone(), version.clone())].clone())
            }
        };
        let versions_retriever = |name: &PackageName, source: &Source| {
            if name == "y" {
                return Err(RetrievalError::Unusable(Errors::from_msg(
                    "the project file is missing".to_string(),
                )));
            }
            Ok(versions_in(&registry, name, source))
        };
        assert_eq!(
            resolve_root(
                BuildConfigType::Build,
                &package_retriever(&[("x", "*")]),
                &versions_retriever,
            )
            .map(|chosen| chosen
                .into_iter()
                .map(|(name, v)| format!("{}@{}", name, v.version))
                .collect::<Vec<_>>()),
            Ok(vec!["x@1.0.0".to_string()])
        );
        let message = resolve_root(
            BuildConfigType::Build,
            &package_retriever(&[("y", "*")]),
            &versions_retriever,
        )
        .unwrap_err();
        assert!(
            message.contains("the project file is missing")
                && !message.contains("Failed to resolve dependencies"),
            "{}",
            message
        );
    }

    /// A requirement no existing version satisfies is explained with the requirement and the
    /// versions that do exist, once for all the versions that have it.
    #[test]
    fn a_requirement_no_version_satisfies_is_explained() {
        let registry = registry(&[
            ("x", "1.0.0", &[("y", "^2.0.0")]),
            ("x", "1.1.0", &[("y", "^2.0.0")]),
            ("y", "1.0.0", &[]),
            ("y", "1.1.0", &[]),
        ]);
        assert_eq!(
            resolve(&registry, &[("x", "*")]).unwrap_err(),
            "error: Failed to resolve dependencies.\n\
             Because \"root\" requires \"x\" and every version of \"x\" requires \"y\" ^2.0.0, \
             which no version satisfies (https://example.com/y.git has only \"y\" 1.0.0 and \
             1.1.0), the requirements of \"root\" cannot be satisfied."
        );
    }

    /// A requirement the root writes that no existing version satisfies stops the resolution with
    /// the requirement and the versions that do exist.
    #[test]
    fn a_requirement_of_the_root_no_version_satisfies_is_reported() {
        let registry = registry(&[("y", "1.0.0", &[]), ("y", "1.1.0", &[])]);
        assert_eq!(
            resolve(&registry, &[("y", "^2.0.0")]).unwrap_err(),
            "error: \"root\" requires \"y\" ^2.0.0, which no version satisfies \
             (https://example.com/y.git has only \"y\" 1.0.0 and 1.1.0)."
        );
    }

    /// Under the test configuration the root brings in its test dependencies, and every other
    /// package brings in the dependencies of its `build` section alone.
    ///
    /// The root requires `x`, and `t` for its tests; `x` requires `u` for its own tests.
    #[test]
    fn only_the_root_brings_in_its_test_dependencies() {
        let package_retriever = |name: &PackageName, _: &SourcedVersion, mode: BuildConfigType| {
            let (build, test): (&[(&str, &str)], &[(&str, &str)]) = match name.as_str() {
                ROOT => (&[("x", "*")], &[("t", "*")]),
                "x" => (&[], &[("u", "*")]),
                _ => (&[], &[]),
            };
            let mut deps = build
                .iter()
                .map(|(dep, req)| dependency(dep, req))
                .collect::<Vec<_>>();
            if mode == BuildConfigType::Test {
                deps.extend(test.iter().map(|(dep, req)| dependency(dep, req)));
            }
            Ok(Package {
                deps,
                fix_version: VersionReq::STAR,
            })
        };
        let versions_retriever = |_: &PackageName, _: &Source| Ok(vec![Version::new(1, 0, 0)]);
        let chosen = resolve_root(
            BuildConfigType::Test,
            &package_retriever,
            &versions_retriever,
        )
        .map(|chosen| chosen.into_iter().map(|(name, _)| name).collect::<Vec<_>>());
        assert_eq!(chosen, Ok(vec!["t".to_string(), "x".to_string()]));
    }

    /// Versions of a package left out for different reasons are explained one by one, each with its
    /// own reason.
    ///
    /// `x` 1.0.0 cannot be read, and `x` 2.0.0 requires a newer Fix.
    #[test]
    fn versions_left_out_for_different_reasons_are_explained_each() {
        let mut registry = registry(&[("x", "1.0.0", &[]), ("x", "2.0.0", &[])]);
        registry
            .get_mut(&("x".to_string(), at("2.0.0", source_of("x"))))
            .unwrap()
            .fix_version = VersionReq::parse("^1.7.0").unwrap();
        let message = resolve_with(&registry, &[dependency("x", "*")], &[("x", "1.0.0")])
            .chosen
            .unwrap_err();
        assert!(
            message.contains("\"x\" 1.0.0 could not be read (the tag is broken)")
                && message.contains("\"x\" 2.0.0 requires Fix ^1.7.0"),
            "{}",
            message
        );
    }

    /// Every version of a package left out for rejecting the running compiler is explained in one
    /// sentence, however many there are.
    #[test]
    fn many_versions_rejecting_the_compiler_are_explained_together() {
        let mut registry = registry(&[
            ("x", "1.0.0", &[]),
            ("x", "1.1.0", &[]),
            ("x", "1.2.0", &[]),
            ("x", "1.3.0", &[]),
        ]);
        for (version, req) in [
            ("1.0.0", "^1.7.0"),
            ("1.1.0", "^1.8.0"),
            ("1.2.0", "^1.9.0"),
            ("1.3.0", "^1.9.0"),
        ] {
            registry
                .get_mut(&("x".to_string(), at(version, source_of("x"))))
                .unwrap()
                .fix_version = VersionReq::parse(req).unwrap();
        }
        assert_eq!(
            resolve(&registry, &[("x", "*")]).unwrap_err(),
            "error: Failed to resolve dependencies.\n\
             This is Fix 1.6.1.\n\
             \n\
             Because \"root\" requires \"x\" and no version of \"x\" supports Fix 1.6.1 (1.0.0 \
             requires Fix ^1.7.0, 1.1.0 requires Fix ^1.8.0, 1.2.0 requires Fix ^1.9.0, 1.3.0 \
             requires Fix ^1.9.0), the requirements of \"root\" cannot be satisfied.\n\
             HINT: Use a newer Fix, or require versions that support Fix 1.6.1."
        );
    }

    /// A version requiring a package whose source cannot be read is explained with the package, its
    /// source, and why it could not be read.
    ///
    /// Every version of `x` requires `y`, whose versions cannot be read.
    #[test]
    fn a_requirement_on_an_unreadable_package_is_explained() {
        let registry = registry(&[("x", "1.0.0", &[("y", "*")]), ("x", "2.0.0", &[("y", "*")])]);
        let package_retriever =
            |name: &PackageName, version: &SourcedVersion, _: BuildConfigType| {
                if name == ROOT {
                    return Ok(Package {
                        deps: vec![dependency("x", "*")],
                        fix_version: VersionReq::STAR,
                    });
                }
                Ok(registry[&(name.clone(), version.clone())].clone())
            };
        let versions_retriever = |name: &PackageName, source: &Source| {
            if name == "y" {
                return Err(RetrievalError::Unusable(Errors::from_msg(
                    "the project file is missing".to_string(),
                )));
            }
            Ok(versions_in(&registry, name, source))
        };
        assert_eq!(
            resolve_root(
                BuildConfigType::Build,
                &package_retriever,
                &versions_retriever,
            )
            .unwrap_err(),
            "error: Failed to resolve dependencies.\n\
             Because \"root\" requires \"x\" and every version of \"x\" requires \"y\" from \
             https://example.com/y.git, which could not be read (the project file is missing), the \
             requirements of \"root\" cannot be satisfied."
        );
    }

    /// A requirement on a package with no version at its source is explained as such.
    #[test]
    fn a_requirement_on_a_package_with_no_version_is_explained() {
        let registry = registry(&[("x", "1.0.0", &[("y", "^1.0.0")])]);
        assert_eq!(
            resolve(&registry, &[("x", "*")]).unwrap_err(),
            "error: Failed to resolve dependencies.\n\
             Because \"root\" requires \"x\" and \"x\" 1.0.0 requires \"y\" ^1.0.0, which no \
             version satisfies (https://example.com/y.git has no version of \"y\"), the \
             requirements of \"root\" cannot be satisfied."
        );
    }

    /// Where the explanation meets a package at two sources, the versions it writes of that package
    /// carry their source, and a source whose every version they hold is written alone.
    ///
    /// The root requires `d` 1.0.0 from one repository, and every version of `x` requires `d` from
    /// another.
    #[test]
    fn the_versions_of_a_package_met_at_two_sources_carry_the_source() {
        let other = Source::Git("https://example.com/other/d.git".to_string());
        let mut registry = registry(&[("d", "1.0.0", &[]), ("d", "2.0.0", &[])]);
        for version in ["1.0.0", "2.0.0"] {
            add(&mut registry, "d", at(version, other.clone()), vec![]);
        }
        for (version, req) in [("1.0.0", "^1.0.0"), ("2.0.0", "*")] {
            add(
                &mut registry,
                "x",
                at(version, source_of("x")),
                vec![dependency_from("d", req, other.clone())],
            );
        }
        assert_eq!(
            resolve(&registry, &[("d", "=1.0.0"), ("x", "*")]).unwrap_err(),
            "error: Failed to resolve dependencies.\n\
             Because \"x\" 1.0.0 requires \"d\" ^1.0.0 from https://example.com/other/d.git and \
             \"x\" 2.0.0 requires \"d\" from https://example.com/other/d.git, every version of \
             \"x\" indirectly requires \"d\" from https://example.com/other/d.git.\n\
             So, because \"root\" requires \"d\" =1.0.0 from https://example.com/d.git and \"x\", \
             the requirements of \"root\" cannot be satisfied.\n\
             HINT: Every requirement on one project must name the same source."
        );
    }
    /// A conclusion that a package needs one of several others names them as alternatives.
    ///
    /// `b` 1.0.0 requires `f` 2.0.0 and `b` 1.1.0 requires `g` 2.0.0, so every version of `b`
    /// needs one of the two, and neither both.
    #[test]
    fn a_package_needing_one_of_several_is_explained_with_alternatives() {
        let registry = registry(&[
            ("b", "1.0.0", &[("f", "^2.0.0")]),
            ("b", "1.1.0", &[("g", "^2.0.0")]),
            ("f", "1.0.0", &[]),
            ("f", "2.0.0", &[]),
            ("g", "1.0.0", &[]),
            ("g", "2.0.0", &[]),
        ]);
        assert_eq!(
            resolve(&registry, &[("b", "*"), ("f", "^1.0.0"), ("g", "^1.0.0")]).unwrap_err(),
            "error: Failed to resolve dependencies.\n\
             Because \"b\" 1.0.0 requires \"f\" ^2.0.0 and \"b\" 1.1.0 requires \"g\" ^2.0.0, \
             every version of \"b\" indirectly requires \"f\" 2.0.0 or \"g\" 2.0.0.\n\
             And because \"root\" requires \"b\", \"root\" indirectly requires \"f\" 2.0.0 or \
             \"g\" 2.0.0.\n\
             So, because \"root\" also requires \"f\" ^1.0.0 and \"g\" ^1.0.0, the requirements of \
             \"root\" cannot be satisfied."
        );
    }

    /// A requirement of `*`, which leaves out pre-releases, is written out where the versions it
    /// accepts are not every version.
    #[test]
    fn a_requirement_of_any_version_leaving_out_a_pre_release_is_written_out() {
        let only_a_pre_release = registry(&[("y", "1.0.0-alpha", &[])]);
        assert_eq!(
            resolve(&only_a_pre_release, &[("y", "*")]).unwrap_err(),
            "error: \"root\" requires \"y\" *, which no version satisfies \
             (https://example.com/y.git has only \"y\" 1.0.0-alpha)."
        );
        let with_a_pre_release = registry(&[
            ("x", "1.0.0", &[("y", "*")]),
            ("y", "1.0.0", &[]),
            ("y", "2.0.0-beta.1", &[]),
        ]);
        let message =
            resolve(&with_a_pre_release, &[("x", "*"), ("y", "=2.0.0-beta.1")]).unwrap_err();
        assert!(
            message.contains("\"root\" requires \"x\", which requires \"y\" *,"),
            "{}",
            message
        );
    }

    /// Versions of one package that fetch a dependency from different sources leave no hint about
    /// sources, since no two of them are used together.
    ///
    /// `y` 1.0.0 requires `d` from one repository and `y` 2.0.0 from another, and every `d`
    /// requires a newer Fix.
    #[test]
    fn alternative_versions_fetching_from_two_sources_give_no_hint_about_sources() {
        let first = Source::Git("https://example.com/first/d.git".to_string());
        let second = Source::Git("https://example.com/second/d.git".to_string());
        let mut registry = registry(&[]);
        for (version, source) in [("1.0.0", &first), ("2.0.0", &second)] {
            add(
                &mut registry,
                "y",
                at(version, source_of("y")),
                vec![dependency_from("d", "*", source.clone())],
            );
            add(&mut registry, "d", at("1.0.0", source.clone()), vec![]);
            registry
                .get_mut(&("d".to_string(), at("1.0.0", source.clone())))
                .unwrap()
                .fix_version = VersionReq::parse("^1.7.0").unwrap();
        }
        let message = resolve(&registry, &[("y", "*")]).unwrap_err();
        assert!(
            message.contains("HINT: Use a newer Fix")
                && !message.contains("HINT: Every requirement on one project must name the same"),
            "{}",
            message
        );
    }
    /// Depender versions that write different requirements accepting the same versions are
    /// explained with those versions, since no one requirement is the one every version writes.
    #[test]
    fn requirements_written_differently_are_explained_by_their_versions() {
        let registry = registry(&[
            ("collection", "1.0.0", &[("common", "^1.1.0")]),
            ("collection", "1.1.0", &[("common", ">=1.1.0, <2.0.0")]),
            ("common", "1.0.0", &[]),
            ("common", "1.1.0", &[]),
        ]);
        assert_eq!(
            resolve(&registry, &[("collection", "^1.0.0"), ("common", "=1.0.0")]).unwrap_err(),
            "error: Failed to resolve dependencies.\n\
             Because \"root\" requires \"collection\" ^1.0.0, which requires \"common\" 1.1.0, \
             \"root\" indirectly requires \"common\" 1.1.0.\n\
             So, because \"root\" also requires \"common\" =1.0.0, the requirements of \"root\" \
             cannot be satisfied."
        );
    }
}
