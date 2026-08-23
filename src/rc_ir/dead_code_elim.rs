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
                // A name this program defines by neither. Reachability is then whatever the name
                // reaches inside this program, which is nothing — the walk resolves a mention
                // against the definitions it was handed, and answers for the program it was given.
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
pub(crate) fn collect_mentions(node: &RcExprNode, mention: &mut impl FnMut(&FullName)) {
    grow_stack(|| collect_mentions_inner(node, mention))
}

/// Call `mention` on the names one node holds, then descend into its continuation and arms.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::types::type_funptr;
    use crate::fixstd::builtin::{make_i64_ty, InlineLLVMMakeStructBody};
    use crate::rc_ir::ast::{MatchArm, RcFunc, RcGlobalInit, RcState, RcVar};
    use std::sync::Arc;

    /// The name lowering gives a symbol of the program under test.
    fn global_name(name: &str) -> FullName {
        FullName::from_strs(&["Main"], name)
    }

    /// A variable of type `I64` under `name`, carrying no source location or debug name.
    fn var(name: FullName) -> RcVar {
        RcVar {
            name,
            ty: make_i64_ty(),
            source: None,
            debug_name: None,
            skip_null_check: false,
        }
    }

    /// A body that mentions each of `mentions` — as the reference of a closure value — and returns
    /// the last value it bound. A body mentioning nothing returns its own parameter.
    fn body_mentioning(mentions: &[FullName]) -> RcExprNode {
        let last = FullName::local(&format!("v{}", mentions.len()));
        let mut body = RcExprNode {
            expr: Arc::new(RcExpr::Ret(var(last))),
            source: None,
        };
        for (i, mentioned) in mentions.iter().enumerate().rev() {
            body = RcExprNode {
                expr: Arc::new(RcExpr::Let(
                    var(FullName::local(&format!("v{}", i + 1))),
                    RcRhs::Closure(
                        FuncRef {
                            name: mentioned.clone(),
                        },
                        vec![],
                    ),
                    body,
                )),
                source: None,
            };
        }
        body
    }

    /// A function of one `I64` parameter whose body mentions each of `mentions`.
    fn func(name: FullName, mentions: &[FullName]) -> RcFunc {
        RcFunc {
            name: FuncRef { name },
            fn_ty: type_funptr(vec![make_i64_ty()], make_i64_ty()),
            params: vec![var(FullName::local("v0"))],
            capture: None,
            ret_ty: make_i64_ty(),
            body: body_mentioning(mentions),
            source: None,
            borrowed_units: Set::default(),
            inline_into_callers: false,
        }
    }

    /// A global value whose initializer mentions each of `mentions`.
    fn global(symbol: FullName, mentions: &[FullName]) -> RcGlobalInit {
        RcGlobalInit {
            symbol,
            ty: make_i64_ty(),
            init: body_mentioning(mentions),
            owns_storage: true,
        }
    }

    /// A program of `funcs` and `globals` reached through `roots`.
    fn prog(funcs: Vec<RcFunc>, globals: Vec<RcGlobalInit>, roots: &[FullName]) -> RcProgram {
        RcProgram {
            funcs: funcs.into_iter().map(|f| (f.name.clone(), f)).collect(),
            globals,
            roots: roots.iter().cloned().collect(),
        }
    }

    /// The names of `prog`'s functions, sorted, so that a comparison does not read the order the
    /// function table happens to hold them in.
    fn func_names(prog: &RcProgram) -> Vec<String> {
        let mut names: Vec<String> = prog.funcs.keys().map(|f| f.name.to_string()).collect();
        names.sort();
        names
    }

    /// The names of `prog`'s globals, sorted.
    fn global_names(prog: &RcProgram) -> Vec<String> {
        let mut names: Vec<String> = prog.globals.iter().map(|g| g.symbol.to_string()).collect();
        names.sort();
        names
    }

    /// A function no root reaches is dropped, and so is one whose only caller was itself dropped.
    #[test]
    fn test_a_function_no_root_reaches_is_dropped() {
        let (main, used, unused, unused_callee) = (
            global_name("main"),
            global_name("used"),
            global_name("unused"),
            global_name("unused_callee"),
        );
        let mut prog = prog(
            vec![
                func(main.clone(), &[used.clone()]),
                func(used.clone(), &[]),
                func(unused.clone(), &[unused_callee.clone()]),
                func(unused_callee.clone(), &[]),
            ],
            vec![],
            &[main.clone()],
        );

        eliminate_unreachable(&mut prog);

        assert_eq!(
            func_names(&prog),
            vec![main.to_string(), used.to_string()],
            "the root and what it reaches survive; the unreached function and the one only it \
             called are dropped"
        );
    }

    /// A global the reached code mentions survives together with what its initializer mentions, and
    /// a global nothing mentions is dropped together with what only its initializer mentioned.
    #[test]
    fn test_a_global_survives_exactly_when_something_reaches_it() {
        let (main, table, helper, orphan, orphan_helper) = (
            global_name("main"),
            global_name("table"),
            global_name("helper"),
            global_name("orphan"),
            global_name("orphan_helper"),
        );
        let mut prog = prog(
            vec![
                func(main.clone(), &[table.clone()]),
                func(helper.clone(), &[]),
                func(orphan_helper.clone(), &[]),
            ],
            vec![
                global(table.clone(), &[helper.clone()]),
                global(orphan.clone(), &[orphan_helper.clone()]),
            ],
            &[main.clone()],
        );

        eliminate_unreachable(&mut prog);

        assert_eq!(
            global_names(&prog),
            vec![table.to_string()],
            "the global the root mentions survives, and the one nothing mentions is dropped"
        );
        assert_eq!(
            func_names(&prog),
            vec![helper.to_string(), main.to_string()],
            "the function a surviving global's initializer mentions survives, and the one only the \
             dropped global's initializer mentioned is dropped"
        );
    }

    /// A name this program defines by neither a function nor a global — the symbol of another
    /// compilation unit — leaves the walk to carry on past it.
    #[test]
    fn test_a_name_of_another_compilation_unit_is_walked_past() {
        let (main, elsewhere, used) = (
            global_name("main"),
            FullName::from_strs(&["Other"], "elsewhere"),
            global_name("used"),
        );
        let mut prog = prog(
            vec![
                func(main.clone(), &[elsewhere.clone(), used.clone()]),
                func(used.clone(), &[]),
            ],
            vec![],
            &[main.clone()],
        );

        eliminate_unreachable(&mut prog);

        assert_eq!(
            func_names(&prog),
            vec![main.to_string(), used.to_string()],
            "the name no definition of this program answers is passed over, and the mention after \
             it is still followed"
        );
    }

    /// Mutual recursion is walked once: a reachable cycle is kept and an unreachable one is
    /// dropped, and neither makes the walk revisit a function it has already reached.
    #[test]
    fn test_a_cycle_is_walked_once() {
        let (main, even, odd, dead_a, dead_b) = (
            global_name("main"),
            global_name("even"),
            global_name("odd"),
            global_name("dead_a"),
            global_name("dead_b"),
        );
        let mut prog = prog(
            vec![
                func(main.clone(), &[even.clone()]),
                func(even.clone(), &[odd.clone()]),
                func(odd.clone(), &[even.clone()]),
                func(dead_a.clone(), &[dead_b.clone()]),
                func(dead_b.clone(), &[dead_a.clone()]),
            ],
            vec![],
            &[main.clone()],
        );

        eliminate_unreachable(&mut prog);

        assert_eq!(
            func_names(&prog),
            vec![even.to_string(), main.to_string(), odd.to_string()],
            "the cycle the root reaches survives whole, and the cycle nothing reaches is dropped"
        );
    }

    /// Every place a body can name a definition is followed: the `Var`, `App`, `Llvm` and `Match`
    /// right-hand sides, the body of a match arm, the `Retain`, `Release`, `Eval` and `Destructure`
    /// nodes, and the `Ret`. A global is lowered to an atom carrying its own name
    /// (`Lowerer::lower_var`), so any of these can be the one place a definition is named from.
    #[test]
    fn test_every_mention_site_is_followed() {
        fn node(expr: RcExpr) -> RcExprNode {
            RcExprNode {
                expr: Arc::new(expr),
                source: None,
            }
        }

        let names: Vec<FullName> = [
            "renamed",
            "callee",
            "argument",
            "operand",
            "scrutinee",
            "named_in_arm",
            "retained",
            "released",
            "evaluated",
            "destructured",
            "returned",
        ]
        .iter()
        .map(|n| global_name(n))
        .collect();
        let (main, unreached) = (global_name("main"), global_name("unreached"));
        let local = |n: &str| var(FullName::local(n));
        let at = |i: usize| var(names[i].clone());

        let mut body = node(RcExpr::Ret(at(10)));
        body = node(RcExpr::Destructure(
            at(9),
            vec![(0, local("field"))],
            RcState::Unknown,
            body,
        ));
        body = node(RcExpr::Eval(at(8), body));
        body = node(RcExpr::Release(at(7), vec![], RcState::Unknown, body));
        body = node(RcExpr::Retain(at(6), vec![], RcState::Unknown, body));
        body = node(RcExpr::Let(
            local("matched"),
            RcRhs::Match(
                at(4),
                vec![MatchArm {
                    tag: Some(0),
                    payload: local("payload"),
                    payload_state: RcState::Unknown,
                    body: node(RcExpr::Ret(at(5))),
                }],
            ),
            body,
        ));
        body = node(RcExpr::Let(
            local("operation"),
            RcRhs::Llvm(
                Box::new(InlineLLVMMakeStructBody {
                    field_names: vec![names[3].clone()],
                }),
                vec![at(3)],
            ),
            body,
        ));
        body = node(RcExpr::Let(
            local("called"),
            RcRhs::App(at(1), vec![at(2)]),
            body,
        ));
        body = node(RcExpr::Let(local("moved"), RcRhs::Var(at(0)), body));

        let mut funcs = vec![RcFunc {
            body,
            ..func(main.clone(), &[])
        }];
        funcs.extend(names.iter().map(|n| func(n.clone(), &[])));
        funcs.push(func(unreached.clone(), &[]));
        let mut prog = prog(funcs, vec![], &[main.clone()]);

        eliminate_unreachable(&mut prog);

        let mut expected: Vec<String> = names.iter().map(|n| n.to_string()).collect();
        expected.push(main.to_string());
        expected.sort();
        assert_eq!(
            func_names(&prog),
            expected,
            "every function the root's body names, at whichever kind of node names it, should \
             survive, and the one no node names should be dropped"
        );
    }
}
