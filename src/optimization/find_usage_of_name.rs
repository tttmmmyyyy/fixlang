// This module provides functionality to find how a name is used in an expression.

use std::sync::Arc;

use crate::ast::{
    expr::ExprNode,
    name::FullName,
    traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
};

pub enum UsageType {
    // The name is passed as an argument to a call. The first component names the function called,
    // and is `None` where the callee is an expression rather than a name; the second is the index of
    // the argument the name was passed as.
    FunctionArgument(Option<FullName>, usize),
    // The name is used as a function and is called.
    CalledAsFunction,
    // The name is stored into a field of a struct being built. The first component names the type
    // constructor of that struct, and the second the position of the field among the fields the
    // type constructor declares.
    CapturedInto(FullName, usize),
}

pub fn run(expr: &Arc<ExprNode>, name: &FullName) -> Vec<UsageType> {
    let mut usages = Vec::new();
    let mut finder = UsageFinder {
        name,
        usages: &mut usages,
    };
    finder.traverse(expr);
    usages
}

struct UsageFinder<'a> {
    name: &'a FullName,
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
}

impl ExprVisitor for UsageFinder<'_> {
    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
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
        if fun.is_var() && &fun.get_var().name == self.name {
            self.add_usage(UsageType::CalledAsFunction);
        }
        let fun_name = if fun.is_var() {
            Some(fun.get_var().name.clone())
        } else {
            None
        };
        for (i, arg) in args.iter().enumerate() {
            if arg.is_var() && &arg.get_var().name == self.name {
                self.add_usage(UsageType::FunctionArgument(fun_name.clone(), i));
            }
        }
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_match(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_tyanno(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
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
            if value.is_var() && &value.get_var().name == self.name {
                self.add_usage(UsageType::CapturedInto(tycon.name.clone(), position));
            }
        }
        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_eval(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_eval(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}
