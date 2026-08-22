//! Dead-code elimination over the RC IR: drop every function and global the program cannot reach.
//!
//! Lowering and the passes over the RC IR mint functions of their own — a lifted lambda body, a
//! borrow version, a version specialized on the uniqueness or locality of its inputs — and each one
//! that supersedes another leaves that other behind with no caller. Code generation writes out
//! whatever the program holds, so a function left behind is LLVM IR built, verified and optimized
//! for code no execution reaches.

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
    let globals: Map<FullName, &RcExprNode> =
        prog.globals.iter().map(|g| (g.symbol.clone(), &g.init)).collect();

    let mut reached: Set<FullName> = Set::default();
    let mut pending: Vec<FullName> = vec![];
    for root in &prog.roots {
        if reached.insert(root.clone()) {
            pending.push(root.clone());
        }
    }
    while let Some(name) = pending.pop() {
        let body = match prog.funcs.get(&FuncRef { name: name.clone() }) {
            Some(func) => &func.body,
            None => match globals.get(&name) {
                Some(init) => init,
                // A name defined by no function and no global of this program: a symbol another
                // compilation unit defines, which code generation declares and the linker resolves.
                None => continue,
            },
        };
        collect_mentions(body, &mut |mentioned| {
            if reached.insert(mentioned.clone()) {
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
/// an operand, the value returned. A local variable is mentioned too: local names are globally
/// unique fresh names, so one never collides with the name of a function or a global, and the caller
/// resolves each mention against the program's definitions.
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
                // Code generation materializes the names embedded in the generator and the RC IR
                // passes read the operand list, so both name values this operation needs.
                RcRhs::Llvm(llvm_gen, args) => {
                    llvm_gen.free_vars().iter().for_each(|n| mention(n));
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
