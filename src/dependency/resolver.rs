//! Dependency resolution: choosing a version of every project a root project depends on, directly
//! or transitively, so that every version requirement holds. The search is the PubGrub algorithm of
//! the `pubgrub` crate, which learns the cause of each conflict and so never retries a combination
//! of versions that the cause already rules out.

use crate::{configuration::BuildConfigType, error::Errors, misc::Map};
use pubgrub::{
    DefaultStringReporter, Dependencies, DependencyProvider, PackageResolutionStatistics,
    PubGrubError, Reporter, VersionSet,
};
use semver::{Version, VersionReq};
use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::BTreeSet,
    fmt::{self, Debug, Display},
};

pub type PackageName = String;

/// What the resolver reads of a project at one version.
#[derive(Clone)]
pub struct Package {
    /// The version requirements this version has on other projects.
    pub deps: Vec<Dependency>,
    /// The compiler versions this version builds with, as its `fix_version` field requires.
    pub fix_version: VersionReq,
}

/// A version requirement on a project.
#[derive(Clone)]
pub struct Dependency {
    pub name: PackageName,
    pub requirement: VersionReq,
}

/// Retrieves a project at a version. The build configuration selects the dependencies the project
/// file lists.
pub type PackageRetriever<'a> =
    &'a dyn Fn(&PackageName, &Version, BuildConfigType) -> Result<Package, Errors>;

/// Lists the versions a project exists in.
pub type VersionRetriever<'a> = &'a dyn Fn(&PackageName) -> Result<Vec<Version>, Errors>;

/// Chooses a version of every project the root project depends on, directly or transitively, such
/// that every version requirement holds and every chosen version accepts the running compiler.
/// Among the choices, newer versions are preferred.
///
/// A version that cannot be retrieved, or whose `fix_version` rejects `compiler_version`, is left
/// out of the choice. When no choice exists, the error explains which requirements conflict.
///
/// # Arguments
/// * `mode` — the build configuration whose dependencies the root project brings in. The
///   dependencies of other projects are read from their `build` section.
/// * `compiler_version` — the version of the running compiler.
///
/// # Returns
/// The chosen projects other than the root, sorted by name.
pub fn resolve_dependency(
    root: (&PackageName, &Version),
    mode: BuildConfigType,
    compiler_version: &Version,
    package_retriever: PackageRetriever<'_>,
    versions_retriever: VersionRetriever<'_>,
) -> Result<Vec<(PackageName, Version)>, Errors> {
    let provider = Provider {
        root_name: root.0.clone(),
        root_version: root.1.clone(),
        mode,
        compiler_version: compiler_version.clone(),
        package_retriever,
        versions_retriever,
        known_versions: RefCell::new(Map::from_iter([(root.0.clone(), vec![root.1.clone()])])),
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
        | Err(PubGrubError::ErrorInShouldCancel(source)) => Err(source.0),
    }
}

/// What the PubGrub solver asks about the projects: their versions, and the requirements and
/// `fix_version` of each version.
struct Provider<'a, 'b> {
    root_name: PackageName,
    root_version: Version,
    mode: BuildConfigType,
    compiler_version: Version,
    package_retriever: PackageRetriever<'a>,
    versions_retriever: VersionRetriever<'b>,
    /// The versions of the root project and of each project some retrieved version has a
    /// requirement on, newest first. The solver asks only about these projects.
    known_versions: RefCell<Map<PackageName, Vec<Version>>>,
}

impl Provider<'_, '_> {
    /// The versions of `name`, recorded when the first requirement on it was read.
    fn known_versions_of(&self, name: &PackageName) -> Vec<Version> {
        self.known_versions
            .borrow()
            .get(name)
            .unwrap_or_else(|| panic!("no requirement on \"{}\" has been read", name))
            .clone()
    }

    /// The versions of `dependency.name` that satisfy `dependency.requirement`. The first
    /// requirement read on a project records its versions.
    fn versions_satisfying(&self, dependency: &Dependency) -> Result<Versions, Errors> {
        if !self.known_versions.borrow().contains_key(&dependency.name) {
            let mut versions = (self.versions_retriever)(&dependency.name)?;
            versions.sort();
            versions.reverse();
            self.known_versions
                .borrow_mut()
                .insert(dependency.name.clone(), versions);
        }
        Ok(Versions::Only(
            self.known_versions.borrow()[&dependency.name]
                .iter()
                .filter(|v| dependency.requirement.matches(v))
                .cloned()
                .collect(),
        ))
    }
}

impl DependencyProvider for Provider<'_, '_> {
    type P = PackageName;
    type V = Version;
    type VS = Versions;
    /// Decides first the projects that took part in more conflicts, then those with fewer versions
    /// left to choose from.
    type Priority = (u32, Reverse<usize>);
    /// Why a version is left out of the choice, written to follow "are unavailable".
    type M = String;
    type Err = RetrievalError;

    fn prioritize(
        &self,
        package: &PackageName,
        range: &Versions,
        package_conflicts_counts: &PackageResolutionStatistics,
    ) -> Self::Priority {
        let candidates = self
            .known_versions_of(package)
            .iter()
            .filter(|v| range.contains(v))
            .count();
        (
            package_conflicts_counts.conflict_count(),
            Reverse(candidates),
        )
    }

    fn choose_version(
        &self,
        package: &PackageName,
        range: &Versions,
    ) -> Result<Option<Version>, RetrievalError> {
        Ok(self
            .known_versions_of(package)
            .into_iter()
            .find(|v| range.contains(v)))
    }

    fn get_dependencies(
        &self,
        package: &PackageName,
        version: &Version,
    ) -> Result<Dependencies<PackageName, Versions, String>, RetrievalError> {
        let mode = if *package == self.root_name && *version == self.root_version {
            self.mode
        } else {
            BuildConfigType::Build
        };
        let retrieved = match (self.package_retriever)(package, version, mode) {
            Ok(retrieved) => retrieved,
            Err(errs) => {
                return Ok(Dependencies::Unavailable(format!(
                    "because it could not be retrieved: {}",
                    errs
                )))
            }
        };
        if !retrieved.fix_version.matches(&self.compiler_version) {
            return Ok(Dependencies::Unavailable(format!(
                "because it requires Fix {}, and this is Fix {}",
                retrieved.fix_version, self.compiler_version
            )));
        }
        let mut constraints = vec![];
        for dependency in &retrieved.deps {
            let versions = self
                .versions_satisfying(dependency)
                .map_err(RetrievalError)?;
            constraints.push((dependency.name.clone(), versions));
        }
        Ok(Dependencies::Available(constraints.into_iter().collect()))
    }
}

/// A set of versions of one project, kept as the versions it holds or as the versions it leaves
/// out. A version requirement becomes the set of the existing versions it accepts, so the set
/// follows the requirement's own matching rules, those for pre-releases included.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Versions {
    /// Exactly these versions.
    Only(BTreeSet<Version>),
    /// Every version but these.
    AllBut(BTreeSet<Version>),
}

impl VersionSet for Versions {
    type V = Version;

    fn empty() -> Self {
        Versions::Only(BTreeSet::new())
    }

    fn singleton(v: Version) -> Self {
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

    fn contains(&self, v: &Version) -> bool {
        match self {
            Versions::Only(vs) => vs.contains(v),
            Versions::AllBut(vs) => !vs.contains(v),
        }
    }
}

/// Writes the set as the explanation of a failed resolution shows it.
///
/// # Examples
/// `Only({1.0.0, 1.1.0})` is written `1.0.0 | 1.1.0`, and `AllBut({1.0.0})` is written
/// `other than 1.0.0`.
impl Display for Versions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let join = |vs: &BTreeSet<Version>| {
            vs.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        };
        match self {
            Versions::Only(vs) if vs.is_empty() => write!(f, "(no existing version)"),
            Versions::Only(vs) => write!(f, "{}", join(vs)),
            Versions::AllBut(vs) if vs.is_empty() => write!(f, "(any version)"),
            Versions::AllBut(vs) => write!(f, "other than {}", join(vs)),
        }
    }
}

/// An error that stops the resolution: the versions of a project could not be listed.
pub struct RetrievalError(Errors);

impl Display for RetrievalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for RetrievalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl std::error::Error for RetrievalError {}

#[cfg(test)]
mod tests {
    use super::{resolve_dependency, Dependency, Package, PackageName};
    use crate::configuration::BuildConfigType;
    use crate::error::Errors;
    use crate::misc::Map;
    use semver::{Version, VersionReq};
    use std::cell::Cell;

    /// The requirement `req` on the package `name`.
    fn dependency(name: &str, req: &str) -> Dependency {
        Dependency {
            name: name.to_string(),
            requirement: VersionReq::parse(req).unwrap(),
        }
    }

    /// A registry of packages for the resolver: each entry is a package of one version and the
    /// requirements it has, written as `("name", "version", &[("dep", "requirement")])`. Every
    /// package accepts every compiler.
    fn registry(entries: &[(&str, &str, &[(&str, &str)])]) -> Map<(PackageName, Version), Package> {
        let mut map = Map::default();
        for (name, version, deps) in entries {
            let package = Package {
                deps: deps.iter().map(|(dep, req)| dependency(dep, req)).collect(),
                fix_version: VersionReq::STAR,
            };
            map.insert(
                (name.to_string(), Version::parse(version).unwrap()),
                package,
            );
        }
        map
    }

    /// The outcome of resolving against a registry.
    struct Resolution {
        /// The chosen packages other than the root as `name@version`, sorted, or the error message.
        chosen: Result<Vec<String>, String>,
        /// How many times the resolver retrieved a package at a version.
        retrievals: usize,
    }

    /// Resolves the requirements `root_deps` of a root package against `registry`, with the
    /// compiler at version 1.6.1. Retrieving a package listed in `broken` fails.
    fn resolve_with(
        registry: &Map<(PackageName, Version), Package>,
        root_deps: &[(&str, &str)],
        broken: &[(&str, &str)],
    ) -> Resolution {
        let root_name = "root".to_string();
        let root_version = Version::new(0, 1, 0);
        let root = Package {
            deps: root_deps
                .iter()
                .map(|(dep, req)| dependency(dep, req))
                .collect(),
            fix_version: VersionReq::STAR,
        };
        let retrievals = Cell::new(0);
        let package_retriever = |name: &PackageName, version: &Version, _: BuildConfigType| {
            retrievals.set(retrievals.get() + 1);
            if *name == root_name {
                return Ok(root.clone());
            }
            if broken
                .iter()
                .any(|(n, v)| n == name && Version::parse(v).unwrap() == *version)
            {
                return Err(Errors::from_msg("the tag is broken".to_string()));
            }
            Ok(registry[&(name.clone(), version.clone())].clone())
        };
        let versions_retriever = |name: &PackageName| {
            Ok(registry
                .keys()
                .filter(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
                .collect())
        };
        let chosen = resolve_dependency(
            (&root_name, &root_version),
            BuildConfigType::Build,
            &Version::new(1, 6, 1),
            &package_retriever,
            &versions_retriever,
        )
        .map(|chosen| {
            chosen
                .iter()
                .map(|(name, version)| format!("{}@{}", name, version))
                .collect()
        })
        .map_err(|errs| errs.to_string());
        Resolution {
            chosen,
            retrievals: retrievals.get(),
        }
    }

    /// Resolves the requirements `root_deps` of a root package against `registry`, and returns the
    /// chosen packages other than the root as `name@version`, sorted, or the error message.
    fn resolve(
        registry: &Map<(PackageName, Version), Package>,
        root_deps: &[(&str, &str)],
    ) -> Result<Vec<String>, String> {
        resolve_with(registry, root_deps, &[]).chosen
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
                && message.contains("a 1.0.0 depends on c 1.0.0")
                && message.contains("b 1.0.0 depends on c 2.0.0"),
            "{}",
            message
        );
    }

    /// The registry of a conflict that has nothing to do with `count` independent packages `a0`,
    /// `a1`, ..., each in four versions: `b` in three versions all requires `c` at `c_requirement`,
    /// and `c` exists in 1.0.0 and 2.0.0. The root requires every `a`, `b` and `c`.
    fn registry_with_an_unrelated_conflict(
        count: usize,
        c_requirement: &str,
    ) -> (Map<(PackageName, Version), Package>, Vec<(String, String)>) {
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
            .map(|name| (name.clone(), "*".to_string()))
            .collect::<Vec<_>>();
        root_deps.push(("b".to_string(), "*".to_string()));
        root_deps.push(("c".to_string(), "*".to_string()));
        (registry(&entries), root_deps)
    }

    /// A conflict between a few packages is resolved, or found unresolvable, without trying the
    /// combinations of the versions of packages that take no part in it. Trying them would retrieve
    /// packages 4^12 times here.
    #[test]
    fn a_conflict_does_not_make_the_search_try_unrelated_combinations() {
        for (c_requirement, expected_c) in [("=1.0.0", Some("c@1.0.0")), ("=3.0.0", None)] {
            let (registry, root_deps) = registry_with_an_unrelated_conflict(12, c_requirement);
            let root_deps = root_deps
                .iter()
                .map(|(n, r)| (n.as_str(), r.as_str()))
                .collect::<Vec<_>>();
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
        let requires = |registry: &mut Map<(PackageName, Version), Package>, v: &str, req: &str| {
            registry
                .get_mut(&("x".to_string(), Version::parse(v).unwrap()))
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

    /// A version that cannot be retrieved is left out, and an older version is chosen.
    #[test]
    fn a_version_that_cannot_be_retrieved_is_left_out() {
        let registry = registry(&[("x", "1.0.0", &[]), ("x", "2.0.0", &[])]);
        assert_eq!(
            resolve_with(&registry, &[("x", "*")], &[("x", "2.0.0")]).chosen,
            Ok(vec!["x@1.0.0".to_string()])
        );
    }
}
