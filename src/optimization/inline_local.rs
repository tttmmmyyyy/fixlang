use crate::{
    ast::program::Program,
    optimization::{application_inlining, let_elimination},
};

pub fn run(prg: &mut Program) {
    // Perform let elimination and application inlining as "inlining of local functions."
    // This transforms expressions like `let f = |x| {e0}; in f(y)` to `{e0}[x:=y]`.
    let global_lambda_to_arity = let_elimination::create_global_lambda_to_arity_map(prg);
    for (_name, sym) in &mut prg.symbols {
        let mut expr = sym.expr.as_ref().unwrap().clone();
        loop {
            let mut changed = false;
            // changed |= pull_let::run_on_expr_once(&mut expr);
            changed |= let_elimination::run_on_expr_once(&mut expr, &global_lambda_to_arity);
            changed |= application_inlining::run_on_expr_once(&mut expr);
            if !changed {
                break;
            }
        }
        sym.expr = Some(expr);
    }
}
