/*
let-elimination optimization

This optimization transforms `let x = {e0} in {e1}` into `{e1}[x:={e0}]` if one of the following conditions hold:
1. `e0` is just a name (variable).
2. `x` is used only once in `e1`, AND
2-a. {e0} is a lambda expression and the occurrence of `x` is in an application, OR
2-b. {e0} is a global lambda expression with n-arguments `f = |a1,...,an| ...` or strictly partial application of name expressions to it, and the occurrence of `x` is in an application.

Why conditions 2-a and 2-b are necessary:
These conditions are to prevent the lifetime of values referenced by expression {e0} from being extended due to the evaluation of expression {e0} being delayed.
In 2-a, the only variables whose lifetimes can change are those captured by the lambda expression, and these were already alive until the call site of the lambda expression,
so their lifetimes do not extend.
In 2-b, the name expressions partially applied to the global lambda expression were also already alive until the call site of the lambda expression,
so their lifetimes do not extend.

(1) itself improves the performance of the program. Consider the following example which contains InlineLLVM nodes:

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

use std::sync::Arc;

use crate::{
    ast::{
        expr::ExprNode,
        name::FullName,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult},
    },
    lsp::language_server::write_log,
    misc::{Map, Set},
    optimization::utils::{rename_free_name, substitute_free_name},
    Program,
};

// pub fn run(prg: &mut Program) {
//     let global_lambda_to_arity = create_global_lambda_to_arity_map(prg);
//     for (_name, sym) in &mut prg.symbols {
//         run_on_symbol(sym, &global_lambda_to_arity);
//     }
// }

// fn run_on_symbol(sym: &mut Symbol, global_lambda_to_arity: &Map<FullName, usize>) {
//     let mut remover = LetEliminator {
//         global_lambda_to_arity: global_lambda_to_arity,
//     };
//     let res = remover.traverse(&sym.expr.as_ref().unwrap());
//     if res.changed {
//         sym.expr = Some(res.expr);
//     }
// }

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
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_let(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        // Check if the expression is of the form `let x = {e0} in {e1}`.
        let x = expr.get_let_pat();
        if !x.is_var() {
            return EndVisitResult::unchanged(expr);
        }
        let e0 = expr.get_let_bound();
        if e0.is_var() {
            // Replace all occurrences of `x` in `{e1}` with `{e0}`.
            let x = &x.get_var().name;
            let e0 = &e0.get_var().name;
            let e1 = expr.get_let_value();
            let expr = rename_free_name(&e1, x, e0);
            return EndVisitResult::changed(expr);
        } else {
            // Count the number of occurrences of `x` in `{e1}`.
            let x = &x.get_var().name;
            let e1 = expr.get_let_value();
            let mut probe = FreeOccurrenceProbe::new(x.clone());
            probe.traverse(&e1);
            write_log(&format!(
                "Free occurrences of {} in {}: count = {}, is_applied = {}",
                x.to_string(),
                e1.expr.stringify().to_string(),
                probe.count,
                probe.is_applied
            ));
            if probe.count == 1 && probe.is_applied {
                // TODO: in a future, we remove let bindings if count == 0.
                // Case (2) of the documentation at the top.
                // Check if conditions 2-a or 2-b hold.
                let cond = e0.is_lam(); // 2-a
                let cond = cond
                    || is_global_lambda_strictly_partially_applied_to_names(
                        &e0,
                        &self.global_lambda_to_arity,
                    ); // 2-b
                if cond {
                    let expr = substitute_free_name(&e1, x, &e0);
                    return EndVisitResult::changed(expr);
                }
            }
        }
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_match(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
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
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}

// An ExprVisitor that counts the number of free occurrences of a given name in an expression.
pub struct FreeOccurrenceProbe {
    // The name to count occurrences of.
    target_name: FullName,
    // Local names that are currently shadowed (i.e., not free).
    shadowed: Set<FullName>,
    // Count of free occurrences found so far.
    count: usize,
    // Is the name occurrs as an application function?
    is_applied: bool,
}

impl FreeOccurrenceProbe {
    fn new(target_name: FullName) -> Self {
        Self {
            target_name,
            shadowed: Set::default(),
            count: 0,
            is_applied: false,
        }
    }
}

impl ExprVisitor for FreeOccurrenceProbe {
    fn start_visit_var(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        let var = expr.get_var();

        // If the target name is shadowed, do nothing
        if self.shadowed.contains(&self.target_name) {
            return EndVisitResult::unchanged(expr);
        }

        // If the variable name matches the target name, increment count
        if var.name == self.target_name {
            self.count += 1;
        }

        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        let llvm = expr.get_llvm();

        // If the target name is shadowed, do nothing
        if self.shadowed.contains(&self.target_name) {
            return EndVisitResult::unchanged(expr);
        }

        // Count occurrences in free_vars
        for fv in llvm.generator.free_vars() {
            if fv == self.target_name {
                self.count += 1;
            }
        }

        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        // Check if the applied function is the target name
        if !self.shadowed.contains(&self.target_name) {
            let func = expr.get_app_func();
            if func.is_var() {
                let var = func.get_var();
                if var.name == self.target_name {
                    self.is_applied = true;
                }
            }
        }
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        let params = expr.get_lam_params();

        // Save the current shadowed state
        let bak_shadowed = self.shadowed.clone();

        // Add parameters to shadowed set
        for param in &params {
            self.shadowed.insert(param.name.clone());
        }

        // Visit the body
        let body = expr.get_lam_body();
        self.traverse(&body);

        // Restore the shadowed state
        self.shadowed = bak_shadowed;

        StartVisitResult::Return
    }

    fn end_visit_lam(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // Visit the bound expression first (where the target name is still free)
        let bound = expr.get_let_bound();
        self.traverse(&bound);

        // Save the current shadowed state
        let bak_shadowed = self.shadowed.clone();

        // Add pattern variables to shadowed set
        let pattern = expr.get_let_pat();
        let introduced_names = pattern.pattern.vars();
        for name in introduced_names {
            self.shadowed.insert(name);
        }

        // Visit the value expression
        let value = expr.get_let_value();
        self.traverse(&value);

        // Restore the shadowed state
        self.shadowed = bak_shadowed;

        StartVisitResult::Return
    }

    fn end_visit_let(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // Visit the condition expression first
        let cond = expr.get_match_cond();
        self.traverse(&cond);

        // Visit each match case
        let pat_vals = expr.get_match_pat_vals();
        for (pat, val) in &pat_vals {
            // Save the current shadowed state
            let bak_shadowed = self.shadowed.clone();

            // Add pattern variables to shadowed set
            let introduced_names = pat.pattern.vars();
            for name in introduced_names {
                self.shadowed.insert(name);
            }

            // Visit the value expression
            self.traverse(&val);

            // Restore the shadowed state
            self.shadowed = bak_shadowed;
        }

        StartVisitResult::Return
    }

    fn end_visit_match(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_tyanno(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
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
