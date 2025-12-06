use std::sync::Arc;

use crate::ast::equality::Equality;
use crate::ast::predicate::Predicate;
use crate::ast::program::{EndNode, NameResolutionContext, TypeEnv};
use crate::ast::traits::KindSignature;
use crate::ast::types::TyVar;
use crate::error::Errors;
use crate::sourcefile::SourcePos;

// Qualified predicate. Statement such as "[a : Eq] Array a : Eq".
// Constraints in `[...]` can be trait bound and equality.
#[derive(Clone)]
pub struct QualPred {
    pub pred_constraints: Vec<Predicate>,
    pub eq_constraints: Vec<Equality>,
    pub kind_constraints: Vec<KindSignature>,
    pub predicate: Predicate,
}

impl QualPred {
    // Find the minimum node which includes the specified source code position.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        let node = self.predicate.find_node_at(pos);
        if node.is_some() {
            return node;
        }
        for pred in &self.pred_constraints {
            let node = pred.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        for eq in &self.eq_constraints {
            let node = eq.find_node_at(pos);
            if node.is_some() {
                return node;
            }
        }
        None
    }

    pub fn free_vars_vec(&self, buf: &mut Vec<Arc<TyVar>>) {
        for pred in &self.pred_constraints {
            pred.ty.free_vars_to_vec(buf);
        }
        for eq in &self.eq_constraints {
            eq.free_vars_vec(buf);
        }
        self.predicate.ty.free_vars_to_vec(buf);
        // Apply kind predicates.
        for tv in buf {
            for kind_sign in &self.kind_constraints {
                if tv.name == kind_sign.tyvar {
                    *tv = tv.set_kind(kind_sign.kind.clone());
                }
            }
        }
    }

    pub fn to_string(&self) -> String {
        let mut s = String::default();
        if self.pred_constraints.len() > 0 || self.kind_constraints.len() > 0 {
            s += "[";
        }
        let mut preds = vec![];
        preds.extend(self.kind_constraints.iter().map(|p| p.to_string()));
        preds.extend(self.pred_constraints.iter().map(|p| p.to_string()));
        s += &preds.join(", ");
        if self.pred_constraints.len() > 0 || self.kind_constraints.len() > 0 {
            s += "] ";
        }
        s += &self.predicate.to_string();
        s
    }

    pub fn resolve_namespace(&mut self, ctx: &NameResolutionContext) -> Result<(), Errors> {
        for p in &mut self.pred_constraints {
            p.resolve_namespace(ctx)?;
        }
        for eq in &mut self.eq_constraints {
            eq.resolve_namespace(ctx)?;
        }
        self.predicate.resolve_namespace(ctx)?;
        Ok(())
    }

    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        for p in &mut self.pred_constraints {
            p.resolve_type_aliases(type_env)?;
        }
        for eq in &mut self.eq_constraints {
            eq.resolve_type_aliases(type_env)?;
        }
        self.predicate.resolve_type_aliases(type_env)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct QualPredScheme {
    pub gen_vars: Vec<Arc<TyVar>>,
    pub qual_pred: QualPred,
}
