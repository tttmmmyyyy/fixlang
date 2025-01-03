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
        let var_name = &expr.get_var().name;
        if *var_name == self.from {
            if state.scope.local_names().contains(&self.to.name) {
                self.fail = true;
                return EndVisitResult::noreplace(expr);
            }
            let expr = expr.set_var_var(expr.get_var().set_name(self.to.clone()));
            EndVisitResult::replace(expr)
        } else {
            EndVisitResult::noreplace(expr)
        }
    }
}
