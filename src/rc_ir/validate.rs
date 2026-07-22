//! Debug-only well-formedness checks for the RC IR.
//!
//! [`validate`] runs after each RC-IR-rewriting pass during compiler development (gated on
//! `Configuration::develop_mode`; it is never run in a normal `fix` build). It catches a malformed
//! rewrite — one that leaves a dangling variable use or duplicates a binding name — at the pass that
//! produced it, on any input. That closes a gap the runtime checks leave: valgrind and a uniqueness
//! assertion need a triggering input and reachable code, whereas this is static and total.
//!
//! It checks the structural invariants of the RC IR: within each function every bound name is
//! unique (no shadowing), every variable use resolves to a binding in scope, or to a global — a
//! function or a global value, both referenceable by name (a direct call's callee is a function
//! name, not a local binding) — every `Retain`/`Release` names one reference-counting unit of its
//! variable; a function carries a capture parameter exactly for the closure ABI; every match has at
//! least one arm, with any catch-all arm last; and an `Llvm` operation's embedded operand names match
//! its argument list. Reference-count balance, use-after-consume, and capture-projection order are
//! follow-ups: they need the ownership and consume model, and must be validated against the whole
//! test suite to stay free of false positives.

use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::misc::Set;
use crate::rc_ir::ast::{FieldPath, RcExpr, RcExprNode, RcProgram, RcRhs, RcVar};
use crate::rc_ir::borrow::rc_units;

/// Check the well-formedness of every function and global, panicking on the first violation. A
/// violation is an internal compiler error — the RC IR is malformed — so it aborts rather than
/// returns. `stage` names the pass just run, so a failure points at the culprit.
///
/// `symbol_names` is every symbol name in the whole program. A use naming one refers to a global
/// function or value (a direct call's callee, a funptr atom, or a global operand) — one this
/// compilation unit may not define, since separated compilation splits the program across units — and
/// code generation materializes it, so it is always in scope. Local names are globally-unique fresh
/// names, so admitting the symbol names never masks a dangling local.
pub fn validate(prog: &RcProgram, symbol_names: &Set<FullName>, type_env: &TypeEnv, stage: &str) {
    // The globally-referenceable names: every program symbol, plus this program's own functions and
    // globals — which include the clones borrow-ification and specialization mint (not program
    // symbols) and any unit-local function.
    let mut globals = symbol_names.clone();
    for f in prog.funcs.keys() {
        globals.insert(f.name.clone());
    }
    for g in &prog.globals {
        globals.insert(g.symbol.clone());
    }
    // The functions alone, for a closure's target: a global value is referenceable by name but is
    // not something a closure can be built from.
    let funcs: Set<FullName> = prog.funcs.keys().map(|f| f.name.clone()).collect();

    for func in prog.funcs.values() {
        // A capture parameter is present exactly for the closure ABI: it is the trailing capture-
        // pointer parameter a closure projects its captures from, and the funptr ABI has none. A clone
        // that copies the arrow type but sets the wrong capture would mis-lower the ABI.
        if func.capture.is_some() != func.fn_ty.is_closure() {
            panic!(
                "[RC IR validate] {}: `{}` capture-present={} disagrees with closure-ABI={}",
                stage,
                func.name.name.to_string(),
                func.capture.is_some(),
                func.fn_ty.is_closure(),
            );
        }
        let mut v = Validator::new(
            stage,
            &globals,
            &funcs,
            type_env,
            func.name.name.to_string(),
        );
        for p in func.params.iter().chain(func.capture.iter()) {
            v.bind(&p.name);
        }
        v.check_expr(&func.body);
    }
    for g in &prog.globals {
        let mut v = Validator::new(stage, &globals, &funcs, type_env, g.symbol.to_string());
        v.check_expr(&g.init);
    }
}

/// The per-function state: the names bound anywhere in the function (`seen`, for uniqueness) and the
/// names currently in scope (`scope`, for use resolution).
struct Validator<'a> {
    stage: &'a str,
    globals: &'a Set<FullName>,
    funcs: &'a Set<FullName>,
    type_env: &'a TypeEnv,
    location: String,
    seen: Set<FullName>,
    scope: Set<FullName>,
}

impl<'a> Validator<'a> {
    fn new(
        stage: &'a str,
        globals: &'a Set<FullName>,
        funcs: &'a Set<FullName>,
        type_env: &'a TypeEnv,
        location: String,
    ) -> Self {
        Validator {
            stage,
            globals,
            funcs,
            type_env,
            location,
            seen: Set::default(),
            scope: Set::default(),
        }
    }

    /// Introduce a binding: it must be unique within the function, and it enters scope.
    fn bind(&mut self, name: &FullName) {
        if !self.seen.insert(name.clone()) {
            panic!(
                "[RC IR validate] {}: duplicate binding `{}` in `{}`",
                self.stage,
                name.to_string(),
                self.location
            );
        }
        self.scope.insert(name.clone());
    }

    /// A variable use must resolve to a binding in scope or to a global (a function or global value).
    /// A `Retain`/`Release` path stops at or above a reference-counting unit of its variable — at one
    /// exactly once `split_rc_units` has run, and above one (a whole value, or a subtree holding
    /// several units) before then. Descending past a unit is what must not happen: code generation
    /// projects the path without checking it, so such a path would reference-count a part of the unit
    /// instead of the unit, or a closure's function pointer instead of its capture.
    fn check_rc_unit(&self, var: &RcVar, path: &FieldPath) {
        let units = rc_units(&var.ty, self.type_env);
        if !units.iter().any(|unit| unit.starts_with(path)) {
            panic!(
                "[RC IR validate] {}: reference counting `{}` at {:?} in `{}`, which reaches none of its units {:?}",
                self.stage,
                var.name.to_string(),
                path,
                self.location,
                units
            );
        }
    }

    fn use_var(&self, name: &FullName) {
        if !self.scope.contains(name) && !self.globals.contains(name) {
            panic!(
                "[RC IR validate] {}: use of unbound variable `{}` in `{}`",
                self.stage,
                name.to_string(),
                self.location
            );
        }
    }

    fn check_expr(&mut self, node: &RcExprNode) {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || self.check_expr_inner(node));
    }

    fn check_expr_inner(&mut self, node: &RcExprNode) {
        match node.expr.as_ref() {
            RcExpr::Let(x, rhs, k) => {
                self.check_rhs(rhs);
                self.bind(&x.name);
                self.check_expr(k);
                self.scope.remove(&x.name);
            }
            RcExpr::Retain(v, path, _, k) | RcExpr::Release(v, path, _, k) => {
                self.use_var(&v.name);
                self.check_rc_unit(v, path);
                self.check_expr(k);
            }
            // `Eval` names no RC unit — it only observes its variable — so there is no path to check.
            RcExpr::Eval(v, k) => {
                self.use_var(&v.name);
                self.check_expr(k);
            }
            RcExpr::Destructure(container, fields, k) => {
                self.use_var(&container.name);
                for (_, field) in fields {
                    self.bind(&field.name);
                }
                self.check_expr(k);
                for (_, field) in fields {
                    self.scope.remove(&field.name);
                }
            }
            RcExpr::Ret(v) => self.use_var(&v.name),
        }
    }

    fn check_rhs(&mut self, rhs: &RcRhs) {
        match rhs {
            RcRhs::Var(y) => self.use_var(&y.name),
            RcRhs::App(callee, args) => {
                self.use_var(&callee.name);
                for a in args {
                    self.use_var(&a.name);
                }
            }
            RcRhs::Closure(fref, caps) => {
                // A closure names a function of the program. A rewrite that mints a clone name and
                // forgets to add its body leaves this reference dangling, which code generation only
                // meets much later.
                if !self.funcs.contains(&fref.name) {
                    panic!(
                        "[RC IR validate] {}: closure targets `{}`, which is not a function of the program, in `{}`",
                        self.stage,
                        fref.name.to_string(),
                        self.location
                    );
                }
                for c in caps {
                    self.use_var(&c.name);
                }
            }
            RcRhs::Llvm(llvm_gen, args) => {
                // The generator embeds its operand names — code generation resolves the operands from
                // them — while the `args` list carries the same names, in the same order, for the
                // reference-counting analyses. Lowering builds one from the other and renaming rewrites
                // both, so the two stay identical; a rewrite that updated one and not the other would
                // desync what code generation reads from what the analyses track.
                let embedded_names = llvm_gen.free_vars();
                let arg_names: Vec<FullName> = args.iter().map(|a| a.name.clone()).collect();
                if embedded_names != arg_names {
                    panic!(
                        "[RC IR validate] {}: LLVM operand names {:?} disagree with argument names {:?} in `{}`",
                        self.stage,
                        embedded_names.iter().map(|n| n.to_string()).collect::<Vec<_>>(),
                        arg_names.iter().map(|n| n.to_string()).collect::<Vec<_>>(),
                        self.location,
                    );
                }
                for a in args {
                    self.use_var(&a.name);
                }
            }
            RcRhs::Match(scrutinee, arms) => {
                self.use_var(&scrutinee.name);
                // A match has at least one arm, and a catch-all arm (`tag == None`) — which code
                // generation compiles as the tag switch's default case — is the last arm, so every
                // earlier arm names a variant. A rewrite that moved a catch-all before another arm
                // would shadow the arms after it.
                if arms.is_empty() {
                    panic!(
                        "[RC IR validate] {}: match with no arms in `{}`",
                        self.stage, self.location,
                    );
                }
                for arm in &arms[..arms.len() - 1] {
                    if arm.tag.is_none() {
                        panic!(
                            "[RC IR validate] {}: a catch-all match arm precedes a later arm in `{}`",
                            self.stage, self.location,
                        );
                    }
                }
                // Each arm's payload is in scope only within that arm's body, so bind it, check the
                // body, and unbind it before the next sibling arm.
                for arm in arms {
                    self.bind(&arm.payload.name);
                    self.check_expr(&arm.body);
                    self.scope.remove(&arm.payload.name);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::types::{type_fun, type_funptr};
    use crate::fixstd::builtin::{make_i64_ty, InlineLLVMNullPtrLit};
    use crate::misc::Map;
    use crate::rc_ir::ast::{FuncRef, MatchArm, RcExpr, RcFunc, RcVar};

    fn var(name: &str) -> RcVar {
        RcVar {
            name: FullName::local(name),
            ty: make_i64_ty(),
            source: None,
            debug_name: None,
            skip_null_check: false,
        }
    }

    fn node(expr: RcExpr) -> RcExprNode {
        RcExprNode {
            expr: Box::new(expr),
            source: None,
        }
    }

    /// Check `body` as a function whose only bindings in scope on entry are `params`.
    fn check(body: &RcExprNode, params: &[&str]) {
        let globals = Set::default();
        let type_env = TypeEnv::default();
        let funcs = Set::default();
        let mut v = Validator::new("test", &globals, &funcs, &type_env, "f".to_string());
        for p in params {
            v.bind(&FullName::local(p));
        }
        v.check_expr(body);
    }

    #[test]
    fn accepts_well_formed() {
        // let x = p; ret x   (p is a parameter)
        let body = node(RcExpr::Let(
            var("x"),
            RcRhs::Var(var("p")),
            node(RcExpr::Ret(var("x"))),
        ));
        check(&body, &["p"]);
    }

    #[test]
    #[should_panic(expected = "use of unbound variable")]
    fn rejects_unbound_use() {
        // ret y   (y is never bound)
        check(&node(RcExpr::Ret(var("y"))), &[]);
    }

    #[test]
    #[should_panic(expected = "duplicate binding")]
    fn rejects_duplicate_binding() {
        // let x = p; let x = p; ret x   (x bound twice)
        let inner = node(RcExpr::Let(
            var("x"),
            RcRhs::Var(var("p")),
            node(RcExpr::Ret(var("x"))),
        ));
        let body = node(RcExpr::Let(var("x"), RcRhs::Var(var("p")), inner));
        check(&body, &["p"]);
    }

    #[test]
    fn accepts_use_of_a_global_name() {
        // let r = call g(); ret r   where g is a global (not a local binding)
        let globals: Set<FullName> = [FullName::local("g")].into_iter().collect();
        let type_env = TypeEnv::default();
        let funcs = Set::default();
        let mut v = Validator::new("test", &globals, &funcs, &type_env, "f".to_string());
        let body = node(RcExpr::Let(
            var("r"),
            RcRhs::App(var("g"), vec![]),
            node(RcExpr::Ret(var("r"))),
        ));
        v.check_expr(&body);
    }

    #[test]
    #[should_panic(expected = "capture-present=false disagrees with closure-ABI=true")]
    fn rejects_capture_missing_for_closure_abi() {
        // A closure-typed function with no capture parameter: the closure ABI has a capture pointer.
        let name = FuncRef {
            name: FullName::local("f"),
        };
        let func = RcFunc {
            name: name.clone(),
            fn_ty: type_fun(make_i64_ty(), make_i64_ty()),
            params: vec![var("p")],
            capture: None,
            ret_ty: make_i64_ty(),
            body: node(RcExpr::Ret(var("p"))),
            source: None,
            borrowed_units: Set::default(),
        };
        let mut funcs = Map::default();
        funcs.insert(name.clone(), func);
        let prog = RcProgram {
            funcs,
            globals: vec![],
            entry: name,
        };
        validate(&prog, &Set::default(), &TypeEnv::default(), "test");
    }

    #[test]
    #[should_panic(expected = "match with no arms")]
    fn rejects_empty_match_arms() {
        // let m = match s {}; ret m   (s is a parameter; the match has no arms)
        let body = node(RcExpr::Let(
            var("m"),
            RcRhs::Match(var("s"), vec![]),
            node(RcExpr::Ret(var("m"))),
        ));
        check(&body, &["s"]);
    }

    #[test]
    #[should_panic(expected = "catch-all match arm precedes a later arm")]
    fn rejects_catch_all_before_a_later_arm() {
        // let m = match s { _ -> c; 1 -> p }; ret m   (a catch-all arm before a tagged arm)
        let arms = vec![
            MatchArm {
                tag: None,
                payload: var("c"),
                body: node(RcExpr::Ret(var("c"))),
            },
            MatchArm {
                tag: Some(1),
                payload: var("p"),
                body: node(RcExpr::Ret(var("p"))),
            },
        ];
        let body = node(RcExpr::Let(
            var("m"),
            RcRhs::Match(var("s"), arms),
            node(RcExpr::Ret(var("m"))),
        ));
        check(&body, &["s"]);
    }

    #[test]
    #[should_panic(expected = "capture-present=true disagrees with closure-ABI=false")]
    fn rejects_capture_present_for_funptr_abi() {
        // A funptr-typed function with a capture parameter: the funptr ABI has no capture pointer.
        let name = FuncRef {
            name: FullName::local("f"),
        };
        let func = RcFunc {
            name: name.clone(),
            fn_ty: type_funptr(vec![make_i64_ty()], make_i64_ty()),
            params: vec![var("p")],
            capture: Some(var("cap")),
            ret_ty: make_i64_ty(),
            body: node(RcExpr::Ret(var("p"))),
            source: None,
            borrowed_units: Set::default(),
        };
        let mut funcs = Map::default();
        funcs.insert(name.clone(), func);
        let prog = RcProgram {
            funcs,
            globals: vec![],
            entry: name,
        };
        validate(&prog, &Set::default(), &TypeEnv::default(), "test");
    }

    #[test]
    #[should_panic(expected = "disagree with argument names")]
    fn rejects_llvm_operand_name_mismatch() {
        // let r = <nullptr op with no embedded operands>(x); ret r
        // The op's embedded operand names () disagree with the argument list (x).
        let body = node(RcExpr::Let(
            var("r"),
            RcRhs::Llvm(Box::new(InlineLLVMNullPtrLit {}), vec![var("x")]),
            node(RcExpr::Ret(var("r"))),
        ));
        check(&body, &["x"]);
    }
}
