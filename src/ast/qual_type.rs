use std::sync::Arc;

use crate::ast::equality::Equality;
use crate::ast::name::Name;
use crate::ast::predicate::Predicate;
use crate::ast::program::{EndNode, TypeEnv};
use crate::ast::traits::KindSignature;
use crate::ast::types::{TyVar, TypeNode};
use crate::elaboration::name_resolution::NameResolutionContext;
use crate::error::Errors;
use crate::misc::Set;
use crate::parse::sourcefile::{SourcePos, Span};

/// A type together with the constraints written in front of it, as a type signature holds them:
/// `[a : Show] a -> String`.
///
/// `Scheme::generalize` turns one into the scheme a global value or a trait member is checked
/// against.
#[derive(Clone)]
pub struct QualType {
    /// Trait constraints, e.g. `a : Show`.
    pub preds: Vec<Predicate>,
    /// Equality constraints on associated types, e.g. `Item c = e`.
    pub eqs: Vec<Equality>,
    /// Kind annotations on type variables, e.g. `f : *->*`.
    pub kind_signs: Vec<KindSignature>,
    /// The type the constraints qualify.
    pub ty: Arc<TypeNode>,
}

impl QualType {
    /// Find the minimum node which includes the specified source code position.
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

    /// The signature as a source line writes it: the kind annotations and the trait constraints in
    /// brackets, then the type.
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

    /// Give every trait, type and associated type named in the constraints and in the type its full
    /// name, read in the context the signature is written in.
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

    /// Expand every type alias standing in the constraints and in the type.
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

    /// Appends to `buf` the type variables standing in the constraints and in the type, each one
    /// once.
    ///
    /// Every variable in `buf` afterwards carries the kind this signature's kind annotations give
    /// its name.
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

    /// Whether the type itself determines `var_name`, so that a use site writing that type says
    /// which type the variable stands for.
    ///
    /// The constraints say nothing here, which is what tells this apart from `Scheme::fixed_vars`:
    /// a constraint on an opaque type variable (`Item ?it = c`) is answered by whichever
    /// implementation is chosen, so it leaves a use site with nothing to choose by.
    pub fn ty_fixes_var(&self, var_name: &Name) -> bool {
        let mut fixed_vars = Set::default();
        self.ty.fixed_vars_to_set(&mut fixed_vars);
        fixed_vars.contains(var_name)
    }

    /// The source of a constraint in which `var_name` stands, if this signature has one.
    ///
    /// A constraint on an opaque type variable (`?it : Iterator`, `Item ?it = Elem c`) is passed
    /// over: it constrains the opaque type, and the variables standing in it are arguments of that
    /// constraint.
    pub fn find_var_in_constraint(&self, var_name: &Name) -> Option<Span> {
        for pred in &self.preds {
            // Skip constraints on opaque type variables.
            if pred.on_opaque_tyvar() {
                continue;
            }
            let mut buf = vec![];
            pred.ty.free_vars_to_vec(&mut buf);
            if buf.iter().any(|tv| &tv.name == var_name) {
                return pred.src.clone();
            }
        }
        for eq in &self.eqs {
            if eq.on_opaque_tyvar() {
                continue;
            }
            let mut buf = vec![];
            eq.free_vars_to_vec(&mut buf);
            if buf.iter().any(|tv| &tv.name == var_name) {
                return eq.src.clone();
            }
        }
        None
    }
}
