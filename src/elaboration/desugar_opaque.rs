// Desugaring of opaque type variables in type signatures.
//
// Opaque types (written as `?name` in Fix) hide concrete return types behind generated TyCons.
// This module runs before type-checking. `validate_opaque_member_impl_signatures` reports an
// implementation of a trait member whose type signature disagrees with the declaration, and
// `desugar_opaque_types` then performs three steps.
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
// (see `fill_opaque_concrete_types` in typecheck.rs), and a concrete type written in terms of the
// opaque TyCon it stands for is rejected (see `validate_opaque_types_are_acyclic`). During
// instantiation, `#wrap_opaque` applications are removed and opaque TyCons are replaced with
// concrete types (see `resolve_opaque_type_in_type`, `remove_opaque_wrapper_func`).

use crate::ast::equality::Equality;
use crate::ast::expr::{expr_app, expr_array_lit, expr_var, Expr, ExprNode};
use crate::ast::name::{FullName, Name, NameSpace};
use crate::ast::pattern::{Pattern, PatternNode};
use crate::ast::predicate::Predicate;
use crate::ast::program::{
    impl_signature_mismatch_error, GlobalValue, Program, SymbolExpr, TraitMemberImpl, TypedExpr,
};
use crate::ast::qual_pred::{QualPred, QualPredScheme};
use crate::ast::types::{
    apply_type_args, collect_free_vars, is_opaque_tyvar, kind_arrow, make_tyvar, tycon,
    type_from_tyvar, type_fun, type_tyapp, type_tycon, Kind, OpaqueTyConResolution, Scheme, TyCon,
    TyConInfo, TyConVariant, TyVar, Type, TypeNode,
};
use crate::configuration::Configuration;
use crate::constants::{WRAP_OPAQUE_FUNC_NAME, WRAP_OPAQUE_TYVAR_PREFIX};
use crate::elaboration::typecheck::{Substitution, TypeCheckContext, UnifOrOtherErr};
use crate::error::Errors;
use crate::graph::Graph;
use crate::misc::{insert_to_map_vec, Map, Set};
use crate::parse::sourcefile::Span;
use std::sync::Arc;

/// Information about an opaque type variable in a scheme.
///
/// Example: for `repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it` in module `Std`:
///   tyvar = ?it (kind *)
///   tycon = Std::repeat::?it
///   tycon_vars = [a]
///   tycon_kind = * -> *
struct OpaqueInfo {
    /// The opaque type variable.
    tyvar: Arc<TyVar>,
    /// The generated TyCon (e.g., `Std::repeat::?it`).
    tycon: Arc<TyCon>,
    /// Non-opaque gen_vars from the scheme; become the TyCon's type arguments.
    tycon_vars: Vec<Arc<TyVar>>,
    /// Kind of the TyCon (e.g., `* -> *` when there is one type argument of kind `*`).
    tycon_kind: Arc<Kind>,
}

impl Program {
    /// Report an implementation of a trait member whose declaration has an opaque type, where the
    /// type signature the implementation writes disagrees with that declaration.
    ///
    /// This runs before `desugar_opaque_types`, for two reasons. The desugaring reads the
    /// implementation's type as the declared type with its type variables given types, and a
    /// signature that disagrees with the declaration has no such reading. And the desugaring drops
    /// the constraints a signature puts on an opaque type variable, so `check_scheme_equivalent`,
    /// which compares an implementation's signature with the declaration during type-checking,
    /// reaches those two schemes with the constraints already gone.
    pub fn validate_opaque_member_impl_signatures(
        &self,
        config: &Configuration,
    ) -> Result<(), Errors> {
        // The context is built for the first member that has an opaque type, so that a program
        // having none pays nothing for this.
        let mut tc: Option<TypeCheckContext> = None;

        let mut errors = Errors::empty();
        for (gv_name, opaque_infos) in self.global_values_with_opaque_types() {
            let gv = &self.global_values[&gv_name];
            let SymbolExpr::Method(impls) = &gv.expr else {
                continue;
            };
            let tc = tc.get_or_insert_with(|| self.create_typechecker(config));
            for impl_ in impls {
                errors.eat_err(validate_impl_signature(
                    tc,
                    &gv.scm,
                    impl_,
                    &opaque_infos,
                    &gv.decl_src,
                ));
            }
        }
        errors.to_result()
    }

    /// Each global value whose type has opaque type variables, with the information the desugaring
    /// needs about each of those variables.
    ///
    /// The values come in the order of their names, so that what the desugaring registers and what
    /// a pass over them reports are in one order from one run to the next.
    fn global_values_with_opaque_types(&self) -> Vec<(FullName, Vec<OpaqueInfo>)> {
        let mut gv_names: Vec<&FullName> = self.global_values.keys().collect();
        gv_names.sort();

        let mut targets: Vec<(FullName, Vec<OpaqueInfo>)> = vec![];
        for gv_name in gv_names {
            let gv = &self.global_values[gv_name];
            let opaque_infos = collect_opaque_infos(&gv.scm, gv_name);
            if !opaque_infos.is_empty() {
                targets.push((gv_name.clone(), opaque_infos));
            }
        }
        targets
    }

    /// Desugar opaque type variables. See the module-level comment for an overview.
    // PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn desugar_opaque_types(&mut self) {
        let targets = self.global_values_with_opaque_types();

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
            let decl_src = self.global_values.get(gv_name).unwrap().decl_src.clone();
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
                    deprecation: None,
                },
            );

            // Rewrite the original global value's scheme and insert #wrap_opaque applications.
            let gv = self.global_values.get_mut(gv_name).unwrap();
            gv.scm = new_scm;
            match &mut gv.expr {
                SymbolExpr::Simple(te) => {
                    te.expr = wrap_with_opaque(&wrap_name, te.expr.clone());
                    te.opaque_types =
                        build_opaque_resolutions(opaque_infos, &Substitution::default(), decl_src);
                }
                SymbolExpr::Method(impls) => {
                    for impl_ in impls.iter_mut() {
                        // The impl side of the match is `impl_.scm`, the scheme this implementation
                        // is type-checked against. It supplies the variable names the rhs of an
                        // OpaqueTyConResolution is filled in with, and the lhs is written in those
                        // same names, so that `resolve_opaque_type_in_type` finds the resolution a
                        // type belongs to. A type annotation the user writes on the impl member is
                        // what names those variables.
                        //
                        // The match binds the trait's type variable, because a member's type has to
                        // name it (see `TraitEnv::validate_structure`). One resolution per
                        // implementation follows from that: each lhs is the opaque type
                        // constructor applied to the type that implementation is for.
                        let defn_to_impl = defn_to_impl_substitution(&scm, &impl_.scm);
                        let defn_to_via_defn = defn_to_impl_substitution(&scm, &impl_.scm_via_defn);

                        impl_.scm = rewrite_impl_scheme(&impl_.scm, &defn_to_impl, opaque_infos);
                        impl_.scm_via_defn = rewrite_impl_scheme(
                            &impl_.scm_via_defn,
                            &defn_to_via_defn,
                            opaque_infos,
                        );
                        impl_.expr.expr = wrap_with_opaque(&wrap_name, impl_.expr.expr.clone());
                        impl_.expr.opaque_types = build_opaque_resolutions(
                            opaque_infos,
                            &defn_to_impl,
                            impl_.lhs_srcs.first().cloned(),
                        );
                    }
                }
            }
        }
    }

    /// Add the TyCon that stands for an opaque type variable to the type environment, taking the
    /// scheme's other generalized variables as its type arguments.
    // PROOF: P1, P2 (dev-docs/proof/rc_ir/borrow-cancel)
    fn register_opaque_tycon(&mut self, info: &OpaqueInfo) {
        let ti = TyConInfo {
            punched_from: None,
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

    /// Give the trait environment the constraints the opaque type variables of `scm` carry, written
    /// in terms of the TyCons that stand for them: `?it : Iterator` becomes `?it a : Iterator`, and
    /// `Item ?it = a` becomes `Item (?it a) = a`.
    ///
    /// The type checker proves a constraint on an opaque type from these, at a use site where the
    /// concrete type behind it stays hidden.
    fn add_opaque_constraints(&mut self, scm: &Arc<Scheme>, opaque_infos: &[OpaqueInfo]) {
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
            // The resolution succeeds here: a predicate naming an unknown trait is reported by
            // `TraitEnv::validate_structure`, and a circular alias by
            // `TraitAliasEnv::resolve_alias`, both of which run before this.
            for pred in &scm.predicates {
                if !pred.on_tyvar(&info.tyvar.name) {
                    continue;
                }
                let resolved = pred
                    .resolve_trait_aliases(&self.trait_env.aliases)
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
                    insert_to_map_vec(&mut self.trait_env.opaque_preds, &trait_id, qps);
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
                insert_to_map_vec(&mut self.trait_env.opaque_eqs, &new_eq.assoc_type, eq_scm);
            }
        }
    }

    /// Rejects an opaque type whose concrete type is written in terms of that opaque type itself.
    ///
    /// Type-checking writes the concrete type it found for an opaque TyCon into `self.opaque_types`,
    /// and instantiation puts that type in the TyCon's place (`resolve_opaque_type_in_type`),
    /// repeating while the result is again an opaque TyCon so that a chain of opaque types is
    /// followed to its end. A concrete type that leads, along such a chain, back to the resolution
    /// it came from is the type of no value, and the replacement would never terminate.
    ///
    /// Only the concrete types filled in so far are read, so a run that checks part of the program
    /// reports the cycles lying within that part.
    pub fn validate_opaque_types_are_acyclic(&self, tc: &TypeCheckContext) -> Result<(), Errors> {
        // One node per resolution: an opaque TyCon of a trait member has one resolution per
        // implementation, and which of them applies is decided by the lhs, so a chain that leaves
        // one implementation for another is a chain between two nodes of one TyCon name.
        let mut tycon_names: Vec<&FullName> = self.opaque_types.keys().collect();
        tycon_names.sort();
        let mut resolutions: Vec<&OpaqueTyConResolution> = vec![];
        let mut nodes_of_tycon: Map<&FullName, Vec<usize>> = Map::default();
        for tycon_name in tycon_names {
            for resolution in &self.opaque_types[tycon_name] {
                nodes_of_tycon
                    .entry(tycon_name)
                    .or_default()
                    .push(resolutions.len());
                resolutions.push(resolution);
            }
        }

        // An edge runs to every resolution that could replace an opaque TyCon of the concrete type.
        let mut tc = tc.clone();
        let mut edges: Vec<Vec<usize>> = vec![vec![]; resolutions.len()];
        for (from, resolution) in resolutions.iter().enumerate() {
            let Some(rhs) = &resolution.rhs else {
                continue;
            };
            for application in collect_opaque_applications(rhs, &self.opaque_types) {
                for to in &nodes_of_tycon[&application.tycon_name] {
                    if application.can_be_resolved_by(&mut tc, &resolutions[*to].lhs)? {
                        edges[from].push(*to);
                    }
                }
            }
        }

        // Every resolution of a cycle determines the others and none of them a type, so the
        // strongly connected component is what the report is about. A component of one resolution
        // is a cycle when that resolution's concrete type is written in terms of itself.
        let refers_to_itself: Vec<bool> = edges
            .iter()
            .enumerate()
            .map(|(node, targets)| targets.contains(&node))
            .collect();
        let graph = Graph::new_with_edges(resolutions, edges);
        let component_of_node = graph.compute_sccs();
        let mut nodes_of_component: Map<usize, Vec<usize>> = Map::default();
        for (node, component) in component_of_node.iter().enumerate() {
            nodes_of_component.entry(*component).or_default().push(node);
        }

        let mut errors = Errors::empty();
        for component in 0..nodes_of_component.len() {
            let component_nodes = &nodes_of_component[&component];
            if component_nodes.len() == 1 && !refers_to_itself[component_nodes[0]] {
                continue;
            }
            errors.append(opaque_cycle_error(&graph, component_nodes));
        }
        errors.to_result()
    }
}

/// Report the type signature `impl_` writes for a trait member, where it disagrees with the
/// declaration of that member.
///
/// # Arguments
/// * `defn_scm` — the type of the member as the trait declares it, in the declaration's own names:
///   `[c : ToIter] c -> ?it`.
/// * `opaque_infos` — the opaque type variables of the declaration.
/// * `decl_src` — where the trait declares the member.
fn validate_impl_signature(
    tc: &TypeCheckContext,
    defn_scm: &Arc<Scheme>,
    impl_: &TraitMemberImpl,
    opaque_infos: &[OpaqueInfo],
    decl_src: &Option<Span>,
) -> Result<(), Errors> {
    let impl_src = impl_.lhs_srcs.first().cloned();

    // The signature and the declaration have to describe the same values. The opaque types of the
    // declaration stand for themselves in that comparison: what one of them hides is the
    // implementation's choice, and the signature states what the declaration states about it rather
    // than choosing it, so a constraint on one has to agree with the declaration's.
    let opaque_tyvars: Vec<Arc<TyVar>> =
        opaque_infos.iter().map(|info| info.tyvar.clone()).collect();
    let signatures_agree = UnifOrOtherErr::extract_others(tc.check_scheme_equivalent(
        &impl_.scm,
        &impl_.scm_via_defn,
        &opaque_tyvars,
    ))?
    .is_ok();

    // The desugaring reads the signature as the declared type with its type variables given types.
    // An implementation that writes no signature is given the type built that way, so a type that
    // has no such reading comes from a signature the implementation writes.
    let Some(defn_to_impl) = Substitution::matching_no_kind_check(&defn_scm.ty, &impl_.scm.ty, &[])
    else {
        // The report says which of the two this signature is: one that describes values other than
        // the declaration's, or one that describes the same values in another shape.
        return Err(if signatures_agree {
            signature_shape_error(&impl_.scm, &impl_.scm_via_defn, &impl_src, decl_src)
        } else {
            impl_signature_mismatch_error(&impl_.scm, &impl_.scm_via_defn, &impl_src, decl_src)
        });
    };

    // Each opaque type of the declaration is written as an opaque type variable of the signature's
    // own, and no two of them as one: the desugaring gives each opaque type of the declaration a
    // TyCon of its own, which stands for one type this implementation returns.
    let mut defn_tyvar_by_written_name: Map<Name, Arc<TyVar>> = Map::default();
    for info in opaque_infos {
        // An opaque type the declaration writes in its constraints alone stands nowhere in the
        // signature, so the signature names no counterpart for it and the desugaring leaves it as
        // it is. What the signature says about such a type is compared above, where it stands for
        // itself.
        let Some(written) = defn_to_impl.data.get(&info.tyvar.name) else {
            continue;
        };
        let written_var = match &written.ty {
            Type::TyVar(tv) if is_opaque_tyvar(&tv.name) => tv.clone(),
            _ => {
                return Err(non_opaque_type_for_opaque_type_error(
                    &info.tyvar,
                    written,
                    &impl_src,
                    decl_src,
                ))
            }
        };
        if let Some(first) =
            defn_tyvar_by_written_name.insert(written_var.name.clone(), info.tyvar.clone())
        {
            return Err(one_opaque_type_for_two_error(
                &first,
                &info.tyvar,
                written,
                &impl_src,
                decl_src,
            ));
        }
    }

    if !signatures_agree {
        return Err(impl_signature_mismatch_error(
            &impl_.scm,
            &impl_.scm_via_defn,
            &impl_src,
            decl_src,
        ));
    }

    Ok(())
}

/// The error reported where the type a signature writes is not the type of the declaration with its
/// type variables replaced. A member whose type has an opaque type asks that of its
/// implementations.
///
/// The type `Expected` names is the declaration's at the type the trait is implemented for, so it
/// is the type to write here, up to the names of its type variables.
fn signature_shape_error(
    scm: &Arc<Scheme>,
    scm_via_defn: &Arc<Scheme>,
    impl_src: &Option<Span>,
    decl_src: &Option<Span>,
) -> Errors {
    Errors::from_msg_srcs(
        format!(
            "Type signature in implementation is not the type of the trait definition with its type variables replaced.\n\
             Expected: `{}`\n\
             Found: `{}`\n\
             NOTE: a member whose type has an opaque type asks this of the signature an implementation writes for it, down to an associated type application written as the definition writes it. The type variables may be renamed, and the signature may be left out.",
            scm_via_defn.ty.to_string(),
            scm.ty.to_string(),
        ),
        &[impl_src, decl_src],
    )
}

/// The error reported where a signature writes a type of its own in the place the declaration
/// writes an opaque type.
fn non_opaque_type_for_opaque_type_error(
    opaque_var: &Arc<TyVar>,
    written: &Arc<TypeNode>,
    impl_src: &Option<Span>,
    decl_src: &Option<Span>,
) -> Errors {
    Errors::from_msg_srcs(
        format!(
            "Type signature in implementation writes `{}` where the trait definition writes the opaque type `{}`. \
             NOTE: write an opaque type variable here too, and the compiler takes the type behind it from the implementation's body.",
            written.to_string(),
            opaque_var.name,
        ),
        &[&source_of_type_or(written, impl_src), decl_src],
    )
}

/// The error reported where a signature writes one opaque type variable for two opaque types of the
/// declaration.
fn one_opaque_type_for_two_error(
    first: &Arc<TyVar>,
    second: &Arc<TyVar>,
    written: &Arc<TypeNode>,
    impl_src: &Option<Span>,
    decl_src: &Option<Span>,
) -> Errors {
    Errors::from_msg_srcs(
        format!(
            "Type signature in implementation writes one opaque type `{}` for two opaque types of the trait definition, `{}` and `{}`. \
             NOTE: write an opaque type variable of its own for each opaque type of the definition.",
            written.to_string(),
            first.name,
            second.name,
        ),
        &[&source_of_type_or(written, impl_src), decl_src],
    )
}

/// Where a diagnostic about `ty` points: at the type as it is written, and at `fallback` where the
/// type carries no location.
fn source_of_type_or(ty: &Arc<TypeNode>, fallback: &Option<Span>) -> Option<Span> {
    ty.get_source().clone().or_else(|| fallback.clone())
}

/// The error reported for `cycle_nodes`, the resolutions of one cycle: each one's concrete type is
/// written in terms of another of them, so none of them names a type.
fn opaque_cycle_error(graph: &Graph<&OpaqueTyConResolution>, cycle_nodes: &[usize]) -> Errors {
    let msg = if cycle_nodes.len() == 1 {
        format!(
            "The concrete type of the opaque type `{}` cannot be determined, because the definition gives it a type which contains that opaque type itself.",
            graph.get(cycle_nodes[0]).lhs.to_string(),
        )
    } else {
        let opaque_type_names = cycle_nodes
            .iter()
            .map(|node| format!("`{}`", graph.get(*node).lhs.to_string()))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "The concrete types of the opaque types {} cannot be determined, because they are written in terms of each other.",
            opaque_type_names,
        )
    };
    let srcs: Vec<&Option<Span>> = cycle_nodes
        .iter()
        .map(|node| &graph.get(*node).src)
        .collect();
    Errors::from_msg_srcs(msg, &srcs)
}

/// An opaque TyCon as it appears in a type, applied to the arguments its resolutions take.
struct OpaqueApplication {
    /// The name of the opaque TyCon standing at the head of this application.
    tycon_name: FullName,
    /// The TyCon applied to those arguments, and `None` where fewer of them are applied — a shape
    /// `resolve_opaque_type_in_type` aborts on, so the check reads it as one that any resolution
    /// could resolve and reports a cycle rather than leaving a type the resolution cannot replace.
    applied_type: Option<Arc<TypeNode>>,
}

impl OpaqueApplication {
    /// Whether the resolution whose left hand side is `lhs` could be the one that replaces this
    /// application.
    ///
    /// Instantiation reaches the application with its type variables already substituted and
    /// matches what it holds then against `lhs` (see `resolve_opaque_type_in_type`), so the
    /// resolution can replace it exactly when some instance of the application is an instance of
    /// `lhs`. The application's type variables are renamed apart first: it and `lhs` are written in
    /// two schemes of their own, and a name they happen to share would otherwise tie them to one
    /// type.
    fn can_be_resolved_by(
        &self,
        tc: &mut TypeCheckContext,
        lhs: &Arc<TypeNode>,
    ) -> Result<bool, Errors> {
        let Some(applied_type) = &self.applied_type else {
            return Ok(true);
        };
        let applied_type = tc.instantiate_type(applied_type);
        tc.are_unifiable(&applied_type, lhs)
    }
}

/// Collect the applications of opaque TyCons in `ty`, at every depth.
fn collect_opaque_applications(
    ty: &Arc<TypeNode>,
    opaque_resolutions: &Map<FullName, Vec<OpaqueTyConResolution>>,
) -> Vec<OpaqueApplication> {
    let mut applications = vec![];
    collect_opaque_applications_inner(ty, opaque_resolutions, &mut applications);
    applications
}

/// Appends to `applications` every application of an opaque TyCon met while walking `ty`,
/// descending into the type arguments of one so that an opaque TyCon standing inside another is
/// collected as well.
fn collect_opaque_applications_inner(
    ty: &Arc<TypeNode>,
    opaque_resolutions: &Map<FullName, Vec<OpaqueTyConResolution>>,
    applications: &mut Vec<OpaqueApplication>,
) {
    if let Some(tycon) = ty.toplevel_tycon() {
        if let Some(resolutions) = opaque_resolutions.get(&tycon.name) {
            let arity = opaque_tycon_arity(resolutions);
            let args = ty.collect_type_arguments();
            let applied_type = if args.len() >= arity {
                Some(apply_type_args(&tycon, &args[..arity]))
            } else {
                None
            };
            applications.push(OpaqueApplication {
                tycon_name: tycon.name.clone(),
                applied_type,
            });
            // The arguments are types of their own; the TyCon they are applied to is this
            // application and is covered by the entry just pushed.
            for arg in args {
                collect_opaque_applications_inner(&arg, opaque_resolutions, applications);
            }
            return;
        }
    }
    match &ty.ty {
        Type::TyVar(_) | Type::TyCon(_) => {}
        Type::TyApp(tyfun, arg) => {
            collect_opaque_applications_inner(tyfun, opaque_resolutions, applications);
            collect_opaque_applications_inner(arg, opaque_resolutions, applications);
        }
        Type::AssocTy(_, args) => {
            for arg in args {
                collect_opaque_applications_inner(arg, opaque_resolutions, applications);
            }
        }
    }
}

/// The number of type arguments an opaque TyCon takes, read from the left hand side of its
/// resolutions, which all apply it to the same number of arguments.
fn opaque_tycon_arity(resolutions: &[OpaqueTyConResolution]) -> usize {
    resolutions[0].lhs.collect_type_arguments().len()
}

/// Collect OpaqueInfo for each opaque type variable in the scheme.
///
/// Example: `Std::repeat` with scheme `[?it : Iterator, Item ?it = a] a -> I64 -> ?it`
/// yields one OpaqueInfo with tycon `Std::repeat::?it`, tycon_vars `[a]`, tycon_kind `* -> *`.
// PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
fn collect_opaque_infos(scm: &Arc<Scheme>, gv_name: &FullName) -> Vec<OpaqueInfo> {
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
    /// Build the TyCon applied to its type arguments.
    /// Example: for tycon `Std::repeat::?it` and tycon_vars `[a]`, returns `?it a`.
    fn opaque_tycon_applied(&self) -> Arc<TypeNode> {
        let args: Vec<Arc<TypeNode>> = self
            .tycon_vars
            .iter()
            .map(|tycon_var| type_from_tyvar(tycon_var.clone()))
            .collect();
        apply_type_args(&self.tycon, &args)
    }

    /// Build a substitution mapping the opaque TyVar to the TyCon application.
    /// Example: `?it` -> `?it a` (where `?it` on the right is the TyCon).
    fn tyvar_to_tycon_substitution(&self) -> Substitution {
        Substitution::single(&self.tyvar.name, self.opaque_tycon_applied())
    }
}

/// Build OpaqueTyConResolution entries with the correct lhs and rhs = None.
/// The rhs is filled in later by type-checking (see `fill_opaque_concrete_types`).
///
/// `defn_to_impl` maps trait-definition type variables to impl-specific types.
/// For non-method values, pass `Substitution::default()` (identity).
///
/// `src` is the source of the definition whose type-checking fills in the rhs.
///
/// Example (simple): for `repeat`, lhs = `?it a`.
/// Example (method): for `impl Array a : ToIter`, defn_to_impl maps `c -> Array a`,
/// so lhs = `?it (Array a)`.
fn build_opaque_resolutions(
    opaque_infos: &[OpaqueInfo],
    defn_to_impl: &Substitution,
    src: Option<Span>,
) -> Map<FullName, Vec<OpaqueTyConResolution>> {
    let mut result: Map<FullName, Vec<OpaqueTyConResolution>> = Map::default();
    for info in opaque_infos {
        let lhs = defn_to_impl.substitute_type(&info.opaque_tycon_applied());
        result
            .entry(info.tycon.name.clone())
            .or_default()
            .push(OpaqueTyConResolution {
                lhs,
                rhs: None,
                src: src.clone(),
            });
    }
    result
}

/// Apply a substitution to a scheme's type and remove predicates/equalities on opaque TyVars.
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

    Scheme::new_arc(
        scm.gen_vars.clone(),
        scm.kind_signs.clone(),
        new_preds,
        new_eqs,
        new_ty,
    )
}

/// Rewrite a scheme: replace opaque TyVars with TyCon applications and remove opaque constraints.
///
/// Example: `[?it : Iterator, Item ?it = a] a -> I64 -> ?it`
/// becomes `a -> I64 -> ?it a` (where `?it` is now a TyCon, and opaque constraints are removed).
fn rewrite_scheme(scm: &Arc<Scheme>, opaque_infos: &[OpaqueInfo]) -> Arc<Scheme> {
    // Build combined substitution for all opaque tyvars.
    let mut sub = Substitution::default();
    for info in opaque_infos {
        assert!(sub.merge(&info.tyvar_to_tycon_substitution()));
    }
    apply_opaque_substitution(scm, &sub)
}

/// The types a scheme of an implementation writes for the type variables of the member's declared
/// type: `{c -> Array a, ?it -> ?iter}` for the declared `c -> ?it` against `Array a -> ?iter`.
///
/// Panics where the implementation's type is not the declared type with its type variables given
/// types. Both schemes an implementation carries are such a type: a signature the implementation
/// writes is reported by `validate_opaque_member_impl_signatures` before the desugaring runs, and
/// the scheme `TraitImpl::member_scheme_by_defn` builds is the declared type with the trait's type
/// variable replaced.
fn defn_to_impl_substitution(defn_scm: &Arc<Scheme>, impl_scm: &Arc<Scheme>) -> Substitution {
    Substitution::matching_no_kind_check(&defn_scm.ty, &impl_scm.ty, &[]).expect(
        "the type of an implementation is the declared type with its type variables given types",
    )
}

/// Rewrite a trait impl's scheme. The impl may use different names for opaque type variables
/// than the trait definition (e.g., `?iter` vs `?it`), so `defn_to_impl` carries the name
/// correspondence: it sends the defn names of `c -> ?it` to the impl names of `Array a -> ?iter`.
fn rewrite_impl_scheme(
    impl_scm: &Arc<Scheme>,
    defn_to_impl: &Substitution,
    opaque_infos: &[OpaqueInfo],
) -> Arc<Scheme> {
    // Build substitution: impl's opaque tyvar → TyCon applied to impl's type arguments.
    let mut sub = Substitution::default();
    for info in opaque_infos {
        // Look up the impl's name for this opaque tyvar.
        let impl_opaque_ty = defn_to_impl.substitute_type(&type_from_tyvar(info.tyvar.clone()));
        let impl_opaque_name = match &impl_opaque_ty.ty {
            Type::TyVar(tv) => &tv.name,
            // `validate_opaque_member_impl_signatures` reports an implementation that writes
            // anything but an opaque type variable here.
            _ => panic!(
                "Expected opaque tyvar `{}` to map to a tyvar in impl scheme",
                info.tyvar.name
            ),
        };

        // Build TyCon application using the impl's type expressions for each type argument.
        let mut ty = type_tycon(&info.tycon);
        for defn_gv in &info.tycon_vars {
            let impl_gv_ty = defn_to_impl.substitute_type(&type_from_tyvar(defn_gv.clone()));
            ty = type_tyapp(ty, impl_gv_ty);
        }

        // `validate_opaque_member_impl_signatures` reports an implementation that writes one
        // opaque type variable for two of the declaration, which is what this merge would refuse.
        assert!(sub.merge(&Substitution::single(impl_opaque_name, ty)));
    }

    apply_opaque_substitution(impl_scm, &sub)
}

/// Build the scheme for the #wrap_opaque function.
///
/// #wrap_opaque bridges the concrete implementation type to the opaque type.
/// Its type is `(original_fn_type_with_fresh_vars) -> (rewritten_fn_type_with_opaque_tycons)`.
///
/// Example: for `repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it`:
///   #wrap_opaque : [x : Iterator, Item x = a] (a -> I64 -> x) -> (a -> I64 -> ?it a)
///   where `x` is a fresh variable replacing `?it` in the domain.
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
        let fresh_name = format!(
            "{}{}",
            WRAP_OPAQUE_TYVAR_PREFIX,
            info.tycon.name.to_string()
        );
        let fresh_tv = make_tyvar(&fresh_name, &info.tyvar.kind);
        wrap_gen_vars.push(fresh_tv.clone());
        assert!(opaque_to_fresh.merge(&Substitution::single(
            &info.tyvar.name,
            type_from_tyvar(fresh_tv),
        )));
    }

    // All predicates from the original scheme, with opaque tyvars replaced by fresh vars.
    let wrap_preds: Vec<Predicate> = orig_scm
        .predicates
        .iter()
        .map(|pred| {
            let mut new_pred = pred.clone();
            opaque_to_fresh.substitute_predicate(&mut new_pred);
            new_pred
        })
        .collect();

    // All equalities from the original scheme, with opaque tyvars replaced by fresh vars.
    let wrap_eqs: Vec<Equality> = orig_scm
        .equalities
        .iter()
        .map(|eq| {
            let mut new_eq = eq.clone();
            opaque_to_fresh.substitute_equality(&mut new_eq);
            new_eq
        })
        .collect();

    // Domain type: original function type with opaque tyvars replaced by fresh vars.
    let domain_ty = opaque_to_fresh.substitute_type(&orig_scm.ty);

    // Codomain type: the rewritten function type (with opaque TyCon applications).
    let codomain_ty = new_scm.ty.clone();

    // Wrap type: domain -> codomain.
    let wrap_ty = type_fun(domain_ty, codomain_ty);

    Scheme::new_arc(wrap_gen_vars, vec![], wrap_preds, wrap_eqs, wrap_ty)
}

/// Wrap an expression in a `#wrap_opaque(...)` application.
///
/// The wrapper App inherits the inner expression's source span so that type
/// errors raised while type-checking the body are attributed to the
/// user-written expression rather than appearing without a location.
fn wrap_with_opaque(wrap_name: &FullName, inner: Arc<ExprNode>) -> Arc<ExprNode> {
    let src = inner.source.clone();
    expr_app(expr_var(wrap_name.clone(), None), vec![inner], src)
}

/// Build a placeholder expression for the #wrap_opaque body.
/// Produces `_undefined_internal([])` which type-checks as `a` (any type).
/// #wrap_opaque is removed during instantiation so this is never executed.
fn build_undefined_expr() -> Arc<ExprNode> {
    let mut placeholder_name = FullName::new(
        &NameSpace::new(vec!["Std".to_string()]),
        "_undefined_internal",
    );
    placeholder_name.global_to_absolute();
    let empty_array = expr_array_lit(vec![], None);
    expr_app(expr_var(placeholder_name, None), vec![empty_array], None)
}

/// Replace opaque TyCons in a type with their concrete types.
///
/// Each OpaqueTyConResolution maps `lhs` (e.g., `?it a`) to `rhs` (e.g., `ArrayIterator a`).
/// The lhs is matched against the type to find the appropriate substitution, and
/// the rhs is substituted accordingly.
///
/// Example: `?it (Array I64)` with resolution `?it (Array a) -> ArrayIterator a`
/// is resolved to `ArrayIterator I64`.
// PROOF: P2a, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn resolve_opaque_type_in_type(
    ty: &Arc<TypeNode>,
    opaque_resolutions: &Map<FullName, Vec<OpaqueTyConResolution>>,
) -> Arc<TypeNode> {
    let mut ty = ty.clone();

    // Loop: resolution may produce another opaque tycon at top level.
    loop {
        let tycon = match ty.toplevel_tycon() {
            Some(tycon) => tycon,
            None => break,
        };

        let resolutions = match opaque_resolutions.get(&tycon.name) {
            Some(resolutions) => resolutions,
            None => break, // not an opaque tycon
        };

        let arity = opaque_tycon_arity(resolutions);

        // Split the type args into prefix (arity args) and rest.
        let all_args = ty.collect_type_arguments();
        assert!(
            all_args.len() >= arity,
            "Opaque tycon `{}` expects arity {} but only {} args applied",
            tycon.name.to_string(),
            arity,
            all_args.len()
        );
        let prefix_args: Vec<_> = all_args[..arity]
            .iter()
            .map(|arg| resolve_opaque_type_in_type(arg, opaque_resolutions))
            .collect();
        let rest_args = &all_args[arity..];

        let prefix = apply_type_args(&tycon, &prefix_args);

        // Try matching each resolution's lhs against the prefix.
        let mut matched = false;
        for resolution in resolutions {
            let matching = Substitution::matching_no_kind_check(
                &resolution.lhs,
                &prefix,
                &[], // no fixed tyvars
            );

            if let Some(sub) = matching {
                // Apply the matching to rhs, then apply rest args.
                let rhs = resolution
                    .rhs
                    .as_ref()
                    .expect("opaque type resolution rhs should be filled in by type-checking");
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
                tycon.name.to_string()
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

/// Remove the #wrap_opaque application from the top level of an expression.
/// Transforms `#wrap_opaque(expr)` to `expr`. Only checks the outermost application.
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

/// Recursively replace opaque TyCons in all type annotations of a pattern tree.
fn resolve_opaque_tycon_in_pattern(
    pat: &Arc<PatternNode>,
    opaque_resolutions: &Map<FullName, Vec<OpaqueTyConResolution>>,
) -> Arc<PatternNode> {
    let mut info = pat.info.clone();
    if let Some(ty) = &info.type_ {
        info.type_ = Some(resolve_opaque_type_in_type(ty, opaque_resolutions));
    }
    match &pat.pattern {
        Pattern::Var(v, anno_ty) => Arc::new(PatternNode {
            pattern: Pattern::Var(v.clone(), anno_ty.clone()),
            info,
        }),
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
        Pattern::Union(variant, variant_src, subpat) => Arc::new(PatternNode {
            pattern: Pattern::Union(
                variant.clone(),
                variant_src.clone(),
                resolve_opaque_tycon_in_pattern(subpat, opaque_resolutions),
            ),
            info,
        }),
    }
}

/// Recursively replace opaque TyCons in all type annotations of an expression tree.
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
            expr.set_let_pat(new_pat)
                .set_let_bound(new_val)
                .set_let_value(new_body)
        }
        Expr::If(cond, then_e, else_e) => {
            let new_cond = resolve_opaque_tycon_in_expr(cond, opaque_resolutions);
            let new_then = resolve_opaque_tycon_in_expr(then_e, opaque_resolutions);
            let new_else = resolve_opaque_tycon_in_expr(else_e, opaque_resolutions);
            expr.set_if_cond(new_cond)
                .set_if_then(new_then)
                .set_if_else(new_else)
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
            expr.set_match_cond(new_scrut)
                .set_match_pat_vals(new_branches)
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
        // A variable holds no subexpression. An LLVM expression holds `generic_ty`, which stays
        // written in the type variables of the builtin that carries it, instantiation included.
        Expr::Var(_) | Expr::LLVM(_) => expr,
    }
}
