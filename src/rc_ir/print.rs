//! A pretty-printer for the RC IR, for inspection and debugging.
//!
//! The output is a readable textual rendering of an `RcProgram`: one block per top-level function
//! and per global initializer, with the continuation-nested body printed as a sequence of
//! statements (`let`, `retain`, `release`) terminated by `ret`. When a provenance map is supplied,
//! each variable binding is annotated with the provenance the analysis computed for it.

use crate::ast::name::FullName;
use crate::misc::Map;
use crate::rc_ir::ast::{
    FieldPath, Ownership, OwnershipShape, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcState,
    RcVar,
};
use crate::rc_ir::provenance::Provenance;

/// The optional annotations for the dump: each variable's provenance, and each parameter's inferred
/// ownership.
#[derive(Clone, Copy, Default)]
pub struct Annotations<'a> {
    pub provs: Option<&'a Map<FullName, Provenance>>,
    pub param_ownerships: Option<&'a Map<FullName, OwnershipShape>>,
}

/// Render a whole program with the given annotations.
pub fn program_to_string_annotated(prog: &RcProgram, ann: Annotations) -> String {
    let mut out = String::new();
    out.push_str(&format!("entry {}\n\n", prog.entry.name.to_string()));

    // Print the functions in a deterministic order (by name) so the dump is stable.
    let mut funcs: Vec<&RcFunc> = prog.funcs.values().collect();
    funcs.sort_by(|a, b| a.name.name.to_string().cmp(&b.name.name.to_string()));
    for func in funcs {
        out.push_str(&func_to_string(func, ann));
        out.push('\n');
    }

    for glob in &prog.globals {
        out.push_str(&format!("global {}:\n", glob.symbol.to_string()));
        out.push_str(&expr_to_string(&glob.init, 1, ann));
        out.push('\n');
    }

    out
}

/// A function renders as its signature line (`fn name(params) cap -> ret:`) followed by its
/// indented body.
fn func_to_string(func: &RcFunc, ann: Annotations) -> String {
    let params = func
        .params
        .iter()
        .map(|p| param_to_string(p, ann))
        .collect::<Vec<_>>()
        .join(", ");
    let cap = match &func.capture {
        Some(cap) => format!(", cap {}", param_to_string(cap, ann)),
        None => String::new(),
    };
    let mut out = format!(
        "fn {}({}){} -> {}:\n",
        func.name.name.to_string(),
        params,
        cap,
        func.ret_ty.to_string()
    );
    out.push_str(&expr_to_string(&func.body, 1, ann));
    out
}

/// A variable renders as its name annotated with its type, its source name in a binding position
/// when it has one, and its provenance when a provenance map is supplied.
fn var_to_string(var: &RcVar, ann: Annotations) -> String {
    let dbg = match &var.debug_name {
        Some(name) => format!(" (as {})", name),
        None => String::new(),
    };
    // A map is supplied only after the analysis has run, and the analysis records every binding, so
    // a variable missing from it is one the analysis walked past.
    let prov = match ann.provs {
        Some(provs) => {
            let prov = provs.get(&var.name).unwrap_or_else(|| {
                unreachable!("no provenance recorded for `{}`", var.name.to_string())
            });
            format!(" [{}]", prov.to_string())
        }
        None => String::new(),
    };
    format!(
        "{} : {}{}{}",
        var.name.to_string(),
        var.ty.to_string(),
        dbg,
        prov
    )
}

/// A parameter or capture renders as a variable does, plus its inferred ownership when an ownership
/// map is supplied. The map is keyed by the inputs alone, so it answers for exactly these positions.
fn param_to_string(var: &RcVar, ann: Annotations) -> String {
    let own = match ann.param_ownerships {
        Some(ownerships) => {
            let shape = ownerships.get(&var.name).unwrap_or_else(|| {
                unreachable!(
                    "no ownership inferred for the input `{}`",
                    var.name.to_string()
                )
            });
            format!(" {{{}}}", ownership_shape_to_string(shape))
        }
        None => String::new(),
    };
    format!("{}{}", var_to_string(var, ann), own)
}

/// Render an ownership shape: `own` / `borrow` for a boxed leaf, `u` where there is no boxed leaf,
/// and a parenthesized list for an unboxed aggregate.
fn ownership_shape_to_string(shape: &OwnershipShape) -> String {
    match shape {
        OwnershipShape::NoUnit => "u".to_string(),
        OwnershipShape::Unit(Ownership::Own) => "own".to_string(),
        OwnershipShape::Unit(Ownership::Borrow) => "borrow".to_string(),
        OwnershipShape::Fields(children) => {
            let inner = children
                .iter()
                .map(ownership_shape_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", inner)
        }
    }
}

/// A variable in a position where only its identity matters (operands) renders as just its name.
fn var_name(var: &RcVar) -> String {
    var.name.to_string()
}

/// The indentation prefix for a nesting level: four spaces per level.
fn indent(level: usize) -> String {
    "    ".repeat(level)
}

/// A statement chain renders as one `let` / `retain` / `release` / `destructure` per line, indented
/// at `level`, ending in `ret`.
fn expr_to_string(node: &RcExprNode, level: usize, ann: Annotations) -> String {
    let ind = indent(level);
    match node.expr.as_ref() {
        RcExpr::Let(var, rhs, cont) => {
            let mut out = format!(
                "{}let {} = {}\n",
                ind,
                var_to_string(var, ann),
                rhs_to_string(rhs, level, ann)
            );
            out.push_str(&expr_to_string(cont, level, ann));
            out
        }
        RcExpr::Retain(var, path, state, cont) => {
            let keyword = if var.skip_null_check {
                "retain_nonnull"
            } else {
                "retain"
            };
            let mut out = format!(
                "{}{} {}{}{}\n",
                ind,
                keyword,
                var_name(var),
                path_to_string(path),
                state_to_string(state)
            );
            out.push_str(&expr_to_string(cont, level, ann));
            out
        }
        RcExpr::Release(var, path, state, cont) => {
            let keyword = if var.skip_null_check {
                "release_nonnull"
            } else {
                "release"
            };
            let mut out = format!(
                "{}{} {}{}{}\n",
                ind,
                keyword,
                var_name(var),
                path_to_string(path),
                state_to_string(state)
            );
            out.push_str(&expr_to_string(cont, level, ann));
            out
        }
        RcExpr::Destructure(container, fields, cont) => {
            let binds = fields
                .iter()
                .map(|(idx, var)| format!(".{} -> {}", idx, var_to_string(var, ann)))
                .collect::<Vec<_>>()
                .join(", ");
            let mut out = format!(
                "{}destructure {} {{ {} }}\n",
                ind,
                var_name(container),
                binds
            );
            out.push_str(&expr_to_string(cont, level, ann));
            out
        }
        RcExpr::Eval(var, cont) => {
            let mut out = format!("{}eval {}\n", ind, var_name(var));
            out.push_str(&expr_to_string(cont, level, ann));
            out
        }
        RcExpr::Ret(var) => format!("{}ret {}\n", ind, var_name(var)),
    }
}

/// A `let` right-hand side renders inline; a `match` expands to indented `case` arms.
fn rhs_to_string(rhs: &RcRhs, level: usize, ann: Annotations) -> String {
    match rhs {
        RcRhs::Var(var) => var_name(var),
        RcRhs::App(callee, args) => {
            format!("{}({})", var_name(callee), operands(args))
        }
        RcRhs::Closure(func, caps) => {
            format!("closure {}[{}]", func.name.name.to_string(), operands(caps))
        }
        RcRhs::Llvm(llvm_gen, _args) => {
            // The op's name spells out its operands, so it is the whole right-hand side here.
            llvm_gen.name()
        }
        RcRhs::Match(scrutinee, arms) => {
            let mut out = format!("match {} {{\n", var_name(scrutinee));
            for arm in arms {
                let variant = match arm.tag {
                    Some(tag) => tag.to_string(),
                    None => "_".to_string(),
                };
                out.push_str(&format!(
                    "{}case {}({}):\n",
                    indent(level + 1),
                    variant,
                    var_to_string(&arm.payload, ann)
                ));
                out.push_str(&expr_to_string(&arm.body, level + 2, ann));
            }
            out.push_str(&format!("{}}}", indent(level)));
            out
        }
    }
}

/// A comma-separated list of operand variables, each by name.
fn operands(vars: &[RcVar]) -> String {
    vars.iter().map(var_name).collect::<Vec<_>>().join(", ")
}

/// A path renders as a run of `.index` segments; the empty path (the whole value) renders as the
/// empty string.
fn path_to_string(path: &FieldPath) -> String {
    path.iter().map(|i| format!(".{}", i)).collect::<String>()
}

/// A known reference-counting state renders as a trailing `@local` / `@threaded` / `@global` tag;
/// `Unknown` renders as the empty string.
fn state_to_string(state: &RcState) -> String {
    match state {
        RcState::Unknown => String::new(),
        RcState::Local => " @local".to_string(),
        RcState::Threaded => " @threaded".to_string(),
        RcState::Global => " @global".to_string(),
    }
}
