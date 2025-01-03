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
            return EndVisitResult::unchanged(expr);
        }
        let local_names = state.scope.local_names();
        // If `from` is shadowed, do nothing.
        if local_names.contains(&self.from.to_string()) {
            return EndVisitResult::unchanged(expr);
        }
        // If the `to` is shadowed, raise an error.
        if state.scope.local_names().contains(&self.to.name) {
            self.fail = true;
            return EndVisitResult::unchanged(expr);
        }
        let expr = expr.set_var_var(var.set_name(self.to.clone()));
        EndVisitResult::changed(expr)
    }
}
