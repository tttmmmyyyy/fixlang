// The text a dump of the lambda sets of a program reads as.

use super::{
    concrete::{Elem, SetId, SetStore},
    infer::Analysis,
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
    for c in 0..analysis.components.len() {
        let component = &analysis.components[c];
        if component.symbols.is_empty() {
            continue;
        }
        let symbols = component.symbols.clone();
        let types = component.types.clone();
        let params = component.params.clone();
        let mut param_names: Map<SetVar, String> = Map::default();
        for (i, p) in params.iter().enumerate() {
            param_names.insert(*p, format!("p{}", i));
        }
        let solver = &mut analysis.solver;
        for (s, ty) in symbols.iter().zip(types.iter()) {
            let text = layout.annotate(ty, &mut |v| param_names[&solver.find(v)].clone());
            writeln!(out, "{} : {}", s.to_string(), text).unwrap();
        }
        for (i, p) in params.iter().enumerate() {
            let members = describe_members(solver, *p, &param_names, &lambda_names);
            writeln!(out, "  p{} = {{{}}}", i, members).unwrap();
        }
        writeln!(out).unwrap();
    }

    writeln!(out, "# Copies").unwrap();
    writeln!(out).unwrap();
    let mut named_sets: Vec<SetId> = vec![];
    for (k, instance) in spec.instances.iter().enumerate() {
        let component = &analysis.components[instance.component];
        named_sets.extend(instance.args.iter().copied());
        if component.symbols.is_empty() {
            writeln!(out, "copy {}: the entry and the exported values", k).unwrap();
            continue;
        }
        let symbols = component.symbols.clone();
        let types = component.types.clone();
        let params = component.params.clone();
        let mut param_sets: Map<SetVar, SetId> = Map::default();
        for (i, p) in params.iter().enumerate() {
            param_sets.insert(*p, instance.args[i]);
        }
        let solver = &mut analysis.solver;
        for (s, ty) in symbols.iter().zip(types.iter()) {
            let text = layout.annotate(ty, &mut |v| format!("#{}", param_sets[&solver.find(v)]));
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
        for elem in spec.store.elems(id) {
            if let Elem::Lam(_, args) = elem {
                stack.extend(args.iter().copied());
            }
        }
    }
    lines.sort();
    for (id, text) in lines {
        writeln!(out, "#{} = {{{}}}", id, text).unwrap();
    }
    out
}

/// The members of the variable `v`, each lambda with the variables its owner is given by, and
/// each `Member::Dynamic` with its reason.
fn describe_members(
    solver: &mut Solver,
    v: SetVar,
    param_names: &Map<SetVar, String>,
    lambda_names: &[String],
) -> String {
    let members = solver.members(v).to_vec();
    let mut texts: Vec<String> = vec![];
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
                        param_names
                            .get(&root)
                            .cloned()
                            .unwrap_or_else(|| format!("v{}", Solver::index(root)))
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
        if !texts.contains(&text) {
            texts.push(text);
        }
    }
    texts.join(", ")
}

/// The elements of the set `id`.
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
