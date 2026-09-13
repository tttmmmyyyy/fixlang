use crate::{
    ast::{
        name::FullName,
        program::{Program, Symbol},
    },
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
/// # Arguments
/// * `global_lambda_to_arity` - how many parameters each global lambda takes, which is what decides
///   whether a `let` binding one of them may be eliminated.
pub fn run_on_symbol(sym: &mut Symbol, global_lambda_to_arity: &Map<FullName, usize>) {
    assert_recorded_arity_holds(sym, global_lambda_to_arity);
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
    assert_recorded_arity_holds(sym, global_lambda_to_arity);
}

/// Assert that what `global_lambda_to_arity` records for `sym` is still true of `sym`.
///
/// A caller builds the map from a snapshot of the symbols and then reads it while rewriting those
/// same symbols, so the arity it records for one of them has to stay a lower bound on the
/// parameters that symbol's leading lambdas take. A recorded arity above that count would read a
/// saturated application of the symbol as a strictly partial one
/// (`is_global_lambda_strictly_partially_applied_to_names`), and `let_elimination` would eliminate
/// a `let` that has to stay.
fn assert_recorded_arity_holds(sym: &Symbol, global_lambda_to_arity: &Map<FullName, usize>) {
    let Some(recorded) = global_lambda_to_arity.get(&sym.name) else {
        return;
    };
    let arity = sym.expr.as_ref().unwrap().lam_sequence_arity();
    assert!(
        *recorded <= arity,
        "the arity recorded for `{}` is {}, but its leading lambdas take {}",
        sym.name.to_string(),
        recorded,
        arity
    );
}
