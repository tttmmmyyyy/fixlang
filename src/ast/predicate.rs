use crate::ast::kind_scope::{KindEnv, KindScope};
use crate::ast::program::EndNode;
use crate::ast::traits::{TraitAliasEnv, TraitId};
use crate::ast::type_env::TypeEnv;
use crate::ast::types::{is_opaque_tyvar, TyVar, Type, TypeNode};
use crate::elaboration::name_resolution::NameResolutionContext;
use crate::elaboration::typecheck::InstantiationSite;
use crate::error::Errors;
use crate::parse::sourcefile::{SourcePos, Span};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// A trait constraint on a type, such as `String : Show` or `a : Eq`.
#[derive(Clone, Serialize, Deserialize)]
pub struct Predicate {
    /// The trait the type has to implement.
    pub trait_id: TraitId,
    /// The type the constraint is on.
    pub ty: Arc<TypeNode>,
    /// The span of the whole constraint in the source, such as `I64 : Describable`.
    pub src: Option<Span>,
    /// The span of the trait name alone, such as `Describable` in `I64 : Describable`.
    pub trait_src: Option<Span>,
    /// For a constraint that type checking has to deduce, the use of the global value whose type
    /// requires it. A constraint deduced on the way to another one is required by the same use.
    #[serde(skip)]
    pub required_by: Option<Arc<InstantiationSite>>,
}

impl PartialEq for Predicate {
    /// Compares `trait_id` and `ty`. The spans and `required_by` take no part, so two copies of one
    /// constraint, written or required at different places, compare equal.
    fn eq(&self, other: &Self) -> bool {
        self.trait_id == other.trait_id && self.ty == other.ty
    }
}

impl Eq for Predicate {}

impl Hash for Predicate {
    /// Hashes the trait and the type, agreeing with the equality of `PartialEq`.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.trait_id.hash(state);
        self.ty.hash(state);
    }
}

impl Predicate {
    /// Appends to `buf` the type variables standing in the constrained type, in order of
    /// appearance.
    pub fn free_vars_to_vec(&self, buf: &mut Vec<Arc<TyVar>>) {
        self.ty.free_vars_to_vec(buf);
    }

    /// Whether the constrained type is an opaque type variable, one whose name starts with `?`.
    pub fn on_opaque_tyvar(&self) -> bool {
        if let Type::TyVar(tv) = &self.ty.ty {
            is_opaque_tyvar(&tv.name)
        } else {
            false
        }
    }

    /// Whether the constrained type is the type variable `tyvar_name`.
    pub fn on_tyvar(&self, tyvar_name: &str) -> bool {
        if let Type::TyVar(tv) = &self.ty.ty {
            tv.name == tyvar_name
        } else {
            false
        }
    }

    /// This constraint with every global name in the trait and in the type made absolute.
    pub fn global_to_absolute(&self) -> Predicate {
        Predicate {
            trait_id: self.trait_id.global_to_absolute(),
            ty: self.ty.global_to_absolute(),
            src: self.src.clone(),
            trait_src: self.trait_src.clone(),
            required_by: self.required_by.clone(),
        }
    }

    /// Records `source` as where the whole constraint is written.
    pub fn set_source(&mut self, source: Span) {
        self.src = Some(source);
    }

    /// The constraint that `ty` implements `trait_id`, written nowhere and required by no use.
    pub fn make(trait_id: TraitId, ty: Arc<TypeNode>) -> Self {
        Predicate {
            trait_id,
            ty,
            src: None,
            trait_src: None,
            required_by: None,
        }
    }

    /// Gives the trait and every name in the type their full names, read in the context `ctx`
    /// carries.
    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
        self.trait_id.resolve_namespace(ctx, &self.src)?;
        self.ty = self.ty.resolve_namespace(ctx)?;
        Ok(())
    }

    /// Replaces each type alias in the type by the type it stands for.
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        self.ty = self.ty.resolve_type_aliases(type_env)?;
        Ok(())
    }

    /// This constraint written in source syntax, as `ty : Trait`, under the names its own type
    /// variables carry.
    pub fn to_string(&self) -> String {
        format!("{} : {}", self.ty.to_string(), self.trait_id.to_string())
    }

    /// Gives each type variable in the type the kind `scope` assigns it.
    pub fn set_kinds(&mut self, scope: &KindScope) {
        self.ty = self.ty.set_kinds(scope);
    }

    /// Reports a constraint whose type has another kind than the trait asks for.
    pub fn check_kinds(&self, kind_env: &KindEnv) -> Result<(), Errors> {
        let expected = &kind_env.traits_and_aliases[&self.trait_id];
        let found = self.ty.kind(kind_env)?;
        if *expected != found {
            return Err(Errors::from_msg_srcs(
                format!(
                    "Kind mismatch in `{}`. Expect: {}, found: {}.",
                    self.to_string(),
                    expected.to_string(),
                    found.to_string()
                ),
                &[&self.src],
            ));
        }
        Ok(())
    }

    /// The constraints this one stands for: itself when its trait is a trait, and, when its trait is
    /// a trait alias, one constraint for each trait the alias reaches through the aliases it names.
    pub fn resolve_trait_aliases(&self, aliases: &TraitAliasEnv) -> Result<Vec<Predicate>, Errors> {
        if !aliases.is_alias(&self.trait_id) {
            return Ok(vec![self.clone()]);
        }
        let trait_ids = aliases.resolve_alias(&self.trait_id)?;
        let mut res = vec![];
        for trait_id in trait_ids {
            let mut p = self.clone();
            p.trait_id = trait_id;
            res.push(p);
        }
        Ok(res)
    }

    /// The smallest node of this constraint covering `pos`: a node of the type, or else the trait.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        if self.src.is_none() {
            return None;
        }
        let src = self.src.as_ref().unwrap();
        if !src.includes_pos_lsp(pos) {
            return None;
        }
        let node = self.ty.find_node_at(pos);
        if node.is_some() {
            return node;
        }
        Some(EndNode::Trait(self.trait_id.clone()))
    }
}
