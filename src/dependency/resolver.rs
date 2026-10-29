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
    DefaultStringReporter, Dependencies, DependencyConstraints, DependencyProvider, DerivationTree,
    Derived, External, PackageResolutionStatistics, PubGrubError, ReportFormatter, Reporter, Term,
    VersionSet,
};
use semver::{Version, VersionReq};
use std::{
    cell::{Cell, Ref, RefCell},
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
    path::PathBuf,
    sync::Arc,
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
    type P = PackageName;
    type V = SourcedVersion;
    type VS = Versions;
    /// Decides first the projects that took part in more conflicts, then those with fewer versions
    /// left to choose from.
    type Priority = (u32, Reverse<usize>);
    type M = Unavailability;
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
                requirements: BTreeMap::from([(version.clone(), retrieved.fix_version)]),
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

/// Why a version of a project is left out of the choice.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Unavailability {
    /// The version could not be read, for the reason given.
    Unreadable(String),
    /// The `fix_version` of each of the versions, none of which accepts `compiler`, the version of
    /// the running compiler.
    RejectsCompiler {
        requirements: BTreeMap<SourcedVersion, VersionReq>,
        compiler: Version,
    },
    /// The version requires `name` from `source`, which could not be read for the reason given.
    DependencyUnreadable {
        name: PackageName,
        source: Source,
        message: String,
    },
    /// The version requires `name` `requirement` from `source`, where `existing` are the versions,
    /// none of which satisfies it.
    NoVersionSatisfies {
        name: PackageName,
        requirement: VersionReq,
        source: Source,
        existing: Vec<Version>,
    },
}

/// Writes the reason as a sentence about the version, with its subject left out.
///
/// # Examples
/// `requires Fix ^1.7.0, and this is Fix 1.6.1`, or `requires "json" ^2.0.0 from
/// https://github.com/x/json.git, where the versions are 1.0.0 and 1.1.0`.
impl Display for Unavailability {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Unavailability::Unreadable(message) => write!(f, "could not be read: {}", message),
            Unavailability::RejectsCompiler {
                requirements,
                compiler,
            } => {
                let requirements = requirements
                    .iter()
                    .map(|(version, requirement)| {
                        if requirements.len() == 1 {
                            requirement.to_string()
                        } else {
                            format!("{} at {}", requirement, version.version)
                        }
                    })
                    .collect::<Vec<_>>();
                write!(
                    f,
                    "requires Fix {}, and this is Fix {}",
                    join_with("and", &requirements),
                    compiler
                )
            }
            Unavailability::DependencyUnreadable {
                name,
                source,
                message,
            } => write!(
                f,
                "requires \"{}\" from {}, which could not be read: {}",
                name, source, message
            ),
            Unavailability::NoVersionSatisfies {
                name,
                requirement,
                source,
                existing,
            } => {
                let existing = match existing.as_slice() {
                    [] => "no version exists".to_string(),
                    [only] => format!("the version is {}", only),
                    versions => format!(
                        "the versions are {}",
                        join_with(
                            "and",
                            &versions.iter().map(Version::to_string).collect::<Vec<_>>()
                        )
                    ),
                };
                write!(
                    f,
                    "requires {} from {}, where {}",
                    written_requirement(name, requirement),
                    source,
                    existing
                )
            }
        }
    }
}

/// The project `name` quoted, followed by `requirement` unless it accepts every version.
///
/// # Examples
/// `"json" ^1.0.0`, or `"json"` for `*`.
fn written_requirement(name: &PackageName, requirement: &VersionReq) -> String {
    if *requirement == VersionReq::STAR {
        format!("\"{}\"", name)
    } else {
        format!("\"{}\" {}", name, requirement)
    }
}

/// Joins `items` into one phrase, with `conjunction` before the last.
///
/// # Examples
/// `["1.0.0", "1.1.0", "1.2.0"]` joined with `or` is `1.0.0, 1.1.0 or 1.2.0`.
fn join_with(conjunction: &str, items: &[String]) -> String {
    match items {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} {} {}", rest.join(", "), conjunction, last),
    }
}

impl Unavailability {
    /// One reason for leaving out the versions left out for `self` and for `other`, if they are
    /// left out for one reason: the same one, or the rejection of the compiler, whose requirements
    /// are put together.
    fn merge(&self, other: &Unavailability) -> Option<Unavailability> {
        match (self, other) {
            (
                Unavailability::RejectsCompiler {
                    requirements: requirements1,
                    compiler,
                },
                Unavailability::RejectsCompiler {
                    requirements: requirements2,
                    ..
                },
            ) => {
                let mut requirements = requirements1.clone();
                requirements.extend(requirements2.clone());
                Some(Unavailability::RejectsCompiler {
                    requirements,
                    compiler: compiler.clone(),
                })
            }
            _ if self == other => Some(self.clone()),
            _ => None,
        }
    }
}

/// Merges, throughout `tree`, two versions of one project left out for one reason into one, so
/// that the explanation states the reason once for all of them.
fn merge_left_out_versions(tree: &mut DerivationTree<PackageName, Versions, Unavailability>) {
    let DerivationTree::Derived(derived) = tree else {
        return;
    };
    merge_left_out_versions(Arc::make_mut(&mut derived.cause1));
    merge_left_out_versions(Arc::make_mut(&mut derived.cause2));
    let (
        DerivationTree::External(External::Custom(name1, _, reason1)),
        DerivationTree::External(External::Custom(name2, _, reason2)),
    ) = (derived.cause1.as_ref(), derived.cause2.as_ref())
    else {
        return;
    };
    // The two causes leave out the versions of one project, and what they derive is that neither
    // version can be chosen.
    let [(name, Term::Positive(versions))] = derived.terms.iter().collect::<Vec<_>>()[..] else {
        return;
    };
    if name1 != name || name2 != name {
        return;
    }
    let Some(reason) = reason1.merge(reason2) else {
        return;
    };
    *tree = DerivationTree::External(External::Custom(name.clone(), versions.clone(), reason));
}

/// The projects whose sets in `tree` hold or leave out versions at more than one source.
fn projects_met_at_several_sources(
    tree: &DerivationTree<PackageName, Versions, Unavailability>,
) -> Set<PackageName> {
    let mut sources = Map::default();
    collect_sources(tree, &mut sources);
    sources
        .into_iter()
        .filter(|(_, sources)| sources.len() > 1)
        .map(|(name, _)| name)
        .collect()
}

/// Adds to `sources` the sources of the versions each project's sets in `tree` hold or leave out.
fn collect_sources(
    tree: &DerivationTree<PackageName, Versions, Unavailability>,
    sources: &mut Map<PackageName, Set<Source>>,
) {
    let mut add = |name: &PackageName, versions: &Versions| {
        sources
            .entry(name.clone())
            .or_default()
            .extend(versions.listed().iter().map(|v| v.source.clone()));
    };
    match tree {
        DerivationTree::External(External::NotRoot(_, _)) => {}
        DerivationTree::External(External::NoVersions(name, versions))
        | DerivationTree::External(External::Custom(name, versions, _)) => add(name, versions),
        DerivationTree::External(External::FromDependencyOf(name, versions, dep, dep_versions)) => {
            add(name, versions);
            add(dep, dep_versions);
        }
        DerivationTree::Derived(derived) => {
            for (name, term) in &derived.terms {
                match term {
                    Term::Positive(versions) | Term::Negative(versions) => add(name, versions),
                }
            }
            collect_sources(&derived.cause1, sources);
            collect_sources(&derived.cause2, sources);
        }
    }
}

/// Whether some version in `tree` is left out because its `fix_version` rejects the running
/// compiler.
fn rejects_the_compiler(tree: &DerivationTree<PackageName, Versions, Unavailability>) -> bool {
    match tree {
        DerivationTree::External(External::Custom(_, _, reason)) => {
            matches!(reason, Unavailability::RejectsCompiler { .. })
        }
        DerivationTree::External(_) => false,
        DerivationTree::Derived(derived) => {
            rejects_the_compiler(&derived.cause1) || rejects_the_compiler(&derived.cause2)
        }
    }
}

/// The message of a failed resolution: the explanation of `tree`, which derives that the
/// requirements of the root project conflict, preceded by the version of the running compiler when
/// a version is left out for rejecting it, and followed by a `HINT:` line for each kind of conflict
/// the user can act on.
///
/// # Examples
/// ```text
/// Failed to resolve dependencies.
/// Because "app" requires "collection" ^1.0.0, which requires "common" ^1.1.0, "app" indirectly requires "common" 1.1.0.
/// So, because "app" also requires "common" =1.0.0, the requirements of "app" cannot all be satisfied.
/// ```
fn explain(
    provider: &Provider<'_, '_>,
    tree: &DerivationTree<PackageName, Versions, Unavailability>,
) -> String {
    let explainer = Explainer {
        provider,
        met_at_several: projects_met_at_several_sources(tree),
        root_mentioned: Cell::new(false),
    };
    let mut lines = vec!["Failed to resolve dependencies.".to_string()];
    let rejects_the_compiler = rejects_the_compiler(tree);
    if rejects_the_compiler {
        lines.push(format!("This is Fix {}.", provider.compiler_version));
        lines.push(String::new());
    }
    lines.push(DefaultStringReporter::report_with_formatter(
        tree, &explainer,
    ));
    if rejects_the_compiler {
        lines.push(format!(
            "HINT: Use a newer Fix, or require versions that support Fix {}.",
            provider.compiler_version
        ));
    }
    if !explainer.met_at_several.is_empty() {
        lines.push("HINT: Every requirement on one project must name the same source.".to_string());
    }
    lines.join("\n")
}

/// Words the explanation of a failed resolution for the user: a project is named by its name in
/// quotes, a requirement as the project file writes it, and the root project without its version.
///
/// The explanation is a series of lines, each stating a fact the resolution met and what follows
/// from it: a fact is a requirement of a version, or a version left out, and what follows is that a
/// version cannot be used or that a project indirectly requires another. The last line concludes
/// that the requirements of the root project cannot all be satisfied.
struct Explainer<'e, 'a, 'b> {
    provider: &'e Provider<'a, 'b>,
    /// The projects the explanation meets at more than one source, whose versions it writes with
    /// their sources, which tell them apart.
    met_at_several: Set<PackageName>,
    /// Whether the explanation has written a requirement of the root project, so that the next one
    /// is written as a requirement it also has.
    root_mentioned: Cell<bool>,
}

impl Explainer<'_, '_, '_> {
    fn is_root(&self, name: &PackageName) -> bool {
        *name == self.provider.root_name
    }

    /// The versions of `name` the provider has listed that `versions` holds.
    fn known_in(&self, name: &PackageName, versions: &Versions) -> BTreeSet<SourcedVersion> {
        self.provider
            .known_versions_of(name)
            .iter()
            .filter(|v| versions.contains(v))
            .cloned()
            .collect()
    }

    /// Writes `versions`, some of the listed versions of `name`, as alternatives. For a project met
    /// at several sources, the versions of each source are followed by the source, and all the
    /// versions of a source are written as the source alone.
    ///
    /// # Examples
    /// `1.0.0 or 1.1.0`, and for a project met at several sources, `1.0.0 from A or from B`.
    fn list(&self, name: &PackageName, versions: &BTreeSet<SourcedVersion>) -> String {
        if !self.met_at_several.contains(name) {
            let versions = versions
                .iter()
                .map(|v| v.version.to_string())
                .collect::<Vec<_>>();
            return join_with("or", &versions);
        }
        let mut groups: BTreeMap<&Source, Vec<String>> = BTreeMap::new();
        for v in versions {
            groups
                .entry(&v.source)
                .or_default()
                .push(v.version.to_string());
        }
        let groups = groups
            .into_iter()
            .map(|(source, group)| {
                let listed_at_source = self.provider.versions_at(name, source).len();
                if group.len() == listed_at_source {
                    format!("from {}", source)
                } else {
                    format!("{} from {}", join_with("or", &group), source)
                }
            })
            .collect::<Vec<_>>();
        join_with("or", &groups)
    }

    /// `name` at `versions`, as the subject of a sentence.
    ///
    /// # Examples
    /// `"json" 1.0.0`, `"json" 1.0.0 or 1.1.0`, `every version of "json"`, or `"app"` for the root.
    fn subject(&self, name: &PackageName, versions: &Versions) -> String {
        if self.is_root(name) {
            return format!("\"{}\"", name);
        }
        let held = self.known_in(name, versions);
        let listed = self.provider.known_versions_of(name).len();
        if held.len() == listed && listed > 1 {
            format!("every version of \"{}\"", name)
        } else {
            format!("\"{}\" {}", name, self.list(name, &held))
        }
    }

    /// `name` at `versions`, as what a project requires.
    ///
    /// # Examples
    /// `"common" 1.1.0`, or `"common"` for every version.
    fn object(&self, name: &PackageName, versions: &Versions) -> String {
        let held = self.known_in(name, versions);
        let listed = self.provider.known_versions_of(name).len();
        if held.len() == listed && !self.met_at_several.contains(name) {
            format!("\"{}\"", name)
        } else {
            format!("\"{}\" {}", name, self.list(name, &held))
        }
    }

    /// What `depender` at `depender_versions` requires of `dep`: the requirement its project file
    /// writes, when every one of those versions writes the same one and it accepts exactly the
    /// listed versions in `dep_versions`, and those versions otherwise.
    fn required(
        &self,
        depender: &PackageName,
        depender_versions: &Versions,
        dep: &PackageName,
        dep_versions: &Versions,
    ) -> String {
        self.written_requirement_on(depender, depender_versions, dep, dep_versions)
            .unwrap_or_else(|| self.object(dep, dep_versions))
    }

    /// The requirement on `dep` that every version of `depender` in `depender_versions` writes, if
    /// they all write one and the same, and it accepts exactly the listed versions in
    /// `dep_versions`.
    fn written_requirement_on(
        &self,
        depender: &PackageName,
        depender_versions: &Versions,
        dep: &PackageName,
        dep_versions: &Versions,
    ) -> Option<String> {
        let dependers = if self.is_root(depender) {
            BTreeSet::from([self.provider.root_version.clone()])
        } else {
            self.known_in(depender, depender_versions)
        };
        let all_requirements = self.provider.requirements.borrow();
        let mut written: Option<&Dependency> = None;
        for version in &dependers {
            let requirement = all_requirements
                .get(&(depender.clone(), version.clone()))?
                .iter()
                .find(|requirement| requirement.name == *dep)?;
            match written {
                None => written = Some(requirement),
                Some(written) => {
                    if written.requirement != requirement.requirement
                        || written.source != requirement.source
                    {
                        return None;
                    }
                }
            }
        }
        let written = written?;
        let accepted = self
            .provider
            .known_versions_of(dep)
            .iter()
            .filter(|v| v.source == written.source && written.requirement.matches(&v.version))
            .cloned()
            .collect::<BTreeSet<_>>();
        if accepted != self.known_in(dep, dep_versions) {
            return None;
        }
        let requirement = written_requirement(dep, &written.requirement);
        Some(if self.met_at_several.contains(dep) {
            format!("{} from {}", requirement, written.source)
        } else {
            requirement
        })
    }

    /// The verb of a requirement `depender` has: `also requires` for a requirement of the root
    /// project after the explanation has written one.
    fn requires(&self, depender: &PackageName) -> &'static str {
        if self.is_root(depender) && self.root_mentioned.replace(true) {
            "also requires"
        } else {
            "requires"
        }
    }

    /// Whether `terms` says that the requirements of the root project cannot all be satisfied,
    /// which the last line of the explanation concludes.
    fn concludes(&self, terms: &pubgrub::Map<PackageName, Term<Versions>>) -> bool {
        terms.is_empty()
            || (terms.len() == 1
                && matches!(terms.get(&self.provider.root_name), Some(Term::Positive(_))))
    }

    /// The opening of a line that adds a fact to the lines before it: `So, because` on the last
    /// line, and `And because` on the others.
    fn and_because(&self, terms: &pubgrub::Map<PackageName, Term<Versions>>) -> &'static str {
        if self.concludes(terms) {
            "So, because"
        } else {
            "And because"
        }
    }

    /// Two facts, joined: a requirement of a project followed by one of the project it requires is
    /// written as one chain, and two requirements of one project as one requirement of both.
    /// Otherwise, a requirement of the root project comes first.
    fn two_facts(
        &self,
        fact1: &External<PackageName, Versions, Unavailability>,
        fact2: &External<PackageName, Versions, Unavailability>,
    ) -> String {
        use External::FromDependencyOf;
        if let (FromDependencyOf(p1, s1, d1, t1), FromDependencyOf(p2, s2, d2, t2)) = (fact1, fact2)
        {
            for ((p, s, q, t), (q2, s2, r, u)) in [
                ((p1, s1, d1, t1), (p2, s2, d2, t2)),
                ((p2, s2, d2, t2), (p1, s1, d1, t1)),
            ] {
                let required = self.known_in(q, t);
                if q == q2 && !required.is_empty() && required.is_subset(&self.known_in(q, s2)) {
                    return format!(
                        "{} {} {}, which requires {}",
                        self.subject(p, s),
                        self.requires(p),
                        self.required(p, s, q, t),
                        self.required(q, s2, r, u)
                    );
                }
            }
            if p1 == p2 && s1 == s2 {
                return format!(
                    "{} {} {} and {}",
                    self.subject(p1, s1),
                    self.requires(p1),
                    self.required(p1, s1, d1, t1),
                    self.required(p2, s2, d2, t2)
                );
            }
        }
        let is_root_requirement = |fact: &External<_, _, _>| matches!(fact, FromDependencyOf(p, _, _, _) if self.is_root(p));
        let (first, second) = if is_root_requirement(fact2) && !is_root_requirement(fact1) {
            (fact2, fact1)
        } else {
            (fact1, fact2)
        };
        let first = self.format_external(first);
        format!("{} and {}", first, self.format_external(second))
    }

    /// The sentence saying that `name` at `versions` is left out for `reason`.
    fn unavailable(
        &self,
        name: &PackageName,
        versions: &Versions,
        reason: &Unavailability,
    ) -> String {
        let subject = self.subject(name, versions);
        match reason {
            Unavailability::RejectsCompiler { requirements, .. } if requirements.len() == 1 => {
                let requirement = requirements.values().next().unwrap();
                format!("{} requires Fix {}", subject, requirement)
            }
            Unavailability::RejectsCompiler {
                requirements,
                compiler,
            } => {
                let each = requirements
                    .iter()
                    .map(|(version, requirement)| {
                        format!("{} requires Fix {}", version.version, requirement)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                if self.known_in(name, versions).len()
                    == self.provider.known_versions_of(name).len()
                {
                    format!(
                        "no version of \"{}\" supports Fix {} ({})",
                        name, compiler, each
                    )
                } else {
                    format!("{} does not support Fix {} ({})", subject, compiler, each)
                }
            }
            Unavailability::NoVersionSatisfies {
                name: dep,
                requirement,
                source,
                existing,
            } => {
                let existing = if existing.is_empty() {
                    format!("has no version of \"{}\"", dep)
                } else {
                    let existing = existing.iter().map(Version::to_string).collect::<Vec<_>>();
                    format!("has only \"{}\" {}", dep, join_with("and", &existing))
                };
                format!(
                    "{} requires {}, which no version satisfies ({} {})",
                    subject,
                    written_requirement(dep, requirement),
                    source,
                    existing
                )
            }
            Unavailability::Unreadable(_) | Unavailability::DependencyUnreadable { .. } => {
                format!("{} {}", subject, reason)
            }
        }
    }
}

impl ReportFormatter<PackageName, Versions, Unavailability> for Explainer<'_, '_, '_> {
    type Output = String;

    fn format_external(
        &self,
        external: &External<PackageName, Versions, Unavailability>,
    ) -> String {
        match external {
            External::NotRoot(name, _) => {
                format!("the dependencies of \"{}\" are being resolved", name)
            }
            External::NoVersions(name, versions) => {
                format!("no version of \"{}\" in {} exists", name, versions)
            }
            External::FromDependencyOf(depender, depender_versions, dep, dep_versions) => {
                let subject = self.subject(depender, depender_versions);
                format!(
                    "{} {} {}",
                    subject,
                    self.requires(depender),
                    self.required(depender, depender_versions, dep, dep_versions)
                )
            }
            External::Custom(name, versions, reason) => self.unavailable(name, versions, reason),
        }
    }

    /// Writes what an incompatibility derives, which is that its terms cannot all hold: the versions
    /// of a positive term cannot be used, and the projects of the positive terms require those of
    /// the negative ones.
    fn format_terms(&self, terms: &pubgrub::Map<PackageName, Term<Versions>>) -> String {
        if self.concludes(terms) {
            return format!(
                "the requirements of \"{}\" cannot all be satisfied",
                self.provider.root_name
            );
        }
        let mut terms = terms.iter().collect::<Vec<_>>();
        terms.sort_by_key(|(name, _)| (!self.is_root(name), name.to_string()));
        let used = terms
            .iter()
            .filter_map(|(name, term)| match term {
                Term::Positive(versions) => Some((*name, versions)),
                Term::Negative(_) => None,
            })
            .collect::<Vec<_>>();
        let required = terms
            .iter()
            .filter_map(|(name, term)| match term {
                Term::Negative(versions) => Some(self.object(name, versions)),
                Term::Positive(_) => None,
            })
            .collect::<Vec<_>>();
        let root_used = used.iter().any(|(name, _)| self.is_root(name));
        // A version used alone is the subject of the sentence, and several used together are what
        // the sentence is about.
        if let ([(name, versions)], true) = (&used[..], required.is_empty()) {
            let held = self.known_in(name, versions).len();
            let listed = self.provider.known_versions_of(name).len();
            if held == listed && listed > 1 {
                return format!("no version of \"{}\" can be used", name);
            }
        }
        let used = if let [(name, versions)] = used[..] {
            vec![self.subject(name, versions)]
        } else {
            used.iter()
                .map(|(name, versions)| self.object(name, versions))
                .collect()
        };
        match (used.len(), required.is_empty()) {
            (0, _) => format!("{} must be used", join_with("or", &required)),
            (1, true) => format!("{} cannot be used", used[0]),
            (_, true) if root_used => {
                format!("{} cannot use {}", used[0], join_with("and", &used[1..]))
            }
            (_, true) => format!("{} cannot be used together", join_with("and", &used)),
            (count, false) => {
                if root_used {
                    self.root_mentioned.set(true);
                }
                let verb = if count == 1 {
                    "indirectly requires"
                } else {
                    "together indirectly require"
                };
                format!(
                    "{} {} {}",
                    join_with("and", &used),
                    verb,
                    join_with("and", &required)
                )
            }
        }
    }

    fn explain_both_external(
        &self,
        external1: &External<PackageName, Versions, Unavailability>,
        external2: &External<PackageName, Versions, Unavailability>,
        current_terms: &pubgrub::Map<PackageName, Term<Versions>>,
    ) -> String {
        let facts = self.two_facts(external1, external2);
        format!("Because {}, {}.", facts, self.format_terms(current_terms))
    }

    fn explain_both_ref(
        &self,
        ref_id1: usize,
        derived1: &Derived<PackageName, Versions, Unavailability>,
        ref_id2: usize,
        derived2: &Derived<PackageName, Versions, Unavailability>,
        current_terms: &pubgrub::Map<PackageName, Term<Versions>>,
    ) -> String {
        let fact1 = self.format_terms(&derived1.terms);
        let fact2 = self.format_terms(&derived2.terms);
        format!(
            "Because {} ({}) and {} ({}), {}.",
            fact1,
            ref_id1,
            fact2,
            ref_id2,
            self.format_terms(current_terms)
        )
    }

    fn explain_ref_and_external(
        &self,
        ref_id: usize,
        derived: &Derived<PackageName, Versions, Unavailability>,
        external: &External<PackageName, Versions, Unavailability>,
        current_terms: &pubgrub::Map<PackageName, Term<Versions>>,
    ) -> String {
        let fact1 = self.format_terms(&derived.terms);
        let fact2 = self.format_external(external);
        format!(
            "Because {} ({}) and {}, {}.",
            fact1,
            ref_id,
            fact2,
            self.format_terms(current_terms)
        )
    }

    fn and_explain_external(
        &self,
        external: &External<PackageName, Versions, Unavailability>,
        current_terms: &pubgrub::Map<PackageName, Term<Versions>>,
    ) -> String {
        let fact = self.format_external(external);
        format!(
            "{} {}, {}.",
            self.and_because(current_terms),
            fact,
            self.format_terms(current_terms)
        )
    }

    fn and_explain_ref(
        &self,
        ref_id: usize,
        derived: &Derived<PackageName, Versions, Unavailability>,
        current_terms: &pubgrub::Map<PackageName, Term<Versions>>,
    ) -> String {
        let fact = self.format_terms(&derived.terms);
        format!(
            "{} {} ({}), {}.",
            self.and_because(current_terms),
            fact,
            ref_id,
            self.format_terms(current_terms)
        )
    }

    fn and_explain_prior_and_external(
        &self,
        prior_external: &External<PackageName, Versions, Unavailability>,
        external: &External<PackageName, Versions, Unavailability>,
        current_terms: &pubgrub::Map<PackageName, Term<Versions>>,
    ) -> String {
        let facts = self.two_facts(prior_external, external);
        format!(
            "{} {}, {}.",
            self.and_because(current_terms),
            facts,
            self.format_terms(current_terms)
        )
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
             cannot all be satisfied."
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
             \"root\" cannot all be satisfied."
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
             So, because \"root\" requires \"foo\" ^1.0.0, the requirements of \"root\" cannot all \
             be satisfied."
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
             So, because \"root\" requires \"web\" ^1.0.0, the requirements of \"root\" cannot all be \
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
             So, because \"root\" also requires \"x\", the requirements of \"root\" cannot all be \
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
            message.contains("the project file is missing") && !message.contains("unavailable"),
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
             1.1.0), the requirements of \"root\" cannot all be satisfied."
        );
    }

    /// A requirement the root writes that no existing version satisfies stops the resolution with
    /// the requirement and the versions that do exist.
    #[test]
    fn a_requirement_of_the_root_no_version_satisfies_is_reported() {
        let registry = registry(&[("y", "1.0.0", &[]), ("y", "1.1.0", &[])]);
        assert_eq!(
            resolve(&registry, &[("y", "^2.0.0")]).unwrap_err(),
            "error: \"root\" requires \"y\" ^2.0.0 from https://example.com/y.git, where the \
             versions are 1.0.0 and 1.1.0."
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
}
