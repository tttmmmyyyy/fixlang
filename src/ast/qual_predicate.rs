use std::sync::Arc;

use crate::ast::equality::Equality;
use crate::ast::name::Name;
use crate::ast::program::{EndNode, NameResolutionContext, TypeEnv};
use crate::ast::traits::{KindSignature, Predicate};
use crate::ast::types::{Kind, KindEnv, TyVar, Type, TypeNode};
use crate::error::Errors;
use crate::misc::Map;
use crate::sourcefile::SourcePos;

// Qualified predicate. Statement such as "[a : Eq] Array a : Eq".
// Constraints in `[...]` can be trait bound and equality.
#[derive(Clone)]
pub struct QualPredicate {
    pub pred_constraints: Vec<Predicate>,
    pub eq_constraints: Vec<Equality>,
    pub kind_constraints: Vec<KindSignature>,
    pub predicate: Predicate,
}

impl QualPredicate {
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

    pub fn extend_kind_scope(
        scope: &mut Map<Name, Arc<Kind>>,
        preds: &Vec<Predicate>,
        eqs: &Vec<Equality>,
        kind_signs: &Vec<KindSignature>,
        kind_env: &KindEnv,
    ) -> Result<(), String> {
        fn insert(
            scope: &mut Map<Name, Arc<Kind>>,
            tyvar: String,
            kind: Arc<Kind>,
        ) -> Result<(), String> {
            if scope.contains_key(&tyvar) {
                if scope[&tyvar] != kind {
                    return Err(format!("Kind mismatch on type variable `{}`.", tyvar));
                }
            } else {
                scope.insert(tyvar, kind);
            }
            Ok(())
        }
        fn extend_by_assoc_ty_application(
            scope: &mut Map<Name, Arc<Kind>>,
            assoc_ty_app: Arc<TypeNode>,
            kind_env: &KindEnv,
        ) -> Result<(), String> {
            match &assoc_ty_app.ty {
                Type::AssocTy(assoc_ty, args) => {
                    let kind_info = kind_env.assoc_tys.get(assoc_ty).unwrap();
                    if args.len() != kind_info.param_kinds.len() {
                        return Err(format!(
                            "Invalid number of arguments for associated type `{}`. Expect: {}, found: {}.",
                            assoc_ty.name.to_string(),
                            kind_info.param_kinds.len(),
                            args.len()
                        ));
                    }
                    for (arg, kind) in args.iter().zip(kind_info.param_kinds.iter()) {
                        match &arg.ty {
                            Type::TyVar(tv) => {
                                insert(scope, tv.name.clone(), kind.clone())?;
                            }
                            Type::AssocTy(_, _) => {
                                extend_by_assoc_ty_application(scope, arg.clone(), kind_env)?;
                            }
                            _ => {}
                        }
                    }
                }
                _ => unreachable!("Associated type application expected."),
            }
            Ok(())
        }

        for kp in kind_signs {
            let tyvar = kp.tyvar.clone();
            let kind = kp.kind.clone();
            insert(scope, tyvar, kind)?;
        }
        for pred in preds {
            match &pred.ty.ty {
                Type::TyVar(tv) => {
                    let trait_id = &pred.trait_id;
                    if !kind_env.traits_and_aliases.contains_key(trait_id) {
                        panic!("Unknown trait: {}", trait_id.to_string());
                    }
                    let kind = kind_env.traits_and_aliases[trait_id].clone();
                    insert(scope, tv.name.clone(), kind)?;
                }
                Type::AssocTy(_, _) => {
                    extend_by_assoc_ty_application(scope, pred.ty.clone(), kind_env)?;
                }
                _ => {
                    // Do nothing.
                }
            }
        }
        for eq in eqs {
            extend_by_assoc_ty_application(scope, eq.lhs(), kind_env)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct QualPredScheme {
    pub gen_vars: Vec<Arc<TyVar>>,
    pub qual_pred: QualPredicate,
}
