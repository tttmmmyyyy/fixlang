//! Debug-only well-formedness checks for the RC IR.
//!
//! [`validate`] runs after each RC-IR-rewriting pass during compiler development (gated on
//! `Configuration::develop_mode`; it is never run in a normal `fix` build). It catches a malformed
//! rewrite — one that leaves a dangling variable use or duplicates a binding name — at the pass that
//! produced it, on any input. That closes a gap the runtime checks leave: valgrind and a uniqueness
//! assertion need a triggering input and reachable code, whereas this is static and total.
//!
//! It checks the structural invariants of the RC IR: every name is bound once in the whole program
//! (no shadowing, and no name shared by two functions or globals); every variable use resolves to a
//! binding in scope, or to a global — a function or a global value, both referenceable by name (a
//! direct call names its callee by that name); every `Retain`/`Release` names one
//! reference-counting unit of its variable; a function carries a capture parameter exactly for the
//! closure ABI; every match has at least one arm, with any catch-all arm last, and its arms bind
//! a variable exactly over an unboxed scrutinee; a read of a payload known to be the variant the
//! union holds sits in the arm that selected that variant; a `Builtin`
//! operation's embedded operand names match its argument list; a closure value stores the capture
//! layout its target function projects; the container of a `DestructureUnboxed` is unboxed, and
//! the node names each of its fields; and no `Release` names a value its read lends.

use crate::ast::name::FullName;
use crate::ast::type_env::TypeEnv;
use crate::ast::types::TypeNode;
use crate::fixstd::builtin::{CaptureProjectOp, NoStorageValueOp, UnionAsOp};
use crate::misc::{grow_stack, Map, Set};
use crate::rc_ir::ast::{FieldPath, FuncRef, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcVar};
use crate::rc_ir::ownership::{names_every_field, rc_units};
use std::sync::Arc;

/// Check the well-formedness of every function and global, panicking on the first violation. A
/// violation is an internal compiler error — the RC IR is malformed — so it aborts rather than
/// returns. `stage` names the pass just run, so a failure points at the culprit.
///
/// `symbol_names` is every symbol name in the whole program. A use naming one refers to a global
/// function or value (a direct call's callee, a func atom, or a global operand) — one this
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
    let mut bound_in: Map<FullName, String> = Map::default();

    for func in prog.funcs.values() {
        // A capture parameter is present exactly for the closure ABI: it is the trailing capture-
        // pointer parameter a closure projects its captures from, and the func ABI has none. A
        // clone that copies the function's type but sets the wrong capture would mis-lower the ABI.
        if func.capture.is_some() != func.fn_ty.is_arrow() {
            panic!(
                "[RC IR validate] {}: `{}` capture-present={} disagrees with closure-ABI={}",
                stage,
                func.name.name.to_string(),
                func.capture.is_some(),
                func.fn_ty.is_arrow(),
            );
        }
        let mut v = Validator::new(
            stage,
            &globals,
            prog,
            &capture_layouts,
            type_env,
            func.name.name.to_string(),
            &mut bound_in,
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
            &mut bound_in,
        );
        v.check_expr(&g.init);
    }
}

/// The capture-object layout each function's capture projections read: the field types every
/// projection of that function records, which is the layout a closure value targeting it must store.
/// A function that projects no capture has no entry, and a closure targeting one stores nothing.
///
/// Checking the projections of a function against each other, and against its capture parameter,
/// happens here, where they are gathered.
fn capture_layouts(prog: &RcProgram, stage: &str) -> Map<FuncRef, Vec<Arc<TypeNode>>> {
    let mut out = Map::default();
    for func in prog.funcs.values() {
        let mut layout: Option<Vec<Arc<TypeNode>>> = None;
        for_each_rhs(&func.body, &mut |rhs| {
            let RcRhs::Builtin(op, _) = rhs else {
                return;
            };
            let Some(proj) = op.as_any().downcast_ref::<CaptureProjectOp>() else {
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
fn check_capture_projection(
    func: &RcFunc,
    proj: &CaptureProjectOp,
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
        | RcExpr::DestructureUnboxed(_, _, k) => for_each_rhs(k, f),
        RcExpr::Ret(_) => {}
    })
}

/// The state of the structural check over one function body or global initializer.
struct Validator<'a, 'b> {
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
    /// Every name bound so far anywhere in the program, with the function or global that binds it; a
    /// second binding of one is a duplicate, in the same body or in another. Names are unique across
    /// the whole program because the passes key facts about a variable by its name alone — the
    /// ownership shape of every parameter (`param_ownership_shapes`) among them — so a name bound in
    /// two places would give one variable the facts of the other.
    bound_in: &'b mut Map<FullName, String>,
    /// The names bound to a value made by `NoStorageValueOp`. A value of a type that occupies no
    /// storage holds no boxed value, since a pointer takes storage, so no reference count acts on
    /// one. `lower_lam` leaves such a capture out of the closure on that ground, and this is where
    /// the ground is checked: every pass that runs between the stages this validator is called at
    /// must place no node on one.
    made_without_storage: Set<FullName>,
    /// The names bound to a value its read lends (`BuiltinOp::lends`). A lent value holds no
    /// reference of its own, so nothing releases it: a release naming one would drop the
    /// reference its container holds.
    lent: Set<FullName>,
    /// The variant each scrutinee is known to hold at the node being checked: the tag of the arm
    /// the node is in, for every match around it. A read that checks no tag
    /// (`UnionAsOp::variant_is_known`) reads the variant recorded here.
    selected_variants: Map<FullName, usize>,
    /// The names currently in scope, which a use must resolve to.
    scope: Set<FullName>,
}

impl<'a, 'b> Validator<'a, 'b> {
    /// A validator of one function body or global initializer, which `location` names in a failure
    /// message.
    ///
    /// # Arguments
    /// * `bound_in` — the names the bodies checked before this one bind, each with the function or
    ///   global that binds it. The validator adds this body's bindings to it.
    fn new(
        stage: &'a str,
        globals: &'a Set<FullName>,
        prog: &'a RcProgram,
        capture_layouts: &'a Map<FuncRef, Vec<Arc<TypeNode>>>,
        type_env: &'a TypeEnv,
        location: String,
        bound_in: &'b mut Map<FullName, String>,
    ) -> Self {
        Validator {
            stage,
            globals,
            prog,
            capture_layouts,
            type_env,
            location,
            bound_in,
            made_without_storage: Set::default(),
            lent: Set::default(),
            selected_variants: Map::default(),
            scope: Set::default(),
        }
    }

    /// Introduce a binding: it must be the only binding of its name in the program, and it enters
    /// scope.
    fn bind(&mut self, name: &FullName) {
        if let Some(first) = self.bound_in.insert(name.clone(), self.location.clone()) {
            panic!(
                "[RC IR validate] {}: duplicate binding `{}` in `{}`, which `{}` binds too",
                self.stage,
                name.to_string(),
                self.location,
                first
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
        self.use_callee(name);
        self.borrows_nothing(name, "is used as a value");
    }

    /// A use of a name as the callee of a direct call, where naming a borrowing function is what
    /// borrow-ification's routing does.
    fn use_callee(&self, name: &FullName) {
        if !self.scope.contains(name) && !self.globals.contains(name) {
            panic!(
                "[RC IR validate] {}: use of unbound variable `{}` in `{}`",
                self.stage,
                name.to_string(),
                self.location
            );
        }
    }

    /// A function whose address becomes a value is reached by an indirect call, which resolves to no
    /// `RcFunc`, so the cancellation analysis reads every argument position of it as owning
    /// (`resolve_callee_params` answers `None` and `rhs_consumes` takes the position as consuming).
    /// A borrowing version does not dispose the argument at a borrowed position, so a release of the
    /// caller's would be cancelled against a consume that never happens and the reference would leak.
    /// Borrow-ification reaches its versions by routing direct calls, and by nothing else.
    fn borrows_nothing(&self, name: &FullName, how: &str) {
        let target = self.prog.funcs.get(&FuncRef { name: name.clone() });
        let Some(target) = target else {
            return;
        };
        if !target.borrowed_units.is_empty() {
            panic!(
                "[RC IR validate] {}: `{}` borrows {} unit(s) and {} in `{}`",
                self.stage,
                name.to_string(),
                target.borrowed_units.len(),
                how,
                self.location,
            );
        }
    }

    /// Check an expression and everything under it, holding a binding in scope for exactly the
    /// continuation or arm body it covers.
    fn check_expr(&mut self, node: &RcExprNode) {
        grow_stack(|| self.check_expr_inner(node));
    }

    /// One node of the walk: the uses it makes, the bindings it introduces, and its continuation.
    fn check_expr_inner(&mut self, node: &RcExprNode) {
        match node.expr.as_ref() {
            RcExpr::Let(x, rhs, k) => {
                self.check_rhs(x, rhs);
                if let RcRhs::Builtin(op, _) = rhs {
                    if op.as_any().downcast_ref::<NoStorageValueOp>().is_some() {
                        self.made_without_storage.insert(x.name.clone());
                    }
                    if op.lends() {
                        self.lent.insert(x.name.clone());
                    }
                }
                self.bind(&x.name);
                self.check_expr(k);
                self.scope.remove(&x.name);
            }
            RcExpr::Retain(v, path, _, k) | RcExpr::Release(v, path, _, k) => {
                self.use_var(&v.name);
                if self.made_without_storage.contains(&v.name) {
                    panic!(
                        "[RC IR validate] {}: a reference count acts on `{}`, a value of `{}`, which occupies no storage, in `{}`",
                        self.stage,
                        v.name.to_string(),
                        v.ty.to_string(),
                        self.location,
                    );
                }
                if matches!(node.expr.as_ref(), RcExpr::Release(..)) && self.lent.contains(&v.name)
                {
                    panic!(
                        "[RC IR validate] {}: a release names `{}`, a value its read lends, in `{}`",
                        self.stage,
                        v.name.to_string(),
                        self.location,
                    );
                }
                self.check_rc_unit(v, path);
                self.check_expr(k);
            }
            // `Eval` names no RC unit — it only observes its variable — so there is no path to check.
            RcExpr::Eval(v, k) => {
                self.use_var(&v.name);
                self.check_expr(k);
            }
            RcExpr::DestructureUnboxed(container, fields, k) => {
                self.use_var(&container.name);
                // A boxed container keeps its fields, and lowering reads them out of it with
                // getters.
                if container.ty.is_box(self.type_env) {
                    panic!(
                        "[RC IR validate] {}: a `DestructureUnboxed` takes the boxed `{}` apart, in `{}`",
                        self.stage,
                        container.ty.to_string(),
                        self.location,
                    );
                }
                // A field the `DestructureUnboxed` leaves out would be dropped inside code generation, out
                // of reach of cancellation, so lowering binds every field.
                if !names_every_field(container, fields, self.type_env) {
                    panic!(
                        "[RC IR validate] {}: a `DestructureUnboxed` of the unboxed `{}` names fields {:?} of it, in `{}`",
                        self.stage,
                        container.ty.to_string(),
                        fields.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
                        self.location,
                    );
                }
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
    /// closure's target function and stored capture layout, a `Builtin` operation's operand names,
    /// and a match's arms.
    fn check_rhs(&mut self, x: &RcVar, rhs: &RcRhs) {
        match rhs {
            RcRhs::Var(y) => self.use_var(&y.name),
            RcRhs::App(callee, args) => {
                self.use_callee(&callee.name);
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
                // A closure is reached by an indirect call, which resolves to no `RcFunc`, so the
                // cancellation analysis reads every argument position of it as owning
                // (`resolve_callee_params` answers `None` and `rhs_consumes` takes the position as
                // consuming). A target that borrows a unit does not dispose the argument there, so
                // the caller's release would be cancelled against a consume that never happens, and
                // the reference would leak. The borrowing versions are reached by the direct calls
                // borrow-ification routes to them, and by nothing else.
                self.borrows_nothing(&fref.name, "is the target of a closure");
                // The closure stores its captures in this order, and the target reads them out by
                // slot index against its own copy of the layout. The two are redundant stores of one
                // layout, so a rewrite that reordered, retyped, added, or dropped the captures at one
                // end alone would leave every projection reading the wrong slot.
                match self.capture_layouts.get(fref) {
                    Some(layout) => {
                        let stored: Vec<Arc<TypeNode>> =
                            caps.iter().map(|c| c.ty.clone()).collect();
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
                    // A target that projects nothing reads no slot, so a closure targeting it fills
                    // none: the capture object it would read is the null pointer. A closure storing
                    // captures for such a target hands them to a body that never takes them out.
                    None => {
                        if !caps.is_empty() {
                            panic!(
                                "[RC IR validate] {}: closure stores captures {:?} where `{}` projects none, in `{}`",
                                self.stage,
                                caps.iter().map(|c| c.ty.clone()).collect::<Vec<_>>(),
                                fref.name.to_string(),
                                self.location,
                            );
                        }
                    }
                }
                for c in caps {
                    self.use_var(&c.name);
                }
            }
            RcRhs::Builtin(op, args) => {
                // A payload read that checks no tag sits in the arm that selected its variant, on
                // that match's scrutinee: anywhere else it would read whatever variant the union
                // holds as the one it names.
                if let Some(read) = op.as_any().downcast_ref::<UnionAsOp>() {
                    let selected = args
                        .first()
                        .and_then(|union| self.selected_variants.get(&union.name));
                    if read.variant_is_known() && selected != Some(&read.variant_index()) {
                        panic!(
                            "[RC IR validate] {}: `{}` reads a variant no enclosing match arm \
                             selected in `{}`",
                            self.stage,
                            op.name(),
                            self.location,
                        );
                    }
                }
                // The op embeds its operand names — code generation resolves the operands from
                // them — while the `args` list carries the same names, in the same order, for the
                // reference-counting analyses. Lowering builds one from the other and renaming rewrites
                // both, so the two stay identical; a rewrite that updated one and not the other would
                // desync what code generation reads from what the analyses track.
                let embedded_names = op.free_vars();
                let arg_names: Vec<FullName> = args.iter().map(|a| a.name.clone()).collect();
                if embedded_names != arg_names {
                    panic!(
                        "[RC IR validate] {}: builtin operand names {:?} disagree with argument names {:?} in `{}`",
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
                let prov = op.result_prov(&x.ty, &arg_tys, self.type_env);
                for origins in prov.leaves() {
                    if origins.len() > 1 {
                        panic!(
                            "[RC IR validate] {}: `{}` declares {} sources for one result leaf in `{}`",
                            self.stage,
                            op.name(),
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
                // The variable an arm binds is in scope only within that arm's body, so bind it,
                // check the body, and unbind it before the next sibling arm.
                for arm in arms {
                    // A match consumes an unboxed scrutinee into the variable each arm binds,
                    // and borrows a boxed one, which its arms read with ops.
                    if arm.binder.is_some() == scrutinee.ty.is_box(self.type_env) {
                        panic!(
                            "[RC IR validate] {}: an arm of the match on `{}`, of type `{}`, binds \
                             a variable over a boxed scrutinee, or none over an unboxed one, in `{}`",
                            self.stage,
                            scrutinee.name.to_string(),
                            scrutinee.ty.to_string(),
                            self.location,
                        );
                    }
                    if let Some(binder) = &arm.binder {
                        self.bind(&binder.name);
                    }
                    let outer_selection = arm
                        .tag
                        .and_then(|tag| self.selected_variants.insert(scrutinee.name.clone(), tag));
                    self.check_expr(&arm.body);
                    if arm.tag.is_some() {
                        match outer_selection {
                            Some(tag) => self.selected_variants.insert(scrutinee.name.clone(), tag),
                            None => self.selected_variants.remove(&scrutinee.name),
                        };
                    }
                    if let Some(binder) = &arm.binder {
                        self.scope.remove(&binder.name);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{validate, Validator};
    use crate::ast::builtin_op::BuiltinOp;
    use crate::ast::name::FullName;
    use crate::ast::type_env::TypeEnv;
    use crate::ast::typedecl::Field;
    use crate::ast::types::{
        kind_star, tycon, type_arrow, type_func, type_tyapp, type_tycon, TyCon, TyConInfo,
        TyConVariant, TypeNode,
    };
    use crate::fixstd::builtin::{
        builtin_tycons, make_array_ty, make_dynamic_object_ty, make_i64_ty, make_ptr_ty,
        CaptureProjectOp, NullPtrLitOp, StructGetOp, UnionAsOp,
    };
    use crate::misc::{Map, Set};
    use crate::rc_ir::ast::{
        FuncRef, MatchArm, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcState, RcVar,
    };
    use std::sync::Arc;

    /// A type environment holding the built-in type constructors, which a check consults to tell a
    /// boxed type from an unboxed one, and `Test::Pair`, an unboxed struct of an `Array I64` and an
    /// `I64`.
    fn type_env() -> TypeEnv {
        let mut tycons = builtin_tycons();
        tycons.insert(
            TyCon::new(FullName::from_strs(&["Test"], "Pair")),
            TyConInfo {
                kind: kind_star(),
                variant: TyConVariant::Struct,
                is_unbox: true,
                tyvars: vec![],
                fields: vec![
                    Field::make(
                        "fst".to_string(),
                        type_tyapp(make_array_ty(), make_i64_ty()),
                        None,
                    ),
                    Field::make("snd".to_string(), make_i64_ty(), None),
                ],
                source: None,
                document: None,
                punched_from: None,
            },
        );
        TypeEnv::new(tycons, Map::default())
    }

    /// `Test::Pair`, the unboxed struct `type_env` declares.
    fn pair_ty() -> Arc<TypeNode> {
        type_tycon(&tycon(FullName::from_strs(&["Test"], "Pair")))
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
        RcExprNode::new(expr, None)
    }

    /// A program with no functions and no globals, for a check that names none of its own.
    fn empty_prog() -> RcProgram {
        RcProgram {
            funcs: Map::default(),
            globals: vec![],
            roots: Set::default(),
            retains_sunk: false,
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
            retains_sunk: false,
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
        let mut bound_in = Map::default();
        let mut v = Validator::new(
            "test",
            &globals,
            &prog,
            &capture_layouts,
            &type_env,
            "f".to_string(),
            &mut bound_in,
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

    /// A name two functions each bind is caught, as a name bound twice in one function is: the
    /// passes key facts about a variable by its name alone, so a name bound in two functions would
    /// give the variable of one the facts of the other.
    #[test]
    #[should_panic(expected = "duplicate binding")]
    fn rejects_a_name_two_functions_bind() {
        // f(p) = ret p;  g(p) = ret p   (p bound in both)
        let ret_p = |name: &str| {
            func(
                name,
                type_func(vec![make_i64_ty()], make_i64_ty()),
                vec![var("p")],
                None,
                node(RcExpr::Ret(var("p"))),
            )
        };
        validate_prog(vec![ret_p("f"), ret_p("g")]);
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
    /// agreement between a function's type and its parameter list.
    #[test]
    #[should_panic(expected = "capture-present=false disagrees with closure-ABI=true")]
    fn rejects_capture_missing_for_closure_abi() {
        // A closure-typed function with no capture parameter: the closure ABI has a capture pointer.
        validate_prog(vec![func(
            "f",
            type_arrow(make_i64_ty(), make_i64_ty()),
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
                tag: None,
                binder: Some(var("c")),
                body: node(RcExpr::Ret(var("c"))),
            },
            MatchArm {
                tag: Some(1),
                binder: Some(var("p")),
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

    /// A func-ABI function carrying a capture parameter is caught, the other direction of the
    /// agreement between a function's type and its parameter list.
    #[test]
    #[should_panic(expected = "capture-present=true disagrees with closure-ABI=false")]
    fn rejects_capture_present_for_func_abi() {
        // A func-typed function with a capture parameter: the func ABI has no capture pointer.
        validate_prog(vec![func(
            "f",
            type_func(vec![make_i64_ty()], make_i64_ty()),
            vec![var("p")],
            Some(var("cap")),
            node(RcExpr::Ret(var("p"))),
        )]);
    }

    /// An operation whose embedded operand names differ from its argument list is caught, so what
    /// code generation reads and what the reference-counting analyses track stay the same names.
    #[test]
    #[should_panic(expected = "disagree with argument names")]
    fn rejects_builtin_operand_name_mismatch() {
        // let r = <nullptr op with no embedded operands>(x); ret r
        // The op's embedded operand names () disagree with the argument list (x).
        let body = node(RcExpr::Let(
            var("r"),
            RcRhs::Builtin(Box::new(NullPtrLitOp {}), vec![var("x")]),
            node(RcExpr::Ret(var("r"))),
        ));
        check(&body, &["x"]);
    }

    /// A release of a value its read lends is caught: the value holds no reference of its own, so
    /// the release would drop the one its container holds.
    #[test]
    #[should_panic(expected = "a value its read lends")]
    fn rejects_a_release_of_a_lent_value() {
        // let x = <lending getter of field 0>(c); release x; ret x
        let read = StructGetOp::of_field(&FullName::local("c"), 0).lending();
        let body = node(RcExpr::Let(
            var("x"),
            RcRhs::Builtin(read, vec![var("c")]),
            node(RcExpr::Release(
                var("x"),
                vec![],
                RcState::Unknown,
                node(RcExpr::Ret(var("x"))),
            )),
        ));
        check(&body, &["c"]);
    }

    /// A payload read that checks no tag is caught outside the arm that selected its variant:
    /// there the union may hold another variant, which the read would take for the one it names.
    #[test]
    #[should_panic(expected = "reads a variant no enclosing match arm selected")]
    fn rejects_a_known_variant_read_outside_its_arm() {
        // let p = <read of variant 0, known>(u); ret p   (u is a parameter; no match selects it)
        let read = UnionAsOp::of_known_variant(&FullName::local("u"), 0);
        let body = node(RcExpr::Let(
            var("p"),
            RcRhs::Builtin(Box::new(read), vec![var("u")]),
            node(RcExpr::Ret(var("p"))),
        ));
        check(&body, &["u"]);
    }

    /// A local variable of a boxed type, standing for a boxed scrutinee or container: the checks
    /// it is used in read whether the type is boxed and nothing else of it.
    fn boxed_var(name: &str) -> RcVar {
        var_of(name, make_dynamic_object_ty())
    }

    /// A payload read that checks no tag is caught in an arm that selected another variant: there
    /// the union holds that other variant, which the read would take for the one it names.
    #[test]
    #[should_panic(expected = "reads a variant no enclosing match arm selected")]
    fn rejects_a_known_variant_read_in_the_arm_of_another_variant() {
        // let m = match u { 1 -> let p = <read of variant 0, known>(u); ret p }; ret m
        let read = UnionAsOp::of_known_variant(&FullName::local("u"), 0);
        let arm = MatchArm {
            tag: Some(1),
            binder: None,
            body: node(RcExpr::Let(
                var("p"),
                RcRhs::Builtin(Box::new(read), vec![boxed_var("u")]),
                node(RcExpr::Ret(var("p"))),
            )),
        };
        let body = node(RcExpr::Let(
            var("m"),
            RcRhs::Match(boxed_var("u"), vec![arm]),
            node(RcExpr::Ret(var("m"))),
        ));
        check(&body, &["u"]);
    }

    /// A payload read that checks no tag is caught after the match whose arm selected its variant:
    /// the selection holds inside the arm alone.
    #[test]
    #[should_panic(expected = "reads a variant no enclosing match arm selected")]
    fn rejects_a_known_variant_read_after_the_match_that_selected_it() {
        // let m = match u { 0 -> ret c }; let p = <read of variant 0, known>(u); ret p
        let read = UnionAsOp::of_known_variant(&FullName::local("u"), 0);
        let arm = MatchArm {
            tag: Some(0),
            binder: None,
            body: node(RcExpr::Ret(var("c"))),
        };
        let after = node(RcExpr::Let(
            var("p"),
            RcRhs::Builtin(Box::new(read), vec![boxed_var("u")]),
            node(RcExpr::Ret(var("p"))),
        ));
        let body = node(RcExpr::Let(
            var("m"),
            RcRhs::Match(boxed_var("u"), vec![arm]),
            after,
        ));
        check(&body, &["u", "c"]);
    }

    /// A payload read that checks no tag is caught where an inner match on the same union has
    /// closed: the variant its arm selected holds inside that arm alone, and after it the union
    /// is known to hold the variant the outer arm selected.
    #[test]
    #[should_panic(expected = "reads a variant no enclosing match arm selected")]
    fn rejects_a_known_variant_read_after_an_inner_match_that_selected_it() {
        // let m = match u { 0 -> let i = match u { 1 -> ret c };
        //                        let p = <read of variant 1, known>(u); ret p }; ret m
        let read = UnionAsOp::of_known_variant(&FullName::local("u"), 1);
        let inner_arm = MatchArm {
            tag: Some(1),
            binder: None,
            body: node(RcExpr::Ret(var("c"))),
        };
        let after_inner = node(RcExpr::Let(
            var("p"),
            RcRhs::Builtin(Box::new(read), vec![boxed_var("u")]),
            node(RcExpr::Ret(var("p"))),
        ));
        let outer_arm = MatchArm {
            tag: Some(0),
            binder: None,
            body: node(RcExpr::Let(
                var("i"),
                RcRhs::Match(boxed_var("u"), vec![inner_arm]),
                after_inner,
            )),
        };
        let body = node(RcExpr::Let(
            var("m"),
            RcRhs::Match(boxed_var("u"), vec![outer_arm]),
            node(RcExpr::Ret(var("m"))),
        ));
        check(&body, &["u", "c"]);
    }

    /// An arm binding a variable over a boxed scrutinee is caught: the match borrows a boxed
    /// scrutinee, so nothing is moved into a variable of the arm.
    #[test]
    #[should_panic(
        expected = "binds a variable over a boxed scrutinee, or none over an unboxed one"
    )]
    fn rejects_an_arm_binding_a_variable_over_a_boxed_scrutinee() {
        // let m = match u { 0(p) -> ret c }; ret m   (u is boxed)
        let arm = MatchArm {
            tag: Some(0),
            binder: Some(var("p")),
            body: node(RcExpr::Ret(var("c"))),
        };
        let body = node(RcExpr::Let(
            var("m"),
            RcRhs::Match(boxed_var("u"), vec![arm]),
            node(RcExpr::Ret(var("m"))),
        ));
        check(&body, &["u", "c"]);
    }

    /// An arm binding no variable over an unboxed scrutinee is caught: the match consumes an
    /// unboxed scrutinee, and the variable the arm binds is what carries it away.
    #[test]
    #[should_panic(
        expected = "binds a variable over a boxed scrutinee, or none over an unboxed one"
    )]
    fn rejects_an_arm_binding_no_variable_over_an_unboxed_scrutinee() {
        // let m = match s { 0 -> ret c }; ret m   (s is unboxed)
        let arm = MatchArm {
            tag: Some(0),
            binder: None,
            body: node(RcExpr::Ret(var("c"))),
        };
        let body = node(RcExpr::Let(
            var("m"),
            RcRhs::Match(var("s"), vec![arm]),
            node(RcExpr::Ret(var("m"))),
        ));
        check(&body, &["s", "c"]);
    }

    /// A lifted closure function `f` reading captured value `cap_idx` of a capture object laid out as
    /// `cap_tys`, out of `read_var` — its capture parameter, in a well-formed function. It disposes of
    /// the capture it owns, so its reference counting balances.
    fn projecting_func(read_var: &RcVar, cap_idx: usize, cap_tys: Vec<Arc<TypeNode>>) -> RcFunc {
        let capture = var_of("cap", make_dynamic_object_ty());
        let proj = Box::new(CaptureProjectOp {
            assumed_state: RcState::Unknown,
            cap_name: read_var.name.clone(),
            cap_idx,
            cap_tys,
        });
        let body = node(RcExpr::Let(
            var("c"),
            RcRhs::Builtin(proj, vec![read_var.clone()]),
            node(RcExpr::Release(
                capture.clone(),
                vec![],
                RcState::Unknown,
                node(RcExpr::Ret(var("c"))),
            )),
        ));
        func(
            "f",
            type_arrow(make_i64_ty(), make_i64_ty()),
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
            type_func(vec![stored.ty.clone()], make_i64_ty()),
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

    /// A closure targeting a function that borrows is caught. An indirect call resolves to no
    /// `RcFunc`, so the cancellation analysis reads its argument positions as owning; a borrowing
    /// target would leave the caller's release cancelled against a consume that never happens.
    #[test]
    #[should_panic(expected = "is the target of a closure")]
    fn rejects_a_closure_targeting_a_borrowing_version() {
        let capture = var_of("cap", make_dynamic_object_ty());
        let mut target = projecting_func(&capture, 0, vec![make_ptr_ty()]);
        target.borrowed_units.insert((FullName::local("p"), vec![]));
        validate_prog(vec![
            closure_building_func(var_of("v", make_ptr_ty())),
            target,
        ]);
    }

    /// A borrowing function named in a value position is caught. Its address reaches an indirect
    /// call, which the cancellation analysis reads as owning every argument position.
    #[test]
    #[should_panic(expected = "is used as a value")]
    fn rejects_a_borrowing_function_named_as_a_value() {
        // `g` returns `f` itself rather than calling it.
        let body = node(RcExpr::Ret(var_of("f", type_func(vec![], make_i64_ty()))));
        let mut target = func(
            "f",
            type_func(vec![], make_i64_ty()),
            vec![],
            None,
            body.clone(),
        );
        target.borrowed_units.insert((FullName::local("p"), vec![]));
        let caller = func("g", type_func(vec![], make_i64_ty()), vec![], None, body);
        validate_prog(vec![caller, target]);
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

    /// A `DestructureUnboxed` that leaves one of its container's fields out is caught: code
    /// generation would drop that field inside the node, where no cancellation reaches the release.
    #[test]
    #[should_panic(expected = "a `DestructureUnboxed` of the unboxed")]
    fn rejects_destructure_of_an_unboxed_container_leaving_a_field_out() {
        // destructure p { 1 -> s }; ret s   (p : Test::Pair; field 0 is left out)
        let body = node(RcExpr::DestructureUnboxed(
            var_of("p", pair_ty()),
            vec![(1, var("s"))],
            node(RcExpr::Ret(var("s"))),
        ));
        check(&body, &["p"]);
    }

    /// A `DestructureUnboxed` of a boxed container is caught: a boxed container keeps its fields,
    /// which getters read out of it.
    #[test]
    #[should_panic(expected = "a `DestructureUnboxed` takes the boxed")]
    fn rejects_destructure_unboxed_of_a_boxed_container() {
        // destructure a { 0 -> s }; ret s   (a is boxed)
        let body = node(RcExpr::DestructureUnboxed(
            boxed_var("a"),
            vec![(0, var("s"))],
            node(RcExpr::Ret(var("s"))),
        ));
        check(&body, &["a"]);
    }
}
