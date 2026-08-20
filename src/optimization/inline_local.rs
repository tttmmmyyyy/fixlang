use crate::{
    ast::{name::FullName, program::{Program, Symbol}},
    misc::Map,
    optimization::{application_inlining, let_elimination},
};

/// Inline the local functions of every symbol of `prg`.
pub fn run(prg: &mut Program) {
    let global_lambda_to_arity = let_elimination::create_global_lambda_to_arity_map(&prg.symbols);
    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym, &global_lambda_to_arity);
    }
}

/// Inline the local functions of one symbol: eliminate a `let` that binds a lambda, and reduce an
/// application of a lambda, until neither applies. Together the two turn `let f = |x| {e0}; f(y)`
/// into `{e0}[x := y]`.
///
/// # Parameters
/// * `sym` - the symbol whose expression is rewritten.
/// * `global_lambda_to_arity` - how many parameters each global lambda takes, which is what decides
///   whether a `let` binding one of them may be eliminated.
pub fn run_on_symbol(sym: &mut Symbol, global_lambda_to_arity: &Map<FullName, usize>) {
    let mut expr = sym.expr.as_ref().unwrap().clone();
    loop {
        let mut changed = false;
        changed |= let_elimination::run_on_expr_once(&mut expr, global_lambda_to_arity);
        changed |= application_inlining::run_on_expr_once(&mut expr);
        if !changed {
            break;
        }
    }
    sym.expr = Some(expr);
}
