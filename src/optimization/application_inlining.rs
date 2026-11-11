/*
Application inlining.

This optimization tries to reduce cost of "create lambda and apply" expressions.

1. Moves application into inner.

For example, in case of `if` expression,

```
(if c {{expr0}} else {{expr1}})({expr2})
```

is transformed into

```
let v = {expr2} in if c {{expr0}(v)} else {{expr1}(v)}
```

An argument that is already a variable is pushed in as it is, so the `let` above appears only for an
argument that has something to evaluate. See `PushedArg`. An application of an uncurried function
pushes all of its arguments in at once, binding them in the order they are listed.

Where the function is itself a `let`, the arguments' `let`s go on the side that leaves the call
evaluating its arguments in the order they are written. `ExprNode::app_order` says which side that
is.

2. Replaces application of lambda expression to an expression with let binding.

The expression

```
(|x| {expr0})({expr1})
```

is transformed into

```
let x = {expr1} in {expr0}
```

The `let` here is the lambda's own binder: `x` keeps its name and its scope, so this rewrite leaves
the number of bindings as it found it. A lambda of an uncurried function, `|{{x, y}}| {expr0}`
applied to `({expr1}, {expr2})`, becomes `let x = {expr1} in let y = {expr2} in {expr0}`.
*/

use super::rename::{
    generate_new_names, rename_lam_param_avoiding, rename_let_pattern_avoiding,
    rename_match_pattern_avoiding,
};
use crate::ast::{
    name::FullName,
    expr::{
        expr_eval_typed, expr_if_typed, expr_let_typed, expr_match_typed, expr_var, var_var,
        AppSourceCodeOrderType, Expr, ExprNode,
    },
    pattern::PatternNode,
    traverse::{EndVisitResult, ExprVisitor, VisitState},
};
use crate::misc::Set;
use std::sync::Arc;

/// Optimizes an expression to a fixpoint, repeating the traversal for as long as it keeps finding
/// applications to rewrite.
pub fn run_on_expr(mut expr: Arc<ExprNode>) -> Arc<ExprNode> {
    while run_on_expr_once(&mut expr) {}
    expr
}

/// Optimizes an expression in place by a single traversal, and reports whether it rewrote anything.
pub fn run_on_expr_once(expr: &mut Arc<ExprNode>) -> bool {
    let mut inliner = AppInliner {};
    let res = inliner.traverse(expr);
    *expr = res.expr;
    res.changed
}

/// The visitor that carries out the two rewrites of this optimization. It acts on an application
/// once its subexpressions have been visited, and leaves every other kind of expression as it is.
struct AppInliner {}

/// The value that the subexpressions of a function receive when an application is pushed into them,
/// together with the binding, if any, that the rewritten expression must be wrapped in.
///
/// A variable argument is handed over as it is: a variable reference has nothing to evaluate, so
/// mentioning it once per branch keeps both the work done and the order it is done in. Any other
/// argument is bound to a fresh name first, so it is evaluated exactly once, before the function's
/// own body.
///
/// Handing a variable over unchanged is what keeps this transformation linear. Binding every
/// argument would add one binding for each `let` an application is pushed through, so pushing `n`
/// arguments into a chain of `let`s — what uncurrying's eta expansion does to a function of `n`
/// parameters — would grow the chain as `2^n`.
struct PushedArg {
    /// The expression that stands for the argument in each subexpression of the function.
    value: Arc<ExprNode>,
    /// The pattern and bound expression of the `let` that evaluates the argument, present for an
    /// argument that was given a fresh name.
    binding: Option<(Arc<PatternNode>, Arc<ExprNode>)>,
}

impl PushedArg {
    /// Prepares the arguments of an application for being pushed into the subexpressions of its
    /// function, one `PushedArg` per argument, in the order they are listed.
    ///
    /// The binder of a fresh name wraps the whole rewritten expression, so its scope covers `func`,
    /// what every other argument is pushed in as, and the arguments listed after it. The fresh name
    /// therefore avoids the names free in `func` and in the other arguments, and the other fresh
    /// names.
    ///
    /// # Arguments
    /// * `func` — the function the application is pushed into.
    fn new_all(args: &[Arc<ExprNode>], func: &Arc<ExprNode>) -> Vec<Self> {
        let mut fresh_names = Set::default();
        let mut pushed = vec![];
        for (i, arg) in args.iter().enumerate() {
            if arg.is_var() {
                pushed.push(PushedArg {
                    value: arg.clone(),
                    binding: None,
                });
                continue;
            }
            let mut black_list = func.free_vars();
            for (j, other) in args.iter().enumerate() {
                if j != i {
                    black_list.extend(other.free_vars());
                }
            }
            black_list.extend(fresh_names.iter().cloned());
            let ty = arg.type_.as_ref().unwrap().clone();
            let name = generate_new_names(&black_list, 1)[0].clone();
            fresh_names.insert(name.clone());
            let pat = PatternNode::make_var(var_var(name.clone()), None).set_type(ty.clone());
            pushed.push(PushedArg {
                value: expr_var(name, None).set_type(ty),
                binding: Some((pat, arg.clone())),
            });
        }
        pushed
    }

    /// What each of `pushed` stands for in the subexpressions of the function.
    fn values(pushed: &[Self]) -> Vec<Arc<ExprNode>> {
        pushed.iter().map(|arg| arg.value.clone()).collect()
    }

    /// The names free in what any of `pushed` stands for.
    fn free_vars_of_values(pushed: &[Self]) -> Set<FullName> {
        let mut names = Set::default();
        for arg in pushed {
            names.extend(arg.value.free_vars());
        }
        names
    }

    /// Wraps the rewritten expression in the bindings of `pushed`, the argument listed first
    /// outermost, so that the arguments are evaluated in the order they are listed.
    fn wrap_all(pushed: Vec<Self>, expr: Arc<ExprNode>) -> Arc<ExprNode> {
        pushed
            .into_iter()
            .rev()
            .fold(expr, |expr, arg| match arg.binding {
                Some((pat, bound)) => expr_let_typed(pat, bound, expr),
                None => expr,
            })
    }
}

impl ExprVisitor for AppInliner {
    /// Rewrites an application whose function is a lambda, a `let`, an `if`, a `match` or an
    /// `eval`, and asks for the result to be visited again, so that the application the rewrite
    /// moves inward is rewritten in turn.
    ///
    /// Where each arm puts the argument's `let` decides which parts of the function are evaluated
    /// before the argument. `FreeOccurrenceProbe::target_is_evaluated_ahead_of_an_argument` in
    /// `let_elimination` relies on these placements, so a change to one of them goes there too.
    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // Get the arguments of the application and the function applied to them. An application
        // carries one argument, or every argument of an uncurried function.
        let args = expr.get_app_args();
        let func = expr.get_app_func();
        match &*func.expr {
            Expr::Lam(params, _body) => {
                // The expression is of the form `(|x| {expr})({a})`, or `(|{{x, y}}| {expr})({a},
                // {b})` for an uncurried function. Replace it with `let x = {a} in {expr}`, or
                // `let x = {a} in let y = {b} in {expr}`.
                assert_eq!(
                    params.len(),
                    args.len(),
                    "a lambda of {} parameters is applied to {} arguments",
                    params.len(),
                    args.len()
                );
                // The `let` of a parameter has the arguments listed after it in its scope, so the
                // parameters are renamed away from the names free in those.
                let mut black_list = Set::default();
                for arg in &args[1..] {
                    black_list.extend(arg.free_vars());
                }
                let func = rename_lam_param_avoiding(&black_list, func.clone());
                let (params, body) = func.destructure_lam();
                let expr = params
                    .iter()
                    .zip(args.iter())
                    .rev()
                    .fold(body, |body, (param, arg)| {
                        let pat = PatternNode::make_var(param.clone(), None)
                            .set_type(arg.type_.as_ref().unwrap().clone());
                        expr_let_typed(pat, arg.clone(), body)
                    });
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::Let(_pattern, _bound, _value) => {
                // The expression is of the form `(let {pat} = {bound} in {value})({a})`.
                // Replace it with `let {pat} = {bound} in let x = {a} in {value}(x)`, or with
                // `let x = {a} in let {pat} = {bound} in {value}(x)`.
                let pushed = PushedArg::new_all(&args, &func);

                // Which of the two comes first is the order the call evaluates its arguments in.
                // `f(x, y)` nests as `f(x)(y)`, which leaves the argument written first inside
                // `{bound}`; `x.f(y)` nests as `f(y)(x)`, where the argument written first is
                // `{a}`. Either way the pair of `let`s holds them in the order they are written.
                let bound_first = expr.app_order == AppSourceCodeOrderType::FX;

                // The expressions standing for the arguments are mentioned under `{pat}`, so a
                // name `{pat}` binds and those expressions mention is renamed away first, leaving
                // the name denoting there what it denoted outside. Where `{bound}` comes first,
                // the arguments themselves are evaluated under `{pat}` as well, so the names free
                // in them join the list.
                let mut black_list = PushedArg::free_vars_of_values(&pushed);
                if bound_first {
                    for arg in &args {
                        black_list.extend(arg.free_vars());
                    }
                }
                let func = rename_let_pattern_avoiding(&black_list, func.clone());

                let value_x = expr
                    .set_app_func(func.get_let_value().clone())
                    .set_app_args(PushedArg::values(&pushed)); // {value}(x)
                let expr = if bound_first {
                    expr_let_typed(
                        func.get_let_pat().clone(),
                        func.get_let_bound().clone(),
                        PushedArg::wrap_all(pushed, value_x),
                    )
                } else {
                    PushedArg::wrap_all(
                        pushed,
                        expr_let_typed(
                            func.get_let_pat().clone(),
                            func.get_let_bound().clone(),
                            value_x,
                        ),
                    )
                };
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::If(cond, then, else_) => {
                // The expression is of the form `(if {cond} then {then} else {else})({a})`.
                // Replace it with `let x = {a} in if {cond} then {then}(x) else {else}(x)`.
                let pushed = PushedArg::new_all(&args, &func);

                let then = expr
                    .set_app_func(then.clone())
                    .set_app_args(PushedArg::values(&pushed)); // {then}(x)
                let else_ = expr
                    .set_app_func(else_.clone())
                    .set_app_args(PushedArg::values(&pushed)); // {else}(x)
                let expr = expr_if_typed(cond.clone(), then, else_); // if {cond} then {then}(x) else {else}(x)
                let expr = PushedArg::wrap_all(pushed, expr); // let x = {a} in if {cond} then {then}(x) else {else}(x)
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::Match(_cond, _pats_vals) => {
                // Similar to `if` and `let` cases. The arguments land under the patterns of the
                // arms, so the names they bind are renamed away from them as in the `let` case.
                let pushed = PushedArg::new_all(&args, &func);
                let func = rename_match_pattern_avoiding(
                    &PushedArg::free_vars_of_values(&pushed),
                    func.clone(),
                );

                let mut pats_vals = func.get_match_pat_vals();
                for (_pat, val) in &mut pats_vals {
                    let new_val = expr
                        .set_app_func(val.clone())
                        .set_app_args(PushedArg::values(&pushed));
                    *val = new_val;
                }
                let expr = expr_match_typed(func.get_match_cond().clone(), pats_vals);
                let expr = PushedArg::wrap_all(pushed, expr);
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::Eval(side, main) => {
                // The expression is of the form `(eval {side} in {main})({a})`.
                // Replace it with `let x = {a} in eval {side} in {main}(x)`.
                let pushed = PushedArg::new_all(&args, &func);

                let main_x = expr
                    .set_app_func(main.clone())
                    .set_app_args(PushedArg::values(&pushed)); // {main}(x)
                let eval_expr = expr_eval_typed(side.clone(), main_x); // eval {side} in {main}(x)
                let expr = PushedArg::wrap_all(pushed, eval_expr); // let x = {a} in eval {side} in {main}(x)
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::App(_, _) | Expr::Var(_) | Expr::Builtin(_) => {
                return EndVisitResult::unchanged(expr);
            }
            Expr::TyAnno(_, _) => {
                unreachable!(
                    "a type annotation stands in the function position of an application, which `remove_tyanno` rules out before this pass runs: {}",
                    func.expr.stringify().to_string()
                );
            }
            Expr::ArrayLit(_) | Expr::MakeStruct(_, _) | Expr::FFICall(_, _, _, _, _, _) => {
                unreachable!(
                    "an expression whose type is never a function stands in the function position of an application: {}",
                    func.expr.stringify().to_string()
                );
            }
        }
    }
}
