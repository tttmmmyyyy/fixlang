/*
let-elimination optimization

This optimization transforms `let x = {e0} in {e1}` into `{e1}[x:={e0}]` if one of the following conditions hold:
1. `e0` is just a name (variable).
2. `x` is used only once in `e1`.

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

use crate::{
    ast::{
        name::FullName,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult},
    },
    misc::Set,
    optimization::utils::{rename_free_name, substitute_free_name},
    Program, Symbol,
};

pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym);
    }
}

fn run_on_symbol(sym: &mut Symbol) {
    let mut remover = LetEliminator {};
    let res = remover.traverse(&sym.expr.as_ref().unwrap());
    if res.changed {
        sym.expr = Some(res.expr);
    }
}

struct LetEliminator {}

impl ExprVisitor for LetEliminator {
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
            let mut counter = FreeOccurrenceCounter::new(x.clone());
            let count = counter.count_occurrences(&e1);
            if count == 1 {
                // If `x` is used exactly once in `{e1}`, substitute `x` with `{e0}` in `{e1}`.
                // TODO: in a future, we remove let bindings if count == 0.
                let expr = substitute_free_name(&e1, x, &e0);
                return EndVisitResult::changed(expr);
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
pub struct FreeOccurrenceCounter {
    // The name to count occurrences of.
    target_name: FullName,
    // Local names that are currently shadowed (i.e., not free).
    shadowed: Set<FullName>,
    // Count of free occurrences found so far.
    count: usize,
}

impl FreeOccurrenceCounter {
    fn new(target_name: FullName) -> Self {
        Self {
            target_name,
            shadowed: Set::default(),
            count: 0,
        }
    }

    fn count_occurrences(&mut self, expr: &std::sync::Arc<crate::ExprNode>) -> usize {
        self.traverse(expr);
        self.count
    }
}

impl ExprVisitor for FreeOccurrenceCounter {
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
