use std::sync::Arc;

use crate::{
    ast::{
        name::FullName,
        traverse::{EndVisitResult, ExprVisitor, VisitState},
    },
    ExprNode,
};

// Replace a free variable of an expression to another name.
// If the name `to` is bound at the place `from` appears, returns Err.
pub fn replace_free_var_of_expr(
    expr: &Arc<ExprNode>,
    from: &FullName,
    to: &FullName,
) -> Result<Arc<ExprNode>, ()> {
    let mut replacer = FreeVarReplacer {
        from: from.clone(),
        to: to.clone(),
        fail: false,
    };
    let res = replacer.traverse(expr);
    if replacer.fail {
        return Err(());
    }
    Ok(res.expr)
}

// Old implementation:
//
// // Replace a free variable of an expression to another name.
// // If the name `to` is bound at the place `from` appears, returns Err.
// fn replace_free_var(
//     expr: &Arc<ExprNode>,
//     from: &FullName,
//     to: &FullName,
//     scope: &mut Scope<()>,
// ) -> Result<Arc<ExprNode>, ()> {
//     match &*expr.expr {
//         Expr::Var(v) => {
//             if v.name == *from {
//                 if scope.local_names().contains(&to.name) {
//                     Err(())
//                 } else {
//                     Ok(expr.clone().set_var_var(v.set_name(to.clone())))
//                 }
//             } else {
//                 Ok(expr.clone())
//             }
//         }
//         Expr::LLVM(_) => Ok(expr.clone()),
//         Expr::App(func, args) => {
//             let func = replace_free_var(func, from, to, scope)?;
//             let args = args
//                 .iter()
//                 .map(|arg| replace_free_var(arg, from, to, scope))
//                 .collect::<Result<_, ()>>()?;
//             Ok(expr.set_app_func(func).set_app_args(args))
//         }
//         Expr::Lam(vs, val) => {
//             let val = if vs.iter().any(|v| v.name == *from) {
//                 // This implies that the `from` is shadowed in `val`, so we should not replace val.
//                 val.clone()
//             } else {
//                 for v in vs {
//                     scope.push(&v.name.name, ());
//                 }
//                 let res = replace_free_var(val, from, to, scope)?;
//                 for v in vs {
//                     scope.pop(&v.name.name);
//                 }
//                 res
//             };
//             Ok(expr.set_lam_body(val))
//         }
//         Expr::Let(pat, bound, val) => {
//             let bound = replace_free_var(bound, from, to, scope)?;
//             let val = if pat.pattern.vars().contains(from) {
//                 // This implies that the `from` is shadowed in `val`, so we should not replace val.
//                 val.clone()
//             } else {
//                 for v in pat.pattern.vars() {
//                     scope.push(&v.name, ());
//                 }
//                 let res = replace_free_var(val, from, to, scope)?;
//                 for v in pat.pattern.vars() {
//                     scope.pop(&v.name);
//                 }
//                 res
//             };
//             Ok(expr.set_let_bound(bound).set_let_value(val))
//         }
//         Expr::Match(cond, pat_vals) => {
//             let cond = replace_free_var(cond, from, to, scope)?;
//             let mut new_pat_vals = vec![];
//             for (pat, val) in pat_vals {
//                 if pat.pattern.vars().contains(from) {
//                     // This implies that the `from` is shadowed in `val`, so we should not replace val.
//                     new_pat_vals.push((pat.clone(), val.clone()));
//                     continue;
//                 }
//                 for v in pat.pattern.vars() {
//                     scope.push(&v.name, ());
//                 }
//                 let val = replace_free_var(val, from, to, scope)?;
//                 for v in pat.pattern.vars() {
//                     scope.pop(&v.name);
//                 }
//                 new_pat_vals.push((pat.clone(), val));
//             }
//             Ok(expr.set_match_cond(cond).set_match_pat_vals(new_pat_vals))
//         }
//         Expr::If(c, t, e) => {
//             let c = replace_free_var(c, from, to, scope)?;
//             let t = replace_free_var(t, from, to, scope)?;
//             let e = replace_free_var(e, from, to, scope)?;
//             Ok(expr.set_if_cond(c).set_if_then(t).set_if_else(e))
//         }
//         Expr::TyAnno(e, _) => {
//             let e = replace_free_var(e, from, to, scope)?;
//             Ok(expr.set_tyanno_expr(e))
//         }
//         Expr::MakeStruct(_, fields) => {
//             let mut expr = expr.clone();
//             for (field_name, field_expr) in fields {
//                 let field_expr = replace_free_var(field_expr, from, to, scope)?;
//                 expr = expr.set_make_struct_field(field_name, field_expr);
//             }
//             Ok(expr)
//         }
//         Expr::ArrayLit(elems) => {
//             let mut expr = expr.clone();
//             for (i, e) in elems.iter().enumerate() {
//                 let e = replace_free_var(e, from, to, scope)?;
//                 expr = expr.set_array_lit_elem(e, i);
//             }
//             Ok(expr)
//         }
//         Expr::FFICall(_, _, _, elems, _) => {
//             let mut expr = expr.clone();
//             for (i, e) in elems.iter().enumerate() {
//                 let e = replace_free_var(e, from, to, scope)?;
//                 expr = expr.set_ffi_call_arg(e, i);
//             }
//             Ok(expr)
//         }
//     }
// }

pub struct FreeVarReplacer {
    from: FullName,
    to: FullName,
    fail: bool,
}

impl ExprVisitor for FreeVarReplacer {
    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, state: &mut VisitState) -> EndVisitResult {
        let var = expr.get_var().clone();
        // If the visited variable is not equal to `from`, do nothing.
        if var.name != self.from {
            return EndVisitResult::noreplace(expr);
        }
        let local_names = state.scope.local_names();
        // If `from` is shadowed, do nothing.
        if local_names.contains(&self.from.to_string()) {
            return EndVisitResult::noreplace(expr);
        }
        // If the `to` is shadowed, raise an error.
        if state.scope.local_names().contains(&self.to.name) {
            self.fail = true;
            return EndVisitResult::noreplace(expr);
        }
        let expr = expr.set_var_var(var.set_name(self.to.clone()));
        EndVisitResult::replace(expr)
    }
}
