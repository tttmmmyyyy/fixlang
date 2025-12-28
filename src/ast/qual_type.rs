use std::sync::Arc;

use crate::ast::equality::Equality;
use crate::ast::predicate::Predicate;
use crate::ast::program::{EndNode, TypeEnv};
use crate::ast::traits::KindSignature;
use crate::ast::types::{TyVar, TypeNode};
use crate::error::Errors;
use crate::name_resolution::NameResolutionContext;
use crate::sourcefile::SourcePos;

#[derive(Clone)]
pub struct QualType {
    pub preds: Vec<Predicate>,
    pub eqs: Vec<Equality>,
    pub kind_signs: Vec<KindSignature>,
    pub ty: Arc<TypeNode>,
}

impl QualType {
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        let node = self.ty.find_node_at(pos);
        if node.is_some() {
            return node;
        }
        for pred in &self.preds {
            let node = pred.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        for eq in &self.eqs {
            let node = eq.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        None
    }

    pub fn to_string(&self) -> String {
        let mut s = String::default();
        if self.preds.len() > 0 || self.kind_signs.len() > 0 {
            s += "[";
        }
        let mut preds = vec![];
        preds.extend(self.kind_signs.iter().map(|p| p.to_string()));
        preds.extend(self.preds.iter().map(|p| p.to_string()));
        s += &preds.join(", ");
        if self.preds.len() > 0 || self.kind_signs.len() > 0 {
            s += "] ";
        }
        s += &self.ty.to_string();
        s
    }

    // Resolve namespace.
    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
        for pred in &mut self.preds {
            pred.resolve_namespace(ctx)?;
        }
        for eq in &mut self.eqs {
            eq.resolve_namespace(ctx)?;
        }
        self.ty = self.ty.resolve_namespace(ctx)?;
        Ok(())
    }

    // Resolve type aliases
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        for pred in &mut self.preds {
            pred.resolve_type_aliases(type_env)?;
        }
        for eq in &mut self.eqs {
            eq.resolve_type_aliases(type_env)?;
        }
        self.ty = self.ty.resolve_type_aliases(type_env)?;
        Ok(())
    }

    pub fn free_vars_vec(&self, buf: &mut Vec<Arc<TyVar>>) {
        for pred in &self.preds {
            pred.ty.free_vars_to_vec(buf);
        }
        for eq in &self.eqs {
            eq.free_vars_vec(buf);
        }
        self.ty.free_vars_to_vec(buf);
        // Apply kind predicates.
        for tv in buf {
            for kind_sign in &self.kind_signs {
                if tv.name == kind_sign.tyvar {
                    *tv = tv.set_kind(kind_sign.kind.clone());
                }
            }
        }
    }
}
