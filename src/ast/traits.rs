use crate::ast::equality::{Equality, EqualityScheme};
use crate::ast::expr::ExprNode;
use crate::ast::import::ImportStatement;
use crate::ast::name::{FullName, Name};
use crate::ast::predicate::Predicate;
use crate::ast::program::{EndNode, NameResolutionContext, NameResolutionType, TypeEnv};
use crate::ast::qual_pred::{QualPred, QualPredScheme};
use crate::ast::qual_type::QualType;
use crate::ast::types::{
    type_from_tyvar, type_tyvar, Kind, KindEnv, Scheme, TyAssoc, TyVar, TypeNode,
};
use crate::builtin::make_boxed_trait;
use crate::misc::{insert_to_map_vec, number_to_varname, Map, Set};
use crate::sourcefile::{SourcePos, Span};
use crate::typecheck::{Substitution, TypeCheckContext};
use crate::typecheckcache;
use crate::UnifOrOtherErr;
use crate::{ast::collect_annotation_tyvars::collect_annotation_tyvars, error::Errors};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// The identifier of a trait.
#[derive(Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TraitId {
    pub name: FullName,
}

impl TraitId {
    pub fn from_fullname(name: FullName) -> TraitId {
        TraitId { name }
    }

    pub fn to_string(&self) -> String {
        self.namespaced_name().to_string()
    }

    pub fn namespaced_name(&self) -> FullName {
        self.name.clone()
    }

    pub fn resolve_namespace(
        &mut self,
        ctx: &NameResolutionContext,
        span: &Option<Span>,
    ) -> Result<(), Errors> {
        self.name = ctx.resolve(&self.name, &[NameResolutionType::Trait], span)?;
        Ok(())
    }
}

// Definition of associated type.
#[derive(Clone)]
pub struct AssocTypeDefn {
    // The local name of the associated type.
    pub name: Name,
    // Kind predicates on the definition of the associated type.
    pub kind_signs: Vec<KindSignature>,
    // Type parameters of the associated type.
    // Includes `impl_type`.
    pub params: Vec<Arc<TyVar>>,
    // The kind of the application of the associated type.
    pub kind_applied: Arc<Kind>,
    // Source location of associated type definition.
    #[allow(dead_code)]
    pub src: Option<Span>,
}

impl AssocTypeDefn {
    pub fn param_kinds(&self) -> Vec<Arc<Kind>> {
        self.params.iter().map(|p| p.kind.clone()).collect()
    }

    pub fn set_kinds(&mut self, impl_type_kind: Arc<Kind>) {
        // Set `impl_type_kind` to `parms[0]`.
        self.params[0] = self.params[0].set_kind(impl_type_kind.clone());
        // Set `kind_signs` to `self.params`.
        for param in &mut self.params[1..] {
            // Skip `self`.
            for kind_sign in &self.kind_signs {
                if param.name == kind_sign.tyvar {
                    *param = param.set_kind(kind_sign.kind.clone());
                }
            }
        }
    }
}

// Implementation of associated type.
#[derive(Clone)]
pub struct AssocTypeImpl {
    pub name: Name,
    // Type parameters of the associated type implementation.
    // Includes `impl_type`.
    pub params: Vec<Arc<TyVar>>,
    pub value: Arc<TypeNode>,
    pub source: Option<Span>,
}

impl AssocTypeImpl {
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        if self.source.is_none() {
            return None;
        }
        let src = self.source.as_ref().unwrap();
        if !src.includes_pos(pos) {
            return None;
        }
        self.value.find_node_at(pos)
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        self.value = self.value.resolve_type_aliases(type_env)?;
        Ok(())
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        self.value = self.value.resolve_namespace(ctx)?;
        Ok(())
    }

    pub fn set_kinds(&mut self, trait_inst: &TraitImpl, kind_env: &KindEnv) -> Result<(), Errors> {
        let assoc_ty_name = TyAssoc {
            name: FullName::new(&trait_inst.trait_id().name.to_namespace(), &self.name),
        };
        let param_kinds = &kind_env.assoc_tys.get(&assoc_ty_name).unwrap().param_kinds;
        if self.params.len() != param_kinds.len() {
            return Err(Errors::from_msg_srcs(
                format!(
                    "Invalid number of parameters for associated type `{}`. Expect: {}, found: {}.",
                    self.name,
                    param_kinds.len(),
                    self.params.len()
                ),
                &[&self.source],
            ));
        }
        let mut tvs_in_value = vec![];
        trait_inst.impl_type().free_vars_to_vec(&mut tvs_in_value);
        for (param, kind) in &mut self.params[1..].iter_mut().zip(param_kinds[1..].iter()) {
            *param = param.set_kind(kind.clone());
            tvs_in_value.push(param.clone());
        }
        let mut tv_to_kind = Map::default();
        for tv_in_value in tvs_in_value {
            tv_to_kind.insert(tv_in_value.name.clone(), tv_in_value.kind.clone());
        }
        self.value = self.value.set_kinds(&tv_to_kind);
        Ok(())
    }
}

#[derive(Clone)]
pub struct AssocTypeKindInfo {
    #[allow(dead_code)]
    pub name: TyAssoc,
    pub param_kinds: Vec<Arc<Kind>>, // Includes `self`.
    pub value_kind: Arc<Kind>,
}

// Trait member.
#[derive(Clone)]
pub struct TraitMember {
    pub name: Name,
    // The type of the member.
    // Here, for example, in case "trait a : Show { show : a -> String }",
    // the type of method "show" is "a -> String",
    // and not "[a : Show] a -> String".
    pub qual_ty: QualType,
    // The type of the member, but with aliases retained.
    pub syn_qual_ty: Option<QualType>,
    pub source: Option<Span>,
    // Document of this member.
    // This field is used only If document from `source` is not available.
    pub document: Option<String>,
}

impl TraitMember {
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        if self.source.is_none() {
            return None;
        }
        let src = self.source.as_ref().unwrap();
        if !src.includes_pos(pos) {
            return None;
        }
        self.qual_ty.find_node_at(pos)
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        self.qual_ty.resolve_namespace(ctx)
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        self.syn_qual_ty = Some(self.qual_ty.clone());
        self.qual_ty.resolve_type_aliases(type_env)
    }
}

// Traits definitions.
#[derive(Clone)]
pub struct TraitDefn {
    // Identifier of this trait (i.e. the name).
    pub trait_: TraitId,
    // Type variable used in trait definition.
    pub type_var: Arc<TyVar>,
    // Members of this trait.
    pub members: Vec<TraitMember>,
    // Associated types.
    pub assoc_types: Map<Name, AssocTypeDefn>,
    // Kind signatures at the trait declaration, e.g., "f: *->*" in "trait [f:*->*] f: Functor {}".
    pub kind_signs: Vec<KindSignature>,
    // Source location of trait definition.
    pub source: Option<Span>,
    // Document of this trait.
    // This field is used only If document from `source` is not available.
    pub document: Option<String>,
}

impl TraitDefn {
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        for mi in &self.members {
            let node = mi.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        None
    }

    // Get the document of this trait.
    pub fn get_document(&self) -> Option<String> {
        // Try to get document from the source code.
        let docs = self.source.as_ref().and_then(|src| src.get_document().ok());

        // If the documentation is empty, treat it as None.
        let docs = match docs {
            Some(docs) if docs.is_empty() => None,
            _ => docs,
        };

        // If the document is not available in the source code, use the document field.
        let docs = match docs {
            Some(_) => docs,
            None => self.document.clone(),
        };

        // Again, if the documentation is empty, treat it as None.
        match docs {
            Some(docs) if docs.is_empty() => None,
            _ => docs,
        }
    }

    // Resolve namespace.
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for mi in &mut self.members {
            errors.eat_err(mi.resolve_namespace(ctx));
        }
        errors.to_result()
    }

    // Resolve type aliases
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for mi in &mut self.members {
            errors.eat_err(mi.resolve_type_aliases(type_env));
        }
        errors.to_result()
    }

    // Get type-scheme of a member.
    // Here, for example, in case "trait a: ToString { to_string : a -> String }",
    // this function returns "[a: ToString] a -> String" as type of "to_string" member.
    pub fn member_scheme(&self, name: &Name, syntactic: bool) -> Arc<Scheme> {
        let member = self.members.iter().find(|mi| mi.name == *name).unwrap();
        let mut qual_ty = if syntactic {
            member.syn_qual_ty.as_ref().unwrap().clone()
        } else {
            member.qual_ty.clone()
        };
        let mut vars = vec![];
        qual_ty.free_vars_vec(&mut vars);
        let mut preds = vec![Predicate::make(
            self.trait_.clone(),
            type_from_tyvar(self.type_var.clone()),
        )];
        preds.append(&mut qual_ty.preds);
        Scheme::generalize(&qual_ty.kind_signs, preds, qual_ty.eqs, qual_ty.ty)
    }

    // Get the type of a member.
    // Here, for example, in case "trait a: ToString { to_string: a -> String }",
    // this function returns "a -> String" as type of "to_string" member.
    pub fn member_ty(&self, name: &Name) -> QualType {
        self.members
            .iter()
            .find(|mi| mi.name == *name)
            .unwrap()
            .qual_ty
            .clone()
    }

    // Validate kind_signs and set it to self.type_var.
    // Also, set kinds of parameters of associated type definition.
    pub fn set_trait_kind(&mut self) -> Result<(), Errors> {
        if self.kind_signs.len() >= 2 {
            let span = Span::unite_opt(&self.kind_signs[0].source, &self.kind_signs[1].source);
            return Err(Errors::from_msg_srcs(
                "You can specify at most one constraint of the form `{type-variable} : {kind}` as the assumption of trait definition.".to_string(),
                &[&span],
            ));
        }
        if self.kind_signs.len() > 0 {
            if self.kind_signs[0].tyvar != self.type_var.name {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "The type variable used in the assumption of trait `{}` has to be `{}`.",
                        self.trait_.to_string(),
                        self.type_var.name,
                    ),
                    &[&self.kind_signs[0].source],
                ));
            }
            self.type_var = self.type_var.set_kind(self.kind_signs[0].kind.clone());
        }
        for (_, assoc_ty_defn) in &mut self.assoc_types {
            assoc_ty_defn.set_kinds(self.type_var.kind.clone());
        }
        Ok(())
    }
}

// Trait implementation
#[derive(Clone)]
pub struct TraitImpl {
    // Statement such as "[a: Show, b: Show] (a, b): Show".
    pub qual_pred: QualPred,
    // Member implementation.
    pub members: Map<Name, Arc<ExprNode>>,
    // Type signatures of members, if provided by user.
    pub member_sigs: Map<Name, QualType>,
    // Associated type synonym implementation.
    pub assoc_types: Map<Name, AssocTypeImpl>,
    // Module where this instance is defined.
    pub define_module: Name,
    // Source location where this instance is defined.
    pub source: Option<Span>,
    // Is this instance implememted by user? (not by compiler)
    pub is_user_defined: bool,
}

impl TraitImpl {
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        let node = self.qual_pred.find_node_at(pos);
        if node.is_some() {
            return node;
        }
        for (_assoc_ty_name, assoc_ty_impl) in &self.assoc_types {
            let node = assoc_ty_impl.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        None
    }

    pub fn set_kinds_in_qual_pred_and_member_sigs(
        &mut self,
        kind_env: &KindEnv,
    ) -> Result<(), Errors> {
        let mut scope = Map::default();
        let preds = &self.qual_pred.pred_constraints;
        let eqs = &self.qual_pred.eq_constraints;
        let kind_signs = &self.qual_pred.kind_constraints;
        let res = QualPred::extend_kind_scope(&mut scope, preds, eqs, kind_signs, kind_env);
        if res.is_err() {
            return Err(Errors::from_msg_srcs(res.unwrap_err(), &[&self.source]));
        }
        self.qual_pred.predicate.set_kinds(&scope);
        for pred in &mut self.qual_pred.pred_constraints {
            pred.set_kinds(&scope);
        }
        for eq in &mut self.qual_pred.eq_constraints {
            eq.set_kinds(&scope);
        }
        for (_member_name, member_sig) in &mut self.member_sigs {
            let mut scope = scope.clone();
            let res = QualPred::extend_kind_scope(
                &mut scope,
                &member_sig.preds,
                &member_sig.eqs,
                &member_sig.kind_signs,
                kind_env,
            );
            if res.is_err() {
                return Err(Errors::from_msg_srcs(
                    res.unwrap_err(),
                    &[&member_sig.ty.get_source()],
                ));
            }
            member_sig.ty = member_sig.ty.set_kinds(&scope);
            for pred in &mut member_sig.preds {
                pred.set_kinds(&scope);
            }
            for eq in &mut member_sig.eqs {
                eq.set_kinds(&scope);
            }
        }
        Ok(())
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        self.qual_pred.resolve_namespace(ctx)?;

        let mut errors = Errors::empty();
        for (_assoc_ty_name, assoc_ty_impl) in &mut self.assoc_types {
            errors.eat_err(assoc_ty_impl.resolve_namespace(ctx));
        }
        for (_member_name, member_sig) in &mut self.member_sigs {
            errors.eat_err(member_sig.resolve_namespace(ctx));
        }

        errors.to_result()

        // This function is called only by resolve_namespace_in_declaration, so we don't need to see into expression.
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        errors.eat_err(self.qual_pred.resolve_type_aliases(type_env));
        for (_assoc_ty_name, assoc_ty_impl) in &mut self.assoc_types {
            errors.eat_err(assoc_ty_impl.resolve_type_aliases(type_env));
        }
        for (_member_name, member_sig) in &mut self.member_sigs {
            errors.eat_err(member_sig.resolve_type_aliases(type_env));
        }
        errors.to_result()
    }

    // Get trait id.
    fn trait_id(&self) -> TraitId {
        self.qual_pred.predicate.trait_id.clone()
    }

    // Get mutable trait id.
    fn trait_id_mut(&mut self) -> &mut TraitId {
        &mut self.qual_pred.predicate.trait_id
    }

    // Get type-scheme of a member implementation.
    // Here, for example, in case "impl [a: ToString, b: ToString] (a, b): ToString",
    // this function returns "[a: ToString, b: ToString] (a, b) -> String" as the type of "to_string".
    //
    // Users can also write type annotations in trait implementations.
    // This function trusts and returns the type annotation if the user has written one.
    pub fn member_scheme(&self, member: &Name, trait_defn: &TraitDefn) -> Arc<Scheme> {
        if let Some(qual_ty) = self.member_sigs.get(member) {
            // If type annotation is provided by user, use it.
            let mut preds = self.qual_pred.pred_constraints.clone();
            preds.extend(qual_ty.preds.clone());

            let mut eqs = self.qual_pred.eq_constraints.clone();
            eqs.extend(qual_ty.eqs.clone());

            let mut kind_signs = self.qual_pred.kind_constraints.clone();
            kind_signs.extend(qual_ty.kind_signs.clone());

            Scheme::generalize(&kind_signs, preds, eqs, qual_ty.ty.clone())
        } else {
            // Otherwise, construct the type from trait definition and impl declaration.
            self.member_scheme_by_defn(member, trait_defn)
        }
    }

    // Get type-scheme of a method implementation.
    // Here, for example, in case "impl [a: ToString, b: ToString] (a, b): ToString",
    // this function returns "[a: ToString, b: ToString] (a, b) -> String" as the type of "to_string".
    //
    // Users can also write type annotations in trait implementations.
    // The `by_defn` means to ignore type annotations and construct the type from trait definition and impl declaration.
    fn member_scheme_by_defn(&self, method_name: &Name, trait_defn: &TraitDefn) -> Arc<Scheme> {
        // First, see the trait definition.
        // Let's consider `trait a : ToString { to_string : a -> String }`.
        let tv = &trait_defn.type_var.name; // `a` in the above example.
        let mut method_qualty = trait_defn.member_ty(method_name); // `a -> String` in the above example.

        // Next, see the trait implementation to get the type for which the trait is implemented.
        let impl_type = self.impl_type(); // `(a, b)` in the above example.

        // We are going to substitute `tv` (e.g., `a`) in `method_qualty` (e.g., `a -> String`) with `impl_type` (e.g., `(a, b)`)
        // This is OK if FV(method_qualty) \ {tv} is disjoint from FV(impl_type).
        // Otherwise, we need to rename the type variables in `method_qualty` to avoid name collision.
        // Example:
        // Consider `impl Arrow a : Functor` for `trait f : Functor { map : (a -> b) -> f a -> f b }`.
        // In this case, if we naively substitute `f` in `map : (a -> b) -> f a -> f b` with `Arrow a`,
        // then we get `map : (a -> b) -> Arrow a a -> Arrow a b`, which is wrong.
        // So we first rename `(a -> b) -> f a -> f b` to `(c -> b) -> f c -> f b`.
        let mut fv_method_quality = vec![];
        method_qualty.free_vars_vec(&mut fv_method_quality);
        let fv_impl_type = impl_type.free_vars();
        let mut s = Substitution::default();
        let mut name_no = -1;
        for fv in &fv_method_quality {
            if &fv.name == tv {
                continue;
            }
            if fv_impl_type.contains_key(&fv.name) {
                // Search for a new name that is not in `fv_impl_type`.
                loop {
                    name_no += 1;
                    let new_name = number_to_varname(name_no as usize);
                    if !fv_impl_type.contains_key(&new_name)
                        && fv_method_quality.iter().all(|x| x.name != new_name)
                    {
                        let new_fv = type_tyvar(&new_name, &fv.kind);
                        let merge_succ =
                            s.merge_substitution(&Substitution::single(&fv.name, new_fv));
                        assert!(merge_succ);
                        break;
                    }
                }
            }
        }
        // Rename type variables in `method_qualty`.
        s.substitute_qualtype(&mut method_qualty);

        // Then substitute `tv` with `impl_type`.
        // Now we get `(a, b) -> String` or `(c -> b) -> Arrow a c -> Arrow a b` in the above examples.
        let s = Substitution::single(&tv, impl_type);
        s.substitute_qualtype(&mut method_qualty);

        // Prepare `vars`, `ty`, `preds`, and `eqs` to be generalized.
        let ty = method_qualty.ty.clone();
        let mut kind_signs = self.qual_pred.kind_constraints.clone();
        kind_signs.append(&mut method_qualty.kind_signs.clone());
        let mut preds = self.qual_pred.pred_constraints.clone();
        preds.append(&mut method_qualty.preds);
        let mut eqs = self.qual_pred.eq_constraints.clone();
        eqs.append(&mut method_qualty.eqs);

        // Set source location of the type to the location where the method is implemented.
        let source = self
            .member_expr(method_name)
            .source
            .as_ref()
            .map(|src| src.to_head_character());
        let ty = ty.set_source(source);

        Scheme::generalize(&kind_signs, preds, eqs, ty)
    }

    // Get expression that implements a member.
    pub fn member_expr(&self, name: &Name) -> Arc<ExprNode> {
        self.members.get(name).unwrap().clone()
    }

    // Get the type implementing the trait.
    pub fn impl_type(&self) -> Arc<TypeNode> {
        self.qual_pred.predicate.ty.clone()
    }
}

// Trait Aliases
#[derive(Clone)]
pub struct TraitAlias {
    // Identifier of this trait (i.e., the name).
    pub id: TraitId,
    // Aliased traits and its source span.
    pub value: Vec<(TraitId, Span)>,
    // Source location of alias definition.
    pub source: Option<Span>,
    // Kind of this trait alias.
    pub kind: Arc<Kind>,
}

impl TraitAlias {
    // Get the document of this trait.
    pub fn get_document(&self) -> Option<String> {
        self.source.as_ref().and_then(|src| src.get_document().ok())
    }

    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        for (t, s) in &self.value {
            if s.includes_pos(pos) {
                return Some(EndNode::Trait(t.clone()));
            }
        }
        None
    }

    // Resolve namespace of trait names in value.
    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        for (trait_id, _) in &mut self.value {
            trait_id.resolve_namespace(ctx, &self.source)?;
        }
        Ok(())
    }
}

// Statement such as "f: * -> *".
#[derive(Clone)]
pub struct KindSignature {
    pub tyvar: Name,
    pub kind: Arc<Kind>,
    pub source: Option<Span>,
}

impl KindSignature {
    pub fn to_string(&self) -> String {
        format!("{} : {}", self.tyvar, self.kind.to_string())
    }
}

// Trait alias environment.
#[derive(Clone, Default)]
pub struct TraitAliasEnv {
    pub data: Map<TraitId, TraitAlias>,
}

impl TraitAliasEnv {
    // Check if a trait name is an alias.
    pub fn is_alias(&self, trait_id: &TraitId) -> bool {
        self.data.contains_key(trait_id)
    }

    // Resolve trait aliases.
    pub fn resolve_alias(&self, trait_id: &TraitId) -> Result<Vec<TraitId>, Errors> {
        fn resolve_alias_internal(
            env: &TraitAliasEnv,
            trait_id: &TraitId,
            res: &mut Vec<TraitId>,
            visited: &mut Set<TraitId>,
        ) -> Result<(), Errors> {
            if visited.contains(trait_id) {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "Circular aliasing detected in trait alias `{}`.",
                        trait_id.to_string()
                    ),
                    &[&env.data.get(trait_id).map(|ta| ta.source.clone()).flatten()],
                ));
            }
            visited.insert(trait_id.clone());
            if !env.is_alias(trait_id) {
                res.push(trait_id.clone());
                return Ok(());
            }
            for (t, _) in &env.data.get(trait_id).unwrap().value {
                resolve_alias_internal(env, t, res, visited)?;
            }
            Ok(())
        }

        let mut res = vec![];
        let mut visited = Set::default();
        resolve_alias_internal(self, trait_id, &mut res, &mut visited)?;
        Ok(res)
    }
}

// Trait environments.
#[derive(Clone, Default)]
pub struct TraitEnv {
    pub traits: Map<TraitId, TraitDefn>,
    pub impls: Map<TraitId, Vec<TraitImpl>>,
    pub aliases: TraitAliasEnv,
}

impl TraitEnv {
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        for (_t, ti) in &self.traits {
            let node = ti.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        for (_, insts) in &self.impls {
            for inst in insts {
                let node = inst.find_node_at(pos);
                if node.is_some() {
                    return node;
                }
            }
        }
        for (_, alias) in &self.aliases.data {
            let node = alias.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        None
    }

    // Get of list of trait names including aliases.
    pub fn trait_names(&self) -> Set<FullName> {
        self.traits_with_aliases()
            .into_iter()
            .map(|t| t.name)
            .collect()
    }

    pub fn traits_with_aliases(&self) -> Vec<TraitId> {
        let mut res = vec![];
        for (k, _v) in &self.traits {
            res.push(k.clone());
        }
        for (k, _v) in &self.aliases.data {
            res.push(k.clone());
        }
        res
    }

    pub fn validate(&self, kind_env: KindEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Check name confliction of traits and aliases.
        fn create_conflicting_error(env: &TraitEnv, trait_id: &TraitId) -> Errors {
            let this_src = &env.traits.get(trait_id).unwrap().source;
            let other_src = &env.aliases.data.get(trait_id).unwrap().source;
            Errors::from_msg_srcs(
                format!("Duplicate definition for `{}`", trait_id.to_string()),
                &[this_src, other_src],
            )
        }

        for (trait_id, _) in &self.traits {
            if self.aliases.data.contains_key(trait_id) {
                errors.append(create_conflicting_error(self, trait_id));
            }
        }
        for (trait_id, _) in &self.aliases.data {
            if self.traits.contains_key(trait_id) {
                errors.append(create_conflicting_error(self, trait_id));
            }
        }

        // Validate trait aliases.
        // Check that traits that appear in values of trait aliases define actually exist.
        for (_, ta) in &self.aliases.data {
            for (t, _) in &ta.value {
                if !self.traits.contains_key(t) && !self.aliases.data.contains_key(t) {
                    errors.append(Errors::from_msg_srcs(
                        format!("Unknown trait `{}`.", t.to_string()),
                        &[&ta.source],
                    ));
                }
            }
        }
        // If some errors are found upto here, throw them.
        errors.to_result()?;

        // Circular aliasing will be detected in `TraitEnv::resolve_aliases`, so we don't need to check it here.

        // Forbid unrelated trait member:
        // Check that the type variable in trait definition appears each of the members' type.
        // This assumption is used in `InstanciatedSymbol::dependent_modules`.
        for (_trait_id, trait_defn) in &self.traits {
            for member in &trait_defn.members {
                if !member.qual_ty.ty.contains_tyvar(&trait_defn.type_var) {
                    errors.append(Errors::from_msg_srcs(
                        format!(
                            "Type variable `{}` used in trait definition has to appear in the type of a member `{}`.",
                            trait_defn.type_var.name,
                            member.name,
                        ),
                        &[&member.qual_ty.ty.get_source()],
                    ));
                }
            }
        }
        // If some errors are found upto here, throw them.
        errors.to_result()?;

        // Prepare TypeCheckContext to use `unify`.
        let tc = TypeCheckContext::new(
            TraitEnv::default(),
            TypeEnv::default(),
            kind_env,
            Map::default(),
            Arc::new(typecheckcache::FileCache::new()),
            0,
        );
        // Validate trait implementations.
        for (trait_id, impls) in &self.impls {
            for impl_ in impls.iter() {
                // check implementation is given for trait, not for trait alias.
                if self.aliases.is_alias(&trait_id) {
                    return Err(Errors::from_msg_srcs(
                        "A trait alias cannot be implemented directly. Implement each aliased trait instead.".to_string(),
                        &[&impl_.qual_pred.predicate.source],
                    ));
                }
                // Now `trait_id` is not an alias, so get the trait definition.
                let defn = self.traits.get(trait_id).unwrap();
                errors.eat_err(Self::validate_trait_impl(impl_, defn, &self.aliases));
            }
            // Throw errors if any.
            errors.to_result()?;

            // Check overlapping instance.
            for i in 0..impls.len() {
                for j in (i + 1)..impls.len() {
                    let inst_i = &impls[i];
                    let inst_j = &impls[j];
                    let mut tc = tc.clone();
                    if UnifOrOtherErr::extract_others(
                        tc.unify(&inst_i.impl_type(), &inst_j.impl_type()),
                    )?
                    .is_err()
                    {
                        continue;
                    }
                    let mut msg = format!(
                        "Two trait implementations for `{}` are overlapping.",
                        trait_id.to_string()
                    );
                    if inst_i.trait_id() == make_boxed_trait() {
                        msg +=
                            "NOTE: `Std::Boxed` is automatically implemented for all boxed types by compiler."
                    }
                    errors.append(Errors::from_msg_srcs(
                        msg,
                        &[
                            &inst_i.source.as_ref().map(|s| s.to_head_character()),
                            &inst_j.source.as_ref().map(|s| s.to_head_character()),
                        ],
                    ));
                }
            }
        }

        errors.to_result()
    }

    fn validate_trait_impl(
        impl_: &TraitImpl,
        defn: &TraitDefn,
        aliases: &TraitAliasEnv,
    ) -> Result<(), Errors> {
        let trait_id = &defn.trait_;

        // Check instance head.
        let implemented_ty = &impl_.qual_pred.predicate.ty;
        if !implemented_ty.is_implementable() {
            return Err(Errors::from_msg_srcs(
                        format!(
                            "Implementing trait for type `{}` is not allowed. \
                            The head (in this case, `{}`) of the type should be a type constructor.",
                            implemented_ty.to_string(),
                            implemented_ty.get_head_string(),
                        ),
                        &[&implemented_ty.get_source()],
                    ));
        }

        // Validate the set of trait members.
        let trait_members = &defn.members;
        let impl_members = &impl_.members;
        let member_sigs = &impl_.member_sigs;
        for trait_member in trait_members {
            if !impl_members.contains_key(&trait_member.name) {
                return Err(Errors::from_msg_srcs(
                    format!("Lacking implementation of member `{}`.", trait_member.name),
                    &[&impl_.source],
                ));
            }
        }
        for (impl_member, impl_expr) in impl_members {
            if trait_members
                .iter()
                .find(|mi| mi.name == *impl_member)
                .is_none()
            {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "`{}` is not a member of trait `{}`.",
                        impl_member,
                        trait_id.to_string(),
                    ),
                    &[&impl_expr.source],
                ));
            }
        }

        // Validate the set of associated types.
        let trait_assoc_types = &defn.assoc_types;
        let impl_assoc_types = &impl_.assoc_types;
        for (trait_assoc_type, _) in trait_assoc_types {
            if !impl_assoc_types.contains_key(trait_assoc_type) {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "Lacking implementation of associated type `{}`.",
                        trait_assoc_type,
                    ),
                    &[&impl_.source],
                ));
            }
        }
        for (impl_assoc_type, impl_info) in impl_assoc_types {
            if !trait_assoc_types.contains_key(impl_assoc_type) {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "`{}` is not an associated type of trait `{}`.",
                        impl_assoc_type,
                        trait_id.to_string(),
                    ),
                    &[&impl_info.source],
                ));
            }
            // Validate free variable of associated type implementation.
            let mut allowed_tyvars = vec![];
            impl_.impl_type().free_vars_to_vec(&mut allowed_tyvars);
            for arg in &impl_info.params {
                allowed_tyvars.push(arg.clone());
            }
            for used_tv in impl_info.value.free_vars_vec() {
                if allowed_tyvars
                    .iter()
                    .all(|allowed_tv| allowed_tv.name != used_tv.name)
                {
                    return Err(Errors::from_msg_srcs(
                        format!("Unknown type variable `{}`.", used_tv.name),
                        &[&impl_info.source],
                    ));
                }
            }
        }

        // For members without type signature, type variables used in type annotations in the member
        // must appear in the type being implemented.
        for (method_name, method_expr) in impl_members {
            if !member_sigs.contains_key(method_name) {
                let mut allowed_tyvars = vec![];
                impl_.impl_type().free_vars_to_vec(&mut allowed_tyvars);
                for (used_tv, tv_src) in collect_annotation_tyvars(&method_expr) {
                    if allowed_tyvars
                        .iter()
                        .all(|allowed_tv| allowed_tv.name != used_tv.name)
                    {
                        return Err(Errors::from_msg_srcs(
                            format!("Unknown type variable `{}`.", used_tv.name),
                            &[&tv_src],
                        ));
                    }
                }
            }
        }

        // Validate member type signatures.
        for (member_name, member_sig) in member_sigs {
            // Check the member is defined in the trait.
            if !trait_members
                .iter()
                .find(|mi| &mi.name == member_name)
                .is_some()
            {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "`{}` is not a member of trait `{}`.",
                        member_name,
                        trait_id.to_string(),
                    ),
                    &[&member_sig.ty.get_source()],
                ));
            }

            // Check the member type signature is equivalent to the one in the trait definition.
            let trait_defn = &defn;
            let type_by_defn = impl_.member_scheme_by_defn(member_name, trait_defn);
            let type_by_sig = impl_.member_scheme(member_name, trait_defn);
            if !Scheme::equivalent(&type_by_defn, &type_by_sig, &aliases)? {
                return Err(Errors::from_msg_srcs(
                            format!(
                                "Type signature of member `{}` is not equivalent to the one in the trait definition. \
                                Expected: `{}`, found: `{}`.",
                                member_name,
                                type_by_defn.to_string(),
                                type_by_sig.to_string(),
                            ),
                            &[&member_sig.ty.get_source()],
                        ));
            }
        }

        // Check Orphan rules.
        let instance_def_mod = &impl_.define_module;
        let trait_def_id = trait_id.name.module();
        let ty = &impl_.qual_pred.predicate.ty;
        let type_def_id = ty.toplevel_tycon().unwrap().name.module();
        if trait_def_id != *instance_def_mod && type_def_id != *instance_def_mod {
            return Err(Errors::from_msg_srcs(
                format!(
                    "Implementing trait `{}` for type `{}` in module `{}` is illegal; \
                            it is not allowed to implement an external trait for an external type.",
                    trait_id.to_string(),
                    ty.to_string_normalize(),
                    instance_def_mod.to_string(),
                ),
                &[&impl_.source.as_ref().map(|s| s.to_head_character())],
            ));
        }

        // Check `Std::Boxed` is not implemented by user.
        if trait_id == &make_boxed_trait() && impl_.is_user_defined {
            return Err(Errors::from_msg_srcs(
                        "Implementing `Std::Boxed` by hand is not allowed. It is automatically implemented for all boxed types by compiler.".to_string(),
                        &[&impl_.source],
                    ));
        }
        Ok(())
    }

    pub fn resolve_namespace(
        &mut self,
        ctx: &mut NameResolutionContext,
        imported_modules: &Map<Name, Vec<ImportStatement>>,
    ) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Resolve names in trait aliases.
        for (trait_id, alias_info) in &mut self.aliases.data {
            ctx.import_statements = imported_modules[&trait_id.name.module()].clone();
            errors.eat_err(alias_info.resolve_namespace(ctx));
        }
        errors.to_result()?; // Throw errors if any.

        // Resolve names in trait definitions.
        for (trait_id, trait_info) in &mut self.traits {
            ctx.import_statements = imported_modules[&trait_id.name.module()].clone();
            // Keys in self.traits should already be resolved.
            assert!(
                trait_id.name
                    == ctx
                        .resolve(&trait_id.name, &[NameResolutionType::Trait], &None)
                        .ok()
                        .unwrap()
            );
            errors.eat_err(trait_info.resolve_namespace(ctx));
        }
        errors.to_result()?; // Throw errors if any.

        // Resolve names in trait implementations.
        let impls = std::mem::replace(&mut self.impls, Default::default());
        let mut new_impls: Map<TraitId, Vec<TraitImpl>> = Default::default();
        for (trait_id, insts) in impls {
            for mut inst in insts {
                // Set up NameResolutionContext.
                ctx.import_statements = imported_modules[&inst.define_module].clone();

                // Resolve trait_id's namespace.
                let mut trait_id = trait_id.clone();
                errors.eat_err(
                    trait_id.resolve_namespace(ctx, &inst.qual_pred.predicate.source.clone()),
                );

                // Resolve names in TrantImpl
                inst.trait_id_mut().name = trait_id.name.clone(); // This is a "just in case" process, and may not be necessary.
                errors.eat_err(inst.resolve_namespace(ctx));

                // Insert to new_impls
                if !new_impls.contains_key(&trait_id) {
                    new_impls.insert(trait_id.clone(), vec![]);
                }
                new_impls.get_mut(&trait_id).unwrap().push(inst);
            }
        }

        errors.to_result()?; // Throw errors if any.
        self.impls = new_impls;
        Ok(())
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Resolve aliases in trait definitions.
        for (_, trait_info) in &mut self.traits {
            errors.eat_err(trait_info.resolve_type_aliases(type_env));
        }

        // Resolve aliases in trait implementations.
        let impls = std::mem::replace(&mut self.impls, Default::default());
        let mut new_impls: Map<TraitId, Vec<TraitImpl>> = Default::default();
        for (trait_id, insts) in impls {
            for mut inst in insts {
                // Resolve names in TrantInstance.
                errors.eat_err(inst.resolve_type_aliases(type_env));

                // Insert to new_impls
                if !new_impls.contains_key(&trait_id) {
                    new_impls.insert(trait_id.clone(), vec![]);
                }
                new_impls.get_mut(&trait_id).unwrap().push(inst);
            }
        }
        errors.to_result()?; // Throw errors if any.
        self.impls = new_impls;
        Ok(())
    }

    // Add traits.
    pub fn add(
        &mut self,
        trait_infos: Vec<TraitDefn>,
        trait_impls: Vec<TraitImpl>,
        trait_aliases: Vec<TraitAlias>,
    ) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for trait_info in trait_infos {
            errors.eat_err(self.add_trait(trait_info));
        }
        for trait_impl in trait_impls {
            errors.eat_err(self.add_instance(trait_impl));
        }
        for trait_alias in trait_aliases {
            errors.eat_err(self.add_alias(trait_alias));
        }
        errors.to_result()
    }

    // Add a trait to TraitEnv.
    pub fn add_trait(&mut self, info: TraitDefn) -> Result<(), Errors> {
        // Check Duplicate definition.
        if self.traits.contains_key(&info.trait_) {
            let info1 = self.traits.get(&info.trait_).unwrap();
            return Err(Errors::from_msg_srcs(
                format!(
                    "Duplicate definition for trait {}.",
                    info.trait_.to_string()
                ),
                &[&info1.source, &info.source],
            ));
        }
        self.traits.insert(info.trait_.clone(), info);
        Ok(())
    }

    // Add an instance.
    pub fn add_instance(&mut self, inst: TraitImpl) -> Result<(), Errors> {
        let trait_id = inst.trait_id();
        if !self.impls.contains_key(&trait_id) {
            self.impls.insert(trait_id.clone(), vec![]);
        }
        self.impls.get_mut(&trait_id).unwrap().push(inst);
        Ok(())
    }

    // Add an trait alias.
    fn add_alias(&mut self, alias: TraitAlias) -> Result<(), Errors> {
        // Check duplicate definition.
        if self.aliases.data.contains_key(&alias.id) {
            let alias1 = self.aliases.data.get(&alias.id).unwrap();
            return Err(Errors::from_msg_srcs(
                format!(
                    "Duplicate definition for trait alias {}.",
                    alias.id.to_string()
                ),
                &[&alias1.source, &alias.source],
            ));
        }
        self.aliases.data.insert(alias.id.clone(), alias);
        Ok(())
    }

    pub fn qualified_predicates(&self) -> Map<TraitId, Vec<QualPredScheme>> {
        let mut qps = Map::default();
        for (trait_id, insts) in &self.impls {
            for inst in insts {
                let mut vars = vec![];
                inst.qual_pred.free_vars_vec(&mut vars);
                insert_to_map_vec(
                    &mut qps,
                    trait_id,
                    QualPredScheme {
                        gen_vars: vars,
                        qual_pred: inst.qual_pred.clone(),
                    },
                );
            }
        }
        qps
    }

    // From implementation of associated types, get generalized type equalities.
    pub fn type_equalities(&self) -> Map<TyAssoc, Vec<EqualityScheme>> {
        let mut eq_scms = Map::default();
        for (trait_id, insts) in &self.impls {
            for inst in insts {
                for (assoc_type_name, assoc_type_impl) in &inst.assoc_types {
                    let assoc_type_namespace = trait_id.name.to_namespace();
                    let assoc_type_fullname = FullName::new(&assoc_type_namespace, assoc_type_name);
                    let impl_type = inst.impl_type();
                    let mut args = vec![impl_type];
                    for tv in &assoc_type_impl.params[1..] {
                        args.push(type_from_tyvar(tv.clone()));
                    }
                    let equality = Equality {
                        assoc_type: TyAssoc {
                            name: assoc_type_fullname,
                        },
                        args,
                        value: assoc_type_impl.value.clone(),
                        source: assoc_type_impl.source.clone(),
                    };
                    insert_to_map_vec(&mut eq_scms, &equality.assoc_type, equality.generalize());
                }
            }
        }
        eq_scms
    }

    pub fn assoc_ty_to_arity(&self) -> Map<FullName, usize> {
        let mut assoc_ty_arity = Map::default();
        for (trait_id, trait_info) in &self.traits {
            for (assoc_ty_name, assoc_ty_info) in &trait_info.assoc_types {
                let assoc_type_namespace = trait_id.name.to_namespace();
                let assoc_type_fullname = FullName::new(&assoc_type_namespace, &assoc_ty_name);
                let arity = assoc_ty_info.params.len();
                assoc_ty_arity.insert(assoc_type_fullname, arity);
            }
        }
        assoc_ty_arity
    }

    pub fn assoc_ty_kind_info(&self) -> Map<TyAssoc, AssocTypeKindInfo> {
        let mut assoc_ty_kind_info = Map::default();
        for (trait_id, trait_info) in &self.traits {
            for (assoc_ty_name, assoc_ty_info) in &trait_info.assoc_types {
                let assoc_type_namespace = trait_id.name.to_namespace();
                let assoc_type = TyAssoc {
                    name: FullName::new(&assoc_type_namespace, &assoc_ty_name),
                };
                assoc_ty_kind_info.insert(
                    assoc_type.clone(),
                    AssocTypeKindInfo {
                        name: assoc_type,
                        param_kinds: assoc_ty_info.param_kinds(),
                        value_kind: assoc_ty_info.kind_applied.clone(),
                    },
                );
            }
        }
        assoc_ty_kind_info
    }

    // Set kinds in Trait definitions and TraitAlias definitions.
    pub fn set_kinds_in_trait_and_alias_defns(&mut self) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Set kinds in trait definitions.
        for (_id, ti) in &mut self.traits {
            errors.eat_err(ti.set_trait_kind());
        }

        // Throw errors if any.
        errors.to_result()?;

        // Set kinds in trait aliases definitions.
        let mut resolved_aliases: Map<TraitId, Vec<TraitId>> = Map::default();
        for (id, _) in &self.aliases.data {
            resolved_aliases.insert(id.clone(), self.aliases.resolve_alias(id)?);
            // If circular aliasing is detected, throw it immediately.
        }
        for (id, ta) in &mut self.aliases.data {
            let mut kinds = resolved_aliases
                .get(id)
                .unwrap()
                .iter()
                .map(|id| self.traits.get(id).unwrap().type_var.kind.clone());
            let kind = kinds.next().unwrap();
            for k in kinds {
                if k != kind {
                    errors.append(Errors::from_msg_srcs(
                        format!(
                            "Kind mismatch in the definition of trait alias `{}`.",
                            id.to_string()
                        ),
                        &[&ta.source],
                    ));
                }
            }
            ta.kind = kind;
        }
        errors.to_result()
    }

    pub fn set_kinds_in_trait_instances(&mut self, kind_env: &KindEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for (_trait_id, trait_impls) in &mut self.impls {
            for inst in trait_impls {
                errors.eat_err(inst.set_kinds_in_qual_pred_and_member_sigs(kind_env));
                let mut assoc_tys = std::mem::replace(&mut inst.assoc_types, Map::default());
                for (_, assoc_ty_impl) in &mut assoc_tys {
                    errors.eat_err(assoc_ty_impl.set_kinds(&inst, kind_env));
                }
                inst.assoc_types = assoc_tys;
            }
        }
        errors.to_result()
    }

    pub fn trait_kind_map_with_aliases(&self) -> Map<TraitId, Arc<Kind>> {
        let mut res: Map<TraitId, Arc<Kind>> = Map::default();
        for (id, ti) in &self.traits {
            res.insert(id.clone(), ti.type_var.kind.clone());
        }
        for (id, ta) in &self.aliases.data {
            res.insert(id.clone(), ta.kind.clone());
        }
        res
    }

    pub fn import(&mut self, other: TraitEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for (_, ti) in other.traits {
            if let Err(es) = self.add_trait(ti) {
                errors.append(es);
            }
        }
        for (_, insts) in other.impls {
            for inst in insts {
                errors.eat_err(self.add_instance(inst));
            }
        }
        for (_, alias) in other.aliases.data {
            if let Err(es) = self.add_alias(alias) {
                errors.append(es);
            }
        }
        errors.to_result()?;
        Ok(())
    }
}
