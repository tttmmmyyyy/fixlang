/*
# let-elimination optimization

## Overview

This optimization transforms `let x = {e0} in {e1}` into `{e1}[x:={e0}]` if one of the following conditions hold:
1. `e0` is just a name (variable).
2. `x` is used only once in `e1`, not appear as arguments to LLVM expression, and any of the following sub-conditions hold:
2-a. {e0} is a lambda expression and the occurrence of `x` is in an application
2-b. {e0} is strictly partial application (i.e. # of args < n) of names to a global lambda expression with n-arguments `f = |a1,...,an| ...`,
     and the occurrence of `x` is in an application
2-c. {e1} evaluates `x` "before any other local names", and `x` is not captured by a lambda expression in {e1}
3. `x`  does not appear in {e1}

## Why conditions 2-* are necessary

These conditions are to prevent the lifetime of values referenced by expression {e0} from being extended due to the evaluation of expression {e0} being delayed.

In 2-a, the only variables whose lifetimes can change are those captured by the lambda expression, and these were already alive until the call site of the lambda expression,
so their lifetimes do not extend.

In 2-b, the name expressions partially applied to the global lambda expression were also already alive until the call site of the lambda expression,
so their lifetimes do not extend.

For the definition of "evaluates before any other local names", see the implementation of `FreeOccurrenceProbe`.

## Effects

This transformation in case 1., i.e., transforming `let x = y in {e1}` into `{e1}[x:=y]` even improves the performance of the program.
Consider the following example which contains InlineLLVM nodes:

```
let x = arr; // Retain `arr` here, because it will be used later.
let n = LLVM<x.Array::@(i)>; // Release `x` here, because it will not be used later.
let y = arr;
let m = LLVM<y.Array::@(j)>;
```

After removing renaming, the code will look like this:

```
let n = LLVM<arr.Array::@(i)>; // By the implementation of `LLVM<arr.@(i)>`, the array will not be retained nor released since `arr` will be used later.
let m = LLVM<arr.Array::@(j)>;
```

and the cost for retaining and releasing an array is saved.
*/

use crate::{
    ast::{
        expr::ExprNode,
        name::FullName,
        program::Symbol,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    constants::CAP_NAME,
    misc::Map,
    optimization::rename::{rename_free_name, substitute_free_name},
};
use std::sync::Arc;

/// How many parameters each global lambda takes, keyed by its name. A name is in the map when its
/// symbol is a lambda expression, and the count runs over the whole sequence of lambdas that
/// expression opens with, so `|x| |y| {e0}` counts two.
pub fn create_global_lambda_to_arity_map(symbols: &Map<FullName, Symbol>) -> Map<FullName, usize> {
    let mut global_lambda_to_arity: Map<FullName, usize> = Map::default();
    for (name, sym) in symbols {
        let expr = sym.expr.as_ref().unwrap();
        if expr.is_lam() {
            global_lambda_to_arity.insert(name.clone(), expr.lam_sequence_arity());
        }
    }
    global_lambda_to_arity
}

/// Runs the let-elimination transformation once over `expr`, and answers whether it rewrote
/// anything.
///
/// # Arguments
/// * `global_lambda_to_arity` — how many parameters each global lambda takes. An empty map leaves
///   the transformation to conditions 1, 2-a, 2-c and 3.
pub fn run_on_expr_once(
    expr: &mut Arc<ExprNode>,
    global_lambda_to_arity: &Map<FullName, usize>,
) -> bool {
    let mut eliminator = LetEliminator {
        global_lambda_to_arity,
    };
    let res = eliminator.traverse(expr);
    *expr = res.expr;
    res.changed
}

/// The walk that eliminates the `let`s and the renaming `match`es of an expression, by the
/// conditions the module documentation lists.
struct LetEliminator<'a> {
    /// How many parameters each global lambda takes, which is what decides whether a bound
    /// expression is a strictly partial application of one.
    global_lambda_to_arity: &'a Map<FullName, usize>,
}

impl<'a> ExprVisitor for LetEliminator<'a> {
    // `ExprVisitor` declares every method without a default. The eliminating is done in
    // `end_visit_let` and `end_visit_match`; every other method here is passed through, visiting
    // the children and leaving the expression as it is.

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    /// Eliminates a `let` binding a name, putting the bound expression where the name is read,
    /// where one of the conditions the module documentation lists holds.
    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // Check if the expression is of the form `let x = {e0} in {e1}`.
        let pat = expr.get_let_pat();
        if !pat.is_var() {
            return EndVisitResult::unchanged(expr);
        }
        // The pattern is just a name.

        let e0 = expr.get_let_bound();
        if e0.is_var() {
            // Case 1 of the documentation at the top.

            // Replace all occurrences of `x` in `{e1}` with `{e0}`.
            let x = &pat.get_var().name;
            let e0 = &e0.get_var().name;
            let e1 = expr.get_let_value();
            let expr = rename_free_name(&e1, x, e0);
            return EndVisitResult::changed(expr);
        }
        // Inspect occurrences of `x` in `{e1}`.
        let x = &pat.get_var().name;
        let e1 = expr.get_let_value();
        let mut probe = FreeOccurrenceProbe::new(x.clone());
        probe.traverse(&e1);

        if probe.count == 1 && !probe.is_argument_to_llvm {
            // Case 2 of the documentation at the top.
            let mut any_sub_condition_holds = false;

            if e0.is_lam() && probe.is_applied {
                // Case 2-a of the documentation at the top.
                any_sub_condition_holds = true;
            }

            if !any_sub_condition_holds
                && is_global_lambda_strictly_partially_applied_to_names(
                    &e0,
                    &self.global_lambda_to_arity,
                )
                && probe.is_applied
            {
                // Case 2-b of the documentation at the top.
                any_sub_condition_holds = true;
            }

            if probe.used_before_any_other_local_names && !probe.is_captured_by_lambda {
                // Case 2-c of the documentation at the top.
                any_sub_condition_holds = true;
            }

            if any_sub_condition_holds {
                let expr = substitute_free_name(&e1, x, &e0);
                return EndVisitResult::changed(expr);
            }
        }

        if probe.count == 0 {
            // Case 3 of the documentation at the top.
            let e1 = expr.get_let_value();
            return EndVisitResult::changed(e1);
        }

        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    /// Replaces a `match` of one arm, whose condition is a name and whose pattern binds a name,
    /// with the arm's body, the name the pattern binds renamed to the name the condition reads.
    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // Check if the expression is of the form `match x { y -> {expr} }`.
        let cond = expr.get_match_cond();
        if !cond.is_var() {
            return EndVisitResult::unchanged(expr);
        }
        let pat_vals = expr.get_match_pat_vals();
        if pat_vals.len() != 1 {
            return EndVisitResult::unchanged(expr);
        }
        let (pat, val) = &pat_vals[0];
        if !pat.is_var() {
            return EndVisitResult::unchanged(expr);
        }

        // Replace, in `{val}`, all occurrences of the name `{pat}` binds with the name `{cond}` reads.
        let pat_name = &pat.get_var().name;
        let cond_name = &cond.get_var().name;
        let expr = rename_free_name(&val, pat_name, cond_name);
        EndVisitResult::changed(expr)
    }

    fn start_visit_tyanno(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_eval(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_eval(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}

/// A walk that inspects the free occurrences of a given name in an expression.
///
/// Everything it reports is a property of those occurrences, so a subexpression the name does not
/// occur free in leaves all of them as they stand. The traversal skips such subexpressions, which
/// keeps the cost of a probe proportional to the region where the name is used. That region ends at
/// a binder giving the name to another binding, so no occurrence of another binding is reached.
struct FreeOccurrenceProbe {
    /// The name whose occurrences are counted.
    target_name: FullName,
    /// How many free occurrences the walk has found so far.
    count: usize,
    /// Does the name stand as the function of an application?
    is_applied: bool,
    /// Is every occurrence of `target_name` evaluated "before any other local names"?
    used_before_any_other_local_names: bool,
    /// Is any occurrence of `target_name` captured by a lambda expression?
    is_captured_by_lambda: bool,
    /// Does any occurrence of `target_name` stand as an argument of an LLVM expression?
    is_argument_to_llvm: bool,
}

impl FreeOccurrenceProbe {
    /// A probe that has found nothing yet, for occurrences of `target_name`.
    fn new(target_name: FullName) -> Self {
        // The traversal locates occurrences by the free variables of each subexpression, so it can
        // only probe a name that free variables account for. `CAP_NAME` is the one name they do
        // not: a lambda expression binds it implicitly, and it is absent from the free variables of
        // every lambda expression whose body reads it.
        assert!(
            !(target_name.is_local() && target_name.name == CAP_NAME),
            "occurrences of `{}` cannot be probed",
            CAP_NAME
        );
        Self {
            target_name,
            count: 0,
            is_applied: false,
            used_before_any_other_local_names: true,
            is_captured_by_lambda: false,
            is_argument_to_llvm: false,
        }
    }

    /// Does the target name occur free in `expr`? The traversal visits `expr` only if it does.
    fn target_occurs_in(&self, expr: &Arc<ExprNode>) -> bool {
        expr.has_free_var(&self.target_name)
    }

    /// Does one of `exprs` the target name is absent from read a local name? Asked of expressions
    /// evaluated as a group, such as the fields of a struct expression: any of them may be
    /// evaluated before the one holding the target name.
    fn another_local_name_is_read_in<'a>(
        &self,
        exprs: impl IntoIterator<Item = &'a Arc<ExprNode>>,
    ) -> bool {
        exprs
            .into_iter()
            .any(|expr| !self.target_occurs_in(expr) && expr.has_free_local_var())
    }

    /// Is the target name read in one of `later` while `earlier`, evaluated ahead of them, reads a
    /// local name? That is the shape in which the target name stops being the first local name the
    /// expression evaluates.
    fn target_is_read_after_a_local_name<'a>(
        &self,
        earlier: &Arc<ExprNode>,
        later: impl IntoIterator<Item = &'a Arc<ExprNode>>,
    ) -> bool {
        later.into_iter().any(|expr| self.target_occurs_in(expr)) && earlier.has_free_local_var()
    }
}

impl ExprVisitor for FreeOccurrenceProbe {
    // `ExprVisitor` declares every method without a default. Every method not documented below is
    // passed through: the children are visited, and the expression itself is left as it is.

    /// Whether the traversal enters `expr`: only where the target name occurs free in it, since a
    /// subexpression leaving every occurrence as it stands has nothing to report.
    fn should_visit(&self, expr: &Arc<ExprNode>) -> bool {
        self.target_occurs_in(expr)
    }

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    /// Counts one occurrence: a variable expression is reached only where it is an occurrence of
    /// the target name.
    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let var = expr.get_var();
        assert!(
            var.name == self.target_name,
            "visited a variable expression of `{}` while probing `{}`",
            var.name.to_string(),
            self.target_name.to_string()
        );
        self.count += 1;
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    /// Counts each argument of the LLVM expression that is the target name, and records that the
    /// target name stands as an argument of an LLVM expression. Such an expression is reached only
    /// where it takes the target name, which it may do more than once.
    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let occurrence_count = expr
            .get_llvm()
            .generator
            .free_vars()
            .iter()
            .filter(|fv| **fv == self.target_name)
            .count();
        assert!(
            occurrence_count > 0,
            "visited an LLVM expression taking no `{}` as an argument",
            self.target_name.to_string()
        );
        self.is_argument_to_llvm = true;
        self.count += occurrence_count;

        EndVisitResult::unchanged(expr)
    }

    /// A target name read in the argument of an application, while the function position reads a
    /// local name, leaves `used_before_any_other_local_names` false.
    fn start_visit_app(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if self.target_is_read_after_a_local_name(&expr.get_app_func(), expr.get_app_args().iter())
        {
            self.used_before_any_other_local_names = false;
        }

        StartVisitResult::VisitChildren
    }

    /// Records that the target name stands as the function of an application.
    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let func = expr.get_app_func();
        if func.is_var() && func.get_var().name == self.target_name {
            self.is_applied = true;
        }
        EndVisitResult::unchanged(expr)
    }

    /// Records that a lambda expression captures the target name. Such an expression is reached
    /// only where the target name is free in it, so its parameters are names other than the target
    /// name, and its body is visited with the target name still standing for the same binding.
    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        self.is_captured_by_lambda = true;

        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    /// Visits the bound expression, and the value expression unless the pattern gives the target
    /// name to another binding. The bound expression is evaluated ahead of the value, so a target
    /// name read in the value while the bound expression reads a local name leaves
    /// `used_before_any_other_local_names` false.
    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        let target_rebound = expr
            .get_let_pat()
            .pattern
            .vars()
            .contains(&self.target_name);

        if !target_rebound
            && self
                .target_is_read_after_a_local_name(&expr.get_let_bound(), [&expr.get_let_value()])
        {
            self.used_before_any_other_local_names = false;
        }

        // Visit the bound expression, where the target name is still free.
        self.traverse(&expr.get_let_bound());

        // Visit the value expression, unless {pat} gives the target name to another binding.
        if !target_rebound {
            self.traverse(&expr.get_let_value());
        }

        StartVisitResult::Return
    }

    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    /// The condition of an `if` is evaluated ahead of its branches, so a target name read in a
    /// branch while the condition reads a local name leaves `used_before_any_other_local_names`
    /// false.
    fn start_visit_if(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if self.target_is_read_after_a_local_name(
            &expr.get_if_cond(),
            [&expr.get_if_then(), &expr.get_if_else()],
        ) {
            self.used_before_any_other_local_names = false;
        }

        StartVisitResult::VisitChildren
    }

    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    /// Visits the condition and the arms whose pattern leaves the target name standing for the
    /// binding under inspection. The condition is evaluated ahead of the arms, so a target name
    /// read in one of them while the condition reads a local name leaves
    /// `used_before_any_other_local_names` false.
    fn start_visit_match(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        let vals_not_rebinding_target = expr
            .get_match_pat_vals()
            .into_iter()
            .filter(|(pat, _val)| !pat.pattern.vars().contains(&self.target_name))
            .map(|(_pat, val)| val)
            .collect::<Vec<_>>();

        if self.target_is_read_after_a_local_name(
            &expr.get_match_cond(),
            vals_not_rebinding_target.iter(),
        ) {
            self.used_before_any_other_local_names = false;
        }

        // Visit the condition expression first
        self.traverse(&expr.get_match_cond());

        // Visit the value expression of each such arm
        for val in vals_not_rebinding_target {
            self.traverse(&val);
        }

        StartVisitResult::Return
    }

    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_tyanno(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    /// A struct expression is reached only where the target name appears in some field, and the
    /// fields are evaluated as a group, so another field reading a local name leaves
    /// `used_before_any_other_local_names` false.
    fn start_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if self.another_local_name_is_read_in(
            expr.get_make_struct_fields()
                .iter()
                .map(|(_name, _src, field)| field),
        ) {
            self.used_before_any_other_local_names = false;
        }

        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    /// An array literal is reached only where the target name appears in some element, and the
    /// elements are evaluated as a group, so another element reading a local name leaves
    /// `used_before_any_other_local_names` false.
    fn start_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if self.another_local_name_is_read_in(expr.get_array_lit_elements().iter()) {
            self.used_before_any_other_local_names = false;
        }
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    /// An FFI call is reached only where the target name appears in some argument, and the
    /// arguments are evaluated as a group, so another argument reading a local name leaves
    /// `used_before_any_other_local_names` false.
    fn start_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if self.another_local_name_is_read_in(expr.get_ffi_call_args().iter()) {
            self.used_before_any_other_local_names = false;
        }
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    /// The side expression of an `eval` is evaluated ahead of its main expression, so a target name
    /// read in the main expression while the side expression reads a local name leaves
    /// `used_before_any_other_local_names` false.
    fn start_visit_eval(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if self.target_is_read_after_a_local_name(&expr.get_eval_side(), [&expr.get_eval_main()]) {
            self.used_before_any_other_local_names = false;
        }
        StartVisitResult::VisitChildren
    }

    fn end_visit_eval(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}

/// Is `expr` a global lambda, or an application of names to one that leaves it short of the
/// parameters it takes?
fn is_global_lambda_strictly_partially_applied_to_names(
    expr: &Arc<ExprNode>,
    global_lambda_to_arity: &Map<FullName, usize>,
) -> bool {
    if expr.is_var() {
        let name = &expr.get_var().name;
        if let Some(_arity) = global_lambda_to_arity.get(name) {
            return true;
        }
    } else if expr.is_app() {
        let (func, args) = expr.destructure_app();
        if func.is_var() {
            let name = &func.get_var().name;
            if let Some(arity) = global_lambda_to_arity.get(name) {
                // Check if the number of arguments is less than the arity (strictly partial application).
                if *arity <= args.len() {
                    return false;
                }
                // Check if all arguments are name expressions.
                return args.iter().all(|arg| arg.is_var());
            }
        }
    }
    false
}
