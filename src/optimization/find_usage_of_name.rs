// This module provides functionality to find how a name is used in an expression.

use crate::ast::{
    expr::ExprNode,
    name::FullName,
    traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
};
use std::sync::Arc;

// A use of the name being searched for, told apart by what receives the name there.
pub enum UsageType {
    // The name is passed as an argument to a call. The first component names the function called,
    // and is `None` where the callee is an expression rather than a name; the second is the index of
    // the argument the name was passed as.
    FunctionArgument(Option<FullName>, usize),
    // The name is used as a function and is called, with the number of arguments the call supplies.
    CalledAsFunction { arg_count: usize },
    // The name is stored into a field of a struct being built. The first component names the type
    // constructor of that struct, and the second the position of the field among the fields the
    // type constructor declares.
    CapturedInto(FullName, usize),
    // The name stands where none of the above receives it: the bound value of a `let`, a branch of
    // an `if` or a `match`, an element of an array literal, under a type annotation, either side of
    // an `eval`, or an operand of an inline-LLVM operation. What holds it there passes it on whole,
    // so the position says nothing about the name beyond its being there.
    Elsewhere,
}

// Every use of `name` in `expr`, in the order the walk meets them, one entry per place the name is
// written. An occurrence of a local `name` standing under an inner binding of the same name is a use
// of that binding, and stays out of the result.
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
    // The name whose uses are collected.
    name: &'a FullName,
    // The uses met so far, in the order the walk met them.
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

    // Whether `expr` is the name being searched for, written there.
    fn is_the_name(&self, expr: &Arc<ExprNode>) -> bool {
        expr.is_var() && &expr.get_var().name == self.name
    }
}

impl ExprVisitor for UsageFinder<'_> {
    // The name standing anywhere the calls and the struct expressions below did not already take it
    // from is held as a value there, which is what `Elsewhere` records.
    fn start_visit_var(
        &mut self,
        expr: &Arc<ExprNode>,
        state: &mut VisitState,
    ) -> StartVisitResult {
        if !self.shadowed(state) && self.is_the_name(expr) {
            self.add_usage(UsageType::Elsewhere);
        }
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    // An inline-LLVM operation names the values it operates on rather than holding them as
    // subexpressions, so the walk reaches them here instead of at a `Var` of its own.
    fn start_visit_llvm(
        &mut self,
        expr: &Arc<ExprNode>,
        state: &mut VisitState,
    ) -> StartVisitResult {
        if !self.shadowed(state) {
            let operands = expr.get_llvm().generator.free_vars();
            let written = operands
                .iter()
                .filter(|operand| *operand == self.name)
                .count();
            for _ in 0..written {
                self.add_usage(UsageType::Elsewhere);
            }
        }
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    // A call records the name once for standing as the callee, and once for each argument position
    // it is passed at, with the callee named where the callee is a name.
    //
    // A call of `n` arguments is written as `n` nested applications, and this reads the whole nest
    // at its outermost node, so the walk goes on into the places the nest holds rather than into the
    // nest itself. Were the nest walked, each of its prefixes would read the same callee and record
    // it again.
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
            self.add_usage(UsageType::CalledAsFunction {
                arg_count: args.len(),
            });
        } else {
            self.visit_expr(&fun, state);
        }
        let fun_name = if fun.is_var() {
            Some(fun.get_var().name.clone())
        } else {
            None
        };
        for (i, arg) in args.iter().enumerate() {
            if self.is_the_name(arg) {
                self.add_usage(UsageType::FunctionArgument(fun_name.clone(), i));
            } else {
                self.visit_expr(arg, state);
            }
        }
        StartVisitResult::Return
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

    // A struct being built records the name once for each field it is stored into, by the position
    // that field holds among the fields the type constructor declares.
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
            } else {
                self.visit_expr(value, state);
            }
        }
        StartVisitResult::Return
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::{
        expr_abs, expr_app, expr_array_lit, expr_eval, expr_ffi_call, expr_if, expr_let, expr_llvm,
        expr_make_struct, expr_match, expr_tyanno, expr_var, var_var, Var,
    };
    use crate::ast::pattern::PatternNode;
    use crate::ast::types::{tycon, type_tycon};
    use crate::fixstd::builtin::InlineLLVMMakeStructBody;

    /// A call of `func` supplying `arg_count` arguments, written one argument at a time, with `name`
    /// standing at argument index `at_index`.
    fn call_with_name_at(
        func: &FullName,
        arg_count: usize,
        at_index: usize,
        name: &FullName,
    ) -> Arc<ExprNode> {
        let mut expr = expr_var(func.clone(), None);
        for index in 0..arg_count {
            let arg = if index == at_index {
                expr_var(name.clone(), None)
            } else {
                expr_var(FullName::local(&format!("a{}", index)), None)
            };
            expr = expr_app(expr, vec![arg], None);
        }
        expr
    }

    /// A call is read at its outermost node, so an argument is recorded once however many arguments
    /// the call supplies and wherever among them it stands. A caller counting the uses of a name
    /// reads the count this way round.
    #[test]
    fn an_argument_is_recorded_once_wherever_it_stands() {
        let func = FullName::from_strs(&["Main"], "f");
        let name = FullName::from_strs(&["Main"], "x");
        let arg_count = 4;
        for at_index in 0..arg_count {
            let expr = call_with_name_at(&func, arg_count, at_index, &name);
            let indices = run(&expr, &name)
                .iter()
                .map(|usage| match usage {
                    UsageType::FunctionArgument(callee, index) => {
                        assert_eq!(callee.as_ref(), Some(&func));
                        *index
                    }
                    _ => panic!("a name standing only as an argument is recorded only as one"),
                })
                .collect::<Vec<_>>();
            assert_eq!(
                indices,
                vec![at_index],
                "an argument at index {} of a call of {} arguments",
                at_index,
                arg_count
            );
        }
    }

    /// A binder of the local `name`.
    fn binder(name: &str) -> Arc<Var> {
        var_var(FullName::local(name))
    }

    /// The walk records one use per place the name is written, at every position an expression has,
    /// because a caller putting an expression where the name stands puts it in every one of them.
    /// The positions no call and no struct expression takes the name from record it as held.
    #[test]
    fn every_position_records_one_use() {
        let name = FullName::from_strs(&["Main"], "f");
        let here = || expr_var(name.clone(), None);
        let other = || expr_var(FullName::from_strs(&["Main"], "g"), None);
        let ty = type_tycon(&tycon(FullName::from_strs(&["Main"], "T")));
        let cases: Vec<(&str, Arc<ExprNode>, usize)> = vec![
            ("the name itself", here(), 1),
            ("another name", other(), 0),
            (
                "the body of a lambda",
                expr_abs(vec![binder("x")], here(), None),
                1,
            ),
            (
                "the bound value and the body of a `let`",
                expr_let(
                    PatternNode::make_var(binder("x"), None),
                    here(),
                    here(),
                    None,
                ),
                2,
            ),
            (
                "the condition and both branches of an `if`",
                expr_if(here(), here(), here(), None),
                3,
            ),
            (
                "the scrutinee and every arm of a `match`",
                expr_match(
                    here(),
                    vec![
                        (PatternNode::make_var(binder("x"), None), here()),
                        (PatternNode::make_var(binder("y"), None), other()),
                        (PatternNode::make_var(binder("z"), None), here()),
                    ],
                    None,
                ),
                3,
            ),
            (
                "under a type annotation",
                expr_tyanno(here(), ty.clone(), None),
                1,
            ),
            (
                "two elements of an array literal",
                expr_array_lit(vec![here(), other(), here()], None),
                2,
            ),
            (
                "two arguments of an FFI call",
                expr_ffi_call(
                    "puts".to_string(),
                    tycon(FullName::from_strs(&["Main"], "T")),
                    vec![],
                    false,
                    vec![here(), other(), here()],
                    false,
                    None,
                ),
                2,
            ),
            (
                "both sides of an `eval`",
                expr_eval(here(), here(), None),
                2,
            ),
            (
                "two operands of an inline-LLVM operation",
                expr_llvm(
                    Box::new(InlineLLVMMakeStructBody {
                        field_names: vec![
                            name.clone(),
                            FullName::from_strs(&["Main"], "g"),
                            name.clone(),
                        ],
                    }),
                    ty.clone(),
                    None,
                ),
                2,
            ),
        ];
        for (position, expr, expected) in cases {
            let usages = run(&expr, &name);
            assert_eq!(
                usages.len(),
                expected,
                "wrong number of uses for the name written at {}",
                position
            );
            assert!(
                usages
                    .iter()
                    .all(|usage| matches!(usage, UsageType::Elsewhere)),
                "the name written at {} is held there, so every use of it is recorded as held",
                position
            );
        }
    }

    /// A call takes the name from the callee position and from each argument position, so those
    /// record what the call does with it rather than that it is held.
    #[test]
    fn a_call_records_its_callee_once_and_each_argument_once() {
        let func = FullName::from_strs(&["Main"], "f");
        let arg_count = 4;
        let call = call_with_name_at(&func, arg_count, 1, &FullName::from_strs(&["Main"], "x"));
        let callee_usages = run(&call, &func);
        assert!(
            matches!(
                callee_usages.as_slice(),
                [UsageType::CalledAsFunction { arg_count: n }] if *n == arg_count
            ),
            "a callee is recorded once for the whole call, with the arguments it supplies"
        );
        let struct_ty = tycon(FullName::from_strs(&["Main"], "T"));
        let held = FullName::from_strs(&["Main"], "x");
        let made = expr_make_struct(
            struct_ty.clone(),
            vec![
                ("a".to_string(), expr_var(held.clone(), None)),
                ("b".to_string(), expr_var(func.clone(), None)),
            ],
        );
        assert!(
            matches!(
                run(&made, &held).as_slice(),
                [UsageType::CapturedInto(name, 0)] if *name == struct_ty.name
            ),
            "a name stored into a field is recorded at the field it is stored in"
        );
    }
}
