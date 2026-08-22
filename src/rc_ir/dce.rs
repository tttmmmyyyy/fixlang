//! Dead-code elimination over the RC IR: drop every function and global the program cannot reach.
//!
//! Lowering and the passes over the RC IR mint functions of their own — a lifted lambda body, a
//! borrow version, a version specialized on the uniqueness or locality of its inputs — and each one
//! that supersedes another leaves that other behind with no caller. Code generation writes out
//! whatever the program holds, so a function left behind is LLVM IR built and verified for code no
//! execution reaches. The back end drops it in turn, but only once its own dead-code elimination
//! runs, which is after that IR has been built, verified and read.

use crate::ast::name::FullName;
use crate::misc::{grow_stack, Map, Set};
use crate::rc_ir::ast::{FuncRef, RcExpr, RcExprNode, RcProgram, RcRhs};

/// Drop the functions and globals `prog.roots` does not reach.
///
/// Reachability is the ordinary one over the graph whose vertices are the program's functions and
/// globals and whose edges are the names one body mentions. Fix expressions are pure and a global is
/// a call-once initializer run when a reader first asks for it, so a global no reader mentions
/// computes a value nothing observes, and it is dropped like an uncalled function.
pub fn eliminate_unreachable(prog: &mut RcProgram) {
    let globals: Map<FullName, &RcExprNode> = prog
        .globals
        .iter()
        .map(|g| (g.symbol.clone(), &g.init))
        .collect();

    // A function and a global are named under a namespace: a program symbol carries the one its
    // module and namespace declarations give it, and a lifted lambda is named under its own symbol's
    // (`Lowerer::fresh_closure_ref`). A local is minted with no namespace, so a mention carrying none
    // names no definition and the walk passes over it without a lookup.
    for name in prog.funcs.keys().map(|f| &f.name).chain(globals.keys()) {
        assert!(
            name.is_global(),
            "RC IR dead-code elimination reads `{}` as a local name, so no mention can reach it",
            name.to_string()
        );
    }

    let mut reached: Set<FullName> = prog.roots.clone();
    let mut pending: Vec<FullName> = prog.roots.iter().cloned().collect();
    while let Some(name) = pending.pop() {
        let body = match prog.funcs.get(&FuncRef { name: name.clone() }) {
            Some(func) => &func.body,
            None => match globals.get(&name) {
                Some(init) => init,
                // A name this program defines by neither: a symbol another compilation unit defines,
                // which code generation declares and the linker resolves.
                None => continue,
            },
        };
        collect_mentions(body, &mut |mentioned| {
            if mentioned.is_global() && reached.insert(mentioned.clone()) {
                pending.push(mentioned.clone());
            }
        });
    }

    prog.funcs.retain(|fref, _| reached.contains(&fref.name));
    prog.globals.retain(|g| reached.contains(&g.symbol));
}

/// Call `mention` on every name `node` mentions.
///
/// A name is mentioned as the reference of a closure value, or as a variable — the callee of a call,
/// an operand, the value returned. Local variables are mentioned along with the rest; the caller
/// decides which of the mentions can name a definition.
fn collect_mentions(node: &RcExprNode, mention: &mut impl FnMut(&FullName)) {
    grow_stack(|| collect_mentions_inner(node, mention))
}

fn collect_mentions_inner(node: &RcExprNode, mention: &mut impl FnMut(&FullName)) {
    match node.expr.as_ref() {
        RcExpr::Let(_, rhs, k) => {
            match rhs {
                RcRhs::Var(v) => mention(&v.name),
                RcRhs::App(callee, args) => {
                    mention(&callee.name);
                    args.iter().for_each(|a| mention(&a.name));
                }
                RcRhs::Closure(fref, caps) => {
                    mention(&fref.name);
                    caps.iter().for_each(|c| mention(&c.name));
                }
                // The names the generator embeds are the operand list again, in the same order —
                // `validate` checks it — so reading the operands reads every name the operation
                // holds, without cloning the generator to ask it for them.
                RcRhs::Llvm(_, args) => {
                    args.iter().for_each(|a| mention(&a.name));
                }
                RcRhs::Match(scrut, arms) => {
                    mention(&scrut.name);
                    for arm in arms {
                        collect_mentions(&arm.body, mention);
                    }
                }
            }
            collect_mentions(k, mention);
        }
        RcExpr::Retain(v, _, _, k) | RcExpr::Release(v, _, _, k) | RcExpr::Eval(v, k) => {
            mention(&v.name);
            collect_mentions(k, mention);
        }
        RcExpr::Destructure(container, _, _, k) => {
            mention(&container.name);
            collect_mentions(k, mention);
        }
        RcExpr::Ret(v) => mention(&v.name),
    }
}
