// Putting the body of a global function where a caller calls it, at the calls where that moves the
// body rather than copying it.

use super::{
    find_usage_of_name::{self, UsageType},
    inline_local,
    let_elimination::create_global_lambda_to_arity_map,
    rename::substitute_free_name,
};
use crate::{
    ast::{expr::ExprNode, name::FullName, program::Symbol},
    misc::Map,
};
use std::sync::Arc;

/// Put the body of each function `callees_by_caller` lists for a symbol where that symbol calls it,
/// where that moves the body: the symbol is the only one naming the function, and names it once, as
/// the callee of a call supplying every argument. Then reduce the application the body is left in.
///
/// A body goes in one level deep: every body is read as it stands before any of them is put
/// anywhere, and the calls a placed body makes are left as calls. What the program grows by is
/// therefore bounded by the bodies listed, and a function that reaches itself through the bodies it
/// receives is placed once.
///
/// # Arguments
/// * `callees_by_caller` - for each symbol, the functions whose bodies are to be put where it calls
///   them.
pub fn move_bodies_into_callers(
    symbols: &mut Map<FullName, Symbol>,
    callees_by_caller: &Map<FullName, Vec<FullName>>,
) {
    let bodies = callees_by_caller
        .iter()
        .flat_map(|(caller, callees)| callees.iter().map(move |callee| (caller, callee)))
        .map(|(caller, callee)| {
            let sym = symbols.get(callee).unwrap_or_else(|| {
                panic!(
                    "{} is to receive the body of {}, which is no symbol",
                    caller.to_string(),
                    callee.to_string()
                )
            });
            (callee.clone(), sym.expr.as_ref().unwrap().clone())
        })
        .collect::<Map<FullName, Arc<ExprNode>>>();

    // How many symbols name each global. A function named by one symbol alone is one whose body has
    // nowhere else to be, so putting it there moves it rather than copying it.
    let mut naming_symbol_counts: Map<FullName, usize> = Map::default();
    for sym in symbols.values() {
        for name in sym.expr.as_ref().unwrap().free_vars() {
            *naming_symbol_counts.entry(name.clone()).or_insert(0) += 1;
        }
    }

    // How many parameters each global function takes, which says whether a call supplies an
    // argument for all of them.
    let arity_map = create_global_lambda_to_arity_map(symbols);

    for (caller, callees) in callees_by_caller {
        // The body goes in where that is a move rather than a copy: the caller is the only symbol
        // naming the function, and it names it as the callee of one saturated call, so the body ends
        // up in one place and the function itself falls to dead-symbol elimination. Where the
        // function is named anywhere else, or called more than once, placing the body would
        // duplicate it, and how much duplication is worth its gain is the judgement `inline` makes.
        let caller_expr = symbols[caller].expr.as_ref().unwrap().clone();
        let moved_bodies = callees
            .iter()
            .filter(|callee| {
                is_moved_by_placing(callee, &caller_expr, &naming_symbol_counts, &arity_map)
            })
            .map(|callee| (callee.clone(), bodies[callee].clone()))
            .collect::<Map<FullName, Arc<ExprNode>>>();
        if moved_bodies.is_empty() {
            continue;
        }
        // The function is named by the call alone, so putting the body where its name stands leaves
        // the body applied to the arguments the call supplies.
        let mut expr = caller_expr;
        for (callee, body) in moved_bodies {
            expr = substitute_free_name(&expr, &callee, &body);
        }
        let sym = symbols.get_mut(caller).unwrap();
        sym.expr = Some(expr);
        // Reduce what the substitution left: a lambda applied to the arguments the call supplies.
        // Left standing, that application is a closure the program builds on the heap and calls
        // through.
        inline_local::run_on_symbol(sym, &arity_map);
    }
}

/// Whether putting `callee`'s body where `caller_expr` writes its name moves the body rather than
/// copying it: the caller whose body is `caller_expr` is the only symbol naming `callee`, it writes
/// the name once, and that one place is the callee of a call supplying every argument.
///
/// The body goes into every place the name is written, so one place is what makes putting it there a
/// move. What that one place is decides the rest: a name passed as an argument, captured, held as a
/// value, or called with fewer arguments than the function takes leaves a closure where it stands,
/// which is a body that has to stay.
///
/// # Arguments
/// * `caller_expr` - the body of the caller, in which the uses of `callee` are counted.
/// * `naming_symbol_counts` - how many symbols of the program name each global.
/// * `arity_map` - how many parameters each global lambda takes.
fn is_moved_by_placing(
    callee: &FullName,
    caller_expr: &Arc<ExprNode>,
    naming_symbol_counts: &Map<FullName, usize>,
    arity_map: &Map<FullName, usize>,
) -> bool {
    let naming_symbol_count = *naming_symbol_counts.get(callee).unwrap_or_else(|| {
        panic!(
            "the body of {} is to be placed, but no symbol names it",
            callee.to_string()
        )
    });
    if naming_symbol_count != 1 {
        return false;
    }
    let arity = match arity_map.get(callee) {
        Some(arity) => *arity,
        None => return false,
    };
    // The walk records one use per place the name is written, so the caller writes the callee's
    // name once, and writes it as the callee of a call supplying every parameter, exactly when this
    // is the whole of what it says about the name.
    matches!(
        find_usage_of_name::run(caller_expr, callee).as_slice(),
        [UsageType::CalledAsFunction { arg_count }] if *arg_count == arity
    )
}

/// The shapes of a caller at which placing a body moves it, and the shapes at which it would copy it.
#[cfg(test)]
mod tests {
    use super::is_moved_by_placing;
    use crate::ast::expr::{expr_app, expr_let, expr_var, var_var, ExprNode};
    use crate::ast::name::FullName;
    use crate::ast::pattern::PatternNode;
    use crate::constants::{CLOSURE_LAM_SUFFIX, INSTANCIATED_NAME_SEPARATOR};
    use crate::misc::Map;
    use std::sync::Arc;

    /// The name of the global function the `index`-th lambda of `Main::main` was lifted to.
    fn lifted(index: u32) -> FullName {
        FullName::from_strs(
            &["Main"],
            &format!(
                "main{}0123abcd{}{}",
                INSTANCIATED_NAME_SEPARATOR, CLOSURE_LAM_SUFFIX, index
            ),
        )
    }

    /// `func` applied to `arg_count` arguments, written one argument at a time.
    fn call(func: &FullName, arg_count: usize) -> Arc<ExprNode> {
        let mut expr = expr_var(func.clone(), None);
        for index in 0..arg_count {
            let arg = expr_var(FullName::local(&format!("a{}", index)), None);
            expr = expr_app(expr, vec![arg], None);
        }
        expr
    }

    /// The two tables the placement rule reads: `lambda` is named by `naming_symbol_count` symbols
    /// of the program and takes `arity` parameters.
    fn tables(
        lambda: &FullName,
        naming_symbol_count: usize,
        arity: usize,
    ) -> (Map<FullName, usize>, Map<FullName, usize>) {
        (
            [(lambda.clone(), naming_symbol_count)]
                .into_iter()
                .collect(),
            [(lambda.clone(), arity)].into_iter().collect(),
        )
    }

    /// A caller naming the lambda as the callee of one call that supplies every argument is the
    /// shape the body moves at.
    #[test]
    fn one_saturated_call_moves_the_body() {
        let lambda = lifted(0);
        let (naming_symbol_counts, arity_map) = tables(&lambda, 1, 2);
        assert!(is_moved_by_placing(
            &lambda,
            &call(&lambda, 2),
            &naming_symbol_counts,
            &arity_map
        ));
    }

    /// A lambda a second symbol names is one whose body has somewhere else to be.
    #[test]
    fn a_lambda_two_symbols_name_keeps_its_body() {
        let lambda = lifted(0);
        let (naming_symbol_counts, arity_map) = tables(&lambda, 2, 2);
        assert!(!is_moved_by_placing(
            &lambda,
            &call(&lambda, 2),
            &naming_symbol_counts,
            &arity_map
        ));
    }

    /// A call supplying fewer arguments than the lambda takes leaves a closure where the name stood,
    /// so the body stays where it is.
    #[test]
    fn a_call_short_of_an_argument_keeps_the_body() {
        let lambda = lifted(0);
        let (naming_symbol_counts, arity_map) = tables(&lambda, 1, 3);
        assert!(!is_moved_by_placing(
            &lambda,
            &call(&lambda, 2),
            &naming_symbol_counts,
            &arity_map
        ));
    }

    /// A call supplying more arguments than the lambda takes calls what the lambda returns.
    #[test]
    fn a_call_past_the_last_parameter_keeps_the_body() {
        let lambda = lifted(0);
        let (naming_symbol_counts, arity_map) = tables(&lambda, 1, 2);
        assert!(!is_moved_by_placing(
            &lambda,
            &call(&lambda, 3),
            &naming_symbol_counts,
            &arity_map
        ));
    }

    /// Two calls name the lambda in two places, so the body stays where it is even where the two
    /// together supply as many arguments as one saturated call would.
    #[test]
    fn two_calls_keep_the_body() {
        let lambda = lifted(0);
        let (naming_symbol_counts, arity_map) = tables(&lambda, 1, 2);
        let other_callee = FullName::from_strs(&["Main"], "g#0123abcd");
        let caller_expr = expr_app(
            expr_app(expr_var(other_callee, None), vec![call(&lambda, 1)], None),
            vec![call(&lambda, 1)],
            None,
        );
        assert!(!is_moved_by_placing(
            &lambda,
            &caller_expr,
            &naming_symbol_counts,
            &arity_map
        ));
    }

    /// A lambda handed to a call as an argument is one the body would have to stay behind for.
    #[test]
    fn a_lambda_passed_as_an_argument_keeps_its_body() {
        let lambda = lifted(0);
        let (naming_symbol_counts, arity_map) = tables(&lambda, 1, 2);
        let other_callee = FullName::from_strs(&["Main"], "g#0123abcd");
        let caller_expr = expr_app(
            expr_app(
                expr_var(other_callee, None),
                vec![expr_var(lambda.clone(), None)],
                None,
            ),
            vec![call(&lambda, 2)],
            None,
        );
        assert!(!is_moved_by_placing(
            &lambda,
            &caller_expr,
            &naming_symbol_counts,
            &arity_map
        ));
    }

    /// A lambda a `let` also binds is written in two places, so putting the body where the name
    /// stands writes it into both rather than moving it. `find_usage_of_name` records nothing for
    /// the `let`, which is why the rule counts the places the name is written.
    #[test]
    fn a_lambda_a_let_also_binds_keeps_its_body() {
        let lambda = lifted(0);
        let (naming_symbol_counts, arity_map) = tables(&lambda, 1, 2);
        let caller_expr = expr_let(
            PatternNode::make_var(var_var(FullName::local("v")), None),
            expr_var(lambda.clone(), None),
            call(&lambda, 2),
            None,
        );
        assert!(!is_moved_by_placing(
            &lambda,
            &caller_expr,
            &naming_symbol_counts,
            &arity_map
        ));
    }

    /// A lambda the arity table does not answer for is one the rule cannot judge.
    #[test]
    fn a_lambda_the_arity_table_does_not_answer_for_keeps_its_body() {
        let lambda = lifted(0);
        let naming_symbol_counts = [(lambda.clone(), 1)].into_iter().collect();
        assert!(!is_moved_by_placing(
            &lambda,
            &call(&lambda, 2),
            &naming_symbol_counts,
            &Map::default()
        ));
    }
}
