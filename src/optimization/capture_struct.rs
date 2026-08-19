// Shared machinery for lifting a lambda that captures local variables into a global function: the
// unboxed struct that threads the captured environment through the call, and generation of a
// collision-free global name for the lifted function.
//
// The same description serves a struct the program declares, which `of_declared` reads into this
// shape so that a pass can narrow the fields of either kind alike.

use crate::{
    ast::{
        expr::{expr_make_struct, expr_var, var_var, ExprNode},
        name::FullName,
        pattern::PatternNode,
        typedecl::Field,
        types::{kind_star, type_tycon, TyCon, TyConInfo, TyConVariant, TypeNode},
    },
    misc::Set,
};
use std::sync::Arc;

// A struct with named fields, together with the type constructor a value of it is built and
// destructured with: the one carrying a lambda's captured environment across the call to its lifted
// global function, or one the program declares.
//
// A minted type constructor lives in the namespace of the function the capture struct is built for,
// and is named after it, so that a value of it says which function consumes it. Two lambdas
// capturing the same names at the same types are distinct here, which is what lets a reader of the
// type answer "what do I call this with".
#[derive(Clone)]
pub struct CaptureStruct {
    // The type constructor a value of this struct is built and destructured with.
    pub tycon: Arc<TyCon>,
    // The definition of `tycon`, which the caller registers into the program's type environment.
    pub tycon_info: TyConInfo,
    // The type of a value of this struct, which is `tycon` applied to its arguments.
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
    // * `owner` - the function this capture struct is built for. It is a global name of its own, so
    //   it alone tells one capture struct from another.
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

    // The struct a value of an existing declaration is carried by, at the instance `ty`. Nothing is
    // minted: the type constructor and its declaration are the ones the program already has.
    //
    // # Arguments
    // * `tycon` - the type constructor of the declaration.
    // * `tycon_info` - that declaration.
    // * `ty` - the type of the value, which is `tycon` applied to its arguments.
    // * `fields` - the declared field names paired with their types at that instance.
    pub fn of_declared(
        tycon: Arc<TyCon>,
        tycon_info: &TyConInfo,
        ty: Arc<TypeNode>,
        fields: &[(FullName, Arc<TypeNode>)],
    ) -> Self {
        Self {
            tycon,
            tycon_info: tycon_info.clone(),
            ty,
            fields: fields.to_vec(),
        }
    }

    // The type constructor of this struct with each field punched out, paired with its declaration.
    //
    // A punched field is a hole: it keeps its slot in the layout and holds no value. `act_` and
    // `mod_` hold the rest of a struct in one of these while the field is out, so a struct that
    // stands in for one the program declares brings these along with it.
    pub fn punched_forms(&self) -> Vec<(TyCon, TyConInfo)> {
        (0..self.tycon_info.fields.len())
            .map(|punched_at| {
                let mut tycon = self.tycon.as_ref().clone();
                tycon.into_punched_type_name(punched_at);
                let mut tycon_info = self.tycon_info.clone();
                tycon_info.fields[punched_at].is_punched = true;
                tycon_info.punched_from = Some(self.tycon.as_ref().clone());
                (tycon, tycon_info)
            })
            .collect()
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

// A fresh global name for a lifted function, derived from `base` (the symbol being processed) plus
// `suffix` and `counter`. `counter` advances and the chosen name is inserted into `global_names`, so
// repeated calls stay collision-free against existing globals and against each other.
pub fn fresh_global_name(
    base: &FullName,
    suffix: &str,
    counter: &mut u32,
    global_names: &mut Set<FullName>,
) -> FullName {
    loop {
        let mut name = base.clone();
        *name.name_as_mut() += &format!("{}{}", suffix, *counter);
        *counter += 1;
        if global_names.insert(name.clone()) {
            return name;
        }
    }
}
