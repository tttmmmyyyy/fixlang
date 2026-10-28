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
    DefaultStringReporter, Dependencies, DependencyConstraints, DependencyProvider,
    PackageResolutionStatistics, PubGrubError, Reporter, VersionSet,
};
use semver::{Version, VersionReq};
use std::{
    cell::{Ref, RefCell},
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

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
pub type VersionRetriever<'a> = &'a dyn Fn(&PackageName, &Source) -> Result<Vec<Version>, Errors>;

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
            Err(Errors::from_msg(format!(
                "Failed to resolve dependencies.\n{}",
                DefaultStringReporter::report(&tree)
            )))
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
    fn versions_satisfying(&self, dependency: &Dependency) -> Result<Versions, Errors> {
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
        Ok(Versions::Only(
            self.known_versions.borrow()[&dependency.name]
                .iter()
                .filter(|v| {
                    v.source == dependency.source && dependency.requirement.matches(&v.version)
                })
                .cloned()
                .collect(),
        ))
    }
}

impl DependencyProvider for Provider<'_, '_> {
    type P = PackageName;
    type V = SourcedVersion;
    type VS = Versions;
    /// Decides first the projects that took part in more conflicts, then those with fewer versions
    /// left to choose from.
    type Priority = (u32, Reverse<usize>);
    /// Why a version is left out of the choice, written to follow "are unavailable".
    type M = String;
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
    /// rejects the running compiler, is unavailable, with the reason.
    fn get_dependencies(
        &self,
        package: &PackageName,
        version: &SourcedVersion,
    ) -> Result<Dependencies<PackageName, Versions, String>, Errors> {
        let mode = if *package == self.root_name && *version == self.root_version {
            self.mode
        } else {
            BuildConfigType::Build
        };
        let retrieved = match (self.package_retriever)(package, version, mode) {
            Ok(retrieved) => retrieved,
            Err(RetrievalError::Unusable(errs)) => {
                return Ok(Dependencies::Unavailable(format!(
                    "because it could not be read: {}",
                    errs.to_string().trim_end()
                )))
            }
            Err(RetrievalError::Fatal(errs)) => return Err(errs),
        };
        if !retrieved.fix_version.matches(&self.compiler_version) {
            return Ok(Dependencies::Unavailable(format!(
                "because it requires Fix {}, and this is Fix {}",
                retrieved.fix_version, self.compiler_version
            )));
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
        let constraints = retrieved
            .deps
            .iter()
            .map(|dependency| {
                Ok((
                    dependency.name.clone(),
                    self.versions_satisfying(dependency)?,
                ))
            })
            .collect::<Result<DependencyConstraints<_, _>, Errors>>()?;
        Ok(Dependencies::Available(constraints))
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

/// Writes the set as the explanation of a failed resolution shows it, the versions of each source
/// together.
///
/// # Examples
/// `Only({1.0.0 at A, 1.1.0 at A})` is written `1.0.0 | 1.1.0 from A`, and `AllBut({1.0.0 at A})`
/// is written `other than 1.0.0 from A`.
impl Display for Versions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let by_source = |vs: &BTreeSet<SourcedVersion>| {
            let mut groups: BTreeMap<&Source, Vec<String>> = BTreeMap::new();
            for v in vs {
                groups
                    .entry(&v.source)
                    .or_default()
                    .push(v.version.to_string());
            }
            groups
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
        .map_err(|errs| errs.to_string())
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

    /// Requirements that no choice of versions satisfies fail the resolution, and the message names
    /// the two packages whose requirements conflict.
    #[test]
    fn unsatisfiable_requirements_are_explained() {
        let registry = registry(&[
            ("a", "1.0.0", &[("c", "=1.0.0")]),
            ("b", "1.0.0", &[("c", "=2.0.0")]),
            ("c", "1.0.0", &[]),
            ("c", "2.0.0", &[]),
        ]);
        let message = resolve(&registry, &[("a", "*"), ("b", "*")]).unwrap_err();
        assert!(
            message.contains("Failed to resolve dependencies.")
                && message.contains("depends on c 1.0.0 from https://example.com/c.git")
                && message.contains("depends on c 2.0.0 from https://example.com/c.git"),
            "{}",
            message
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
    /// packages 4^12 times here.
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
    /// that accepts it is chosen. When no version accepts it, the message says so.
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
        let message = resolve(&registry, &[("x", "*")]).unwrap_err();
        assert!(
            message.contains("requires Fix ^1.7.0, and this is Fix 1.6.1"),
            "{}",
            message
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
    /// both sources.
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
        let message = resolve(&registry, &[("d", "*"), ("x", "*")]).unwrap_err();
        assert!(
            message.contains("https://example.com/d.git")
                && message.contains("https://example.com/other/d.git"),
            "{}",
            message
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
                return Err(Errors::from_msg("the repository is gone".to_string()));
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
}
