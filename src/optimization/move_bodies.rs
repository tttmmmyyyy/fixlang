// Putting the body of a global lambda where a caller calls it, at the calls where that moves the
// body rather than copying it.

use super::{
    find_usage_of_name::{self, UsageType},
    inline_local,
    let_elimination::create_global_lambda_to_arity_map,
    rename::substitute_free_names,
};
use crate::{
    ast::{expr::ExprNode, name::FullName, program::Symbol},
    misc::{map_on_compiler_threads, Map},
};
use std::sync::Arc;

/// Put the body of each global lambda `callees_by_caller` lists for a symbol where that symbol calls
/// it, where that moves the body: the symbol is the only one naming the lambda, and names it once, as
/// the callee of a call. Then reduce the application the body is left in.
///
/// A body goes in one level deep: every body is read as it stands before any of them is put
/// anywhere, and the calls a placed body makes are left as calls. What the program grows by is
/// therefore bounded by the bodies listed, and a lambda that reaches itself through the bodies it
/// receives is placed once.
///
/// The callers are rewritten independently of one another, on up to `num_threads` threads.
///
/// # Arguments
/// * `callees_by_caller` - for each symbol, the global lambdas whose bodies are to be put where it
///   calls them. Listing a global that is not a lambda is a bug of the caller: such a global is a
///   value the program computes once, and putting its body at a call would compute it at each call.
pub fn move_bodies_into_callers(
    symbols: &mut Map<FullName, Symbol>,
    callees_by_caller: &Map<FullName, Vec<FullName>>,
    num_threads: usize,
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
            let body = sym.expr.as_ref().unwrap();
            assert!(
                body.is_lam(),
                "{} is to receive the body of {}, which is no lambda",
                caller.to_string(),
                callee.to_string()
            );
            (callee.clone(), body.clone())
        })
        .collect::<Map<FullName, Arc<ExprNode>>>();

    // How many symbols name each global. A lambda named by one symbol alone is one whose body has
    // nowhere else to be, so putting it there moves it rather than copying it.
    let mut naming_symbol_counts: Map<FullName, usize> = Map::default();
    for sym in symbols.values() {
        for name in sym.expr.as_ref().unwrap().free_vars() {
            *naming_symbol_counts.entry(name.clone()).or_insert(0) += 1;
        }
    }

    // How many parameters each global lambda takes, which the reduction of a placed body reads.
    let arity_map = create_global_lambda_to_arity_map(symbols);

    let rewritten_callers = map_on_compiler_threads(
        callees_by_caller.iter().collect(),
        num_threads,
        |(caller, callees)| {
            // The body goes in where that is a move rather than a copy: the caller is the only
            // symbol naming the lambda, and it names it as the callee of one call, so the body
            // ends up in one place and the lambda itself falls to dead-symbol
            // elimination. Where the lambda is named anywhere else, or called more than once,
            // placing the body would duplicate it, and how much duplication is worth its gain is the
            // judgement `inline` makes.
            let caller_expr = symbols[caller].expr.as_ref().unwrap();
            let moved_bodies = callees
                .iter()
                .filter(|callee| is_moved_by_placing(callee, caller_expr, &naming_symbol_counts))
                .map(|callee| (callee.clone(), bodies[callee].clone()))
                .collect::<Map<FullName, Arc<ExprNode>>>();
            if moved_bodies.is_empty() {
                return None;
            }
            // The lambda is named by the call alone, so putting the body where its name stands
            // leaves the body applied to the arguments the call supplies, however many they are.
            let mut sym = symbols[caller].clone();
            sym.expr = Some(substitute_free_names(caller_expr, moved_bodies));
            // Reduce what the substitution left: a lambda applied to the arguments the call
            // supplies. Left standing, that application is a closure the program builds on the heap
            // and calls through.
            inline_local::run_on_symbol(&mut sym, &arity_map);
            Some((caller, sym))
        },
    );
    for (caller, sym) in rewritten_callers.into_iter().flatten() {
        *symbols.get_mut(caller).unwrap() = sym;
    }
}

/// Whether putting the body of the global lambda `callee` where `caller_expr` writes its name moves
/// the body rather than copying it: the caller whose body is `caller_expr` is the only symbol naming
/// `callee`, it writes the name once, and that one place is the callee of a call.
///
/// The body goes into every place the name is written, so one place is what makes putting it there a
/// move. What that one place is decides the rest: a name passed as an argument, captured, or held as
/// a value leaves the lambda itself standing there, which is a body that has to stay. A call
/// supplying fewer arguments than the parameters the nested lambdas of the body declare leaves the
/// rest of them as a lambda in the caller, holding what the partial application would hold; one
/// supplying more applies what the body returns to the remaining arguments. Either way the body is in
/// one place.
///
/// # Arguments
/// * `caller_expr` - the body of the caller, in which the uses of `callee` are counted.
/// * `naming_symbol_counts` - how many symbols of the program name each global.
fn is_moved_by_placing(
    callee: &FullName,
    caller_expr: &Arc<ExprNode>,
    naming_symbol_counts: &Map<FullName, usize>,
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
    // The walk records one use per place the name is written, so the caller writes the callee's
    // name once, and writes it as the callee of a call, exactly when this is the whole of what it
    // says about the name.
    matches!(
        find_usage_of_name::run(caller_expr, callee).as_slice(),
        [UsageType::CalledAsFunction]
    )
}

/// The shapes of a caller at which placing a body moves it, and the shapes at which it would copy it.
#[cfg(test)]
mod tests {
    use super::is_moved_by_placing;
    use crate::ast::expr::{expr_app, expr_let, expr_var, var_var, ExprNode};
    use crate::ast::name::FullName;
    use crate::ast::pattern::PatternNode;
    use crate::constants::{INSTANCIATED_NAME_SEPARATOR, LS_LAM_SUFFIX};
    use crate::misc::Map;
    use std::sync::Arc;

    /// The name of the global lambda the `index`-th lambda of `Main::main` was lifted to.
    fn lifted(index: u32) -> FullName {
        FullName::from_strs(
            &["Main"],
            &format!(
                "main{}0123abcd{}{}",
                INSTANCIATED_NAME_SEPARATOR, LS_LAM_SUFFIX, index
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

    /// The table the placement rule reads: `lambda` is named by `naming_symbol_count` symbols of the
    /// program.
    fn naming_symbol_counts(lambda: &FullName, naming_symbol_count: usize) -> Map<FullName, usize> {
        [(lambda.clone(), naming_symbol_count)]
            .into_iter()
            .collect()
    }

    /// A caller naming the lambda as the callee of one call is the shape the body moves at, however
    /// many arguments the call supplies: the parameters a short call leaves over stand as a lambda in
    /// the caller, and the arguments past the last parameter are applied to what the body returns.
    #[test]
    fn one_call_moves_the_body() {
        let lambda = lifted(0);
        let counts = naming_symbol_counts(&lambda, 1);
        for arg_count in 1..=3 {
            assert!(
                is_moved_by_placing(&lambda, &call(&lambda, arg_count), &counts),
                "a call supplying {} arguments should move the body",
                arg_count
            );
        }
    }

    /// A lambda a second symbol names is one whose body has somewhere else to be.
    #[test]
    fn a_lambda_two_symbols_name_keeps_its_body() {
        let lambda = lifted(0);
        let counts = naming_symbol_counts(&lambda, 2);
        assert!(!is_moved_by_placing(&lambda, &call(&lambda, 2), &counts));
    }

    /// Two calls name the lambda in two places, so the body stays where it is, since placing it
    /// would write it into both.
    #[test]
    fn two_calls_keep_the_body() {
        let lambda = lifted(0);
        let counts = naming_symbol_counts(&lambda, 1);
        let other_callee = FullName::from_strs(&["Main"], "g#0123abcd");
        let caller_expr = expr_app(
            expr_app(expr_var(other_callee, None), vec![call(&lambda, 1)], None),
            vec![call(&lambda, 1)],
            None,
        );
        assert!(!is_moved_by_placing(&lambda, &caller_expr, &counts));
    }

    /// A lambda handed to a call as an argument is one the body would have to stay behind for.
    #[test]
    fn a_lambda_passed_as_an_argument_keeps_its_body() {
        let lambda = lifted(0);
        let counts = naming_symbol_counts(&lambda, 1);
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
        assert!(!is_moved_by_placing(&lambda, &caller_expr, &counts));
    }

    /// A lambda a `let` also binds is written in two places, so putting the body where the name
    /// stands writes it into both rather than moving it. `find_usage_of_name` records nothing for
    /// the `let`, which is why the rule counts the places the name is written.
    #[test]
    fn a_lambda_a_let_also_binds_keeps_its_body() {
        let lambda = lifted(0);
        let counts = naming_symbol_counts(&lambda, 1);
        let caller_expr = expr_let(
            PatternNode::make_var(var_var(FullName::local("v")), None),
            expr_var(lambda.clone(), None),
            call(&lambda, 2),
            None,
        );
        assert!(!is_moved_by_placing(&lambda, &caller_expr, &counts));
    }
}
