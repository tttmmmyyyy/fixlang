use crate::ast::name::Name;
use crate::ast::types::{is_inference_tyvar, type_from_tyvar, Scheme, TyVar, TypeNode};
use crate::elaboration::typecheck::{Substitution, UnificationErr};
use crate::misc::{Map, Set};
use std::sync::Arc;

/// The names one error report prints for the type variables the inference created (`#a3`): `t1`,
/// `t2`, ... in the order the report first mentions them. A name that another type variable of the
/// report already has is skipped, so two variables of one report never print alike.
pub struct ReportTyVarNames {
    /// Every type variable the report mentions, each once, in the order it first mentions them.
    vars: Vec<Arc<TyVar>>,
    /// Replaces each type variable the inference created by one under its printed name.
    renaming: Substitution,
    /// The printed name of each type variable the inference created, by its own name.
    names: Map<Name, Name>,
}

impl ReportTyVarNames {
    /// Names the type variables of a report.
    ///
    /// # Arguments
    /// * `vars` — every type variable the report mentions, in the order it mentions them, the ones
    ///   a scheme generalizes included.
    pub fn new(vars: &[Arc<TyVar>]) -> Self {
        let mut seen = Set::default();
        let vars = vars
            .iter()
            .filter(|tv| seen.insert(tv.name.clone()))
            .cloned()
            .collect::<Vec<_>>();
        let taken = vars
            .iter()
            .filter(|tv| !is_inference_tyvar(&tv.name))
            .map(|tv| tv.name.clone())
            .collect::<Set<_>>();
        let mut renaming = Substitution::default();
        let mut names = Map::default();
        let mut next_no = 1;
        for tv in vars.iter().filter(|tv| is_inference_tyvar(&tv.name)) {
            let name = loop {
                let name = format!("t{}", next_no);
                next_no += 1;
                if !taken.contains(&name) {
                    break name;
                }
            };
            let replacement = type_from_tyvar(tv.set_name(name.clone()));
            let merged = renaming.merge(&Substitution::single(&tv.name, replacement));
            assert!(
                merged,
                "type variable `{}` named twice in one report",
                tv.name
            );
            names.insert(tv.name.clone(), name);
        }
        ReportTyVarNames {
            vars,
            renaming,
            names,
        }
    }

    /// Every type variable the report mentions, each once, in the order it first mentions them.
    pub fn vars(&self) -> &[Arc<TyVar>] {
        &self.vars
    }

    /// The name the report prints for the type variable `name`.
    pub fn name(&self, name: &Name) -> Name {
        match self.names.get(name) {
            Some(printed) => printed.clone(),
            None => {
                assert!(
                    !is_inference_tyvar(name),
                    "type variable `{}` is missing from the variables the report was named for",
                    name
                );
                name.clone()
            }
        }
    }

    /// `ty` as the report prints it.
    pub fn ty(&self, ty: &Arc<TypeNode>) -> String {
        self.renaming.substitute_type(ty).to_string()
    }

    /// `scm` as the report prints it.
    pub fn scheme(&self, scm: &Arc<Scheme>) -> String {
        self.renaming.substitute_scheme(scm).to_string()
    }

    /// `err` with its type variables under the names the report prints.
    pub fn rename_unification_error(&self, err: &UnificationErr) -> UnificationErr {
        self.renaming.substitute_unification_error(err)
    }
}
