// Unwrap newtype pattern, i.e., type A = unbox struct { data : B } to B.

use crate::ast::{
    expr::{expr_let, expr_let_typed, expr_llvm, expr_match_typed, Expr, ExprNode},
    inline_llvm::LLVMGenerator,
    program::{Program, Symbol, TypeEnv},
    traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
};
use std::sync::Arc;

pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym, prg.type_env.clone());
    }
    prg.type_env.unwrap_newtype_tycons();
}

fn run_on_symbol(sym: &mut Symbol, type_env: TypeEnv) {
    let mut remover = NewtypeUnwrapper { type_env: type_env };
    let res = remover.traverse(&sym.expr.as_ref().unwrap());
    if res.changed {
        sym.expr = Some(res.expr);
    }
    todo!("replace symbol's type")
}

fn run_on_inferred_type(expr: &Arc<ExprNode>, type_env: &TypeEnv) -> Arc<ExprNode> {
    let type_ = expr.type_.as_ref().unwrap();
    let type_ = type_.unwrap_newtype(type_env);
    expr.set_type(type_)
}

struct NewtypeUnwrapper {
    type_env: TypeEnv,
}

impl ExprVisitor for NewtypeUnwrapper {
    fn start_visit_tyanno(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &self.type_env);

        let ty = expr.get_tyanno_ty();
        let ty = ty.unwrap_newtype(&self.type_env);
        let expr = expr.set_tyanno_ty(ty);

        EndVisitResult::changed(expr)
    }

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &self.type_env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let mut expr = run_on_inferred_type(&expr, &self.type_env);
        if let Expr::LLVM(llvm) = expr.expr.as_ref() {
            let mut llvm = llvm.as_ref().clone();
            llvm.ty = llvm.ty.unwrap_newtype(&self.type_env);
            match &mut llvm.generator {
                LLVMGenerator::StructGetBody(body) => {
                    todo!()
                }
                LLVMGenerator::StructSetBody(body) => {
                    todo!()
                }
                LLVMGenerator::StructPunchBody(body) => {
                    todo!()
                }
                LLVMGenerator::StructPlugInBody(body) => {
                    todo!()
                }
                _ => {}
            }
            expr = expr.set_llvm(llvm);
        } else {
            unreachable!()
        }

        EndVisitResult::changed(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &self.type_env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &self.type_env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let mut expr = run_on_inferred_type(&expr, &self.type_env);
        if let Expr::Let(pat, body, val) = expr.expr.as_ref() {
            let pat = pat.unwrap_newtype(&self.type_env);
            expr = expr_let_typed(pat, body.clone(), val.clone());
        } else {
            unreachable!()
        }
        EndVisitResult::changed(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &self.type_env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let mut expr = run_on_inferred_type(&expr, &self.type_env);
        if let Expr::Match(scrut, arms) = expr.expr.as_ref() {
            let arms = arms
                .iter()
                .map(|(pat, arm_expr)| (pat.unwrap_newtype(&self.type_env), arm_expr.clone()))
                .collect();
            expr = expr_match_typed(scrut.clone(), arms);
        } else {
            unreachable!()
        }
        EndVisitResult::changed(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        let mut expr = expr.clone();
        if let Expr::MakeStruct(tycon, fields) = expr.expr.as_ref() {
            let ti = self.type_env.tycons.get(tycon).unwrap();
            if ti.is_newtype_pattern() {
                expr = fields[0].1.clone();
            }
        } else {
            unreachable!()
        }
        EndVisitResult::changed(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &self.type_env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &self.type_env);
        EndVisitResult::changed(expr)
    }
}
