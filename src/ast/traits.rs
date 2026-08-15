use crate::ast::collect_annotation_tyvars::collect_annotation_tyvars;
use crate::ast::deprecation::DeprecationInfo;
use crate::ast::equality::{Equality, EqualityScheme};
use crate::ast::expr::ExprNode;
use crate::ast::kind_scope::{KindEnv, KindScope};
use crate::ast::name::{FullName, Name};
use crate::ast::predicate::Predicate;
use crate::ast::program::{EndNode, TypeEnv};
use crate::ast::qual_pred::{QualPred, QualPredScheme};
use crate::ast::qual_type::QualType;
use crate::ast::types::{
    is_opaque_tyvar, type_from_tyvar, type_tyvar, AssocType, Kind, Scheme, TyVar, TypeNode,
};
use crate::constants::ERR_MISSING_TRAIT_IMPL;
use crate::elaboration::name_resolution::{NameResolutionContext, NameResolutionType};
use crate::elaboration::typecheck::{Substitution, TypeCheckContext, UnifOrOtherErr};
use crate::elaboration::typecheckcache::FileCache;
use crate::error::{Error, Errors};
use crate::fixstd::builtin::make_boxed_trait;
use crate::misc::{generate_fresh_varnames, insert_to_map_vec, Map, Set};
use crate::parse::sourcefile::{SourcePos, Span};
use serde::{Deserialize, Serialize};
use std::mem;
use std::sync::Arc;

/// Information about missing items in a trait implementation, used for error messages and quick fixes.
#[derive(Clone, Serialize, Deserialize)]
pub struct MissingTraitImplInfo {
    /// The members and associated types the trait declares and the implementation leaves out.
    pub items: Vec<MissingTraitImplItem>,
    /// The impl type (e.g. `Main::MyData`).
    pub impl_type: Arc<TypeNode>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MissingTraitImplItem {
    Member(MissingMember),
    AssocType(MissingAssocType),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MissingMember {
    pub name: FullName,
    // The type of the member with the trait type variable substituted by the impl type.
    pub ty: Arc<TypeNode>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MissingAssocType {
    pub name: FullName,
    // Number of type parameters beyond the impl type parameter.
    pub num_extra_params: usize,
}

impl MissingTraitImplInfo {
    // Build the error message for missing items.
    pub fn error_message(&self) -> String {
        let names: Vec<String> = self
            .items
            .iter()
            .map(|item| match item {
                MissingTraitImplItem::Member(m) => format!("member `{}`", m.name.name),
                MissingTraitImplItem::AssocType(a) => {
                    format!("associated type `{}`", a.name.name)
                }
            })
            .collect();
        format!("Missing implementation of {}.", names.join(", "))
    }

    // Serialize to a serde_json::Value for use as diagnostic data.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }

    // Deserialize from a serde_json::Value.
    pub fn from_json(value: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(value.clone()).ok()
    }
}

/// The identifier of a trait, which is the full name it is declared under.
#[derive(Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TraitId {
    /// The trait's name, qualified by the namespace holding the declaration, e.g. `Std::ToString`.
    pub name: FullName,
}

impl TraitId {
    /// The identifier of the trait declared under `name`.
    pub fn from_fullname(name: FullName) -> TraitId {
        TraitId { name }
    }

    /// Splits `member_fullname`, written as `<trait-namespace>::<TraitName>::<member>`, into the
    /// id of the trait and the bare member name. Every namespaced name splits: the leading part is
    /// taken for a trait's name without asking whether a trait of that name is declared.
    ///
    /// Inverse of `FullName::new(&trait_id.name.to_namespace(), &member_name)`,
    /// which is how a trait member's `GlobalValue` is keyed in `Program`.
    pub fn split_member_fullname(member_fullname: &FullName) -> Option<(TraitId, Name)> {
        if member_fullname.namespace.names.is_empty() {
            return None;
        }
        let trait_id = TraitId::from_fullname(member_fullname.namespace.clone().to_fullname());
        Some((trait_id, member_fullname.name.clone()))
    }

    /// The trait's full name as text, e.g. `Std::ToString`.
    pub fn to_string(&self) -> String {
        self.namespaced_name().to_string()
    }

    /// The trait's name together with the namespace holding its declaration.
    pub fn namespaced_name(&self) -> FullName {
        self.name.clone()
    }

    /// Replaces the trait's name with the full name `ctx` resolves it to.
    ///
    /// # Arguments
    /// * `span` — the source the report points at when the name names no trait, or several.
    pub fn resolve_namespace(
        &mut self,
        ctx: &mut NameResolutionContext,
        span: &Option<Span>,
    ) -> Result<(), Errors> {
        self.name = ctx.resolve(&self.name, &[NameResolutionType::Trait], span)?;
        Ok(())
    }

    /// This trait id with its name marked as an absolute path, i.e. read from the root as
    /// `::Std::ToString`.
    pub fn global_to_absolute(&self) -> TraitId {
        let mut name = self.name.clone();
        name.global_to_absolute();
        TraitId { name }
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
    // Source location of the entire associated type definition (e.g., `type Item a` in `type Item a;`).
    // This span is needed for `get_document()` to find doc comments placed above the definition.
    pub src: Option<Span>,
    // Source location of the associated type name only (e.g., `Item` in `type Item a;`).
    pub name_src: Option<Span>,
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
    // The impl_type as written by the user in the associated type line (e.g., `Main::MyType` in `type Item Main::MyType = ...;`).
    // This is used for post-name-resolution validation against the trait impl's impl_type.
    pub impl_type_as_written: Arc<TypeNode>,
    // Source location of the entire associated type implementation (e.g., `type Item MyIter = I64` in `type Item MyIter = I64;`).
    // This span is needed for `get_document()` to find doc comments placed above the implementation.
    pub source: Option<Span>,
    // Source span of the associated type name only (e.g., `Item` in `type Item MyIter = I64;`).
    pub name_src: Option<Span>,
}

impl AssocTypeImpl {
    // Find the minimum node which includes the specified source code position.
    // `trait_id` is the trait that this associated type implementation belongs to.
    pub fn find_node_at(&self, pos: &SourcePos, trait_id: &TraitId) -> Option<EndNode> {
        if self.source.is_none() {
            return None;
        }
        let src = self.source.as_ref().unwrap();
        if !src.includes_pos_lsp(pos) {
            return None;
        }
        // Check if cursor is on the associated type name itself (LHS of the impl).
        if let Some(ns) = &self.name_src {
            if ns.includes_pos_lsp(pos) {
                let full_name = FullName::new(&trait_id.name.to_namespace(), &self.name);
                return Some(EndNode::AssocType(AssocType {
                    name: full_name,
                    src: Some(ns.clone()),
                }));
            }
        }
        self.value.find_node_at(pos)
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        self.value = self.value.resolve_type_aliases(type_env)?;
        self.impl_type_as_written = self.impl_type_as_written.resolve_type_aliases(type_env)?;
        Ok(())
    }

    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
        self.value = self.value.resolve_namespace(ctx)?;
        self.impl_type_as_written = self.impl_type_as_written.resolve_namespace(ctx)?;
        Ok(())
    }

    pub fn set_kinds(&mut self, trait_inst: &TraitImpl, kind_env: &KindEnv) -> Result<(), Errors> {
        let assoc_ty_name = AssocType {
            name: FullName::new(&trait_inst.trait_id().name.to_namespace(), &self.name),
            src: None,
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
        let mut scope = KindScope::new();
        for tv_in_value in tvs_in_value {
            scope
                .insert(tv_in_value.name.clone(), tv_in_value.kind.clone())
                .map_err(|e| Errors::from_msg_srcs(e, &[&self.source]))?;
        }
        self.value = self.value.set_kinds(&scope);
        Ok(())
    }
}

#[derive(Clone)]
pub struct AssocTypeKindInfo {
    #[allow(dead_code)]
    pub name: AssocType,
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
    // Source location of this member declaration.
    // The left hand side of the member declaration: e.g., `to_string` for "to_string : a -> String".
    pub decl_src: Option<Span>,
    // Document of this member.
    // This field is used only If document from `decl_src` is not available.
    pub document: Option<String>,
    /// Deprecation metadata, set during elaboration when a matching
    /// `DEPRECATED[...]` pragma exists.
    pub deprecation: Option<DeprecationInfo>,
}

impl TraitMember {
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        self.qual_ty.find_node_at(pos)
    }

    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
        self.qual_ty.resolve_namespace(ctx)
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        self.syn_qual_ty = Some(self.qual_ty.clone());
        self.qual_ty.resolve_type_aliases(type_env)
    }
}

/// The declaration of a trait, i.e. `trait a : Functor { ... }` and what it writes between the
/// braces.
#[derive(Clone)]
pub struct TraitDefn {
    /// The name this trait is declared under.
    pub trait_: TraitId,
    /// The type variable the declaration writes to the left of the trait's name, which the members'
    /// types are written in terms of: `a` in `trait a : Functor`.
    pub type_var: Arc<TyVar>,
    /// The members the trait declares, in the order the declaration writes them.
    pub members: Vec<TraitMember>,
    /// The associated types the trait declares, by their local names.
    pub assoc_types: Map<Name, AssocTypeDefn>,
    /// The kind signatures written as the assumption of the declaration, e.g. `f : *->*` in
    /// `trait [f : *->*] f : Functor {}`.
    pub kind_signs: Vec<KindSignature>,
    /// The source span of the whole declaration, from the `trait` keyword to the closing `}`.
    pub source: Option<Span>,
    /// The source span of the trait's name alone, e.g. `Functor` in `trait a : Functor { ... }`.
    pub name_src: Option<Span>,
    /// The trait's document, carried here for a trait whose `source` is unavailable; otherwise the
    /// document is read from the source code.
    pub document: Option<String>,
}

impl TraitDefn {
    /// The innermost node covering `pos` among the trait's name, its associated type declarations
    /// and its members.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        // Check if cursor is on the trait name itself (LHS of the trait definition).
        if let Some(ns) = &self.name_src {
            if ns.includes_pos_lsp(pos) {
                return Some(EndNode::Trait(self.trait_.clone()));
            }
        }
        // Check associated type definitions.
        for (assoc_name, assoc_defn) in &self.assoc_types {
            if let Some(ns) = &assoc_defn.name_src {
                if ns.includes_pos_lsp(pos) {
                    let full_name = FullName::new(&self.trait_.name.to_namespace(), assoc_name);
                    return Some(EndNode::AssocType(AssocType {
                        name: full_name,
                        src: Some(ns.clone()),
                    }));
                }
            }
        }
        for member in &self.members {
            let node = member.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        None
    }

    /// The trait's document: the doc comment written above the declaration, and the `document`
    /// field where the source carries none.
    pub fn get_document(&self) -> Option<String> {
        /// `docs` with an empty document read as absent.
        fn nonempty(docs: Option<String>) -> Option<String> {
            docs.filter(|docs| !docs.is_empty())
        }

        // Prefer the document written in the source code, and fall back to the `document` field.
        let from_source = nonempty(self.source.as_ref().and_then(|src| src.get_document().ok()));
        nonempty(from_source.or_else(|| self.document.clone()))
    }

    /// Gives the names in the members' type signatures the full names `ctx` resolves them to,
    /// reporting every name whose resolution fails.
    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for member in &mut self.members {
            errors.eat_err(member.resolve_namespace(ctx));
        }
        errors.to_result()
    }

    /// Replaces the type aliases in the members' type signatures with the types they stand for,
    /// keeping each signature as written in the member's `syn_qual_ty`.
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for member in &mut self.members {
            errors.eat_err(member.resolve_type_aliases(type_env));
        }
        errors.to_result()
    }

    /// The type scheme of the member `name`, constrained by the trait itself:
    /// `[a : ToString] a -> String` for `to_string` of
    /// `trait a : ToString { to_string : a -> String }`. Panics when the trait declares no member
    /// of that name.
    ///
    /// # Arguments
    /// * `syntactic` — when true, the member's type is the one written in source, with the type
    ///   aliases in it left unexpanded.
    pub fn member_scheme(&self, name: &Name, syntactic: bool) -> Arc<Scheme> {
        let member = self
            .members
            .iter()
            .find(|member| member.name == *name)
            .unwrap();
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

    /// The type of the member `name` as the declaration writes it, carrying no constraint from the
    /// trait itself: `a -> String` for `to_string` of
    /// `trait a : ToString { to_string : a -> String }`. Panics when the trait declares no member
    /// of that name.
    pub fn member_ty(&self, name: &Name) -> QualType {
        self.members
            .iter()
            .find(|member| member.name == *name)
            .unwrap()
            .qual_ty
            .clone()
    }

    /// Gives the trait's type variable the kind the declaration's assumption states, and gives the
    /// parameters of the associated type declarations the kinds that follow from it. Reports an
    /// assumption holding more than one kind signature, and one naming a type variable other than
    /// the trait's own.
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

/// An implementation of a trait, i.e. `impl [a : Show, b : Show] (a, b) : Show { ... }` and what it
/// writes between the braces.
#[derive(Clone)]
pub struct TraitImpl {
    /// The head of the implementation with its context: `[a : Show, b : Show] (a, b) : Show`. The
    /// trait implemented and the type it is implemented for are read from here.
    pub qual_pred: QualPred,
    /// The expression implementing each member, by the member's local name.
    pub members: Map<Name, Arc<ExprNode>>,
    /// The source spans of each left-hand side naming a member: in
    /// `impl MyType : ToString { to_string : MyType -> String; to_string = ...; }`, both
    /// occurrences of `to_string`.
    pub member_lhs_srcs: Map<Name, Vec<Span>>,
    /// The type signatures the implementation writes for its members, by the member's local name.
    pub member_sigs: Map<Name, QualType>,
    /// The implementation of each associated type, by the associated type's local name.
    pub assoc_types: Map<Name, AssocTypeImpl>,
    /// The module this implementation is written in, which the orphan rule is checked against.
    pub define_module: Name,
    /// The source span of the whole implementation.
    pub source: Option<Span>,
    /// Whether the user wrote this implementation; the compiler generates the others, such as
    /// `Std::Boxed` for every boxed type.
    pub is_user_defined: bool,
}

impl TraitImpl {
    /// The innermost node covering `pos` among the head with its context, the associated type
    /// implementations and the members' type signatures.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        let trait_id = self.trait_id();
        let node = self.qual_pred.find_node_at(pos);
        if node.is_some() {
            return node;
        }
        for (_assoc_ty_name, assoc_ty_impl) in &self.assoc_types {
            let node = assoc_ty_impl.find_node_at(pos, &trait_id);
            if node.is_some() {
                return node;
            }
        }
        for (_member_name, member_sig) in &self.member_sigs {
            let node = member_sig.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        None
    }

    /// Gives every type variable of the head, of its context and of the members' type signatures
    /// the kind that the constraints around it and the traits it is bound by determine. Reports a
    /// type variable the constraints give two different kinds, and an associated type applied to
    /// the wrong number of arguments.
    pub fn set_kinds_in_qual_pred_and_member_sigs(
        &mut self,
        kind_env: &KindEnv,
    ) -> Result<(), Errors> {
        let mut kind_scope = KindScope::new();
        let preds = &self.qual_pred.pred_constraints;
        let eqs = &self.qual_pred.eq_constraints;
        let kind_signs = &self.qual_pred.kind_constraints;
        let extend_result = kind_scope.extend(preds, eqs, kind_signs, kind_env);
        if extend_result.is_err() {
            return Err(Errors::from_msg_srcs(
                extend_result.unwrap_err(),
                &[&self.source],
            ));
        }
        self.qual_pred.predicate.set_kinds(&kind_scope);
        for pred in &mut self.qual_pred.pred_constraints {
            pred.set_kinds(&kind_scope);
        }
        for eq in &mut self.qual_pred.eq_constraints {
            eq.set_kinds(&kind_scope);
        }
        for (_member_name, member_sig) in &mut self.member_sigs {
            let mut member_kind_scope = kind_scope.clone();
            let extend_result = member_kind_scope.extend(
                &member_sig.preds,
                &member_sig.eqs,
                &member_sig.kind_signs,
                kind_env,
            );
            if extend_result.is_err() {
                return Err(Errors::from_msg_srcs(
                    extend_result.unwrap_err(),
                    &[&member_sig.ty.get_source()],
                ));
            }
            member_sig.ty = member_sig.ty.set_kinds(&member_kind_scope);
            for pred in &mut member_sig.preds {
                pred.set_kinds(&member_kind_scope);
            }
            for eq in &mut member_sig.eqs {
                eq.set_kinds(&member_kind_scope);
            }
        }
        Ok(())
    }

    /// Gives the names of the head, of its context, of the associated type implementations and of
    /// the members' type signatures the full names `ctx` resolves them to, reporting every name
    /// whose resolution fails. The names inside the members' bodies are resolved when the bodies
    /// are type-checked.
    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        errors.eat_err(self.qual_pred.resolve_namespace(ctx));
        for (_assoc_ty_name, assoc_ty_impl) in &mut self.assoc_types {
            errors.eat_err(assoc_ty_impl.resolve_namespace(ctx));
        }
        for (_member_name, member_sig) in &mut self.member_sigs {
            errors.eat_err(member_sig.resolve_namespace(ctx));
        }

        errors.to_result()
    }

    /// Replaces the type aliases in the head, in its context, in the associated type
    /// implementations and in the members' type signatures with the types they stand for.
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

    /// The trait this implementation's head names.
    fn trait_id(&self) -> TraitId {
        self.qual_pred.predicate.trait_id.clone()
    }

    /// The type scheme of this implementation of the member `member_name`:
    /// `[a : ToString, b : ToString] (a, b) -> String` for `to_string` of
    /// `impl [a : ToString, b : ToString] (a, b) : ToString`.
    ///
    /// A type signature the implementation writes for the member is taken as the member's type;
    /// otherwise the type comes from the trait's declaration and this implementation's head.
    pub fn member_scheme(&self, member_name: &Name, trait_defn: &TraitDefn) -> Arc<Scheme> {
        if let Some(qual_ty) = self.member_sigs.get(member_name) {
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
            self.member_scheme_by_defn(member_name, trait_defn)
        }
    }

    /// The type scheme of this implementation of the member `member_name`, as the trait's
    /// declaration and this implementation's head give it:
    /// `[a : ToString, b : ToString] (a, b) -> String` for `to_string` of
    /// `impl [a : ToString, b : ToString] (a, b) : ToString`. The type comes from the declaration
    /// and the head alone, whatever type signature the implementation writes for the member.
    pub fn member_scheme_by_defn(&self, member_name: &Name, trait_defn: &TraitDefn) -> Arc<Scheme> {
        // First, see the trait definition.
        // Let's consider `trait a : ToString { to_string : a -> String }`.
        let tyvar_name = &trait_defn.type_var.name; // `a` in the above example.
        let mut member_qualty = trait_defn.member_ty(member_name); // `a -> String` in the above example.

        // Next, see the trait implementation to get the type for which the trait is implemented.
        let impl_type = self.impl_type(); // `(a, b)` in the above example.

        // We are going to substitute `tyvar_name` (e.g., `a`) in `member_qualty` (e.g., `a -> String`) with `impl_type` (e.g., `(a, b)`)
        // This is OK if FV(member_qualty) \ {tyvar_name} is disjoint from FV(impl_type).
        // Otherwise, we need to rename the type variables in `member_qualty` to avoid name collision.
        // Example:
        // Consider `impl Arrow a : Functor` for `trait f : Functor { map : (a -> b) -> f a -> f b }`.
        // In this case, if we naively substitute `f` in `map : (a -> b) -> f a -> f b` with `Arrow a`,
        // then we get `map : (a -> b) -> Arrow a a -> Arrow a b`, which is wrong.
        // So we first rename `(a -> b) -> f a -> f b` to `(c -> b) -> f c -> f b`.
        let mut fv_member_qualty = vec![];
        member_qualty.free_vars_vec(&mut fv_member_qualty);
        let fv_impl_type = impl_type.free_vars();
        // Collect type variables that need renaming (those that collide with fv_impl_type).
        let vars_to_rename: Vec<_> = fv_member_qualty
            .iter()
            .filter(|fv| &fv.name != tyvar_name && fv_impl_type.contains_key(&fv.name))
            .collect();
        let used_names: Set<String> = fv_impl_type
            .keys()
            .chain(fv_member_qualty.iter().map(|fv| &fv.name))
            .cloned()
            .collect();
        let new_names = generate_fresh_varnames(vars_to_rename.len(), &used_names);
        let mut rename_subst = Substitution::default();
        for (fv, new_name) in vars_to_rename.iter().zip(new_names.iter()) {
            let new_fv = type_tyvar(new_name, &fv.kind);
            let merge_succ = rename_subst.merge(&Substitution::single(&fv.name, new_fv));
            assert!(merge_succ);
        }
        // Rename type variables in `member_qualty`.
        rename_subst.substitute_qualtype(&mut member_qualty);

        // Then substitute `tyvar_name` with `impl_type`.
        // Now we get `(a, b) -> String` or `(c -> b) -> Arrow a c -> Arrow a b` in the above examples.
        let impl_subst = Substitution::single(&tyvar_name, impl_type);
        impl_subst.substitute_qualtype(&mut member_qualty);

        // Prepare `vars`, `ty`, `preds`, and `eqs` to be generalized.
        let ty = member_qualty.ty.clone();
        let mut kind_signs = self.qual_pred.kind_constraints.clone();
        kind_signs.append(&mut member_qualty.kind_signs.clone());
        let mut preds = self.qual_pred.pred_constraints.clone();
        preds.append(&mut member_qualty.preds);
        let mut eqs = self.qual_pred.eq_constraints.clone();
        eqs.append(&mut member_qualty.eqs);

        Scheme::generalize(&kind_signs, preds, eqs, ty)
    }

    /// Get expression that implements a member.
    /// Panics when this implementation has no member of that name.
    pub fn member_expr(&self, name: &Name) -> Arc<ExprNode> {
        self.members.get(name).unwrap().clone()
    }

    /// The type the trait is implemented for, i.e., the head of this implementation: `(a, b)` in
    /// `impl [a : ToString, b : ToString] (a, b) : ToString`.
    pub fn impl_type(&self) -> Arc<TypeNode> {
        self.qual_pred.predicate.ty.clone()
    }
}

/// The declaration of a trait alias, i.e. `trait MyAlias = Foo + Bar;`.
#[derive(Clone)]
pub struct TraitAlias {
    /// The name this alias is declared under.
    pub id: TraitId,
    /// The traits the alias stands for, each with the span naming it: `Foo` and `Bar` above. A
    /// trait named here may itself be an alias.
    pub value: Vec<(TraitId, Span)>,
    /// The source span of the whole declaration, from the `trait` keyword to the final semicolon.
    pub source: Option<Span>,
    /// The source span of the alias's name alone, e.g. `MyAlias` in `trait MyAlias = Foo + Bar;`.
    pub name_src: Option<Span>,
    /// The kind of the type variable the aliased traits constrain, which all of them share.
    pub kind: Arc<Kind>,
}

impl TraitAlias {
    /// The alias's document, i.e. the doc comment written above the declaration.
    pub fn get_document(&self) -> Option<String> {
        self.source.as_ref().and_then(|src| src.get_document().ok())
    }

    /// The innermost node covering `pos` among the alias's name and the traits it stands for.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        // Check if cursor is on the trait alias name itself (LHS of the alias definition).
        if let Some(ns) = &self.name_src {
            if ns.includes_pos_lsp(pos) {
                return Some(EndNode::Trait(self.id.clone()));
            }
        }
        for (t, s) in &self.value {
            if s.includes_pos_lsp(pos) {
                return Some(EndNode::Trait(t.clone()));
            }
        }
        None
    }

    /// Gives the traits the alias stands for the full names `ctx` resolves their names to.
    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
        for (trait_id, _) in &mut self.value {
            trait_id.resolve_namespace(ctx, &self.source)?;
        }
        Ok(())
    }
}

// Statement such as "f: * -> *".
#[derive(Clone, Serialize, Deserialize)]
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

/// The trait aliases a program declares, and the expansion of an alias into the traits it stands
/// for.
#[derive(Clone, Default)]
pub struct TraitAliasEnv {
    /// Every declared alias, keyed by the name it is declared under. A trait name absent from here
    /// names a trait of its own.
    pub data: Map<TraitId, TraitAlias>,
}

impl TraitAliasEnv {
    /// Whether `trait_id` names an alias. Trait names divide into the aliases declared here and the
    /// traits `TraitEnv::traits` holds.
    pub fn is_alias(&self, trait_id: &TraitId) -> bool {
        self.data.contains_key(trait_id)
    }

    /// The traits an alias stands for: each trait reachable from it that is not itself an alias,
    /// once, in the order the definitions name them.
    ///
    /// Reports an alias that stands for itself, directly or through other aliases, since expanding
    /// such a one does not terminate.
    pub fn resolve_alias(&self, trait_id: &TraitId) -> Result<Vec<TraitId>, Errors> {
        /// Walks the aliases reachable from `trait_id` and pushes onto `res` each trait it reaches
        /// that is not an alias.
        ///
        /// `on_path` holds the aliases the walk has entered and not yet left, so an alias found in
        /// it is one the walk is already inside: that, and only that, is a circular alias. An alias
        /// reachable along several paths is entered on the first of them and left again before the
        /// second is walked, which is why membership has to be given up on the way back up.
        ///
        /// `resolved` holds the traits already accounted for in `res`: an alias all of whose traits
        /// are pushed, and a non-alias trait pushed itself. It keeps a part of the graph that
        /// several paths share from being walked, or pushed, a second time.
        fn resolve_alias_internal(
            env: &TraitAliasEnv,
            trait_id: &TraitId,
            res: &mut Vec<TraitId>,
            on_path: &mut Set<TraitId>,
            resolved: &mut Set<TraitId>,
        ) -> Result<(), Errors> {
            if resolved.contains(trait_id) {
                return Ok(());
            }
            let Some(alias) = env.data.get(trait_id) else {
                res.push(trait_id.clone());
                resolved.insert(trait_id.clone());
                return Ok(());
            };
            if !on_path.insert(trait_id.clone()) {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "Circular aliasing detected in trait alias `{}`.",
                        trait_id.to_string()
                    ),
                    &[&alias.source],
                ));
            }
            for (t, _) in &alias.value {
                resolve_alias_internal(env, t, res, on_path, resolved)?;
            }
            on_path.remove(trait_id);
            resolved.insert(trait_id.clone());
            Ok(())
        }

        let mut res = vec![];
        let mut on_path = Set::default();
        let mut resolved = Set::default();
        resolve_alias_internal(self, trait_id, &mut res, &mut on_path, &mut resolved)?;
        Ok(res)
    }
}

/// The traits, the trait implementations and the trait aliases a program declares.
#[derive(Clone, Default)]
pub struct TraitEnv {
    /// Every declared trait, by the name it is declared under.
    pub traits: Map<TraitId, TraitDefn>,
    /// The implementations of each trait, filed under the trait their head names.
    pub impls: Map<TraitId, Vec<TraitImpl>>,
    /// Every declared trait alias, and the traits each of them stands for.
    pub aliases: TraitAliasEnv,
    /// The trait constraints an opaque type variable carries, such as `?it : Iterator` of
    /// `[?it : Iterator] I64 -> ?it`, by the trait constraining it.
    pub opaque_preds: Map<TraitId, Vec<QualPredScheme>>,
    /// The associated type equalities an opaque type variable carries, such as `Item ?it = I64` of
    /// `[?it : Iterator, Item ?it = I64] I64 -> ?it`, by the associated type they fix.
    pub opaque_eqs: Map<AssocType, Vec<EqualityScheme>>,
}

impl TraitEnv {
    /// The innermost node covering `pos` among the trait definitions, the trait implementations and
    /// the trait aliases.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        for (_t, trait_defn) in &self.traits {
            let node = trait_defn.find_node_at(pos);
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

    /// The full name of every declared trait and of every declared trait alias.
    pub fn trait_names(&self) -> Set<FullName> {
        self.traits_with_aliases()
            .into_iter()
            .map(|t| t.name)
            .collect()
    }

    /// The identifier of every declared trait and of every declared trait alias.
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

    /// Validates the traits, the trait aliases and the trait implementations, structurally.
    ///
    /// Whether two implementations overlap is asked by `validate_overlapping_instances`, once the
    /// kinds of the type variables in their heads are known.
    pub fn validate_structure(&self) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        /// The report of `trait_id` being declared both as a trait and as a trait alias, pointing
        /// at both declarations. Panics unless both declare it.
        fn create_conflicting_error(env: &TraitEnv, trait_id: &TraitId) -> Errors {
            let this_src = &env.traits.get(trait_id).unwrap().source;
            let other_src = &env.aliases.data.get(trait_id).unwrap().source;
            Errors::from_msg_srcs(
                format!("Duplicate definition for `{}`", trait_id.to_string()),
                &[this_src, other_src],
            )
        }

        // Check name confliction of traits and aliases.
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

        // Circular aliasing is reported by `TraitAliasEnv::resolve_alias`, which
        // `TraitEnv::set_kinds_in_trait_and_alias_defns` calls for every declared alias.

        for (_trait_id, trait_defn) in &self.traits {
            // Forbid opaque type variables in trait definitions.
            if is_opaque_tyvar(&trait_defn.type_var.name) {
                errors.append(Errors::from_msg_srcs(
                    format!(
                        "Opaque type variable `{}` is not allowed in a trait definition.",
                        trait_defn.type_var.name,
                    ),
                    &[&trait_defn.source.as_ref().map(|s| s.to_head_character())],
                ));
            }
            for member in &trait_defn.members {
                // Validate trait member definition.

                // That a use site determines the trait type variable from the
                // member's type is checked by the Fixv well-formedness
                // condition in `Scheme::validate_constraints`: it rejects a
                // member whose type leaves the variable out, and one that
                // mentions it only as an argument of an associated type
                // application.

                // The "impl type" cannot be constrained.
                //
                // This is a restriction mentioned in section 5.1 (Well-formed programs) of the paper "Associated Type Synonyms":
                // > If σ ≡ (∀α.π ⇒ τ) is a method signature in a class declaration for D β, we require that β not ∈ Fv π.
                // This is related to Issue #73.
                if let Some(source) = member
                    .qual_ty
                    .find_var_in_constraint(&trait_defn.type_var.name)
                {
                    errors.append(Errors::from_msg_srcs(
                        format!(
                            "Type variable `{}` used in trait definition cannot be constrained in the type of a member.",
                            trait_defn.type_var.name,
                        ),
                        &[&Some(source)],
                    ));
                }
            }
        }
        // If some errors are found upto here, throw them.
        errors.to_result()?;

        // Validate trait implementations.
        for (trait_id, impls) in &self.impls {
            for impl_ in impls.iter() {
                // check implementation is given for trait, not for trait alias.
                if self.aliases.is_alias(&trait_id) {
                    return Err(Errors::from_msg_srcs(
                        "A trait alias cannot be implemented directly. Implement each aliased trait instead.".to_string(),
                        &[&impl_.qual_pred.predicate.src],
                    ));
                }
                // Now `trait_id` is not an alias, so get the trait definition.
                let defn = self.traits.get(trait_id).unwrap();
                errors.eat_err(Self::validate_trait_impl(impl_, defn));
            }
        }

        errors.to_result()
    }

    /// Reports each pair of implementations of one trait whose heads can denote the same type.
    ///
    /// Which types a head denotes depends on the kinds of the type variables in it, so this runs
    /// once those kinds are set: a variable still carrying the default kind `*` fails to unify with
    /// the type it stands for, and the pair reads as disjoint.
    pub fn validate_overlapping_instances(&self, kind_env: KindEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Prepare TypeCheckContext to use `unify`.
        let tc = TypeCheckContext::new(
            TraitEnv::default(),
            TypeEnv::default(),
            kind_env,
            Map::default(),
            Arc::new(FileCache::new()),
            0,
            false,
        );
        for (trait_id, impls) in &self.impls {
            for i in 0..impls.len() {
                for j in (i + 1)..impls.len() {
                    let inst_i = &impls[i];
                    let inst_j = &impls[j];
                    let mut tc = tc.clone();
                    let type_i = tc.instantiate_type(&inst_i.impl_type());
                    let type_j = tc.instantiate_type(&inst_j.impl_type());
                    if UnifOrOtherErr::extract_others(tc.unify(&type_i, &type_j))?.is_err() {
                        continue;
                    }
                    let mut msg = format!(
                        "Two trait implementations for `{}` are overlapping.",
                        trait_id.to_string()
                    );
                    if inst_i.trait_id() == make_boxed_trait() {
                        msg +=
                            " NOTE: `Std::Boxed` is automatically implemented for all boxed types by compiler."
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

    /// Reports what the implementation `impl_` of the trait `defn` gets wrong: a head no trait may
    /// be implemented for, a member or an associated type the trait leaves undeclared, one the
    /// trait declares and the implementation leaves out, an associated type line writing another
    /// type than the head, a type variable bound nowhere, an implementation of a trait and a type
    /// both foreign to the module writing it, and `Std::Boxed` written by hand.
    ///
    /// The report of the members and associated types left out carries their names and types as
    /// data, from which the editor offers to write them.
    fn validate_trait_impl(impl_: &TraitImpl, defn: &TraitDefn) -> Result<(), Errors> {
        let trait_id = &defn.trait_;

        // Check instance head.
        let impl_ty = &impl_.qual_pred.predicate.ty;
        impl_ty.is_implementable()?;

        // Validate the set of trait members.
        let trait_members = &defn.members;
        let impl_members = &impl_.members;
        let member_sigs = &impl_.member_sigs;

        // Collect missing members and associated types for the quick fix.
        let trait_ns = trait_id.name.to_namespace();
        let mut missing_items: Vec<MissingTraitImplItem> = vec![];
        for trait_member in trait_members {
            if !impl_members.contains_key(&trait_member.name) {
                let scheme = impl_.member_scheme_by_defn(&trait_member.name, defn);
                missing_items.push(MissingTraitImplItem::Member(MissingMember {
                    name: FullName::new(&trait_ns, &trait_member.name),
                    ty: scheme.ty.clone(),
                }));
            }
        }

        /// Reports `member` unless `trait_members` declares it, anchoring the report at `src`.
        fn validate_member_is_declared(
            trait_members: &[TraitMember],
            trait_id: &TraitId,
            member: &Name,
            src: &Option<Span>,
        ) -> Result<(), Errors> {
            if trait_members.iter().any(|mi| mi.name == *member) {
                return Ok(());
            }
            Err(Errors::from_msg_srcs(
                format!(
                    "`{}` is not a member of trait `{}`.",
                    member,
                    trait_id.to_string(),
                ),
                &[src],
            ))
        }

        for (member_name, member_expr) in impl_members {
            validate_member_is_declared(trait_members, trait_id, member_name, &member_expr.source)?;
        }

        // Validate the set of associated types.
        let trait_assoc_types = &defn.assoc_types;
        let impl_assoc_types = &impl_.assoc_types;
        for (trait_assoc_type, assoc_defn) in trait_assoc_types {
            if !impl_assoc_types.contains_key(trait_assoc_type) {
                let num_extra_params = assoc_defn.params.len() - 1; // skip impl_type param
                missing_items.push(MissingTraitImplItem::AssocType(MissingAssocType {
                    name: FullName::new(&trait_ns, trait_assoc_type),
                    num_extra_params,
                }));
            }
        }

        // If there are missing items, report them with quick fix data.
        if !missing_items.is_empty() {
            let info = MissingTraitImplInfo {
                items: missing_items,
                impl_type: impl_.impl_type(),
            };
            let mut err = Error::from_msg_srcs(info.error_message(), &[&impl_.source]);
            err.code = Some(ERR_MISSING_TRAIT_IMPL);
            err.data = Some(info.to_json());
            return Err(Errors::from_err(err));
        }

        for (impl_assoc_type, assoc_ty_impl) in impl_assoc_types {
            if !trait_assoc_types.contains_key(impl_assoc_type) {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "`{}` is not an associated type of trait `{}`.",
                        impl_assoc_type,
                        trait_id.to_string(),
                    ),
                    &[&assoc_ty_impl.source],
                ));
            }
            // Validate that the impl_type written in the associated type line matches the trait impl's impl_type.
            if assoc_ty_impl.impl_type_as_written != impl_.impl_type() {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "The implementation of an associated type should be in the form `type {{AssocTyName}} {{impl_type}} {{type_var1}} ... {{type_varN}} = {{value_type}};`, where {{impl_type}} is `{}` here.",
                        impl_.impl_type().to_string()
                    ),
                    &[&assoc_ty_impl.source],
                ));
            }
            // Validate free variable of associated type implementation.
            let mut allowed_tyvars = vec![];
            impl_.impl_type().free_vars_to_vec(&mut allowed_tyvars);
            for arg in &assoc_ty_impl.params {
                allowed_tyvars.push(arg.clone());
            }
            for used_tv in assoc_ty_impl.value.free_vars_vec() {
                if allowed_tyvars
                    .iter()
                    .all(|allowed_tv| allowed_tv.name != used_tv.name)
                {
                    return Err(Errors::from_msg_srcs(
                        format!("Unknown type variable `{}`.", used_tv.name),
                        &[&assoc_ty_impl.source],
                    ));
                }
            }
        }

        // For members without type signature, type variables used in type annotations in the member
        // must appear in the type being implemented.
        // This prevents users from referencing trait-definition-derived type variables
        // (including opaque type variables like `?it`) that are not visible to the user
        // in the impl context. If the user wants to use such variables, they should
        // provide an explicit type signature on the member of the implementation.
        for (member_name, member_expr) in impl_members {
            if !member_sigs.contains_key(member_name) {
                let mut allowed_tyvars = vec![];
                impl_.impl_type().free_vars_to_vec(&mut allowed_tyvars);
                for (used_tv, tv_src) in collect_annotation_tyvars(&member_expr) {
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
            validate_member_is_declared(
                trait_members,
                trait_id,
                member_name,
                &member_sig.ty.get_source(),
            )?;
        }

        // Check Orphan rules.
        let instance_def_mod = &impl_.define_module;
        let trait_def_mod = trait_id.name.module();
        let type_def_mod = impl_ty.toplevel_tycon().unwrap().name.module();
        if trait_def_mod != *instance_def_mod && type_def_mod != *instance_def_mod {
            return Err(Errors::from_msg_srcs(
                format!(
                    "Implementing trait `{}` for type `{}` in module `{}` is illegal; \
                            it is not allowed to implement an external trait for an external type.",
                    trait_id.to_string(),
                    impl_ty.to_string_normalize(),
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

    /// Gives the names in the trait aliases, in the trait declarations and in the trait
    /// implementations the full names `ctx` resolves them to, and files each implementation under
    /// the trait its resolved head names.
    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Resolve names in trait aliases.
        for (trait_id, alias_info) in &mut self.aliases.data {
            ctx.set_current_module(trait_id.name.module());
            errors.eat_err(alias_info.resolve_namespace(ctx));
        }
        errors.to_result()?; // Throw errors if any.

        // Resolve names in trait definitions.
        for (trait_id, trait_info) in &mut self.traits {
            ctx.set_current_module(trait_id.name.module());
            errors.eat_err(trait_info.resolve_namespace(ctx));
        }
        errors.to_result()?; // Throw errors if any.

        // Resolve names in trait implementations, and file each one under the name its head
        // resolved to.
        let old_impls = mem::replace(&mut self.impls, Default::default());
        let mut new_impls: Map<TraitId, Vec<TraitImpl>> = Default::default();
        for (trait_id_key, trait_impls) in old_impls {
            for mut impl_ in trait_impls {
                // Set up NameResolutionContext.
                ctx.set_current_module(impl_.define_module.clone());

                // `add_instance` keys the map by the implementation's own trait id.
                assert!(
                    impl_.trait_id().name == trait_id_key.name,
                    "`{}` is filed under `{}`.",
                    impl_.trait_id().name.to_string(),
                    trait_id_key.name.to_string()
                );

                errors.eat_err(impl_.resolve_namespace(ctx));

                Self::add_instance_to(&mut new_impls, impl_);
            }
        }

        errors.to_result()?; // Throw errors if any.
        self.impls = new_impls;
        Ok(())
    }

    /// Replaces the type aliases in the trait declarations and in the trait implementations with
    /// the types they stand for.
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Resolve aliases in trait definitions.
        for (_, trait_info) in &mut self.traits {
            errors.eat_err(trait_info.resolve_type_aliases(type_env));
        }

        // Resolve aliases in trait implementations.
        let old_impls = mem::replace(&mut self.impls, Default::default());
        let mut new_impls: Map<TraitId, Vec<TraitImpl>> = Default::default();
        for (_trait_id, trait_impls) in old_impls {
            for mut impl_ in trait_impls {
                errors.eat_err(impl_.resolve_type_aliases(type_env));

                Self::add_instance_to(&mut new_impls, impl_);
            }
        }
        errors.to_result()?; // Throw errors if any.
        self.impls = new_impls;
        Ok(())
    }

    /// Records the given trait declarations, trait implementations and trait aliases, reporting
    /// every name that is declared twice.
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

    /// Records the trait declaration `info`, reporting a trait already declared under that name,
    /// at both declarations.
    pub fn add_trait(&mut self, info: TraitDefn) -> Result<(), Errors> {
        // Check Duplicate definition.
        if self.traits.contains_key(&info.trait_) {
            let declared = self.traits.get(&info.trait_).unwrap();
            return Err(Errors::from_msg_srcs(
                format!(
                    "Duplicate definition for trait {}.",
                    info.trait_.to_string()
                ),
                &[&declared.source, &info.source],
            ));
        }
        self.traits.insert(info.trait_.clone(), info);
        Ok(())
    }

    /// Appends `inst` to `impls`, under the trait it implements.
    fn add_instance_to(impls: &mut Map<TraitId, Vec<TraitImpl>>, inst: TraitImpl) {
        let trait_id = inst.trait_id();
        insert_to_map_vec(impls, &trait_id, inst);
    }

    /// Appends `inst` to the implementations recorded for the trait it implements.
    pub fn add_instance(&mut self, inst: TraitImpl) -> Result<(), Errors> {
        Self::add_instance_to(&mut self.impls, inst);
        Ok(())
    }

    /// Records the trait alias declaration `alias`, reporting an alias already declared under that
    /// name, at both declarations.
    fn add_alias(&mut self, alias: TraitAlias) -> Result<(), Errors> {
        // Check duplicate definition.
        if self.aliases.data.contains_key(&alias.id) {
            let declared = self.aliases.data.get(&alias.id).unwrap();
            return Err(Errors::from_msg_srcs(
                format!(
                    "Duplicate definition for trait alias {}.",
                    alias.id.to_string()
                ),
                &[&declared.source, &alias.source],
            ));
        }
        self.aliases.data.insert(alias.id.clone(), alias);
        Ok(())
    }

    /// The trait constraints a deduction may use, by the trait each one is about: what every
    /// implementation's head states, generalized over the type variables in it, together with the
    /// constraints the opaque type variables carry.
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
        // Merge opaque predicates.
        for (trait_id, opaque_qps) in &self.opaque_preds {
            for qps_entry in opaque_qps {
                insert_to_map_vec(&mut qps, trait_id, qps_entry.clone());
            }
        }
        qps
    }

    /// The type equalities a deduction may use, by the associated type each one is about: what
    /// every implementation of an associated type states, generalized over the type variables in
    /// it, together with the equalities the opaque type variables carry.
    pub fn type_equalities(&self) -> Map<AssocType, Vec<EqualityScheme>> {
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
                        assoc_type: AssocType {
                            name: assoc_type_fullname,
                            src: assoc_type_impl.name_src.clone(),
                        },
                        args,
                        value: assoc_type_impl.value.clone(),
                        src: assoc_type_impl.source.clone(),
                    };
                    insert_to_map_vec(&mut eq_scms, &equality.assoc_type, equality.generalize());
                }
            }
        }
        // Merge opaque equalities.
        for (assoc_type, opaque_eq_scms) in &self.opaque_eqs {
            for eq_scm in opaque_eq_scms {
                insert_to_map_vec(&mut eq_scms, assoc_type, eq_scm.clone());
            }
        }
        eq_scms
    }

    /// The number of type parameters every declared associated type takes, by its full name. The
    /// type the trait is implemented for is the first of them, so `type Item a;` has arity 1.
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

    /// The kinds every declared associated type works with: those of its type parameters and that
    /// of the type an application of it stands for.
    pub fn assoc_ty_kind_info(&self) -> Map<AssocType, AssocTypeKindInfo> {
        let mut assoc_ty_kind_info = Map::default();
        for (trait_id, trait_info) in &self.traits {
            for (assoc_ty_name, assoc_ty_info) in &trait_info.assoc_types {
                let assoc_type_namespace = trait_id.name.to_namespace();
                let assoc_type = AssocType {
                    name: FullName::new(&assoc_type_namespace, &assoc_ty_name),
                    src: None,
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

    /// Gives every trait's type variable, and every alias's, the kind its declaration determines.
    /// Reports an alias standing for traits of two different kinds, and one standing for itself.
    pub fn set_kinds_in_trait_and_alias_defns(&mut self) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // Set kinds in trait definitions.
        for (_id, trait_defn) in &mut self.traits {
            errors.eat_err(trait_defn.set_trait_kind());
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

    /// Gives the type variables of every implementation — those of its head, of its context, of
    /// its members' type signatures and of its associated type implementations — the kinds the
    /// constraints around them determine.
    pub fn set_kinds_in_trait_instances(&mut self, kind_env: &KindEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for (_trait_id, trait_impls) in &mut self.impls {
            for inst in trait_impls {
                errors.eat_err(inst.set_kinds_in_qual_pred_and_member_sigs(kind_env));
                let mut assoc_tys = mem::replace(&mut inst.assoc_types, Map::default());
                for (_, assoc_ty_impl) in &mut assoc_tys {
                    errors.eat_err(assoc_ty_impl.set_kinds(&inst, kind_env));
                }
                inst.assoc_types = assoc_tys;
            }
        }
        errors.to_result()
    }

    /// The kind of the type variable every declared trait and trait alias constrains, by its name.
    pub fn trait_kind_map_with_aliases(&self) -> Map<TraitId, Arc<Kind>> {
        let mut res: Map<TraitId, Arc<Kind>> = Map::default();
        for (id, trait_defn) in &self.traits {
            res.insert(id.clone(), trait_defn.type_var.kind.clone());
        }
        for (id, ta) in &self.aliases.data {
            res.insert(id.clone(), ta.kind.clone());
        }
        res
    }

    /// Takes the trait declarations, the trait implementations and the trait aliases of `other`
    /// into this environment, reporting every name both of them declare.
    pub fn import(&mut self, other: TraitEnv) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for (_, trait_defn) in other.traits {
            if let Err(es) = self.add_trait(trait_defn) {
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
