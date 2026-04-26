// Desugaring of opaque type variables in type signatures.
//
// Opaque types (written as `?name` in Fix) hide concrete return types behind generated TyCons.
// This module runs before type-checking and performs three steps.
//
// === Simple global value example ===
//
// Input: `repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it`
//
// Step 1: Generate TyCon `Std::repeat::?it` with kind `* -> *`, type args `[a]`.
//
// Step 2: Add global constraints:
//   QualPredScheme { gen_vars: [a], pred: ?it a : Iterator }
//   EqualityScheme { gen_vars: [a], eq: Item (?it a) = a }
//
// Step 3: Rewrite scheme and wrap definition:
//   repeat : a -> I64 -> ?it a
//   #wrap_opaque : [x : Iterator, Item x = a] (a -> I64 -> x) -> (a -> I64 -> ?it a)
//   repeat = #wrap_opaque(|x, n| range(0, n).map(|_| x))
//
// === Trait member example ===
//
// Input: trait `c : ToIter` with member
//   `to_iter : [?it : Iterator, Item ?it = Elem c] c -> ?it`
//
// Step 1: Generate TyCon `ToIter::to_iter::?it` with kind `* -> *`, type args `[c]`.
//   (The TyCon's type args are the trait's type variables, not the method's own gen_vars.)
//
// Step 2: Add global constraints:
//   QualPredScheme { gen_vars: [c], pred_constraints: [], pred: ?it c : Iterator }
//   EqualityScheme { gen_vars: [c], eq: Item (?it c) = Elem c }
//   Note: `pred_constraints` is empty (no `c : ToIter` condition) because `?it c` only
//   appears via `to_iter : [c : ToIter] c -> ?it c`, so `c : ToIter` is already guaranteed.
//
// Step 3: One #wrap_opaque is generated per method (shared across all impls):
//   to_iter : [c : ToIter] c -> ?it c
//   #wrap_opaque : [c : ToIter, x : Iterator, Item x = Elem c] (c -> x) -> (c -> ?it c)
//   Each impl wraps its definition independently:
//     impl Array a : ToIter { to_iter = #wrap_opaque(|arr| ArrayIterator { ... }); }
//   The OpaqueTyConResolution lhs is specialized per impl (e.g., `?it (Array a)`),
//   using a defn_to_impl substitution that maps `c -> Array a`.
//
// After type-checking, the concrete type behind `#wrap_opaque`'s domain variable is extracted
// (see `fill_opaque_concrete_types` in typecheck.rs). During instantiation, `#wrap_opaque`
// applications are removed and opaque TyCons are replaced with concrete types
// (see `resolve_opaque_type_in_type`, `remove_opaque_wrapper_func`).

use std::sync::Arc;

use crate::ast::equality::Equality;
use crate::ast::expr::{expr_app, expr_array_lit, expr_var, Expr, ExprNode};
use crate::ast::pattern::{Pattern, PatternNode};
use crate::ast::name::{FullName, Name, NameSpace};
use crate::ast::predicate::Predicate;
use crate::ast::program::{GlobalValue, Program, SymbolExpr, TypedExpr};
use crate::ast::qual_pred::{QualPred, QualPredScheme};
use crate::ast::types::{
    collect_free_vars, is_opaque_tyvar, kind_arrow, make_tyvar, type_from_tyvar,
    type_fun, type_tyapp, type_tycon, tycon, Kind, OpaqueTyConResolution, Scheme, TyCon, TyConInfo,
    TyConVariant, TyVar, Type, TypeNode,
};
use crate::constants::{WRAP_OPAQUE_FUNC_NAME, WRAP_OPAQUE_TYVAR_PREFIX};
use crate::elaboration::typecheck::Substitution;
use crate::misc::{insert_to_map_vec, Map, Set};

// Information about an opaque type variable in a scheme.
//
// Example: for `repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it` in module `Std`:
//   tyvar = ?it (kind *)
//   tycon = Std::repeat::?it
//   tycon_vars = [a]
//   tycon_kind = * -> *
struct OpaqueInfo {
    // The opaque type variable.
    tyvar: Arc<TyVar>,
    // The generated TyCon (e.g., `Std::repeat::?it`).
    tycon: Arc<TyCon>,
    // Non-opaque gen_vars from the scheme; become the TyCon's type arguments.
    tycon_vars: Vec<Arc<TyVar>>,
    // Kind of the TyCon (e.g., `* -> *` when there is one type argument of kind `*`).
    tycon_kind: Arc<Kind>,
}

impl Program {
    // Desugar opaque type variables. See the module-level comment for an overview.
    pub fn desugar_opaque_types(&mut self) {
        let gv_names: Vec<FullName> = self.global_values.keys().cloned().collect();

        // Collect opaque infos for global values that have opaque type variables.
        let mut targets: Vec<(FullName, Vec<OpaqueInfo>)> = vec![];
        for gv_name in &gv_names {
            let gv = self.global_values.get(gv_name).unwrap();
            let opaque_infos = collect_opaque_infos(&gv.scm, gv_name);
            if !opaque_infos.is_empty() {
                targets.push((gv_name.clone(), opaque_infos));
            }
        }

        // Step 1 & 2: Register opaque TyCons and add constraints to TraitEnv.
        for (gv_name, opaque_infos) in &targets {
            let scm = self.global_values.get(gv_name).unwrap().scm.clone();

            for info in opaque_infos {
                self.register_opaque_tycon(info);
            }
            self.add_opaque_constraints(&scm, opaque_infos);
        }

        // Step 3: Rewrite type signatures and generate #wrap_opaque GlobalValues.
        for (gv_name, opaque_infos) in &targets {
            let scm = self.global_values.get(gv_name).unwrap().scm.clone();
            let new_scm = rewrite_scheme(&scm, opaque_infos);

            // Generate one #wrap_opaque per function/method.
            let wrap_name = FullName::new(&gv_name.to_namespace(), WRAP_OPAQUE_FUNC_NAME);
            let wrap_scm = build_wrap_scheme(&scm, &new_scm, opaque_infos);

            self.global_values.insert(
                wrap_name.clone(),
                GlobalValue {
                    scm: wrap_scm,
                    syn_scm: None,
                    expr: SymbolExpr::Simple(TypedExpr::from_expr(build_undefined_expr())),
                    decl_src: None,
                    defn_src: None,
                    document: None,
                    compiler_defined_method: false,
                },
            );

            // Rewrite the original global value's scheme and insert #wrap_opaque applications.
            let gv = self.global_values.get_mut(gv_name).unwrap();
            gv.scm = new_scm;
            match &mut gv.expr {
                SymbolExpr::Simple(te) => {
                    let original_expr = te.expr.clone();
                    te.expr = expr_app(
                        expr_var(wrap_name.clone(), None),
                        vec![original_expr],
                        None,
                    );
                    te.opaque_types = build_opaque_resolutions(
                        opaque_infos,
                        &Substitution::default(),
                    );
                }
                SymbolExpr::Method(impls) => {
                    for impl_ in impls.iter_mut() {
                        // Compute defn_to_impl by matching the trait defn scheme type
                        // (e.g., `c -> ?it`) against the impl scheme type (e.g., `Array a -> ?it`).
                        // We must use impl_.scm (not scm_via_defn) because the lhs of OpaqueTyConResolution
                        // must use the same variable names as the rhs, which is filled during type-checking
                        // against impl_.scm. When a user provides a type annotation on the impl method,
                        // impl_.scm.ty may use different variable names than scm_via_defn.ty; the lhs must
                        // match the type-checking context to ensure resolve_opaque_type_in_type works correctly.
                        let defn_to_impl = Substitution::matching_no_kind_check(
                            &scm.ty,
                            &impl_.scm.ty,
                            &[],
                        )
                        .expect("defn scheme type should match impl scm type");

                        impl_.scm = rewrite_impl_scheme(
                            &impl_.scm,
                            &scm,
                            opaque_infos,
                        );
                        impl_.scm_via_defn =
                            rewrite_impl_scheme(&impl_.scm_via_defn, &scm, opaque_infos);
                        let original_expr = impl_.expr.expr.clone();
                        impl_.expr.expr = expr_app(
                            expr_var(wrap_name.clone(), None),
                            vec![original_expr],
                            None,
                        );
                        impl_.expr.opaque_types = build_opaque_resolutions(
                            opaque_infos,
                            &defn_to_impl,
                        );
                    }
                }
            }
        }
    }

    fn register_opaque_tycon(&mut self, info: &OpaqueInfo) {
        let ti = TyConInfo {
            kind: info.tycon_kind.clone(),
            variant: TyConVariant::Opaque,
            is_unbox: false,
            tyvars: info.tycon_vars.clone(),
            fields: vec![],
            source: None,
            document: None,
        };
        let mut new_tycons = Map::default();
        new_tycons.insert(info.tycon.as_ref().clone(), ti);
        self.type_env.add_tycons(new_tycons);
    }

    fn add_opaque_constraints(
        &mut self,
        scm: &Arc<Scheme>,
        opaque_infos: &[OpaqueInfo],
    ) {
        // Build a combined substitution mapping ALL opaque tyvars to their TyCons.
        // This is needed for equalities that reference multiple opaque types,
        // e.g., `Item ?it = ?e` where both `?it` and `?e` are opaque.
        let mut all_opaque_sub = Substitution::default();
        for info in opaque_infos {
            let sub = info.tyvar_to_tycon_substitution();
            assert!(all_opaque_sub.merge(&sub));
        }

        for info in opaque_infos {
            let sub = info.tyvar_to_tycon_substitution();

            // Extract opaque-related predicates.
            // Resolve trait aliases (e.g., `Additive` -> `Add` + `Zero`) so that
            // each constituent trait is stored separately in `opaque_preds`.
            for pred in &scm.predicates {
                if !pred.on_tyvar(&info.tyvar.name) {
                    continue;
                }
                let resolved = pred.resolve_trait_aliases(&self.trait_env.aliases)
                    .unwrap_or_else(|_| vec![pred.clone()]);
                for resolved_pred in resolved {
                    let mut new_pred = resolved_pred;
                    sub.substitute_predicate(&mut new_pred);
                    let qps = QualPredScheme {
                        gen_vars: info.tycon_vars.clone(),
                        qual_pred: QualPred {
                            pred_constraints: vec![],
                            eq_constraints: vec![],
                            kind_constraints: vec![],
                            predicate: new_pred,
                        },
                    };
                    let trait_id = qps.qual_pred.predicate.trait_id.clone();
                    insert_to_map_vec(
                        &mut self.trait_env.opaque_preds,
                        &trait_id,
                        qps,
                    );
                }
            }

            // Extract opaque-related equalities.
            // Use the combined substitution so that all opaque tyvars (including
            // those on the RHS) are replaced with their TyCons.
            for eq in &scm.equalities {
                if !eq.on_tyvar(&info.tyvar.name) {
                    continue;
                }
                let mut new_eq = eq.clone();
                all_opaque_sub.substitute_equality(&mut new_eq);
                let eq_scm = new_eq.generalize();
                insert_to_map_vec(
                    &mut self.trait_env.opaque_eqs,
                    &new_eq.assoc_type,
                    eq_scm,
                );
            }
        }
    }
}

// Collect OpaqueInfo for each opaque type variable in the scheme.
//
// Example: `Std::repeat` with scheme `[?it : Iterator, Item ?it = a] a -> I64 -> ?it`
// yields one OpaqueInfo with tycon `Std::repeat::?it`, tycon_vars `[a]`, tycon_kind `* -> *`.
fn collect_opaque_infos(
    scm: &Arc<Scheme>,
    gv_name: &FullName,
) -> Vec<OpaqueInfo> {
    // Find all opaque type variables in the scheme.
    let all_vars = collect_free_vars(&scm.predicates, &scm.equalities, &scm.ty);

    let mut seen = Set::<Name>::default();
    let mut opaque_vars = vec![];
    for tv in &all_vars {
        if is_opaque_tyvar(&tv.name) && !seen.contains(&tv.name) {
            seen.insert(tv.name.clone());
            opaque_vars.push(tv.clone());
        }
    }

    // Non-opaque gen_vars become the TyCon's type arguments.
    let gen_vars = scm.gen_vars.clone();

    opaque_vars
        .into_iter()
        .map(|opq_var| {
            // TyCon kind: gen_var kinds → opaque tyvar kind.
            // E.g., for gen_vars [a : *] and opaque tyvar ?it : *, the TyCon kind is * -> *.
            let mut tc_kind: Arc<Kind> = opq_var.kind.clone();
            for gv in gen_vars.iter().rev() {
                tc_kind = kind_arrow(gv.kind.clone(), tc_kind);
            }
            let tycon_name = FullName::new(&gv_name.to_namespace(), &opq_var.name);
            OpaqueInfo {
                tyvar: opq_var.clone(),
                tycon: tycon(tycon_name),
                tycon_vars: gen_vars.clone(),
                tycon_kind: tc_kind,
            }
        })
        .collect()
}

impl OpaqueInfo {
    // Build the TyCon applied to its type arguments.
    // Example: for tycon `Std::repeat::?it` and tycon_vars `[a]`, returns `?it a`.
    fn opaque_tycon_applied(&self) -> Arc<TypeNode> {
        let mut ty = type_tycon(&self.tycon);
        for gv in &self.tycon_vars {
            ty = type_tyapp(ty, type_from_tyvar(gv.clone()));
        }
        ty
    }

    // Build a substitution mapping the opaque TyVar to the TyCon application.
    // Example: `?it` -> `?it a` (where `?it` on the right is the TyCon).
    fn tyvar_to_tycon_substitution(&self) -> Substitution {
        Substitution::single(&self.tyvar.name, self.opaque_tycon_applied())
    }
}

// Build OpaqueTyConResolution entries with the correct lhs and rhs = None.
// The rhs is filled in later by type-checking (see `fill_opaque_concrete_types`).
//
// `defn_to_impl` maps trait-definition type variables to impl-specific types.
// For non-method values, pass `Substitution::default()` (identity).
//
// Example (simple): for `repeat`, lhs = `?it a`.
// Example (method): for `impl Array a : ToIter`, defn_to_impl maps `c -> Array a`,
// so lhs = `?it (Array a)`.
fn build_opaque_resolutions(
    opaque_infos: &[OpaqueInfo],
    defn_to_impl: &Substitution,
) -> Map<FullName, Vec<OpaqueTyConResolution>> {
    let mut result: Map<FullName, Vec<OpaqueTyConResolution>> = Map::default();
    for info in opaque_infos {
        let lhs = defn_to_impl.substitute_type(&info.opaque_tycon_applied());
        result
            .entry(info.tycon.name.clone())
            .or_default()
            .push(OpaqueTyConResolution { lhs, rhs: None });
    }
    result
}

// Apply a substitution to a scheme's type and remove predicates/equalities on opaque TyVars.
fn apply_opaque_substitution(scm: &Arc<Scheme>, sub: &Substitution) -> Arc<Scheme> {
    let new_ty = sub.substitute_type(&scm.ty);

    let new_preds: Vec<Predicate> = scm
        .predicates
        .iter()
        .filter(|p| !p.on_opaque_tyvar())
        .cloned()
        .collect();

    let new_eqs: Vec<Equality> = scm
        .equalities
        .iter()
        .filter(|eq| !eq.on_opaque_tyvar())
        .cloned()
        .collect();

    Scheme::new_arc(scm.gen_vars.clone(), scm.kind_signs.clone(), new_preds, new_eqs, new_ty)
}

// Rewrite a scheme: replace opaque TyVars with TyCon applications and remove opaque constraints.
//
// Example: `[?it : Iterator, Item ?it = a] a -> I64 -> ?it`
// becomes `a -> I64 -> ?it a` (where `?it` is now a TyCon, and opaque constraints are removed).
fn rewrite_scheme(scm: &Arc<Scheme>, opaque_infos: &[OpaqueInfo]) -> Arc<Scheme> {
    // Build combined substitution for all opaque tyvars.
    let mut sub = Substitution::default();
    for info in opaque_infos {
        assert!(sub.merge(&info.tyvar_to_tycon_substitution()));
    }
    apply_opaque_substitution(scm, &sub)
}

// Rewrite a trait impl's scheme. The impl may use different names for opaque type variables
// than the trait definition (e.g., `?iter` vs `?it`), so we compute the name correspondence
// by matching the trait defn scheme type (which uses defn names like `c -> ?it`) against
// `impl_scm.ty` (which uses impl names like `Array a -> ?iter`).
fn rewrite_impl_scheme(
    impl_scm: &Arc<Scheme>,
    defn_scm: &Arc<Scheme>,
    opaque_infos: &[OpaqueInfo],
) -> Arc<Scheme> {
    // Match trait defn scheme type against impl scheme type to find the defn→impl name mapping.
    // E.g., defn `c -> ?it` against impl `Array a -> ?iter` gives {c → Array a, ?it → ?iter}.
    let defn_to_impl = Substitution::matching_no_kind_check(
        &defn_scm.ty,
        &impl_scm.ty,
        &[],
    ).expect("defn scheme type should match impl scheme type");

    // Build substitution: impl's opaque tyvar → TyCon applied to impl's type arguments.
    let mut sub = Substitution::default();
    for info in opaque_infos {
        // Look up the impl's name for this opaque tyvar.
        let impl_opaque_ty = defn_to_impl.substitute_type(&type_from_tyvar(info.tyvar.clone()));
        let impl_opaque_name = match &impl_opaque_ty.ty {
            Type::TyVar(tv) => &tv.name,
            _ => panic!("Expected opaque tyvar `{}` to map to a tyvar in impl scheme", info.tyvar.name),
        };

        // Build TyCon application using the impl's type expressions for each type argument.
        let mut ty = type_tycon(&info.tycon);
        for defn_gv in &info.tycon_vars {
            let impl_gv_ty = defn_to_impl.substitute_type(&type_from_tyvar(defn_gv.clone()));
            ty = type_tyapp(ty, impl_gv_ty);
        }

        assert!(sub.merge(&Substitution::single(impl_opaque_name, ty)));
    }

    apply_opaque_substitution(impl_scm, &sub)
}

// Build the scheme for the #wrap_opaque function.
//
// #wrap_opaque bridges the concrete implementation type to the opaque type.
// Its type is `(original_fn_type_with_fresh_vars) -> (rewritten_fn_type_with_opaque_tycons)`.
//
// Example: for `repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it`:
//   #wrap_opaque : [x : Iterator, Item x = a] (a -> I64 -> x) -> (a -> I64 -> ?it a)
//   where `x` is a fresh variable replacing `?it` in the domain.
fn build_wrap_scheme(
    orig_scm: &Arc<Scheme>,
    new_scm: &Arc<Scheme>,
    opaque_infos: &[OpaqueInfo],
) -> Arc<Scheme> {
    // For each opaque tyvar, introduce a fresh type variable for the domain side.
    // E.g., for opaque TyCon `Std::repeat::?it`, fresh var is `#Std::repeat::?it`.
    let mut opaque_to_fresh = Substitution::default();
    let mut wrap_gen_vars = orig_scm.gen_vars.clone();

    for info in opaque_infos.iter() {
        // This name is parsed back via `strip_prefix(WRAP_OPAQUE_TYVAR_PREFIX)` in
        // `fill_opaque_concrete_types` (typecheck.rs). Keep the two in sync.
        let fresh_name = format!("{}{}", WRAP_OPAQUE_TYVAR_PREFIX, info.tycon.name.to_string());
        let fresh_tv = make_tyvar(&fresh_name, &info.tyvar.kind);
        wrap_gen_vars.push(fresh_tv.clone());
        assert!(opaque_to_fresh.merge(&Substitution::single(
            &info.tyvar.name,
            type_from_tyvar(fresh_tv),
        )));
    }

    // All predicates from the original scheme, with opaque tyvars replaced by fresh vars.
    let wrap_preds: Vec<Predicate> = orig_scm.predicates.iter().map(|pred| {
        let mut new_pred = pred.clone();
        opaque_to_fresh.substitute_predicate(&mut new_pred);
        new_pred
    }).collect();

    // All equalities from the original scheme, with opaque tyvars replaced by fresh vars.
    let wrap_eqs: Vec<Equality> = orig_scm.equalities.iter().map(|eq| {
        let mut new_eq = eq.clone();
        opaque_to_fresh.substitute_equality(&mut new_eq);
        new_eq
    }).collect();

    // Domain type: original function type with opaque tyvars replaced by fresh vars.
    let domain_ty = opaque_to_fresh.substitute_type(&orig_scm.ty);

    // Codomain type: the rewritten function type (with opaque TyCon applications).
    let codomain_ty = new_scm.ty.clone();

    // Wrap type: domain -> codomain.
    let wrap_ty = type_fun(domain_ty, codomain_ty);

    Scheme::new_arc(wrap_gen_vars, vec![], wrap_preds, wrap_eqs, wrap_ty)
}

// Build a placeholder expression for the #wrap_opaque body.
// Produces `_undefined_internal([])` which type-checks as `a` (any type).
// #wrap_opaque is removed during instantiation so this is never executed.
fn build_undefined_expr() -> Arc<ExprNode> {
    let mut placeholder_name = FullName::new(
        &NameSpace::new(vec!["Std".to_string()]),
        "_undefined_internal",
    );
    placeholder_name.global_to_absolute();
    let empty_array = expr_array_lit(vec![], None);
    expr_app(expr_var(placeholder_name, None), vec![empty_array], None)
}

// Replace opaque TyCons in a type with their concrete types.
//
// Each OpaqueTyConResolution maps `lhs` (e.g., `?it a`) to `rhs` (e.g., `ArrayIterator a`).
// The lhs is matched against the type to find the appropriate substitution, and
// the rhs is substituted accordingly.
//
// Example: `?it (Array I64)` with resolution `?it (Array a) -> ArrayIterator a`
// is resolved to `ArrayIterator I64`.
pub fn resolve_opaque_type_in_type(
    ty: &Arc<TypeNode>,
    opaque_resolutions: &Map<FullName, Vec<OpaqueTyConResolution>>,
) -> Arc<TypeNode> {
    let mut ty = ty.clone();

    // Loop: resolution may produce another opaque tycon at top level.
    loop {
        let tc = match ty.toplevel_tycon() {
            Some(tc) => tc,
            None => break,
        };

        let resolutions = match opaque_resolutions.get(&tc.name) {
            Some(r) => r,
            None => break, // not an opaque tycon
        };

        // Compute arity from the lhs of the first resolution.
        let arity = resolutions[0].lhs.collect_type_argments().len();

        // Split the type args into prefix (arity args) and rest.
        let all_args = ty.collect_type_argments();
        assert!(
            all_args.len() >= arity,
            "Opaque tycon `{}` expects arity {} but only {} args applied",
            tc.name.to_string(), arity, all_args.len()
        );
        let prefix_args: Vec<_> = all_args[..arity]
            .iter()
            .map(|arg| resolve_opaque_type_in_type(arg, opaque_resolutions))
            .collect();
        let rest_args = &all_args[arity..];

        // Rebuild the prefix: TyCon applied to prefix_args.
        let mut prefix = type_tycon(&tc);
        for arg in &prefix_args {
            prefix = type_tyapp(prefix, arg.clone());
        }

        // Try matching each resolution's lhs against the prefix.
        let mut matched = false;
        for oct in resolutions {
            let matching = Substitution::matching_no_kind_check(
                &oct.lhs,
                &prefix,
                &[], // no fixed tyvars
            );

            if let Some(sub) = matching {
                // Apply the matching to rhs, then apply rest args.
                let rhs = oct.rhs.as_ref().expect(
                    "opaque type resolution rhs should be filled in by type-checking",
                );
                let mut resolved = sub.substitute_type(rhs);
                for arg in rest_args {
                    resolved = type_tyapp(resolved, arg.clone());
                }
                ty = resolved;
                matched = true;
                break;
            }
        }

        if !matched {
            panic!(
                "No matching OpaqueTyConResolution found for opaque tycon `{}`",
                tc.name.to_string()
            );
        }
    }

    // Recurse into sub-nodes.
    match &ty.ty {
        Type::TyVar(_) | Type::TyCon(_) => ty,
        Type::TyApp(fun, arg) => {
            let new_fun = resolve_opaque_type_in_type(fun, opaque_resolutions);
            let new_arg = resolve_opaque_type_in_type(arg, opaque_resolutions);
            if Arc::ptr_eq(&new_fun, fun) && Arc::ptr_eq(&new_arg, arg) {
                ty
            } else {
                ty.set_tyapp_fun(new_fun).set_tyapp_arg(new_arg)
            }
        }
        Type::AssocTy(_assoc_ty, args) => {
            let new_args: Vec<Arc<TypeNode>> = args
                .iter()
                .map(|a| resolve_opaque_type_in_type(a, opaque_resolutions))
                .collect();
            ty.set_assocty_args(new_args)
        }
    }
}

// Remove the #wrap_opaque application from the top level of an expression.
// Transforms `#wrap_opaque(expr)` to `expr`. Only checks the outermost application.
pub fn remove_opaque_wrapper_func(expr: Arc<ExprNode>) -> Arc<ExprNode> {
    if let Expr::App(func, args) = expr.expr.as_ref() {
        if args.len() == 1 {
            if let Expr::Var(var) = func.expr.as_ref() {
                if var.name.name.starts_with(WRAP_OPAQUE_FUNC_NAME) {
                    return args[0].clone();
                }
            }
        }
    }
    expr
}

// Recursively replace opaque TyCons in all type annotations of a pattern tree.
fn resolve_opaque_tycon_in_pattern(
    pat: &Arc<PatternNode>,
    opaque_resolutions: &Map<FullName, Vec<OpaqueTyConResolution>>,
) -> Arc<PatternNode> {
    let mut info = pat.info.clone();
    if let Some(ty) = &info.type_ {
        info.type_ = Some(resolve_opaque_type_in_type(ty, opaque_resolutions));
    }
    match &pat.pattern {
        Pattern::Var(v, anno_ty) => {
            Arc::new(PatternNode {
                pattern: Pattern::Var(v.clone(), anno_ty.clone()),
                info,
            })
        }
        Pattern::Struct(tc, field_to_pat) => {
            let mut new_field_to_pat = field_to_pat.clone();
            for (_, _, subpat) in new_field_to_pat.iter_mut() {
                *subpat = resolve_opaque_tycon_in_pattern(subpat, opaque_resolutions);
            }
            Arc::new(PatternNode {
                pattern: Pattern::Struct(tc.clone(), new_field_to_pat),
                info,
            })
        }
        Pattern::Union(variant, variant_src, subpat) => {
            Arc::new(PatternNode {
                pattern: Pattern::Union(
                    variant.clone(),
                    variant_src.clone(),
                    resolve_opaque_tycon_in_pattern(subpat, opaque_resolutions),
                ),
                info,
            })
        }
    }
}

// Recursively replace opaque TyCons in all type annotations of an expression tree.
pub fn resolve_opaque_tycon_in_expr(
    expr: &Arc<ExprNode>,
    opaque_resolutions: &Map<FullName, Vec<OpaqueTyConResolution>>,
) -> Arc<ExprNode> {
    // Map over all sub-expressions and their types.
    let type_ = expr.type_.as_ref().unwrap();
    let type_ = resolve_opaque_type_in_type(type_, opaque_resolutions);
    let expr = expr.set_type(type_.clone());
    match expr.expr.as_ref() {
        Expr::App(func, args) => {
            let new_func = resolve_opaque_tycon_in_expr(func, opaque_resolutions);
            let new_args: Vec<_> = args
                .iter()
                .map(|a| resolve_opaque_tycon_in_expr(a, opaque_resolutions))
                .collect();
            expr.set_app_func(new_func).set_app_args(new_args)
        }
        Expr::Lam(_vars, body) => {
            let new_body = resolve_opaque_tycon_in_expr(body, opaque_resolutions);
            expr.set_lam_body(new_body)
        }
        Expr::Let(pat, val, body) => {
            let new_pat = resolve_opaque_tycon_in_pattern(pat, opaque_resolutions);
            let new_val = resolve_opaque_tycon_in_expr(val, opaque_resolutions);
            let new_body = resolve_opaque_tycon_in_expr(body, opaque_resolutions);
            expr.set_let_pat(new_pat).set_let_bound(new_val).set_let_value(new_body)
        }
        Expr::If(cond, then_e, else_e) => {
            let new_cond = resolve_opaque_tycon_in_expr(cond, opaque_resolutions);
            let new_then = resolve_opaque_tycon_in_expr(then_e, opaque_resolutions);
            let new_else = resolve_opaque_tycon_in_expr(else_e, opaque_resolutions);
            expr.set_if_cond(new_cond).set_if_then(new_then).set_if_else(new_else)
        }
        Expr::Match(scrut, branches) => {
            let new_scrut = resolve_opaque_tycon_in_expr(scrut, opaque_resolutions);
            let new_branches: Vec<_> = branches
                .iter()
                .map(|(pat, body)| {
                    (
                        resolve_opaque_tycon_in_pattern(pat, opaque_resolutions),
                        resolve_opaque_tycon_in_expr(body, opaque_resolutions),
                    )
                })
                .collect();
            expr.set_match_cond(new_scrut).set_match_pat_vals(new_branches)
        }
        Expr::TyAnno(inner, ty) => {
            let new_inner = resolve_opaque_tycon_in_expr(inner, opaque_resolutions);
            let new_ty = resolve_opaque_type_in_type(ty, opaque_resolutions);
            expr.set_tyanno_expr(new_inner).set_tyanno_ty(new_ty)
        }
        Expr::ArrayLit(elems) => {
            let new_elems: Vec<_> = elems
                .iter()
                .map(|e| resolve_opaque_tycon_in_expr(e, opaque_resolutions))
                .collect();
            expr.set_array_lit_elems(new_elems)
        }
        Expr::Eval(side, main) => {
            let new_side = resolve_opaque_tycon_in_expr(side, opaque_resolutions);
            let new_main = resolve_opaque_tycon_in_expr(main, opaque_resolutions);
            expr.set_eval_side(new_side).set_eval_main(new_main)
        }
        Expr::MakeStruct(_tc, fields) => {
            let mut new_fields = fields.clone();
            for (_, _, e) in new_fields.iter_mut() {
                *e = resolve_opaque_tycon_in_expr(e, opaque_resolutions);
            }
            expr.set_make_struct_fields(new_fields)
        }
        Expr::FFICall(_name, _ret_ty, _param_tys, _va_args, args, _is_ios) => {
            let new_args: Vec<_> = args
                .iter()
                .map(|a| resolve_opaque_tycon_in_expr(a, opaque_resolutions))
                .collect();
            expr.set_ffi_call_args(new_args)
        }
        _ => expr,
    }
}
