// The text a dump of the lambda sets of a program reads as.

use super::{
    concrete::{Elem, SetId, SetStore},
    infer::{slot_roots, Analysis},
    slots::SlotLayout,
    solver::{Member, Owner, SetVar, Solver},
    specialize::Specialization,
};
use crate::misc::{Map, Set};
use std::fmt::Write;

/// The text of a dump of `analysis` and `spec`: the scheme of each symbol, with what each of its
/// lambda sets holds and why one is Dynamic, then the copies the program asks for, each symbol's
/// type annotated with the concrete sets of the copy, and the sets those name.
pub fn render(analysis: &mut Analysis, spec: &Specialization, layout: &SlotLayout) -> String {
    let mut out = String::new();
    let lambda_names: Vec<String> = analysis.lambdas.iter().map(|l| l.to_string()).collect();

    writeln!(out, "# Schemes").unwrap();
    writeln!(out).unwrap();
    for component in &analysis.components {
        if component.symbols.is_empty() {
            continue;
        }
        let solver = &mut analysis.solver;
        // The parameters are named `p<i>` and the pinned variables of the types `q<i>`.
        let mut var_names: Map<SetVar, String> = Map::default();
        for (i, p) in component.params.iter().enumerate() {
            var_names.insert(*p, format!("p{}", i));
        }
        let pinned: Vec<SetVar> = slot_roots(solver, &component.types)
            .into_iter()
            .filter(|v| !var_names.contains_key(v))
            .collect();
        for (i, v) in pinned.iter().enumerate() {
            var_names.insert(*v, format!("q{}", i));
        }
        for (s, ty) in component.symbols.iter().zip(&component.types) {
            let text = layout.annotate(ty, &mut |v| var_names[&solver.find(v)].clone());
            writeln!(out, "{} : {}", s.to_string(), text).unwrap();
        }
        for v in component.params.iter().chain(&pinned) {
            let members = describe_members(solver, *v, &var_names, &lambda_names);
            writeln!(out, "  {} = {{{}}}", var_names[v], members).unwrap();
        }
        writeln!(out).unwrap();
    }

    writeln!(out, "# Copies").unwrap();
    writeln!(out).unwrap();
    let mut named_sets: Vec<SetId> = vec![];
    for (k, instance) in spec.instances.iter().enumerate() {
        let component = &analysis.components[instance.component];
        if component.symbols.is_empty() {
            writeln!(out, "copy {}: the entry and the exported values", k).unwrap();
            continue;
        }
        let solver = &mut analysis.solver;
        for (s, ty) in component.symbols.iter().zip(&component.types) {
            let text = layout.annotate(ty, &mut |v| {
                let id = instance.set_of(component, solver.find(v));
                named_sets.push(id);
                format!("#{}", id)
            });
            writeln!(out, "copy {}: {} : {}", k, s.to_string(), text).unwrap();
        }
    }
    writeln!(out).unwrap();

    writeln!(out, "# Sets").unwrap();
    writeln!(out).unwrap();
    let mut shown: Set<SetId> = Set::default();
    let mut stack: Vec<SetId> = named_sets;
    let mut lines: Vec<(SetId, String)> = vec![];
    while let Some(id) = stack.pop() {
        if !shown.insert(id) {
            continue;
        }
        lines.push((id, describe_set(&spec.store, id, &lambda_names)));
        for (_, args) in spec.store.lam_elems(id) {
            stack.extend(args.iter().copied());
        }
    }
    lines.sort();
    for (id, text) in lines {
        writeln!(out, "#{} = {{{}}}", id, text).unwrap();
    }
    out
}

/// The members of the variable `v`, each lambda with the variables its owner is given by, and
/// each `Member::Dynamic` with its reason. A variable without a name in `var_names` is written
/// `v<index>`.
///
/// # Examples
/// A variable holding a lambda of `Main::f` whose owner is given `p0`, and a Dynamic member, reads
/// as `Main::f@lam1[p0], Dynamic (<reason>)`.
fn describe_members(
    solver: &mut Solver,
    v: SetVar,
    var_names: &Map<SetVar, String>,
    lambda_names: &[String],
) -> String {
    let members = solver.members(v).to_vec();
    let mut texts: Vec<String> = vec![];
    let mut seen: Set<String> = Set::default();
    for member in members {
        let text = match member {
            Member::Lam {
                lam,
                owner: Owner::Instance(args),
            } => {
                let args: Vec<String> = args
                    .iter()
                    .map(|a| {
                        let root = solver.find(*a);
                        var_names
                            .get(&root)
                            .cloned()
                            .unwrap_or_else(|| format!("v{}", root.index()))
                    })
                    .collect();
                format!("{}[{}]", lambda_names[lam as usize], args.join(", "))
            }
            Member::Lam {
                lam,
                owner: Owner::Current,
            } => format!("{}[current]", lambda_names[lam as usize]),
            Member::Dynamic(reason) => format!("Dynamic ({})", reason.to_string()),
        };
        if seen.insert(text.clone()) {
            texts.push(text);
        }
    }
    texts.join(", ")
}

/// The elements of the set `id`: each lambda with the sets its owner is given, and `Elem::Dynamic`
/// as `Dynamic`.
///
/// # Examples
/// A set holding two lambdas of `Main::f`, the owner of the second given the set `#2`, reads as
/// `Main::f@lam0[], Main::f@lam1[#2]`. The set holding `Elem::Dynamic` reads as `Dynamic`.
fn describe_set(store: &SetStore, id: SetId, lambda_names: &[String]) -> String {
    store
        .elems(id)
        .iter()
        .map(|e| match e {
            Elem::Dynamic => "Dynamic".to_string(),
            Elem::Lam(lam, args) => {
                let args: Vec<String> = args.iter().map(|a| format!("#{}", a)).collect();
                format!("{}[{}]", lambda_names[*lam as usize], args.join(", "))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}
