// This module implements an algorithm of dependency resolution.

use crate::{
    configuration::BuildConfigType, error::Errors, metafiles::project_file::ProjectFile,
    misc::info_msg,
};
use semver::{Version, VersionReq};

pub type PackageName = String;

// Package of specific version.
#[derive(Clone)]
pub struct Package {
    pub name: PackageName,
    pub version: Version,
    pub deps: Vec<Dependency>,
}

// Dependency, i.e., a version requirement for a package.
#[derive(Clone)]
pub struct Dependency {
    pub name: PackageName,
    pub requirement: VersionReq,
}

// Package retriever function.
// It takes a package of a specific version and returns its package information.
pub type PackageRetriever<'a> =
    &'a dyn Fn(&PackageName, &Version, BuildConfigType) -> Result<Package, Errors>;

// Version retriever function.
// It takes a package name and returns a list of versions which exist.
pub type VersionRetriever<'a> = &'a dyn Fn(&PackageName) -> Result<Vec<Version>, Errors>;

// Find a version of every package the root project depends on, directly or transitively, such that
// every package's requirements are satisfied.
// Returns the root package and the chosen packages, or None if no choice of versions satisfies all
// the requirements.
// If retriever functions rise an error, returns the error.
pub fn resolve_dependency<'a, 'b>(
    root_proj: &ProjectFile,
    package_retriever: PackageRetriever<'a>,
    versions_retriever: VersionRetriever<'b>,
    mode: BuildConfigType,
) -> Result<Option<Vec<Package>>, Errors> {
    let root = package_retriever(
        &root_proj.general.name,
        &root_proj.general.version(),
        mode,
    )?;
    let requirements = root.deps.clone();
    resolve_requirements(
        vec![root],
        requirements,
        package_retriever,
        versions_retriever,
        0,
    )
}

// Choose versions for the packages named by `requirements` and not yet in `fixed`, together with
// everything they depend on, so that every requirement is satisfied.
// `requirements` holds the dependencies of the packages in `fixed`; the ones on packages already in
// `fixed` are checked against the chosen version.
// The search is depth-first and complete: when a choice leads to a conflict, the next version of the
// most recently chosen package is tried, and so on back to the first choice.
// Returns `fixed` extended by the chosen packages, or None if no choice satisfies all the
// requirements.
// If retriever functions rise an error, returns the error.
fn resolve_requirements<'a, 'b>(
    fixed: Vec<Package>,
    requirements: Vec<Dependency>,
    package_retriever: PackageRetriever<'a>,
    versions_retriever: VersionRetriever<'b>,
    depth: usize,
) -> Result<Option<Vec<Package>>, Errors> {
    let indent = " ".repeat(depth);

    // Check the requirements on the packages whose versions are already chosen, and keep the others.
    let mut open: Vec<Dependency> = vec![];
    for req in requirements {
        match fixed.iter().find(|p| p.name == req.name) {
            Some(package) if req.requirement.matches(&package.version) => {}
            Some(package) => {
                info_msg(&format!(
                    "{}Accepted \"{}@{}\" conflicts with the requirement `{}`. Backtrack.",
                    indent, package.name, package.version, req.requirement
                ));
                return Ok(None);
            }
            None => open.push(req),
        }
    }
    if open.is_empty() {
        return Ok(Some(fixed));
    }

    // Choose the package with the fewest versions satisfying all its open requirements, so that a
    // conflict shows itself after as few choices as possible.
    let mut names: Vec<&PackageName> = vec![];
    for req in &open {
        if !names.contains(&&req.name) {
            names.push(&req.name);
        }
    }
    let mut chosen: Option<(PackageName, Vec<Version>)> = None;
    for name in names {
        let candidates = candidate_versions(name, &open, versions_retriever)?;
        if chosen
            .as_ref()
            .map_or(true, |(_, best)| candidates.len() < best.len())
        {
            chosen = Some((name.clone(), candidates));
        }
    }
    let (name, candidates) = chosen.unwrap();
    let reqs_text = open
        .iter()
        .filter(|req| req.name == name)
        .map(|req| format!("`{}`", req.requirement))
        .collect::<Vec<_>>()
        .join(", ");
    if candidates.is_empty() {
        info_msg(&format!(
            "{}No version of \"{}\" satisfies {}. Backtrack.",
            indent, name, reqs_text
        ));
        return Ok(None);
    }
    info_msg(&format!(
        "{}Resolving \"{}\" to satisfy {}.",
        indent, name, reqs_text
    ));

    // The requirements on `name` hold for every candidate, so they are dropped.
    open.retain(|req| req.name != name);
    for version in &candidates {
        info_msg(&format!("{} Trying \"{}@{}\".", indent, name, version));
        let package = package_retriever(&name, version, BuildConfigType::Build)?;
        let mut requirements = open.clone();
        requirements.extend(package.deps.iter().cloned());
        let mut next_fixed = fixed.clone();
        next_fixed.push(package);
        let res = resolve_requirements(
            next_fixed,
            requirements,
            package_retriever,
            versions_retriever,
            depth + 1,
        )?;
        if res.is_some() {
            return Ok(res);
        }
        info_msg(&format!("{} Reject \"{}@{}\".", indent, name, version));
    }
    info_msg(&format!(
        "{}No version of \"{}\" was available. Backtrack.",
        indent, name
    ));
    Ok(None)
}

// The versions of the package `name` that satisfy every requirement on it in `requirements`, newest
// first.
fn candidate_versions<'b>(
    name: &PackageName,
    requirements: &[Dependency],
    versions_retriever: VersionRetriever<'b>,
) -> Result<Vec<Version>, Errors> {
    let mut versions = versions_retriever(name)?
        .into_iter()
        .filter(|v| {
            requirements
                .iter()
                .filter(|req| &req.name == name)
                .all(|req| req.requirement.matches(v))
        })
        .collect::<Vec<_>>();
    versions.sort();
    versions.reverse();
    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::misc::Map;

    /// A registry of packages for the resolver: each entry is a package of one version and the
    /// requirements it has, written as `("name", "version", &[("dep", "requirement")])`.
    fn registry(
        entries: &[(&str, &str, &[(&str, &str)])],
    ) -> Map<(PackageName, Version), Package> {
        let mut map = Map::default();
        for (name, version, deps) in entries {
            let package = Package {
                name: name.to_string(),
                version: Version::parse(version).unwrap(),
                deps: deps
                    .iter()
                    .map(|(dep, req)| Dependency {
                        name: dep.to_string(),
                        requirement: VersionReq::parse(req).unwrap(),
                    })
                    .collect(),
            };
            map.insert((package.name.clone(), package.version.clone()), package);
        }
        map
    }

    /// Resolves the requirements `root_deps` of a root package against `registry`, and returns the
    /// chosen packages other than the root as `name@version`, sorted.
    fn resolve(
        registry: &Map<(PackageName, Version), Package>,
        root_deps: &[(&str, &str)],
    ) -> Option<Vec<String>> {
        let package_retriever = |name: &PackageName, version: &Version, _: BuildConfigType| {
            Ok(registry[&(name.clone(), version.clone())].clone())
        };
        let versions_retriever = |name: &PackageName| {
            Ok(registry
                .keys()
                .filter(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
                .collect())
        };
        let root = Package {
            name: "root".to_string(),
            version: Version::new(0, 1, 0),
            deps: vec![],
        };
        let requirements = root_deps
            .iter()
            .map(|(dep, req)| Dependency {
                name: dep.to_string(),
                requirement: VersionReq::parse(req).unwrap(),
            })
            .collect();
        let fixed = resolve_requirements(
            vec![root],
            requirements,
            &package_retriever,
            &versions_retriever,
            0,
        )
        .unwrap_or_else(|_| panic!("the retrievers of the test registry do not fail"))?;
        let mut chosen = fixed[1..]
            .iter()
            .map(|p| format!("{}@{}", p.name, p.version))
            .collect::<Vec<_>>();
        chosen.sort();
        Some(chosen)
    }

    /// When a package's dependency accepts several versions of a package and the root pins an older
    /// one, the pinned version is chosen even though the dependency is resolved first.
    #[test]
    fn a_pinned_version_wins_over_a_newer_version_a_dependency_accepts() {
        let registry = registry(&[
            ("collection", "1.0.0", &[("common", "^1.0.0")]),
            ("common", "1.0.0", &[]),
            ("common", "1.1.0", &[]),
        ]);
        assert_eq!(
            resolve(&registry, &[("collection", "=1.0.0"), ("common", "=1.0.0")]),
            Some(vec![
                "collection@1.0.0".to_string(),
                "common@1.0.0".to_string()
            ])
        );
    }

    /// When the conflict shows itself only after a version of another package has been chosen, the
    /// search returns to that choice and tries an older version.
    ///
    /// `a` has fewer versions than `b`, so it is chosen first, and its dependency `c` is chosen
    /// next at its newest version 2.0.0. Every version of `b` requires `c` 1.0.0, so the search has
    /// to go back to `c`.
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
            Some(vec![
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
            Some(vec!["x@1.0.0".to_string(), "y@1.0.0".to_string()])
        );
    }

    /// Requirements that no choice of versions satisfies are reported as unresolvable.
    #[test]
    fn unsatisfiable_requirements_are_not_resolved() {
        let registry = registry(&[
            ("a", "1.0.0", &[("c", "=1.0.0")]),
            ("b", "1.0.0", &[("c", "=2.0.0")]),
            ("c", "1.0.0", &[]),
            ("c", "2.0.0", &[]),
        ]);
        assert_eq!(resolve(&registry, &[("a", "*"), ("b", "*")]), None);
    }
}
