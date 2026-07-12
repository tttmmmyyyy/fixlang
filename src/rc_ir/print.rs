//! A pretty-printer for the RC IR, for inspection and debugging.
//!
//! The output is a readable textual rendering of an `RcProgram`: one block per top-level function
//! and per global initializer, with the continuation-nested body printed as a sequence of
//! statements (`let`, `retain`, `release`) terminated by `ret`.

use crate::rc_ir::ast::{Path, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcState, RcVar};

/// Render a whole program.
pub fn program_to_string(prog: &RcProgram) -> String {
    let mut out = String::new();
    out.push_str(&format!("entry {}\n\n", prog.entry.name.to_string()));

    // Print the functions in a deterministic order (by name) so the dump is stable.
    let mut funcs: Vec<&RcFunc> = prog.funcs.values().collect();
    funcs.sort_by(|a, b| a.name.name.to_string().cmp(&b.name.name.to_string()));
    for func in funcs {
        out.push_str(&func_to_string(func));
        out.push('\n');
    }

    for glob in &prog.globals {
        out.push_str(&format!("global {}:\n", glob.symbol.to_string()));
        out.push_str(&expr_to_string(&glob.init, 1));
        out.push('\n');
    }

    out
}

/// A function renders as its signature line (`fn name(params) cap -> ret:`) followed by its
/// indented body.
fn func_to_string(func: &RcFunc) -> String {
    let params = func
        .params
        .iter()
        .map(var_to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let cap = match &func.cap {
        Some(cap) => format!(", cap {}", var_to_string(cap)),
        None => String::new(),
    };
    let mut out = format!(
        "fn {}({}){} -> {}:\n",
        func.name.name.to_string(),
        params,
        cap,
        func.ret_ty.to_string()
    );
    out.push_str(&expr_to_string(&func.body, 1));
    out
}

/// A variable renders as its name annotated with its type, plus its source name in a binding
/// position when it has one.
fn var_to_string(var: &RcVar) -> String {
    let dbg = match &var.debug_name {
        Some(name) => format!(" (as {})", name),
        None => String::new(),
    };
    format!("{} : {}{}", var.name.to_string(), var.ty.to_string(), dbg)
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
fn expr_to_string(node: &RcExprNode, level: usize) -> String {
    let ind = indent(level);
    match node.expr.as_ref() {
        RcExpr::Let(var, rhs, cont) => {
            let mut out = format!(
                "{}let {} = {}\n",
                ind,
                var_to_string(var),
                rhs_to_string(rhs, level)
            );
            out.push_str(&expr_to_string(cont, level));
            out
        }
        RcExpr::Retain(var, path, state, cont) => {
            let keyword = if var.nonnull {
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
            out.push_str(&expr_to_string(cont, level));
            out
        }
        RcExpr::Release(var, path, state, cont) => {
            let keyword = if var.nonnull {
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
            out.push_str(&expr_to_string(cont, level));
            out
        }
        RcExpr::Destructure(container, fields, cont) => {
            let binds = fields
                .iter()
                .map(|(idx, var)| format!(".{} -> {}", idx, var_to_string(var)))
                .collect::<Vec<_>>()
                .join(", ");
            let mut out = format!(
                "{}destructure {} {{ {} }}\n",
                ind,
                var_name(container),
                binds
            );
            out.push_str(&expr_to_string(cont, level));
            out
        }
        RcExpr::Ret(var) => format!("{}ret {}\n", ind, var_name(var)),
    }
}

/// A `let` right-hand side renders inline; a `match` expands to indented `case` arms.
fn rhs_to_string(rhs: &RcRhs, level: usize) -> String {
    match rhs {
        RcRhs::Var(var) => var_name(var),
        RcRhs::App(callee, args) => {
            format!("{}({})", var_name(callee), operands(args))
        }
        RcRhs::Closure(func, caps) => {
            format!("closure {}[{}]", func.name.name.to_string(), operands(caps))
        }
        RcRhs::Llvm(gen, args) => {
            format!("{}({})", gen.name(), operands(args))
        }
        RcRhs::Match(scrutinee, arms) => {
            let mut out = format!("match {} {{\n", var_name(scrutinee));
            for arm in arms {
                let variant = match arm.variant {
                    Some(tag) => tag.to_string(),
                    None => "_".to_string(),
                };
                out.push_str(&format!(
                    "{}case {}({}):\n",
                    indent(level + 1),
                    variant,
                    var_to_string(&arm.payload)
                ));
                out.push_str(&expr_to_string(&arm.body, level + 2));
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
fn path_to_string(path: &Path) -> String {
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
