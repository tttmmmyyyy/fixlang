// This module impleents an algorithm of dependency resolution.

use crate::error::Errors;
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
pub type PackageRetriever<'a> = &'a dyn Fn(&PackageName, &Version) -> Result<Package, Errors>;

// Version retriever function.
// It takes a package name and returns a list of versions which exist.
pub type VersionRetriever<'a> = &'a dyn Fn(&PackageName) -> Result<Vec<Version>, Errors>;

// Logger function.
pub type Logger<'a> = &'a dyn Fn(&str);

// Resolve a dependency.
// It takes a dependency and version-fixed packages, and returns a list of packages necessary to satisfy the dependency, includeing the given `root_package`.
// If the dependency cannot be resolved, returns None.
// If retriever functions rise an error, returns the error.
pub fn resolve_dependency<'a, 'b>(
    root_package: &Package,
    package_retriever: PackageRetriever<'a>,
    versions_retriever: VersionRetriever<'a>,
    logger: Logger<'b>,
) -> Result<Option<Vec<Package>>, Errors> {
    let mut pkgs = vec![root_package.clone()];
    for dep in &root_package.deps {
        if let Some(fixed) =
            resolve_dependency_inner(dep, &pkgs, package_retriever, versions_retriever, logger, 0)?
        {
            pkgs = fixed;
        } else {
            return Ok(None);
        }
    }
    Ok(Some(pkgs))
}

// Resolve a dependency.
// It takes a dependency and version-fixed packages, and returns a list of packages necessary to satisfy the dependency, includeing the given `fixed`.
// If the dependency cannot be resolved, returns None.
// If retriever functions rise an error, returns the error.
fn resolve_dependency_inner<'a, 'b>(
    dependency: &Dependency,
    fixed: &[Package],
    package_retriever: PackageRetriever<'a>,
    versions_retriever: VersionRetriever<'a>,
    logger: Logger<'b>,
    indent: usize,
) -> Result<Option<Vec<Package>>, Errors> {
    logger(&format!(
        "{}Resolving version requirement: `{} = {}`.",
        " ".repeat(indent),
        dependency.name,
        dependency.requirement
    ));
    let indent = indent + 1;

    // In case the dependent package is already resolved,
    // if the version satisfies the requirement, return an empty list.
    // Otherwise, raise an error.
    if let Some(package) = fixed.iter().find(|p| p.name == dependency.name) {
        if dependency.requirement.matches(&package.version) {
            logger(&format!(
                "{}Already using version `{}` of package `{}`, which satisfies the requirement `{}`. OK.",
                " ".repeat(indent),
                package.version,
                dependency.name,
                dependency.requirement
            ));
            return Ok(Some(fixed.to_vec()));
        } else {
            logger(&format!(
                "{}Already using version `{}` of `{}`, which does not satisfy the requirement `{}`. Backtracking.",
                " ".repeat(indent),
                package.version, dependency.name, dependency.requirement
            ));
            return Ok(None);
        }
    }

    // Find the latest version which can be used.
    let mut vers = versions_retriever(&dependency.name)?;
    vers.sort();
    for version in vers.iter().rev() {
        logger(&format!(
            "{}Trying version `{}` of package `{}`.",
            " ".repeat(indent),
            version,
            dependency.name
        ));
        let indent = indent + 1;

        // Try to use this version. Get the package information.
        let mut resolved = fixed.to_vec();
        let package = package_retriever(&dependency.name, version)?;
        let deps = package.deps.clone();
        resolved.push(package);

        // Try to resolve dependencies for this package.
        // We try first dependencies with fewer possible versions.
        let mut dep_range: Vec<(Dependency, usize)> = vec![]; // Pairs of Dependency and number of possible versions for the package.
        for dep in deps.iter() {
            let vers = versions_retriever(&dep.name)?;
            let count = vers.iter().filter(|v| dep.requirement.matches(v)).count();
            dep_range.push((dep.clone(), count));
        }
        dep_range.sort_by_key(|(_, count)| *count);
        let deps = dep_range.iter().map(|(dep, _)| dep).collect::<Vec<_>>();
        let mut ok = true;
        for dep in deps {
            if let Some(res) = resolve_dependency_inner(
                dep,
                &resolved,
                package_retriever,
                versions_retriever,
                logger,
                indent,
            )? {
                resolved = res;
            } else {
                ok = false;
                break;
            }
        }

        // If all dependecies are resolved, use this version.
        if ok {
            logger(&format!(
                "{}Accept version `{}` of package `{}`.",
                " ".repeat(indent),
                version,
                dependency.name,
            ));
            return Ok(Some(resolved));
        }
        // Otherwise, try the next version.
        logger(&format!(
            "{}Reject version `{}` of package `{}`.",
            " ".repeat(indent),
            version,
            dependency.name,
        ));
    }
    // We have tried all versions, but none of them worked.
    logger(&format!(
        "{}No version of package `{}` was accepted. Backtracking.",
        " ".repeat(indent),
        dependency.name,
    ));
    Ok(None)
}
