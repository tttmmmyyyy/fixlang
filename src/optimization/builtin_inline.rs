use std::sync::Arc;

use crate::{
    ast::{
        name::FullName,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult},
    },
    make_with_retained_name, ExprNode, Program, STD_NAME,
};

/*
This optimization replaces call to some builtin functions, especially builtin functions that take a lambda as an argument, with a LLVM node.

(1) Std::with_retained : (a -> b) -> a -> b

Replace the expression `x.with_retained(|x| {expr})` with an LLVM node represented by the following pseudocode.

```
retain(x)
{expr}
release(x)
```

Since this optimization is not applied to `let f = {expr}; x.with_retained(f)`,
it is desirable that it is converted to `x.with_retained(|x| {expr})` in advance.

*/

pub fn run(prg: &mut Program) {}

struct Visitor {}

impl ExprVisitor for Visitor {
    fn end_visit_app(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        // Check if the expression is of the form `with_retained(|x| {expr}, x)`.
        let fun = expr.get_app_func();
        if !fun.is_app() {
            // `fun` cannot be `with_retained(|x| {expr})`.
            return EndVisitResult::noreplace(expr);
        }
        let (fun, args) = (fun.get_app_func(), fun.get_app_args());
        // Check the function name.
        let fun_name = &fun.get_var().name;
        if *fun_name != make_with_retained_name() {
            return EndVisitResult::noreplace(expr);
        }

        // Check that the argument of `fun` is of the form `|x| {expr}`.
        if args.len() != 1 {
            // If `fun` is `with_retained`, it should have exactly one argument.
            return EndVisitResult::noreplace(expr);
        }
        let f = &args[0];
        if !f.is_lam() {
            return EndVisitResult::noreplace(expr);
        }
        let f_params = f.get_lam_params();
        assert_eq!(f_params.len(), 1);
        let f_param = &f_params[0];
        let f_body = f.get_lam_body();

        let args = expr.get_app_args();
        assert_eq!(args.len(), 1);
        let x = &args[0];

        // Now, the expression is of the form `x.with_retained(|{f_param}| {f_body})`.
        // Replace it with `let {f_param} = x in InlineLLVMWithRetainedInlined { f_param, f_body }`.

        unimplemented!("")
    }
}

fn run_on_expr(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    unimplemented!("")
}
