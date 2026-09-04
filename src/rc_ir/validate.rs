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
//! function or a global value, both referenceable by name (a direct call names its callee by that
//! name) — every `Retain`/`Release` names one reference-counting unit of its variable; a function
//! carries a capture parameter exactly for the closure ABI; every match has at least one arm, with
//! any catch-all arm last; an `Llvm` operation's embedded operand names match its argument list;
//! and a closure value stores the capture layout its target function projects.

use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::fixstd::builtin::InlineLLVMCaptureProjectBody;
use crate::misc::{grow_stack, Map, Set};
use crate::rc_ir::ast::{FieldPath, FuncRef, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcVar};
use crate::rc_ir::ownership::rc_units;
use std::sync::Arc;

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
    let capture_layouts = capture_layouts(prog, stage);

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
            prog,
            &capture_layouts,
            type_env,
            func.name.name.to_string(),
        );
        for p in func.params.iter().chain(func.capture.iter()) {
            v.bind(&p.name);
        }
        v.check_expr(&func.body);
    }
    for g in &prog.globals {
        let mut v = Validator::new(
            stage,
            &globals,
            prog,
            &capture_layouts,
            type_env,
            g.symbol.to_string(),
        );
        v.check_expr(&g.init);
    }
}

/// The capture-object layout each function's capture projections read: the field types every
/// projection of that function records, which is the layout a closure value targeting it must store.
/// A function that projects no capture has no entry — it reads nothing, so any layout suits it.
///
/// Checking the projections of a function against each other, and against its capture parameter,
/// happens here, where they are gathered.
fn capture_layouts(prog: &RcProgram, stage: &str) -> Map<FuncRef, Vec<Arc<TypeNode>>> {
    let mut out = Map::default();
    for func in prog.funcs.values() {
        let mut layout: Option<Vec<Arc<TypeNode>>> = None;
        for_each_rhs(&func.body, &mut |rhs| {
            let RcRhs::Llvm(llvm_gen, _) = rhs else {
                return;
            };
            let Some(proj) = llvm_gen
                .as_any()
                .downcast_ref::<InlineLLVMCaptureProjectBody>()
            else {
                return;
            };
            check_capture_projection(func, proj, layout.as_ref(), stage);
            layout = Some(proj.cap_tys.clone());
        });
        if let Some(layout) = layout {
            out.insert(func.name.clone(), layout);
        }
    }
    out
}

/// A capture projection reads slot `cap_idx` of the function's capture object, whose layout it
/// carries as `cap_tys`. It must read the function's own capture parameter, name a slot that layout
/// has, and agree on the layout with the function's other projections — they are copies of one list,
/// so a rewrite that retyped or reordered the captures of one projection alone would leave the rest
/// reading the old layout.
// PROOF: D/A (dev-docs/proof/rc_ir/borrow-cancel)
fn check_capture_projection(
    func: &RcFunc,
    proj: &InlineLLVMCaptureProjectBody,
    prev_layout: Option<&Vec<Arc<TypeNode>>>,
    stage: &str,
) {
    let location = func.name.name.to_string();
    let capture = func.capture.as_ref().map(|c| &c.name);
    if capture != Some(&proj.cap_name) {
        panic!(
            "[RC IR validate] {}: capture projection reads `{}`, which is not the capture parameter of `{}`",
            stage,
            proj.cap_name.to_string(),
            location,
        );
    }
    if proj.cap_idx >= proj.cap_tys.len() {
        panic!(
            "[RC IR validate] {}: capture projection reads slot {} of a {}-slot capture in `{}`",
            stage,
            proj.cap_idx,
            proj.cap_tys.len(),
            location,
        );
    }
    if let Some(prev_layout) = prev_layout {
        if *prev_layout != proj.cap_tys {
            panic!(
                "[RC IR validate] {}: capture projections of `{}` disagree on the capture layout: {:?} and {:?}",
                stage, location, prev_layout, proj.cap_tys,
            );
        }
    }
}

/// Apply `f` to every right-hand side of an expression, arms of a match included.
fn for_each_rhs(node: &RcExprNode, f: &mut impl FnMut(&RcRhs)) {
    grow_stack(|| match node.expr.as_ref() {
        RcExpr::Let(_, rhs, k) => {
            f(rhs);
            if let RcRhs::Match(_, arms) = rhs {
                for arm in arms {
                    for_each_rhs(&arm.body, f);
                }
            }
            for_each_rhs(k, f);
        }
        RcExpr::Retain(_, _, _, k)
        | RcExpr::Release(_, _, _, k)
        | RcExpr::Eval(_, k)
        | RcExpr::Destructure(_, _, _, k) => for_each_rhs(k, f),
        RcExpr::Ret(_) => {}
    })
}

/// The state of the structural check over one function body or global initializer.
struct Validator<'a> {
    /// The pass whose output is being checked, named in a failure message.
    stage: &'a str,
    /// The names a use may resolve to without a binding: the program's functions and global values.
    globals: &'a Set<FullName>,
    /// The program being checked, in which a closure's target function must be defined.
    prog: &'a RcProgram,
    /// The capture layout each function's projections read (`capture_layouts`), which a closure
    /// value targeting that function must store.
    capture_layouts: &'a Map<FuncRef, Vec<Arc<TypeNode>>>,
    /// The type definitions, which decide where a value's reference-counting units sit.
    type_env: &'a TypeEnv,
    /// The function or global whose body is being checked, named in a failure message.
    location: String,
    /// Every name bound anywhere in this body; a second binding of one is a duplicate.
    seen: Set<FullName>,
    /// The names currently in scope, which a use must resolve to.
    scope: Set<FullName>,
}

impl<'a> Validator<'a> {
    /// A validator of one function body or global initializer, which `location` names in a failure
    /// message.
    fn new(
        stage: &'a str,
        globals: &'a Set<FullName>,
        prog: &'a RcProgram,
        capture_layouts: &'a Map<FuncRef, Vec<Arc<TypeNode>>>,
        type_env: &'a TypeEnv,
        location: String,
    ) -> Self {
        Validator {
            stage,
            globals,
            prog,
            capture_layouts,
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

    /// A variable use must resolve to a binding in scope or to a global (a function or global value).
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

    /// Check an expression and everything under it, holding a binding in scope for exactly the
    /// continuation or arm body it covers.
    fn check_expr(&mut self, node: &RcExprNode) {
        grow_stack(|| self.check_expr_inner(node));
    }

    /// One node of the walk: the uses it makes, the bindings it introduces, and its continuation.
    // PROOF: D/A (dev-docs/proof/rc_ir/borrow-cancel)
    fn check_expr_inner(&mut self, node: &RcExprNode) {
        match node.expr.as_ref() {
            RcExpr::Let(x, rhs, k) => {
                self.check_rhs(x, rhs);
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
            RcExpr::Destructure(container, fields, _state, k) => {
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

    /// Check a right-hand side: the variables it uses, and the invariants its own form carries — a
    /// closure's target function and stored capture layout, an `Llvm` operation's operand names, and
    /// a match's arms.
    // PROOF: D/A (dev-docs/proof/rc_ir/borrow-cancel)
    fn check_rhs(&mut self, x: &RcVar, rhs: &RcRhs) {
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
                if !self.prog.funcs.contains_key(fref) {
                    panic!(
                        "[RC IR validate] {}: closure targets `{}`, which is not a function of the program, in `{}`",
                        self.stage,
                        fref.name.to_string(),
                        self.location
                    );
                }
                // The closure stores its captures in this order, and the target reads them out by
                // slot index against its own copy of the layout. The two are redundant stores of one
                // layout, so a rewrite that reordered, retyped, added, or dropped the captures at one
                // end alone would leave every projection reading the wrong slot.
                if let Some(layout) = self.capture_layouts.get(fref) {
                    let stored: Vec<Arc<TypeNode>> = caps.iter().map(|c| c.ty.clone()).collect();
                    if *layout != stored {
                        panic!(
                            "[RC IR validate] {}: closure stores captures {:?} where `{}` projects {:?}, in `{}`",
                            self.stage,
                            stored,
                            fref.name.to_string(),
                            layout,
                            self.location,
                        );
                    }
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
                // Each result leaf comes from at most one place. The reference-counting analyses
                // read the declaration leaf by leaf and follow the single source back to the object
                // the leaf belongs to; a leaf declaring two sources would let one name stand for two
                // objects, and `cancel` pairs a release with a retain by name.
                let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
                let prov = llvm_gen.result_prov(&x.ty, &arg_tys, self.type_env);
                for origins in prov.leaves() {
                    if origins.len() > 1 {
                        panic!(
                            "[RC IR validate] {}: `{}` declares {} sources for one result leaf in `{}`",
                            self.stage,
                            llvm_gen.name(),
                            origins.len(),
                            self.location,
                        );
                    }
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
                // One arm answers each variant, so a value of that variant reaches one body. A pass
                // that reads the arms by tag takes the first of two arms carrying one tag, and the
                // second would be dead where code generation's switch sends the value to the first.
                let mut tags = Set::default();
                for arm in arms {
                    if let Some(tag) = arm.tag {
                        if !tags.insert(tag) {
                            panic!(
                                "[RC IR validate] {}: two match arms carry variant {} in `{}`",
                                self.stage, tag, self.location,
                            );
                        }
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
    use crate::fixstd::builtin::{
        bulitin_tycons, make_dynamic_object_ty, make_i64_ty, make_ptr_ty, InlineLLVMNullPtrLit,
    };
    use crate::rc_ir::ast::{MatchArm, RcState};

    /// A type environment holding the built-in type constructors, which a check consults to tell a
    /// boxed type from an unboxed one.
    fn type_env() -> TypeEnv {
        TypeEnv::new(bulitin_tycons(), Map::default())
    }

    /// A local variable of type `I64`, which is unboxed and so holds no reference-counting unit.
    fn var(name: &str) -> RcVar {
        var_of(name, make_i64_ty())
    }

    /// A local variable of the given type, carrying no source location or debug name.
    fn var_of(name: &str, ty: Arc<TypeNode>) -> RcVar {
        RcVar {
            name: FullName::local(name),
            ty,
            source: None,
            debug_name: None,
            skip_null_check: false,
        }
    }

    /// An expression node carrying no source span, so a failure message quotes no code.
    fn node(expr: RcExpr) -> RcExprNode {
        RcExprNode {
            expr: Arc::new(expr),
            source: None,
        }
    }

    /// A program with no functions and no globals, for a check that names none of its own.
    fn empty_prog() -> RcProgram {
        RcProgram {
            funcs: Map::default(),
            globals: vec![],
            roots: Set::default(),
        }
    }

    /// A function returning `I64`: `fn_ty` fixes the ABI, and `capture` is the capture parameter the
    /// closure ABI takes.
    fn func(
        name: &str,
        fn_ty: Arc<TypeNode>,
        params: Vec<RcVar>,
        capture: Option<RcVar>,
        body: RcExprNode,
    ) -> RcFunc {
        RcFunc {
            name: FuncRef {
                name: FullName::local(name),
            },
            fn_ty,
            params,
            capture,
            ret_ty: make_i64_ty(),
            body,
            source: None,
            borrowed_units: Set::default(),
            inline_into_callers: false,
        }
    }

    /// Validate a program made of `funcs`, the first of which is its reachability root.
    fn validate_prog(funcs: Vec<RcFunc>) {
        let root = funcs
            .first()
            .expect("a program has at least one function")
            .name
            .name
            .clone();
        let funcs = funcs.into_iter().map(|f| (f.name.clone(), f)).collect();
        let prog = RcProgram {
            funcs,
            globals: vec![],
            roots: [root].into_iter().collect(),
        };
        validate(&prog, &Set::default(), &type_env(), "test");
    }

    /// Check `body` as a function whose only bindings in scope on entry are `params`.
    fn check(body: &RcExprNode, params: &[&str]) {
        check_with_globals(body, params, &[]);
    }

    /// Check `body` as a function whose only bindings in scope on entry are `params`, with `globals`
    /// as the program's global names.
    fn check_with_globals(body: &RcExprNode, params: &[&str], globals: &[&str]) {
        let globals: Set<FullName> = globals.iter().map(|g| FullName::local(g)).collect();
        let type_env = type_env();
        let prog = empty_prog();
        let capture_layouts = Map::default();
        let mut v = Validator::new(
            "test",
            &globals,
            &prog,
            &capture_layouts,
            &type_env,
            "f".to_string(),
        );
        for p in params {
            v.bind(&FullName::local(p));
        }
        v.check_expr(body);
    }

    /// A body binding one variable to a parameter and returning it passes: every use resolves and
    /// every name is bound once.
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

    /// A use of a name that neither a binding nor a global provides is caught — the shape a rewrite
    /// leaves behind when it drops the binding of a variable it still reads.
    #[test]
    #[should_panic(expected = "use of unbound variable")]
    fn rejects_unbound_use() {
        // ret y   (y is never bound)
        check(&node(RcExpr::Ret(var("y"))), &[]);
    }

    /// A name bound a second time in one function is caught, so a name resolves its binding
    /// uniquely and no binding shadows another.
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

    /// A use that no binding in scope answers passes when it names a global, since a function or
    /// global value is referenceable by name from anywhere.
    #[test]
    fn accepts_use_of_a_global_name() {
        // let r = call g(); ret r   where g is a global
        let body = node(RcExpr::Let(
            var("r"),
            RcRhs::App(var("g"), vec![]),
            node(RcExpr::Ret(var("r"))),
        ));
        check_with_globals(&body, &[], &["g"]);
    }

    /// A closure-ABI function without the capture parameter is caught, one direction of the
    /// agreement between a function's arrow type and its parameter list.
    #[test]
    #[should_panic(expected = "capture-present=false disagrees with closure-ABI=true")]
    fn rejects_capture_missing_for_closure_abi() {
        // A closure-typed function with no capture parameter: the closure ABI has a capture pointer.
        validate_prog(vec![func(
            "f",
            type_fun(make_i64_ty(), make_i64_ty()),
            vec![var("p")],
            None,
            node(RcExpr::Ret(var("p"))),
        )]);
    }

    /// A match with no arms is caught: it selects a body for no value, so the expression it binds
    /// has none.
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

    /// A catch-all arm ahead of another arm is caught: code generation compiles it as the tag
    /// switch's default case, which takes every value the arms after it were written for.
    #[test]
    #[should_panic(expected = "catch-all match arm precedes a later arm")]
    fn rejects_catch_all_before_a_later_arm() {
        // let m = match s { _ -> c; 1 -> p }; ret m   (a catch-all arm before a tagged arm)
        let arms = vec![
            MatchArm {
                payload_state: RcState::Unknown,
                tag: None,
                payload: var("c"),
                body: node(RcExpr::Ret(var("c"))),
            },
            MatchArm {
                payload_state: RcState::Unknown,
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

    /// A funptr-ABI function carrying a capture parameter is caught, the other direction of the
    /// agreement between a function's arrow type and its parameter list.
    #[test]
    #[should_panic(expected = "capture-present=true disagrees with closure-ABI=false")]
    fn rejects_capture_present_for_funptr_abi() {
        // A funptr-typed function with a capture parameter: the funptr ABI has no capture pointer.
        validate_prog(vec![func(
            "f",
            type_funptr(vec![make_i64_ty()], make_i64_ty()),
            vec![var("p")],
            Some(var("cap")),
            node(RcExpr::Ret(var("p"))),
        )]);
    }

    /// An operation whose embedded operand names differ from its argument list is caught, so what
    /// code generation reads and what the reference-counting analyses track stay the same names.
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

    /// A lifted closure function `f` reading captured value `cap_idx` of a capture object laid out as
    /// `cap_tys`, out of `read_var` — its capture parameter, in a well-formed function. It disposes of
    /// the capture it owns, so its reference counting balances.
    fn projecting_func(read_var: &RcVar, cap_idx: usize, cap_tys: Vec<Arc<TypeNode>>) -> RcFunc {
        let capture = var_of("cap", make_dynamic_object_ty());
        let proj = Box::new(InlineLLVMCaptureProjectBody {
            assume_local: false,
            cap_name: read_var.name.clone(),
            cap_idx,
            cap_tys,
        });
        let body = node(RcExpr::Let(
            var("c"),
            RcRhs::Llvm(proj, vec![read_var.clone()]),
            node(RcExpr::Release(
                capture.clone(),
                vec![],
                RcState::Unknown,
                node(RcExpr::Ret(var("c"))),
            )),
        ));
        func(
            "f",
            type_fun(make_i64_ty(), make_i64_ty()),
            vec![var("p")],
            Some(capture),
            body,
        )
    }

    /// A function `g` whose body builds a closure value targeting `f`, storing `stored`.
    fn closure_building_func(stored: RcVar) -> RcFunc {
        let body = node(RcExpr::Let(
            var("cl"),
            RcRhs::Closure(
                FuncRef {
                    name: FullName::local("f"),
                },
                vec![stored.clone()],
            ),
            node(RcExpr::Ret(var("cl"))),
        ));
        func(
            "g",
            type_funptr(vec![stored.ty.clone()], make_i64_ty()),
            vec![stored],
            None,
            body,
        )
    }

    /// A closure value storing exactly what its target function projects passes: the two records of
    /// the one capture layout agree.
    #[test]
    fn accepts_a_closure_whose_captures_match_the_projected_layout() {
        let capture = var_of("cap", make_dynamic_object_ty());
        validate_prog(vec![
            closure_building_func(var_of("v", make_ptr_ty())),
            projecting_func(&capture, 0, vec![make_ptr_ty()]),
        ]);
    }

    /// A capture stored at a type the target does not project is caught, so a rewrite that changes
    /// the layout at one end alone cannot escape.
    #[test]
    #[should_panic(expected = "closure stores captures")]
    fn rejects_a_closure_whose_captures_differ_from_the_projected_layout() {
        // `f` projects one captured `I64`, while the closure value stores one `Ptr`.
        let capture = var_of("cap", make_dynamic_object_ty());
        validate_prog(vec![
            closure_building_func(var_of("v", make_ptr_ty())),
            projecting_func(&capture, 0, vec![make_i64_ty()]),
        ]);
    }

    /// A projection reading a variable that is not its function's capture parameter is caught: it
    /// would read a capture object the function was never handed.
    #[test]
    #[should_panic(expected = "is not the capture parameter")]
    fn rejects_a_capture_projection_of_another_variable() {
        // The projection reads the parameter `p` rather than the capture parameter.
        validate_prog(vec![projecting_func(&var("p"), 0, vec![make_i64_ty()])]);
    }

    /// A projection naming a slot one past the layout it carries is caught, so a capture object is
    /// never read out of range.
    #[test]
    #[should_panic(expected = "of a 1-slot capture")]
    fn rejects_a_capture_projection_past_the_layout() {
        // The projection reads slot 1 of a capture object laid out with one slot.
        let capture = var_of("cap", make_dynamic_object_ty());
        validate_prog(vec![projecting_func(&capture, 1, vec![make_i64_ty()])]);
    }
}
