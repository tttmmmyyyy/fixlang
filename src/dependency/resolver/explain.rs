//! The explanation of a failed resolution, written for the user in the words of the project
//! files.

use super::{group_by_source, Dependency, PackageName, Provider, Source, SourcedVersion, Versions};
use crate::misc::{Map, Set};
use pubgrub::{
    DefaultStringReporter, DerivationTree, Derived, External, ReportFormatter, Reporter, Term,
    VersionSet,
};
use semver::{Version, VersionReq};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
    sync::Arc,
};

/// Why a version of a project is left out of the choice.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Unavailability {
    /// The version could not be read, for the reason given.
    Unreadable(String),
    /// The `fix_version` of each version rejects the running compiler.
    RejectsCompiler {
        /// The `fix_version` of each version left out.
        fix_versions: BTreeMap<SourcedVersion, VersionReq>,
        /// The version of the running compiler.
        compiler: Version,
    },
    /// The version requires a project whose versions could not be read.
    DependencyUnreadable {
        /// The project required.
        name: PackageName,
        /// The source the version requires `name` from.
        source: Source,
        /// Why the versions of `name` could not be read.
        message: String,
    },
    /// The version has a requirement that no version at its source satisfies.
    NoVersionSatisfies {
        /// The project required.
        name: PackageName,
        /// The requirement on `name`, as the project file writes it.
        requirement: VersionReq,
        /// The source the version requires `name` from.
        source: Source,
        /// The versions of `name` at `source`.
        existing: Vec<Version>,
    },
}

/// Writes the reason as a sentence about the versions left out, with its subject left out.
///
/// # Examples
/// `requires Fix ^1.7.0, and this is Fix 1.6.1`, or `requires "json" ^2.0.0, which no version
/// satisfies (https://github.com/x/json.git has only "json" 1.0.0 and 1.1.0)`.
impl Display for Unavailability {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Unavailability::Unreadable(message) => write!(f, "could not be read ({})", message),
            Unavailability::RejectsCompiler {
                fix_versions,
                compiler,
            } => match fix_versions.values().collect::<Vec<_>>()[..] {
                [fix_version] => write!(
                    f,
                    "requires Fix {}, and this is Fix {}",
                    fix_version, compiler
                ),
                _ => write!(
                    f,
                    "does not support Fix {} ({})",
                    compiler,
                    self.fix_version_of_each()
                ),
            },
            Unavailability::DependencyUnreadable {
                name,
                source,
                message,
            } => write!(
                f,
                "requires \"{}\" from {}, which could not be read ({})",
                name, source, message
            ),
            Unavailability::NoVersionSatisfies {
                name,
                requirement,
                source,
                existing,
            } => {
                let existing = if existing.is_empty() {
                    format!("has no version of \"{}\"", name)
                } else {
                    let existing = existing.iter().map(Version::to_string).collect::<Vec<_>>();
                    format!("has only \"{}\" {}", name, join_with("and", &existing))
                };
                write!(
                    f,
                    "requires {}, which no version satisfies ({} {})",
                    written_requirement(name, requirement),
                    source,
                    existing
                )
            }
        }
    }
}

impl Unavailability {
    /// The `fix_version` of each version a rejection of the compiler leaves out.
    ///
    /// # Examples
    /// `1.0.0 requires Fix ^1.7.0, 1.1.0 requires Fix ^1.8.0`.
    fn fix_version_of_each(&self) -> String {
        let Unavailability::RejectsCompiler { fix_versions, .. } = self else {
            unreachable!("not a rejection of the compiler: {:?}", self)
        };
        fix_versions
            .iter()
            .map(|(version, fix_version)| {
                format!("{} requires Fix {}", version.version, fix_version)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Combines `self` and `other` into one reason that covers the versions left out for either.
    /// Two equal reasons give that reason, and two rejections of the compiler give one rejection
    /// with the `fix_version`s of both. Any other pair gives `None`.
    fn merge(&self, other: &Unavailability) -> Option<Unavailability> {
        match (self, other) {
            (
                Unavailability::RejectsCompiler {
                    fix_versions: fix_versions1,
                    compiler,
                },
                Unavailability::RejectsCompiler {
                    fix_versions: fix_versions2,
                    compiler: compiler2,
                },
            ) => {
                assert_eq!(compiler, compiler2, "two compilers in one resolution");
                let mut fix_versions = fix_versions1.clone();
                fix_versions.extend(fix_versions2.clone());
                Some(Unavailability::RejectsCompiler {
                    fix_versions,
                    compiler: compiler.clone(),
                })
            }
            _ if self == other => Some(self.clone()),
            _ => None,
        }
    }
}

/// The project `name` quoted, followed by `requirement`.
///
/// # Examples
/// `"json" ^1.0.0`.
fn written_requirement(name: &PackageName, requirement: &VersionReq) -> String {
    format!("\"{}\" {}", name, requirement)
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

/// Throughout `tree`, replaces each derivation from two sets of versions of one project, left out
/// for reasons `Unavailability::merge` combines, with one fact that leaves out both sets. The
/// explanation then states the reason once for all of them.
pub(super) fn merge_left_out_versions(
    tree: &mut DerivationTree<PackageName, Versions, Unavailability>,
) {
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

/// Whether `tree` holds two requirements on one project that name different sources and can be
/// in effect together: requirements of two projects, or of versions one project can be at.
fn requires_one_project_from_two_sources(
    tree: &DerivationTree<PackageName, Versions, Unavailability>,
) -> bool {
    let mut requirements = vec![];
    collect_requirements(tree, &mut requirements);
    let sources = |versions: &Versions| {
        versions
            .listed()
            .iter()
            .map(|v| v.source.clone())
            .collect::<BTreeSet<_>>()
    };
    requirements
        .iter()
        .enumerate()
        .any(|(i, (p1, s1, d1, t1))| {
            requirements[..i].iter().any(|(p2, s2, d2, t2)| {
                d1 == d2
                    && sources(t1).is_disjoint(&sources(t2))
                    && (p1 != p2 || s1.intersection(s2) != Versions::empty())
            })
        })
}

/// Adds to `requirements` every requirement `tree` holds, as the depender, its versions, the
/// project required and the versions required.
fn collect_requirements<'t>(
    tree: &'t DerivationTree<PackageName, Versions, Unavailability>,
    requirements: &mut Vec<(&'t PackageName, &'t Versions, &'t PackageName, &'t Versions)>,
) {
    match tree {
        DerivationTree::External(External::FromDependencyOf(p, s, d, t)) => {
            requirements.push((p, s, d, t))
        }
        DerivationTree::External(_) => {}
        DerivationTree::Derived(derived) => {
            collect_requirements(&derived.cause1, requirements);
            collect_requirements(&derived.cause2, requirements);
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
pub(super) fn explain(
    provider: &Provider<'_, '_>,
    tree: &DerivationTree<PackageName, Versions, Unavailability>,
) -> String {
    let explainer = Explainer {
        provider,
        met_at_several: projects_met_at_several_sources(tree),
        root_requirement_written: Cell::new(false),
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
    if requires_one_project_from_two_sources(tree) {
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
    /// The provider that ran the resolution, which holds the versions it listed and their
    /// requirements.
    provider: &'e Provider<'a, 'b>,
    /// The projects the explanation meets at more than one source, whose versions it writes with
    /// their sources, which tell them apart.
    met_at_several: Set<PackageName>,
    /// Whether the explanation has written a requirement of the root project, so that the next one
    /// is written as a requirement it also has. The reporter of pubgrub asks for the lines in the
    /// order it writes them, and each line is built from its first word to its last.
    root_requirement_written: Cell<bool>,
}

impl Explainer<'_, '_, '_> {
    /// Whether `name` is the project being resolved.
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

    /// Whether `versions` holds every version of `name` the provider has listed.
    fn holds_every_version(&self, name: &PackageName, versions: &Versions) -> bool {
        self.known_in(name, versions).len() == self.provider.known_versions_of(name).len()
    }

    /// Whether `versions` holds every version of `name` the provider has listed, of which there
    /// are several.
    fn holds_every_one_of_several(&self, name: &PackageName, versions: &Versions) -> bool {
        self.provider.known_versions_of(name).len() > 1 && self.holds_every_version(name, versions)
    }

    /// Writes `versions`, some of the listed versions of `name`, as alternatives. For a project met
    /// at several sources, the versions of each source are followed by the source, and all the
    /// versions of a source are written as the source alone.
    ///
    /// # Examples
    /// `1.0.0 or 1.1.0`, and for a project met at several sources, `1.0.0 from A or from B`.
    fn alternatives(&self, name: &PackageName, versions: &BTreeSet<SourcedVersion>) -> String {
        if !self.met_at_several.contains(name) {
            let versions = versions
                .iter()
                .map(|v| v.version.to_string())
                .collect::<Vec<_>>();
            return join_with("or", &versions);
        }
        let groups = group_by_source(versions)
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
        if self.holds_every_one_of_several(name, versions) {
            format!("every version of \"{}\"", name)
        } else {
            format!(
                "\"{}\" {}",
                name,
                self.alternatives(name, &self.known_in(name, versions))
            )
        }
    }

    /// `name` at `versions`, as what a project requires.
    ///
    /// # Examples
    /// `"common" 1.1.0`, or `"common"` for every version.
    fn object(&self, name: &PackageName, versions: &Versions) -> String {
        if self.holds_every_version(name, versions) && !self.met_at_several.contains(name) {
            format!("\"{}\"", name)
        } else {
            format!(
                "\"{}\" {}",
                name,
                self.alternatives(name, &self.known_in(name, versions))
            )
        }
    }

    /// What `depender` at `depender_versions` requires of `dep`. When all those versions write
    /// the same requirement on `dep`, and it accepts exactly the listed versions in
    /// `dep_versions`, this is that requirement. Otherwise it is the versions in `dep_versions`.
    ///
    /// # Examples
    /// `"common" ^1.1.0` as the project file writes it, or `"common" 1.1.0`.
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

    /// The requirement on `dep` that every version of `depender` in `depender_versions` writes.
    /// Gives it only when all those versions write the same one and it accepts exactly the listed
    /// versions in `dep_versions`.
    fn written_requirement_on(
        &self,
        depender: &PackageName,
        depender_versions: &Versions,
        dep: &PackageName,
        dep_versions: &Versions,
    ) -> Option<String> {
        let listed_depender_versions = if self.is_root(depender) {
            BTreeSet::from([self.provider.root_version.clone()])
        } else {
            self.known_in(depender, depender_versions)
        };
        let all_requirements = self.provider.requirements.borrow();
        let mut written: Option<&Dependency> = None;
        for version in &listed_depender_versions {
            let dependency = all_requirements
                .get(&(depender.clone(), version.clone()))?
                .iter()
                .find(|dependency| dependency.name == *dep)?;
            match written {
                None => written = Some(dependency),
                Some(written) => {
                    if written.requirement != dependency.requirement
                        || written.source != dependency.source
                    {
                        return None;
                    }
                }
            }
        }
        let written = written?;
        let accepted = self.provider.listed_versions_accepted_by(written);
        if accepted != self.known_in(dep, dep_versions) {
            return None;
        }
        // `*` leaves out pre-releases, so it is written out unless it accepts every version.
        let requirement = if written.requirement == VersionReq::STAR
            && accepted.len() == self.provider.versions_at(dep, &written.source).len()
        {
            format!("\"{}\"", dep)
        } else {
            written_requirement(dep, &written.requirement)
        };
        Some(if self.met_at_several.contains(dep) {
            format!("{} from {}", requirement, written.source)
        } else {
            requirement
        })
    }

    /// The verb of a requirement `depender` has, which the explanation is about to write:
    /// `also requires` for a requirement of the root project after the explanation has written one.
    fn requirement_verb(&self, depender: &PackageName) -> &'static str {
        if self.is_root(depender) && self.root_requirement_written.replace(true) {
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
            for ((p, s, q, t), (q2, q2_versions, r, u)) in [
                ((p1, s1, d1, t1), (p2, s2, d2, t2)),
                ((p2, s2, d2, t2), (p1, s1, d1, t1)),
            ] {
                let required = self.known_in(q, t);
                if q == q2
                    && !required.is_empty()
                    && required.is_subset(&self.known_in(q, q2_versions))
                {
                    return format!(
                        "{} {} {}, which requires {}",
                        self.subject(p, s),
                        self.requirement_verb(p),
                        self.required(p, s, q, t),
                        self.required(q, q2_versions, r, u)
                    );
                }
            }
            if p1 == p2 && s1 == s2 {
                return format!(
                    "{} {} {} and {}",
                    self.subject(p1, s1),
                    self.requirement_verb(p1),
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

    /// The sentence saying that `name` at `versions` is left out for `reason`. A rejection of the
    /// compiler by one version gives its `fix_version` alone, since the line before the explanation
    /// gives the version of the compiler. A rejection by every version of `name` is written as one
    /// sentence about the project. Every other reason is written as its `Display` writes it.
    ///
    /// # Examples
    /// `"json" 1.1.0 requires Fix ^1.8.0`, or `no version of "json" supports Fix 1.6.1 (1.0.0
    /// requires Fix ^1.7.0, 1.1.0 requires Fix ^1.8.0)`.
    fn unavailability_sentence(
        &self,
        name: &PackageName,
        versions: &Versions,
        reason: &Unavailability,
    ) -> String {
        match reason {
            Unavailability::RejectsCompiler { fix_versions, .. } if fix_versions.len() == 1 => {
                let fix_version = fix_versions.values().next().unwrap();
                format!(
                    "{} requires Fix {}",
                    self.subject(name, versions),
                    fix_version
                )
            }
            Unavailability::RejectsCompiler { compiler, .. }
                if self.holds_every_version(name, versions) =>
            {
                format!(
                    "no version of \"{}\" supports Fix {} ({})",
                    name,
                    compiler,
                    reason.fix_version_of_each()
                )
            }
            _ => format!("{} {}", self.subject(name, versions), reason),
        }
    }
}

impl ReportFormatter<PackageName, Versions, Unavailability> for Explainer<'_, '_, '_> {
    type Output = String;

    /// Writes one fact: a requirement as its project file writes it, a version left out with its
    /// reason, the resolution of the root, or a project with no version to choose.
    fn format_external(
        &self,
        external: &External<PackageName, Versions, Unavailability>,
    ) -> String {
        match external {
            External::NotRoot(name, _) => {
                format!("the dependencies of \"{}\" are being resolved", name)
            }
            External::NoVersions(name, _) => {
                format!("no version of \"{}\" can be chosen", name)
            }
            External::FromDependencyOf(depender, depender_versions, dep, dep_versions) => {
                let subject = self.subject(depender, depender_versions);
                format!(
                    "{} {} {}",
                    subject,
                    self.requirement_verb(depender),
                    self.required(depender, depender_versions, dep, dep_versions)
                )
            }
            External::Custom(name, versions, reason) => {
                self.unavailability_sentence(name, versions, reason)
            }
        }
    }

    /// Writes what an incompatibility derives, which is that its terms cannot all hold: the
    /// versions of a positive term cannot be used, and the projects of the positive terms require
    /// those of the negative ones.
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
        if let ([(name, versions)], true) = (&used[..], required.is_empty()) {
            if self.holds_every_one_of_several(name, versions) {
                return format!("no version of \"{}\" can be used", name);
            }
        }
        // A single project used is the subject of the sentence. Several projects used are joined as
        // the objects of `cannot be used together` or `together indirectly require`.
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
                let together = if used.len() > 2 { " together" } else { "" };
                format!(
                    "{} cannot use {}{}",
                    used[0],
                    join_with("and", &used[1..]),
                    together
                )
            }
            (_, true) => format!("{} cannot be used together", join_with("and", &used)),
            (count, false) => {
                if root_used {
                    self.root_requirement_written.set(true);
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
                    join_with("or", &required)
                )
            }
        }
    }

    /// Writes `Because <fact> and <fact>, <conclusion>.`, where two requirements along one chain
    /// are written as the chain.
    fn explain_both_external(
        &self,
        external1: &External<PackageName, Versions, Unavailability>,
        external2: &External<PackageName, Versions, Unavailability>,
        current_terms: &pubgrub::Map<PackageName, Term<Versions>>,
    ) -> String {
        let facts = self.two_facts(external1, external2);
        format!("Because {}, {}.", facts, self.format_terms(current_terms))
    }

    /// Writes `Because <conclusion> (<n>) and <conclusion> (<m>), <conclusion>.`, citing two
    /// earlier lines by their numbers.
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

    /// Writes `Because <conclusion> (<n>) and <fact>, <conclusion>.`, citing an earlier line by its
    /// number.
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

    /// Writes `And because <fact>, <conclusion>.`, or `So, because` on the last line.
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

    /// Writes `And because <conclusion> (<n>), <conclusion>.`, or `So, because` on the last line,
    /// citing an earlier line by its number.
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

    /// Writes `And because <fact> and <fact>, <conclusion>.`, or `So, because` on the last line,
    /// where two requirements along one chain are written as the chain.
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
