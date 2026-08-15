use std::sync::Arc;
use crate::ast::equality::Equality;
use crate::ast::predicate::Predicate;
use crate::ast::program::{EndNode, TypeEnv};
use crate::ast::traits::KindSignature;
use crate::ast::types::TyVar;
use crate::elaboration::name_resolution::NameResolutionContext;
use crate::error::Errors;
use crate::parse::sourcefile::SourcePos;

/// A predicate together with the constraints under which it holds, as `[a : Eq] Array a : Eq` says
/// that `Array a` implements `Eq` whenever `a` does.
#[derive(Clone)]
pub struct QualPred {
    /// The trait bounds among the constraints, such as `a : Eq`.
    pub pred_constraints: Vec<Predicate>,
    /// The equalities among the constraints, such as `Item it = I64`.
    pub eq_constraints: Vec<Equality>,
    /// The kind signatures among the constraints, such as `f : *->*`.
    pub kind_constraints: Vec<KindSignature>,
    /// The predicate the constraints qualify, i.e. the head: `Array a : Eq` above.
    pub predicate: Predicate,
}

impl QualPred {
    /// The innermost node covering `pos` among the head, the trait constraints and the equality
    /// constraints.
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

    /// Appends to `buf` the type variables free in the constraints and in the head. Every variable
    /// in `buf` that a kind constraint names — those appended here and those `buf` already held —
    /// takes the kind that constraint gives it.
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

    /// Renders this qualified predicate the way it is written in source: the kind constraints and
    /// the trait constraints in `[...]`, then the head.
    ///
    /// # Examples
    /// The trait constraint `a : Eq` with the head `Array a : Eq` renders as
    /// `[a : Eq] Array a : Eq`, and a head carrying no kind or trait constraint renders alone, as
    /// `Array I64 : Eq`.
    pub fn to_string(&self) -> String {
        let mut s = String::default();
        if self.pred_constraints.len() > 0 || self.kind_constraints.len() > 0 {
            s += "[";
        }
        let mut constraints = vec![];
        constraints.extend(self.kind_constraints.iter().map(|c| c.to_string()));
        constraints.extend(self.pred_constraints.iter().map(|c| c.to_string()));
        s += &constraints.join(", ");
        if self.pred_constraints.len() > 0 || self.kind_constraints.len() > 0 {
            s += "] ";
        }
        s += &self.predicate.to_string();
        s
    }

    /// Gives the names in the constraints and in the head the full names `ctx` resolves them to,
    /// reporting every name whose resolution fails.
    pub fn resolve_namespace(&mut self, ctx: &mut NameResolutionContext) -> Result<(), Errors> {
        let mut errors = Errors::empty();
        for pred in &mut self.pred_constraints {
            errors.eat_err(pred.resolve_namespace(ctx));
        }
        for eq in &mut self.eq_constraints {
            errors.eat_err(eq.resolve_namespace(ctx));
        }
        errors.eat_err(self.predicate.resolve_namespace(ctx));
        errors.to_result()
    }

    /// Replaces the type aliases appearing in the constraints and in the head with the types they
    /// stand for.
    pub fn resolve_type_aliases(&mut self, type_env: &TypeEnv) -> Result<(), Errors> {
        for pred in &mut self.pred_constraints {
            pred.resolve_type_aliases(type_env)?;
        }
        for eq in &mut self.eq_constraints {
            eq.resolve_type_aliases(type_env)?;
        }
        self.predicate.resolve_type_aliases(type_env)?;
        Ok(())
    }
}

/// A qualified predicate holding for every instantiation of the type variables it is generalized
/// over, such as `Array a : Eq` generalized over `a`.
#[derive(Clone)]
pub struct QualPredScheme {
    /// The generalized type variables, which a use of this scheme instantiates afresh.
    pub gen_vars: Vec<Arc<TyVar>>,
    /// The qualified predicate the generalized variables appear in.
    pub qual_pred: QualPred,
}
