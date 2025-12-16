use std::sync::Arc;

use crate::ast::kind_scope::{KindEnv, KindScope};
use crate::ast::name::FullName;
use crate::error::Errors;
use crate::misc::Set;
use serde::{Deserialize, Serialize};

use super::*;

// Equality predicate `AssociateType args = value`.
#[derive(Clone, Serialize, Deserialize)]
pub struct Equality {
    pub assoc_type: TyAssoc,
    pub args: Vec<Arc<TypeNode>>,
    pub value: Arc<TypeNode>,
    pub source: Option<Span>,
}

impl Equality {
    // Find the minimum expression node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        if self.source.is_none() {
            return None;
        }
        let src = self.source.as_ref().unwrap();
        if !src.includes_pos(pos) {
            return None;
        }
        let node = self.args.iter().find_map(|arg| arg.find_node_at(pos));
        if node.is_some() {
            return node;
        }
        self.value.find_node_at(pos)
    }

    pub fn free_vars_to_vec(&self, buf: &mut Vec<Arc<TyVar>>) {
        for arg in &self.args {
            arg.free_vars_to_vec(buf);
        }
        self.value.free_vars_to_vec(buf);
    }

    // Collect all referenced type names (both type constructors and associated types).
    pub fn collect_referenced_names(&self, names: &mut Set<FullName>) {
        // Collect the associated type name
        names.insert(self.assoc_type.name.clone());
        // Collect names from arguments
        for arg in &self.args {
            arg.collect_referenced_names(names);
        }
        // Collect names from value
        self.value.collect_referenced_names(names);
    }

    pub fn check_kinds(&self, kind_env: &KindEnv) -> Result<(), Errors> {
        let kind_info = kind_env.assoc_tys.get(&self.assoc_type).unwrap();
        if self.args.len() != kind_info.param_kinds.len() {
            return Err(Errors::from_msg_srcs(
                format!(
                    "Invalid number of arguments for associated type `{}`. Expect: {}, found: {}.",
                    self.assoc_type.name.to_string(),
                    kind_info.param_kinds.len(),
                    self.args.len()
                ),
                &[&self.source],
            ));
        }
        for (arg, expect_kind) in self.args.iter().zip(kind_info.param_kinds.iter()) {
            let found_kind = arg.kind(kind_env)?;
            if *expect_kind != found_kind {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "Kind mismatch in `{}`. Expect: {}, found: {}.",
                        arg.to_string(),
                        expect_kind.to_string(),
                        found_kind.to_string()
                    ),
                    &[&self.source],
                ));
            }
        }
        let found_kind = self.value.kind(kind_env)?;
        if kind_info.value_kind != found_kind {
            return Err(Errors::from_msg_srcs(
                format!(
                    "Kind mismatch in `{}`. Expect: {}, found: {}.",
                    self.value.to_string(),
                    kind_info.value_kind.to_string(),
                    found_kind.to_string()
                ),
                &[&self.source],
            ));
        }
        Ok(())
    }

    pub fn set_kinds(&mut self, scope: &KindScope) {
        for arg in &mut self.args {
            *arg = arg.set_kinds(scope);
        }
        self.value = self.value.set_kinds(scope);
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        for arg in &mut self.args {
            *arg = arg.resolve_type_aliases(type_env)?;
        }
        self.value = self.value.resolve_type_aliases(type_env)?;
        Ok(())
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        self.assoc_type.resolve_namespace(ctx, &self.source)?;
        for arg in &mut self.args {
            *arg = arg.resolve_namespace(ctx)?;
        }
        self.value = self.value.resolve_namespace(ctx)?;
        Ok(())
    }

    pub fn to_string(&self) -> String {
        format!("{} = {}", self.lhs().to_string(), self.value.to_string())
    }

    pub fn free_vars_vec(&self, buf: &mut Vec<Arc<TyVar>>) {
        for arg in &self.args {
            arg.free_vars_to_vec(buf);
        }
        self.value.free_vars_to_vec(buf);
    }

    // Get the type of the left-hand side of the equality.
    pub fn lhs(&self) -> Arc<TypeNode> {
        type_assocty(self.assoc_type.clone(), self.args.clone())
    }

    pub fn generalize(&self) -> EqualityScheme {
        let mut tyvars = vec![];
        for arg in &self.args {
            arg.free_vars_to_vec(&mut tyvars);
        }
        self.value.free_vars_to_vec(&mut tyvars);
        EqualityScheme {
            gen_vars: tyvars,
            equality: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct EqualityScheme {
    pub gen_vars: Vec<Arc<TyVar>>,
    pub equality: Equality,
}
