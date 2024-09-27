// This module impleents an algorithm of dependency resolution.

use crate::{error::Errors, project_file::ProjectFile};
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

pub fn resolve_dependency<'a, 'b, 'c>(
    root_proj: &ProjectFile,
    package_retriever: PackageRetriever<'a>,
    versions_retriever: VersionRetriever<'b>,
    logger: Logger<'c>,
) -> Result<Option<Vec<Package>>, Errors> {
    try_use_package(
        (&root_proj.general.name, &root_proj.general.version()),
        &[],
        package_retriever,
        versions_retriever,
        logger,
        0,
    )
}

// Try to use a package.
// It takes a NEW package `pkg` and other packages which are already version-fixed, and update `fixed` to satisfy the dependency, includeing the given `pkg`.
// If the dependency cannot be resolved, returns None.
// If retriever functions rise an error, returns the error.
fn try_use_package<'a, 'b, 'c>(
    pkg: (&PackageName, &Version),
    fixed: &[Package],
    package_retriever: PackageRetriever<'a>,
    versions_retriever: VersionRetriever<'b>,
    logger: Logger<'c>,
    indent: usize,
) -> Result<Option<Vec<Package>>, Errors> {
    let (pkg_name, pkg_version) = pkg;

    // `fixed` should not contain the given package.
    assert!(!fixed.iter().any(|p| p.name == *pkg_name));

    // Get the package information.
    let package = package_retriever(pkg_name, pkg_version)?;
    let deps = package.deps.clone();

    // Add the package to the fixed list.
    let mut fixed = fixed.to_vec();
    fixed.push(package);

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
        if let Some(res) = try_resolve_dependency(
            dep,
            &fixed,
            package_retriever,
            versions_retriever,
            logger,
            indent,
        )? {
            fixed = res;
        } else {
            ok = false;
            break;
        }
    }

    // If all dependecies are resolved, use this version.
    if ok {
        logger(&format!(
            "{}Accept \"{} = {}\".",
            " ".repeat(indent),
            pkg_name,
            pkg_version,
        ));
        return Ok(Some(fixed));
    }
    return Ok(None);
}

// Resolve a dependency.
// It takes a dependency and version-fixed packages, and update `fixed` to satisfy the dependency.
// If the dependency cannot be resolved, returns None.
// If retriever functions rise an error, returns the error.
fn try_resolve_dependency<'a, 'b, 'c>(
    dependency: &Dependency,
    fixed: &[Package],
    package_retriever: PackageRetriever<'a>,
    versions_retriever: VersionRetriever<'b>,
    logger: Logger<'c>,
    indent: usize,
) -> Result<Option<Vec<Package>>, Errors> {
    logger(&format!(
        "{}Resolving version requirement: \"{}@{}\".",
        " ".repeat(indent),
        dependency.name,
        dependency.requirement
    ));
    let indent = indent + 1;

    // In case the dependent package is already resolved,
    // if the version satisfies the requirement, nothing to do.
    // Otherwise, raise an error.
    if let Some(package) = fixed.iter().find(|p| p.name == dependency.name) {
        if dependency.requirement.matches(&package.version) {
            logger(&format!(
                "{}Already using \"{}@{}\", which satisfies the requirement `{}`. OK.",
                " ".repeat(indent),
                dependency.name,
                package.version,
                dependency.requirement
            ));
            return Ok(Some(fixed.to_vec()));
        } else {
            logger(&format!(
                "{}Already using \"{}@{}\", which does not satisfy the requirement `{}`. Backtracking.",
                " ".repeat(indent),
                dependency.name, package.version, dependency.requirement
            ));
            return Ok(None);
        }
    }

    // Find the latest version which can be used.
    let vers = versions_retriever(&dependency.name)?;
    let mut vers = vers
        .iter()
        .filter(|v| dependency.requirement.matches(v))
        .collect::<Vec<_>>();
    vers.sort();
    for version in vers.iter().rev() {
        logger(&format!(
            "{}Trying \"{}@{}\".",
            " ".repeat(indent),
            dependency.name,
            version,
        ));
        let indent = indent + 1;

        // Try to use this version.
        let fixed = try_use_package(
            (&dependency.name, version),
            fixed,
            package_retriever,
            versions_retriever,
            logger,
            indent,
        )?;
        if fixed.is_some() {
            return Ok(fixed);
        }

        // Otherwise, try the next version.
        logger(&format!(
            "{}Reject version `{}` of `{}`.",
            " ".repeat(indent),
            version,
            dependency.name,
        ));
    }
    // We have tried all versions, but none of them worked.
    logger(&format!(
        "{}No version of project `{}` was accepted. Backtracking.",
        " ".repeat(indent),
        dependency.name,
    ));
    Ok(None)
}
