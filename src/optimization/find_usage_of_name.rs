// This module provides functionality to find how a name is used in an expression.

use crate::ast::{
    expr::ExprNode,
    name::FullName,
    traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
};
use std::sync::Arc;

// One way a name is used, at one occurrence of it.
pub enum UsageType {
    // The name is passed as an argument to a function.
    // The first component is the name of the function called, and the second component is the index
    // of the argument the name was passed as.
    FunctionArgument(FullName, usize),
    // The name is used as a function and is called.
    CalledAsFunction,
    // The name is stored into a field of a struct being built. The first component names the type
    // constructor of that struct, and the second the position of the field among the fields the
    // type constructor declares.
    CapturedInto(FullName, usize),
}

// The uses of `name` within `expr`, one entry per occurrence that `UsageType` has a shape for, in
// the order the walk meets them. An occurrence standing under an inner binding of the same name
// belongs to that binding and is passed over.
pub fn run(expr: &Arc<ExprNode>, name: &FullName) -> Vec<UsageType> {
    let mut usages = Vec::new();
    let mut finder = UsageFinder {
        name,
        usages: &mut usages,
    };
    finder.traverse(expr);
    usages
}

// The walk collecting the uses of one name.
struct UsageFinder<'a> {
    // The name whose uses are being collected.
    name: &'a FullName,
    // The uses met so far, which the walk appends to.
    usages: &'a mut Vec<UsageType>,
}

impl UsageFinder<'_> {
    fn add_usage(&mut self, usage: UsageType) {
        self.usages.push(usage);
    }

    // Whether an inner binding stands between here and the name being searched for, which makes an
    // occurrence here one about something else.
    fn shadowed(&self, state: &VisitState) -> bool {
        self.name.is_local() && state.scope.has_value(&self.name.name)
    }

    // Whether `expr` is an occurrence of the name being searched for.
    fn is_the_name(&self, expr: &Arc<ExprNode>) -> bool {
        expr.is_var() && &expr.get_var().name == self.name
    }
}

impl ExprVisitor for UsageFinder<'_> {
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
        expr: &Arc<ExprNode>,
        state: &mut VisitState,
    ) -> StartVisitResult {
        if self.shadowed(state) {
            return StartVisitResult::VisitChildren;
        }
        let (fun, args) = expr.destructure_app();
        if self.is_the_name(&fun) {
            self.add_usage(UsageType::CalledAsFunction);
        }
        // A call whose callee is not a variable has no name to record the argument against, and the
        // consumer looks the name up among the program's globals, so such a call is passed over.
        if fun.is_var() {
            let fun_name = fun.get_var().name.clone();
            for (i, arg) in args.iter().enumerate() {
                if self.is_the_name(arg) {
                    self.add_usage(UsageType::FunctionArgument(fun_name.clone(), i));
                }
            }
        }
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
        state: &mut VisitState,
    ) -> StartVisitResult {
        if self.shadowed(state) {
            return StartVisitResult::VisitChildren;
        }
        let (tycon, fields) = expr.destructure_make_struct().unwrap();
        for (position, (_, _, value)) in fields.iter().enumerate() {
            if self.is_the_name(value) {
                self.add_usage(UsageType::CapturedInto(tycon.name.clone(), position));
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
