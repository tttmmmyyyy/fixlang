// Shared machinery for passing what a lambda captures as a value: the unboxed struct that carries
// the captured variables, and generation of a collision-free global name for a function a lambda is
// lifted to.

use crate::{
    ast::{
        expr::{expr_make_struct, expr_var, var_var, ExprNode},
        name::FullName,
        pattern::PatternNode,
        traverse::VisitState,
        typedecl::Field,
        types::{kind_star, type_tycon, TyCon, TyConInfo, TyConVariant, TypeNode},
    },
    misc::Set,
};
use std::sync::Arc;

// The unboxed struct that carries a lambda's captured variables to the global function the lambda
// is lifted to.
//
// The type constructor lives in the namespace of `owner`, and its name is `prefix@owner`, so that a
// value of it says which function it was built for. Two lambdas capturing the same names at the same
// types are distinct here, which is what lets a reader of the type answer "what do I call this
// with".
#[derive(Clone)]
pub struct CaptureStruct {
    // The type constructor a value of this struct is built and destructured with.
    pub tycon: Arc<TyCon>,
    // The definition of `tycon`, which the caller registers into the program's type environment.
    pub tycon_info: TyConInfo,
    // The type of a capture struct value, i.e. `tycon` applied to no arguments.
    pub ty: Arc<TypeNode>,
    // Captured names paired with their types, in the caller's order.
    fields: Vec<(FullName, Arc<TypeNode>)>,
}

impl CaptureStruct {
    // Build the capture struct carrying `fields`. It only describes the type; the caller registers
    // `tycon_info` into the program's type environment.
    //
    // # Arguments
    // * `prefix` - the head of the type constructor's name, which says which pass built the capture
    //   struct.
    // * `owner` - the global name of the function this capture struct is built for. Together with
    //   `prefix`, it tells one capture struct from another.
    // * `fields` - the captured names paired with their types, in the order the struct holds them.
    pub fn new(prefix: &str, owner: &FullName, fields: &[(FullName, Arc<TypeNode>)]) -> Self {
        let tycon = Arc::new(TyCon {
            name: FullName::new(&owner.namespace, &format!("{}@{}", prefix, owner.name)),
        });
        let tycon_info = TyConInfo {
            punched_from: None,
            kind: kind_star(),
            variant: TyConVariant::Struct,
            is_unbox: true,
            tyvars: vec![],
            fields: fields
                .iter()
                .map(|(n, t)| Field::make(n.name.clone(), t.clone(), None))
                .collect(),
            source: None,
            document: None,
        };
        let ty = type_tycon(&tycon);
        Self {
            tycon,
            tycon_info,
            ty,
            fields: fields.to_vec(),
        }
    }

    // Expression building the capture struct from the captured variables currently in scope.
    pub fn struct_expr(&self) -> Arc<ExprNode> {
        expr_make_struct(
            self.tycon.clone(),
            self.fields
                .iter()
                .map(|(n, t)| (n.to_string(), expr_var(n.clone(), None).set_type(t.clone())))
                .collect(),
        )
        .set_type(self.ty.clone())
    }

    // The captured names paired with their types, in the order the struct holds them.
    pub fn fields(&self) -> &[(FullName, Arc<TypeNode>)] {
        &self.fields
    }

    // Pattern destructuring the capture struct back into its original captured names.
    pub fn pattern(&self) -> Arc<PatternNode> {
        let field_pats = self
            .fields
            .iter()
            .map(|(n, t)| {
                (
                    n.to_string(),
                    PatternNode::make_var(var_var(n.clone()), None).set_type(t.clone()),
                )
            })
            .collect();
        PatternNode::make_struct(self.tycon.clone(), field_pats).set_type(self.ty.clone())
    }
}

// The local variables `lam` captures, each paired with the type `state` gives it where `lam` is
// written, in the order `ExprNode::lambda_cap_names` lists them.
pub fn captured_fields(lam: &Arc<ExprNode>, state: &VisitState) -> Vec<(FullName, Arc<TypeNode>)> {
    lam.lambda_cap_names()
        .into_iter()
        .map(|name| {
            let ty = state.scope.get_local(&name.name).unwrap().unwrap();
            (name, ty)
        })
        .collect()
}

// The global names a walk over one symbol mints new ones against: the globals of the program as the
// walk found them, shared by the walks over every symbol, and the names this walk has minted.
//
// A minted name is the walk's symbol followed by a suffix ending in a letter and a counter, so the
// digits at its end are the counter and what precedes the suffix is the symbol. Names minted from
// two different symbols therefore differ, and the walks over different symbols mint against the
// program's globals alone, without seeing what one another minted.
#[derive(Clone)]
pub struct GlobalNames {
    // The globals of the program as the walks found them.
    existing: Arc<Set<FullName>>,
    // The names minted through this value.
    minted: Set<FullName>,
}

impl GlobalNames {
    // The names to mint against for a walk over a program whose globals are `existing`.
    pub fn new(existing: Arc<Set<FullName>>) -> Self {
        GlobalNames {
            existing,
            minted: Set::default(),
        }
    }

    // A fresh global name for a lifted function, derived from `base` (the symbol being processed)
    // plus `suffix` and `counter`. `counter` advances and the chosen name joins the minted ones, so
    // repeated calls stay collision-free against existing globals and against each other.
    pub fn fresh(&mut self, base: &FullName, suffix: &str, counter: &mut u32) -> FullName {
        assert!(
            suffix.ends_with(|c: char| !c.is_ascii_digit()),
            "the suffix `{}` of a minted name ends in a digit, which runs it into the counter",
            suffix
        );
        loop {
            let mut name = base.clone();
            *name.name_as_mut() += &format!("{}{}", suffix, *counter);
            *counter += 1;
            if !self.existing.contains(&name) && self.minted.insert(name.clone()) {
                return name;
            }
        }
    }
}
