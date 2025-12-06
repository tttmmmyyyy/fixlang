use std::sync::Arc;

use crate::ast::equality::Equality;
use crate::ast::name::Name;
use crate::ast::predicate::Predicate;
use crate::ast::traits::{AssocTypeKindInfo, KindSignature, TraitId};
use crate::ast::types::{Kind, TyAssoc, TyCon, Type, TypeNode};
use crate::misc::Map;

// Kind environment.
#[derive(Default, Clone)]
pub struct KindEnv {
    pub tycons: Map<TyCon, Arc<Kind>>,
    pub assoc_tys: Map<TyAssoc, AssocTypeKindInfo>,
    pub traits_and_aliases: Map<TraitId, Arc<Kind>>,
}

// Kind scope.
#[derive(Clone, Default)]
pub struct KindScope {
    pub scope: Map<Name, Arc<Kind>>,
}

impl KindScope {
    pub fn new() -> Self {
        Self {
            scope: Map::default(),
        }
    }

    pub fn set_tv(&self, tv: &Arc<crate::ast::types::TyVar>) -> Arc<crate::ast::types::TyVar> {
        if let Some(kind) = self.scope.get(&tv.name) {
            tv.set_kind(kind.clone())
        } else {
            tv.clone()
        }
    }

    pub fn insert(&mut self, tyvar: String, kind: Arc<Kind>) -> Result<(), String> {
        if self.scope.contains_key(&tyvar) {
            if self.scope[&tyvar] != kind {
                return Err(format!("Kind mismatch on type variable `{}`.", tyvar));
            }
        } else {
            self.scope.insert(tyvar, kind);
        }
        Ok(())
    }

    fn extend_by_assoc_ty_application(
        &mut self,
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
                            self.insert(tv.name.clone(), kind.clone())?;
                        }
                        Type::AssocTy(_, _) => {
                            self.extend_by_assoc_ty_application(arg.clone(), kind_env)?;
                        }
                        _ => {}
                    }
                }
            }
            _ => unreachable!("Associated type application expected."),
        }
        Ok(())
    }

    pub fn extend(
        &mut self,
        preds: &Vec<Predicate>,
        eqs: &Vec<Equality>,
        kind_signs: &Vec<KindSignature>,
        kind_env: &KindEnv,
    ) -> Result<(), String> {
        for kp in kind_signs {
            let tyvar = kp.tyvar.clone();
            let kind = kp.kind.clone();
            self.insert(tyvar, kind)?;
        }
        for pred in preds {
            match &pred.ty.ty {
                Type::TyVar(tv) => {
                    let trait_id = &pred.trait_id;
                    if !kind_env.traits_and_aliases.contains_key(trait_id) {
                        panic!("Unknown trait: {}", trait_id.to_string());
                    }
                    let kind = kind_env.traits_and_aliases[trait_id].clone();
                    self.insert(tv.name.clone(), kind)?;
                }
                Type::AssocTy(_, _) => {
                    self.extend_by_assoc_ty_application(pred.ty.clone(), kind_env)?;
                }
                _ => {
                    // Do nothing.
                }
            }
        }
        for eq in eqs {
            self.extend_by_assoc_ty_application(eq.lhs(), kind_env)?;
        }
        Ok(())
    }
}
