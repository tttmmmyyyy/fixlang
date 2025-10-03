// Unwrap newtype pattern, i.e., type A = unbox struct { data : B } to B.

use crate::{
    ast::{
        expr::{expr_let_typed, expr_make_struct, expr_match_typed, expr_var, Expr, ExprNode},
        inline_llvm::LLVMGenerator,
        program::{Program, Symbol, TypeEnv},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
        types::tycon,
    },
    builtin::{make_tuple_name, make_unit_ty},
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

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, state: &mut VisitState) -> EndVisitResult {
        let old_ty = expr.type_.as_ref().unwrap().clone();
        let mut expr = run_on_inferred_type(&expr, &self.type_env);
        let new_ty = expr.type_.as_ref().unwrap().clone();

        let mut llvm = if let Expr::LLVM(llvm) = expr.expr.as_ref() {
            llvm.as_ref().clone()
        } else {
            unreachable!()
        };
        llvm.ty = llvm.ty.unwrap_newtype(&self.type_env);
        expr = expr.set_llvm(llvm.clone());

        // Replace StructGetBody, StructSetBody, StructPunchBody, and StructPlugInBody for structures defined by the newtype pattern.
        match &llvm.generator {
            LLVMGenerator::StructGetBody(body) => {
                // @ : S -> F = |s| GetBody(s)
                // =>
                // @ : F -> F = |s| s
                let field_ty = new_ty;
                let struct_name = body.var_name.clone();
                assert!(struct_name.is_local());
                let struct_ty = state.scope.get_local(&struct_name.name).unwrap().unwrap();
                let struct_ti = struct_ty.toplevel_tycon_info(&self.type_env);
                if struct_ti.is_newtype_pattern() {
                    expr = expr_var(struct_name, expr.source.clone()).set_type(field_ty);
                }
            }
            LLVMGenerator::StructSetBody(body) => {
                // set : F -> S -> S = |f, s| SetBody(f)
                // =>
                // set : F -> F -> F = |f, s| f
                let field_ty = new_ty;
                let struct_ty = old_ty;
                let struct_ti = struct_ty.toplevel_tycon_info(&self.type_env);
                if struct_ti.is_newtype_pattern() {
                    let field_name = body.value_name.clone();
                    expr = expr_var(field_name, expr.source.clone()).set_type(field_ty);
                }
            }
            LLVMGenerator::StructPunchBody(body) => {
                // punch : S -> (F, S*) = |s| Punch(s)
                // =>
                // punch : F -> (F, ()) = |s| (s, ())
                let field_unit_ty = new_ty;
                let struct_name = body.var_name.clone();
                assert!(struct_name.is_local());
                let struct_ty = state.scope.get_local(&struct_name.name).unwrap().unwrap();
                let struct_ti = struct_ty.toplevel_tycon_info(&self.type_env);
                if struct_ti.is_newtype_pattern() {
                    let field_ty = field_unit_ty.collect_type_argments()[0].clone();
                    let unit_ty = make_unit_ty();
                    let struct_expr = expr_var(struct_name, expr.source.clone()).set_type(field_ty);
                    let unit_expr =
                        expr_make_struct(tycon(make_tuple_name(0)), vec![]).set_type(unit_ty);
                    expr = expr_make_struct(
                        tycon(make_tuple_name(2)),
                        vec![("0".to_string(), struct_expr), ("1".to_string(), unit_expr)],
                    )
                    .set_type(field_unit_ty);
                }
            }
            LLVMGenerator::StructPlugInBody(body) => {
                // plug_in : S* -> F -> S = |s, f| PlugIn(s, f)
                // =>
                // plug_in : () -> F -> F = |_, f| f
                let struct_ty = old_ty;
                let struct_ti = struct_ty.toplevel_tycon_info(&self.type_env);
                if struct_ti.is_newtype_pattern() {
                    let field_ty = new_ty;
                    let field_name = body.field_name.clone();
                    assert!(field_name.is_local());
                    expr = expr_var(field_name, expr.source.clone()).set_type(field_ty);
                }
            }
            _ => {}
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
