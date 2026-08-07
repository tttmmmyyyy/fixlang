// Shared machinery for lifting a lambda that captures local variables into a global function: the
// unboxed struct that threads the captured environment through the call, and generation of a
// collision-free global name for the lifted function. `closure_specialization` and
// `defunctionalize_fix` both lift lambdas this way and build on these.

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

// The unboxed struct that carries a lambda's captured environment across the call to its lifted
// global function.
//
// The type constructor lives in the namespace of the function the capture struct is built for, and
// is named after it, so that a value of it says which function consumes it. Two lambdas capturing
// the same names at the same types are distinct here, which is what lets a reader of the type answer
// "what do I call this with".
#[derive(Clone)]
pub struct CaptureStruct {
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
    // * `prefix` - the head of the type constructor's name, which keeps the capture structs of one
    //   caller distinct from those of another.
    // * `owner` - the function this capture struct is built for, which the name is made unique by.
    // * `fields` - the captured names paired with their types, in the order the struct holds them.
    pub fn new(prefix: &str, owner: &FullName, fields: &[(FullName, Arc<TypeNode>)]) -> Self {
        let signature = fields
            .iter()
            .map(|(n, t)| format!("{}:{}", n.to_string(), t.to_string()))
            .collect::<Vec<_>>()
            .join(",");
        let tycon = Arc::new(TyCon {
            name: FullName::new(
                &owner.namespace,
                &format!("{}@{}<{}>", prefix, owner.name, signature),
            ),
        });
        let tycon_info = TyConInfo {
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

    // The same capture struct with the type of one field narrowed, as happens when the lambda in
    // that field turns out to be a known one and is threaded through as its capture list instead of
    // as a closure. The prefix and the owner are given again because the type constructor's name is
    // built from them.
    pub fn with_field_type(
        &self,
        prefix: &str,
        owner: &FullName,
        field: usize,
        ty: Arc<TypeNode>,
    ) -> Self {
        let mut fields = self.fields.clone();
        fields[field].1 = ty;
        CaptureStruct::new(prefix, owner, &fields)
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
