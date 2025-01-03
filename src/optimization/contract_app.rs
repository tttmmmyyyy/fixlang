/*
Contract application optimization.

This optimization tries to reduce cost of "create lambda and apply" expressions.

1. Moves application of variable into inner expression.

For example, if `v` is a variable,

```
(if c {{expr0}} else {{expr1}})(v)
```

is transformed into

```
if c {{expr0}(v)} else {{expr1}(v)}
```

2. Replaces application of lambda expression to a variable expression with substitution.

For example, if `v` is a variable,
```
(|x| {expr})(v)
```

is transformed into

```
{expr}[v/x]
```

Do the above transformations only if `v` is a variable, because if not, `v` is calculated many times and the optimization may increase the cost.
*/

use std::sync::Arc;

use crate::{
    ast::traverse::{EndVisitResult, ExprVisitor, VisitState},
    Expr, ExprNode, InstantiatedSymbol, Program,
};

pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.instantiated_symbols {
        run_on_symbol(sym);
    }
}

fn run_on_symbol(sym: &mut InstantiatedSymbol) {
    loop {
        let mut optimizer = ContractAppOptimizer {};
        let res = optimizer.traverse(&sym.expr.as_ref().unwrap());
        if res.changed {
            sym.expr = Some(res.expr);
        } else {
            break;
        }
    }
}

struct ContractAppOptimizer {}

impl ExprVisitor for ContractAppOptimizer {
    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // Get the argument of the application.
        let args = expr.get_app_args();
        if args.len() > 1 {
            // This optimiza does not support multiple arguments.
            return EndVisitResult::unchanged(expr);
        }
        let arg = args[0].clone();
        if !arg.is_var() {
            return EndVisitResult::unchanged(expr);
        }

        // Get the function applied to the argument.
        let func = expr.get_app_func();
        match &*func.expr {
            Expr::App(_expr_node, _vec) => todo!(),
            Expr::Lam(_vec, _expr_node) => todo!(),
            Expr::Let(_pattern_node, _expr_node, _expr_node1) => todo!(),
            Expr::If(_expr_node, _expr_node1, _expr_node2) => todo!(),
            Expr::Match(_expr_node, _vec) => todo!(),
            Expr::Var(_var) => {
                return EndVisitResult::unchanged(expr);
            }
            Expr::LLVM(_inline_llvm) => {
                return EndVisitResult::unchanged(expr);
            }
            Expr::TyAnno(_expr_node, _type_node) => {
                // If remove tyanno optimization is done, this case should not happen.
                return EndVisitResult::unchanged(expr);
            }
            Expr::ArrayLit(_vec) => {
                return EndVisitResult::unchanged(expr);
            }
            Expr::MakeStruct(_ty_con, _vec) => {
                return EndVisitResult::unchanged(expr);
            }
            Expr::FFICall(_, _ty_con, _vec, _vec1, _) => {
                return EndVisitResult::unchanged(expr);
            }
        }
    }
}
