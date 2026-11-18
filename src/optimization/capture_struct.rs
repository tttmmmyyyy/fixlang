// Shared machinery for passing what a lambda captures as a value: the unboxed struct that carries
// the captured variables to the global function the lambda is lifted to.

use crate::{
    ast::{
        expr::{
            expr_abs, expr_abs_typed, expr_let_typed, expr_make_struct, expr_var, var_var, Expr,
            ExprNode,
        },
        name::FullName,
        pattern::PatternNode,
        traverse::VisitState,
        typedecl::Field,
        types::{kind_star, type_tycon, TyCon, TyConInfo, TyConVariant, TypeNode},
    },
    optimization::rename::rename_lam_param_avoiding,
};
use std::sync::Arc;

/// The unboxed struct that carries a lambda's captured variables to the global function the lambda
/// is lifted to.
///
/// The type constructor lives in the namespace of `owner`, and its name is `prefix@owner`, so that a
/// value of it says which function it was built for. Two lambdas capturing the same names at the same
/// types are distinct here, which is what lets a reader of the type answer "what do I call this
/// with".
#[derive(Clone)]
pub struct CaptureStruct {
    /// The type constructor a value of this struct is built and destructured with.
    pub tycon: Arc<TyCon>,
    /// The definition of `tycon`, which the caller registers into the program's type environment.
    pub tycon_info: TyConInfo,
    /// The type of a capture struct value, i.e. `tycon` applied to no arguments.
    pub ty: Arc<TypeNode>,
    /// Captured names paired with their types, in the caller's order.
    fields: Vec<(FullName, Arc<TypeNode>)>,
}

impl CaptureStruct {
    /// Build the capture struct carrying `fields`. It only describes the type; the caller registers
    /// `tycon_info` into the program's type environment.
    ///
    /// # Arguments
    /// * `prefix` - the head of the type constructor's name, which says which pass built the capture
    ///   struct.
    /// * `owner` - the global name of the function this capture struct is built for. Together with
    ///   `prefix`, it tells one capture struct from another.
    /// * `fields` - the captured names paired with their types, in the order the struct holds them.
    pub fn new(prefix: &str, owner: &FullName, fields: &[(FullName, Arc<TypeNode>)]) -> Self {
        let tycon = Arc::new(TyCon {
            name: Self::tycon_name(prefix, owner),
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

    /// The name of the type constructor `new` gives the capture struct built with `prefix` for
    /// `owner`.
    pub fn tycon_name(prefix: &str, owner: &FullName) -> FullName {
        FullName::new(&owner.namespace, &format!("{}@{}", prefix, owner.name))
    }

    /// Expression building the capture struct from the captured variables currently in scope.
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

    /// The global function a lambda is lifted to: it takes the capture struct as `param`, binds the
    /// captured names to its fields, and evaluates to `body`, the lambda.
    ///
    /// # Examples
    /// For a struct capturing `n`, `body` `|x| x + n` gives `|param| |x| let Cap { n } = param; x + n`.
    pub fn lifted_function(&self, param: &FullName, body: Arc<ExprNode>) -> Arc<ExprNode> {
        let unpacked = expr_let_typed(
            self.pattern(),
            expr_var(param.clone(), None).set_type(self.ty.clone()),
            body,
        );
        internalize_let_to_var_at_head(&expr_abs_typed(
            var_var(param.clone()),
            self.ty.clone(),
            unpacked,
        ))
    }

    /// Pattern destructuring the capture struct back into its original captured names.
    pub fn pattern(&self) -> Arc<PatternNode> {
        let field_pats = self
            .fields
            .iter()
            .map(|(n, t)| (n.to_string(), PatternNode::make_local_var(n, t.clone())))
            .collect();
        PatternNode::make_struct(self.tycon.clone(), field_pats).set_type(self.ty.clone())
    }
}

/// The local variables `lam` captures, each paired with the type `state` gives it where `lam` is
/// written, in the order `ExprNode::lambda_cap_names` lists them.
pub fn captured_fields(lam: &Arc<ExprNode>, state: &VisitState) -> Vec<(FullName, Arc<TypeNode>)> {
    lam.lambda_cap_names()
        .into_iter()
        .map(|name| {
            let ty = state.scope.get_local(&name.name).unwrap().unwrap();
            (name, ty)
        })
        .collect()
}

/// Convert `let a = x in |b| y` to `|b| let a = x in y` if `x` is a variable expression.
fn internalize_let_to_var_one(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    // Check if the expression is in the form of `let a = x in |b| y`.
    if !expr.is_let() {
        return expr.clone();
    }
    let lam = expr.get_let_value();
    if !lam.is_lam() {
        return expr.clone();
    }
    let pat_a = expr.get_let_pat();
    let bound_x = expr.get_let_bound();
    if !bound_x.is_var() {
        return expr.clone();
    }

    // Rename the parameter of the lambda so that it is not contained in `FV(bound_x) + FV(pat_a)`.
    let mut black_list = pat_a.pattern.vars();
    black_list.extend(&mut bound_x.free_vars().into_iter());
    let lam = rename_lam_param_avoiding(&black_list, lam);

    // Construct the expression.
    let params_b = lam.get_lam_params();
    let body_y = lam.get_lam_body();
    let new_expr = expr_let_typed(pat_a.clone(), bound_x.clone(), body_y.clone());
    let new_expr = expr_abs(params_b, new_expr, None);
    new_expr.set_type(expr.type_.clone().unwrap())
}

/// Rewrites the head of `expr` to begin with as many nested lambdas as possible, by moving each
/// `let` binding of a variable that stands between them inside the lambdas that follow it.
fn internalize_let_to_var_at_head(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Lam(_, body) => {
            let body = internalize_let_to_var_at_head(body);
            expr.set_lam_body(body)
        }
        Expr::Let(_, _, val) => {
            // Before applying `internalize_let_to_var_one` into the whole let expression,
            // apply it to the value of the let expression.
            // This increases the chance of applying `internalize_let_to_var_one` by changing the value to a lambda expression.
            let val = internalize_let_to_var_at_head(val);
            let expr = expr.set_let_value_typed(val);

            // Apply `internalize_let_to_var_one` to the whole let expression.
            let expr = internalize_let_to_var_one(&expr);

            // If the whole expression changed into a lambda expression, apply `internalize_let_to_var_at_head` again.
            match &*expr.expr {
                Expr::Lam(_, _) => internalize_let_to_var_at_head(&expr),
                _ => expr,
            }
        }
        _ => expr.clone(),
    }
}
