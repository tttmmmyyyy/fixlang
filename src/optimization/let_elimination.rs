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
        program::Program,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    misc::Map,
    optimization::rename::{rename_free_name, substitute_free_name},
};
use std::sync::Arc;

pub fn create_global_lambda_to_arity_map(prg: &Program) -> Map<FullName, usize> {
    let mut global_lambda_to_arity: Map<FullName, usize> = Map::default();
    for (name, sym) in &prg.symbols {
        let expr = sym.expr.as_ref().unwrap();
        if expr.is_lam() {
            let args = expr.destructure_lam_sequence().0;
            let arity = args.iter().map(|args| args.len()).sum();
            global_lambda_to_arity.insert(name.clone(), arity);
        }
    }
    global_lambda_to_arity
}

// Run let-elimination transformation once on the given expression.
//
// If any transformation is applied, returns true.
//
// - `global_lambda_to_arity`: a map from global lambda names to their arities. If given empty map, conditions 2-b will not be applied.
pub fn run_on_expr_once(
    expr: &mut Arc<ExprNode>,
    global_lambda_to_arity: &Map<FullName, usize>,
) -> bool {
    let mut remover = LetEliminator {
        global_lambda_to_arity,
    };
    let res = remover.traverse(expr);
    *expr = res.expr;
    res.changed
}

struct LetEliminator<'a> {
    global_lambda_to_arity: &'a Map<FullName, usize>,
}

impl<'a> ExprVisitor for LetEliminator<'a> {
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

    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // Check if the expression is of the form `let x = {e0} in {e1}`.
        let x = expr.get_let_pat();
        if !x.is_var() {
            return EndVisitResult::unchanged(expr);
        }
        // The pattern is just a name.

        let e0 = expr.get_let_bound();
        if e0.is_var() {
            // Case 1 of the documentation at the top.

            // Replace all occurrences of `x` in `{e1}` with `{e0}`.
            let x = &x.get_var().name;
            let e0 = &e0.get_var().name;
            let e1 = expr.get_let_value();
            let expr = rename_free_name(&e1, x, e0);
            return EndVisitResult::changed(expr);
        }
        // Inspect occurrences of `x` in `{e1}`.
        let x = &x.get_var().name;
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

        // Replace all occurrences of `pat` in `expr` with `cond`.
        let pat = &pat.get_var().name;
        let cond = &cond.get_var().name;
        let expr = rename_free_name(&val, pat, cond);
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

// An ExprVisitor that inspects the free occurrences of a given name in an expression.
//
// Everything it reports is a property of those occurrences, so a subexpression the name does not
// occur free in leaves all of them as they stand. The traversal skips such subexpressions, which
// keeps the cost of a probe proportional to the region where the name is used rather than to the
// size of the expression it is run on. That region ends at a binder giving the name to another
// binding, so no occurrence of another binding is ever reached.
struct FreeOccurrenceProbe {
    // The name to count occurrences of.
    target_name: FullName,
    // Count of free occurrences found so far.
    count: usize,
    // Is the name occurrs as an application function?
    is_applied: bool,
    // Is all occurrences of `target_name` evaluated "before any other local names"?
    used_before_any_other_local_names: bool,
    // Is any occurrence of `target_name` captured by a lambda expression?
    is_captured_by_lambda: bool,
    // Is any occurrence of `target_name` appear as arguments to LLVM expression?
    is_argument_to_llvm: bool,
}

impl FreeOccurrenceProbe {
    fn new(target_name: FullName) -> Self {
        Self {
            target_name,
            count: 0,
            is_applied: false,
            used_before_any_other_local_names: true,
            is_captured_by_lambda: false,
            is_argument_to_llvm: false,
        }
    }

    // Does the target name occur free in `expr`? The traversal enters `expr` only if it does.
    fn target_occurs_in(&self, expr: &Arc<ExprNode>) -> bool {
        expr.has_free_var(&self.target_name)
    }
}

impl ExprVisitor for FreeOccurrenceProbe {
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

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // A variable expression is entered only when it is an occurrence of the target name.
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

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // An LLVM expression is entered only when it takes the target name as an argument, which it
        // may do more than once.
        self.is_argument_to_llvm = true;
        for fv in expr.get_llvm().generator.free_vars() {
            if fv == self.target_name {
                self.count += 1;
            }
        }

        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // Function application expression {f}({x}).

        // If {x} contains the target name, and {f} contains local name, then set `used_before_any_other_local_names` to false.
        if expr
            .get_app_args()
            .iter()
            .any(|arg| self.target_occurs_in(arg))
            && expr.get_app_func().has_free_local_var()
        {
            self.used_before_any_other_local_names = false;
        }

        StartVisitResult::VisitChildren
    }

    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // Check if the applied function is the target name
        let func = expr.get_app_func();
        if func.is_var() && func.get_var().name == self.target_name {
            self.is_applied = true;
        }
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // A lambda expression is entered only when the target name is free in it, i.e., captured by
        // it. Its parameters are therefore names other than the target name, and the body is
        // visited with the target name still standing for the same binding.
        self.is_captured_by_lambda = true;

        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // Let expression `let {pat} = {bound} in {value}`.
        let target_rebound = expr
            .get_let_pat()
            .pattern
            .vars()
            .contains(&self.target_name);

        // If {value} contains the target name, and {bound} contains local name, then set `used_before_any_other_local_names` to false.
        if !target_rebound
            && self.target_occurs_in(&expr.get_let_value())
            && expr.get_let_bound().has_free_local_var()
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

    fn start_visit_if(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // If expression `if {cond} { {then} } else { {else} }`.

        // if the target name appears in {then} or {else}, and {cond} contains local name, then set `used_before_any_other_local_names` to false.
        if (self.target_occurs_in(&expr.get_if_then())
            || self.target_occurs_in(&expr.get_if_else()))
            && expr.get_if_cond().has_free_local_var()
        {
            self.used_before_any_other_local_names = false;
        }

        StartVisitResult::VisitChildren
    }

    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // Match expression `match {cond} { pat1 => {val1}; pat2 => {val2}; ... }`.
        // The value expressions of the arms whose {pat} leaves the target name standing for the
        // binding under inspection.
        let vals = expr
            .get_match_pat_vals()
            .into_iter()
            .filter(|(pat, _val)| !pat.pattern.vars().contains(&self.target_name))
            .map(|(_pat, val)| val)
            .collect::<Vec<_>>();

        // If the target name appears in any such {val}, and {cond} contains local name, then set `used_before_any_other_local_names` to false.
        if vals.iter().any(|val| self.target_occurs_in(val))
            && expr.get_match_cond().has_free_local_var()
        {
            self.used_before_any_other_local_names = false;
        }

        // Visit the condition expression first
        self.traverse(&expr.get_match_cond());

        // Visit the value expression of each arm
        for val in vals {
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

    fn start_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // A struct expression is entered only when the target name appears in some field.

        // If any other field contains local name, then set `used_before_any_other_local_names` to false.
        for (_, _, field) in &expr.get_make_struct_fields() {
            if !self.target_occurs_in(field) && field.has_free_local_var() {
                // A field contains local name other than the target name.
                self.used_before_any_other_local_names = false;
                break;
            }
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

    fn start_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // An array literal is entered only when the target name appears in some element.

        // If any other element contains local name, then set `used_before_any_other_local_names` to false.
        for element in expr.get_array_lit_elements() {
            if !self.target_occurs_in(&element) && element.has_free_local_var() {
                // An element contains local name other than the target name.
                self.used_before_any_other_local_names = false;
                break;
            }
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

    fn start_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // An FFI call is entered only when the target name appears in some argument.

        // If any other argument contains local name, then set `used_before_any_other_local_names` to false.
        for arg in expr.get_ffi_call_args() {
            if !self.target_occurs_in(&arg) && arg.has_free_local_var() {
                // An argument contains local name other than the target name.
                self.used_before_any_other_local_names = false;
                break;
            }
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

    fn start_visit_eval(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // If the main expression contains the target name, and the sub-expression contains local name, then set `used_before_any_other_local_names` to false.
        if self.target_occurs_in(&expr.get_eval_main()) && expr.get_eval_side().has_free_local_var()
        {
            self.used_before_any_other_local_names = false;
        }
        StartVisitResult::VisitChildren
    }

    fn end_visit_eval(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}

// Check if the expression is a global lambda expression or strictly partial application of name expressions to it.
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
