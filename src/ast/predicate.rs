use std::sync::Arc;

use crate::ast::kind_scope::{KindEnv, KindScope};
use crate::ast::name::GlobalRelativeNames;
use crate::ast::program::{EndNode, TypeEnv};
use crate::ast::traits::{TraitAliasEnv, TraitId};
use crate::ast::types::{TyVar, TypeNode};
use crate::error::Errors;
use crate::name_resolution::NameResolutionContext;
use crate::sourcefile::{SourcePos, Span};
use serde::{Deserialize, Serialize};

// Statement such as "String : Show" or "a : Eq".
#[derive(Clone, Serialize, Deserialize)]
pub struct Predicate {
    pub trait_id: TraitId,
    pub ty: Arc<TypeNode>,
    pub source: Option<Span>,
}

impl Predicate {
    pub fn free_vars_to_vec(&self, buf: &mut Vec<Arc<TyVar>>) {
        self.ty.free_vars_to_vec(buf);
    }

    // Collect names that should be imported.
    pub fn collect_import_names(&self, names: &mut GlobalRelativeNames) {
        // Collect the trait name
        names.add(self.trait_id.name.clone());
        // Collect type names
        self.ty.collect_import_names(names);
    }

    // Convert all global FullNames to absolute paths.
    pub fn global_to_absolute(&self) -> Predicate {
        Predicate {
            trait_id: self.trait_id.global_to_absolute(),
            ty: self.ty.global_to_absolute(),
            source: self.source.clone(),
        }
    }

    pub fn set_source(&mut self, source: Span) {
        self.source = Some(source);
    }

    pub fn make(trait_id: TraitId, ty: Arc<TypeNode>) -> Self {
        Predicate {
            trait_id,
            ty,
            source: None,
        }
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        self.trait_id.resolve_namespace(ctx, &self.source)?;
        self.ty = self.ty.resolve_namespace(ctx)?;
        Ok(())
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        self.ty = self.ty.resolve_type_aliases(type_env)?;
        Ok(())
    }

    pub fn to_string_normalize(&self) -> String {
        format!(
            "{} : {}",
            self.ty.to_string_normalize(),
            self.trait_id.to_string()
        )
    }

    pub fn to_string(&self) -> String {
        format!("{} : {}", self.ty.to_string(), self.trait_id.to_string())
    }

    pub fn set_kinds(&mut self, scope: &KindScope) {
        self.ty = self.ty.set_kinds(scope);
    }

    pub fn check_kinds(&self, kind_env: &KindEnv) -> Result<(), Errors> {
        let expected = &kind_env.traits_and_aliases[&self.trait_id];
        let found = self.ty.kind(kind_env)?;
        if *expected != found {
            return Err(Errors::from_msg_srcs(
                format!(
                    "Kind mismatch in `{}`. Expect: {}, found: {}.",
                    self.to_string_normalize(),
                    expected.to_string(),
                    found.to_string()
                ),
                &[&self.source],
            ));
        }
        Ok(())
    }

    // If the trait used in this predicate is a trait alias, resolve it to a set of predicates that are not using trait aliases.
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

    // Find the minimum expression node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        if self.source.is_none() {
            return None;
        }
        let src = self.source.as_ref().unwrap();
        if !src.includes_pos(pos) {
            return None;
        }
        let node = self.ty.find_node_at(pos);
        if node.is_some() {
            return node;
        }
        Some(EndNode::Trait(self.trait_id.clone()))
    }
}
