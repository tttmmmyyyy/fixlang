// This module provides a way to traverse the AST of a program.

use std::sync::Arc;

use super::{typecheck::Scope, types::TypeNode, Expr, ExprNode};

pub enum StartVisitResult {
    VisitChildren,
    ReplaceAndRevisit(Arc<ExprNode>),
    // Skip, // to be implemented
    // ReplaceAndSkipChildren(Arc<ExprNode>), // to be implemented
    // ReplaceAndVisitChildren(Arc<ExprNode>), // to be implemented
}

// pub enum EndVisitResult {
//     NoReplace,
//     Replace(Arc<ExprNode>),
//     // ReplaceAndRevisit(Arc<ExprNode>), // to be implemented
// }

pub struct EndVisitResult {
    pub expr: Arc<ExprNode>,
    pub changed: bool,
    pub revisit: bool,
}
impl EndVisitResult {
    pub fn unwrap(self, changed: &mut bool) -> Arc<ExprNode> {
        *changed |= self.changed;
        self.expr
    }

    pub fn unchanged(expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult {
            expr: expr.clone(),
            changed: false,
            revisit: false,
        }
    }

    pub fn changed(expr: Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult {
            expr,
            changed: true,
            revisit: false,
        }
    }

    // Revisit the expression if it has been changed.
    pub fn revisit(mut self: EndVisitResult) -> EndVisitResult {
        self.revisit = true;
        self
    }

    fn add_changed(mut self: EndVisitResult, changed: bool) -> EndVisitResult {
        self.changed |= changed;
        self
    }
}

pub struct VisitState {
    pub scope: Scope<Option<Arc<TypeNode>>>,
}

impl Default for VisitState {
    fn default() -> Self {
        VisitState {
            scope: Scope::default(),
        }
    }
}

pub trait ExprVisitor {
    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult;
    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult;

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult;
    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult;

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult;
    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult;

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult;
    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult;

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult;
    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult;

    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult;
    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult;

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult;
    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult;

    fn start_visit_tyanno(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult;
    fn end_visit_tyanno(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState)
        -> EndVisitResult;

    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult;
    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult;

    fn start_visit_array_lit(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult;
    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult;

    fn start_visit_ffi_call(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult;
    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult;

    fn traverse(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        let mut state = VisitState::default();
        self.visit_expr(expr, &mut state)
    }

    fn revisit_if_changed(
        &mut self,
        end_res: EndVisitResult,
        state: &mut VisitState,
    ) -> EndVisitResult {
        if end_res.changed && end_res.revisit {
            self.visit_expr(&end_res.expr, state).add_changed(true)
        } else {
            end_res
        }
    }

    fn visit_expr(&mut self, expr: &Arc<ExprNode>, state: &mut VisitState) -> EndVisitResult {
        match &*expr.expr {
            Expr::Var(_var) => {
                let res = self.start_visit_var(expr, state);
                match res {
                    StartVisitResult::VisitChildren => {
                        // Has no children
                    }
                    StartVisitResult::ReplaceAndRevisit(expr) => {
                        return self.visit_expr(&expr, state).add_changed(true);
                    }
                }
                let res = self.end_visit_var(expr, state);
                self.revisit_if_changed(res, state)
            }
            Expr::LLVM(_lit) => {
                let res = self.start_visit_llvm(expr, state);
                match res {
                    StartVisitResult::VisitChildren => {
                        // Has no children
                    }
                    StartVisitResult::ReplaceAndRevisit(expr) => {
                        return self.visit_expr(&expr, state).add_changed(true);
                    }
                }
                let res = self.end_visit_llvm(expr, state);
                self.revisit_if_changed(res, state)
            }
            Expr::App(func, args) => {
                let mut changed = false;
                let mut expr = expr.clone();

                let res = self.start_visit_app(&expr, state);
                match res {
                    StartVisitResult::VisitChildren => {
                        let func = self.visit_expr(func, state).unwrap(&mut changed);
                        let mut args_new = vec![];
                        for arg in args {
                            let arg = self.visit_expr(&arg, state).unwrap(&mut changed);
                            args_new.push(arg);
                        }
                        if changed {
                            expr = expr.set_app_func(func).set_app_args(args_new);
                        }
                    }
                    StartVisitResult::ReplaceAndRevisit(expr) => {
                        return self.visit_expr(&expr, state).add_changed(true);
                    }
                }
                let res = self.end_visit_app(&expr, state).add_changed(changed);
                self.revisit_if_changed(res, state)
            }
            Expr::Lam(args, body) => {
                let mut changed = false;
                let mut expr = expr.clone();
                let res = self.start_visit_lam(&expr, state);
                match res {
                    StartVisitResult::VisitChildren => {
                        let arg_tys = expr.ty.as_ref().map(|ty| ty.get_lambda_srcs());
                        for (i, arg) in args.iter().enumerate() {
                            state.scope.push(
                                &arg.name.name,
                                arg_tys.as_ref().map(|arg_tys| arg_tys[i].clone()),
                            );
                        }
                        let body = self.visit_expr(body, state).unwrap(&mut changed);
                        for arg in args {
                            state.scope.pop(&arg.name.name);
                        }
                        if changed {
                            expr = expr.set_lam_body(body);
                        }
                    }
                    StartVisitResult::ReplaceAndRevisit(expr) => {
                        return self.visit_expr(&expr, state).add_changed(true);
                    }
                }
                let res = self.end_visit_lam(&expr, state).add_changed(changed);
                self.revisit_if_changed(res, state)
            }
            Expr::Let(pat, bound, val) => {
                let mut changed = false;
                let mut expr = expr.clone();
                let res = self.start_visit_let(&expr, state);
                match res {
                    StartVisitResult::VisitChildren => {
                        let bound = self.visit_expr(bound, state).unwrap(&mut changed);
                        let names = pat.vars_with_types();
                        for (v, opt_ty) in &names {
                            state.scope.push(&v.name, opt_ty.clone());
                        }
                        let val = self.visit_expr(val, state).unwrap(&mut changed);
                        for (v, _) in names.iter().rev() {
                            state.scope.pop(&v.name);
                        }
                        if changed {
                            expr = expr.set_let_bound(bound).set_let_value(val);
                        }
                    }
                    StartVisitResult::ReplaceAndRevisit(expr) => {
                        return self.visit_expr(&expr, state).add_changed(true);
                    }
                }
                let res = self.end_visit_let(&expr, state).add_changed(changed);
                self.revisit_if_changed(res, state)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let mut changed = false;
                let mut expr = expr.clone();
                let res = self.start_visit_if(&expr, state);
                match res {
                    StartVisitResult::VisitChildren => {
                        let cond = self.visit_expr(cond, state).unwrap(&mut changed);
                        let then_expr = self.visit_expr(then_expr, state).unwrap(&mut changed);
                        let else_expr = self.visit_expr(else_expr, state).unwrap(&mut changed);
                        if changed {
                            expr = expr
                                .set_if_cond(cond)
                                .set_if_then(then_expr)
                                .set_if_else(else_expr);
                        }
                    }
                    StartVisitResult::ReplaceAndRevisit(expr) => {
                        return self.visit_expr(&expr, state).add_changed(true);
                    }
                }
                let res = self.end_visit_if(&expr, state).add_changed(changed);
                self.revisit_if_changed(res, state)
            }
            Expr::Match(cond, pat_vals) => {
                let mut changed = false;
                let mut expr = expr.clone();
                let res = self.start_visit_match(&expr, state);
                match res {
                    StartVisitResult::VisitChildren => {
                        let cond = self.visit_expr(cond, state).unwrap(&mut changed);
                        let mut new_pat_vals = vec![];
                        for (pat, val) in pat_vals {
                            let names = pat.vars_with_types();
                            for (v, opt_ty) in &names {
                                state.scope.push(&v.name, opt_ty.clone());
                            }
                            let val = self.visit_expr(&val, state).unwrap(&mut changed);
                            for (v, _) in names.iter().rev() {
                                state.scope.pop(&v.name);
                            }
                            new_pat_vals.push((pat.clone(), val));
                        }
                        if changed {
                            expr = expr.set_match_cond(cond).set_match_pat_vals(new_pat_vals);
                        }
                    }
                    StartVisitResult::ReplaceAndRevisit(expr) => {
                        return self.visit_expr(&expr, state).add_changed(true);
                    }
                }
                let res = self.end_visit_match(&expr, state).add_changed(changed);
                self.revisit_if_changed(res, state)
            }
            Expr::TyAnno(e, _) => {
                let mut changed = false;
                let mut expr = expr.clone();
                let res = self.start_visit_tyanno(&expr, state);
                match res {
                    StartVisitResult::VisitChildren => {
                        let e = self.visit_expr(e, state).unwrap(&mut changed);
                        if changed {
                            expr = expr.set_tyanno_expr(e);
                        }
                    }
                    StartVisitResult::ReplaceAndRevisit(expr) => {
                        return self.visit_expr(&expr, state).add_changed(true);
                    }
                }
                let res = self.end_visit_tyanno(&expr, state).add_changed(changed);
                self.revisit_if_changed(res, state)
            }
            Expr::MakeStruct(_, fields) => {
                let mut changed = false;
                let mut expr = expr.clone();
                let res = self.start_visit_make_struct(&expr, state);
                match res {
                    StartVisitResult::VisitChildren => {
                        let mut new_fields = vec![];
                        for (name, val) in fields {
                            let val = self.visit_expr(&val, state).unwrap(&mut changed);
                            new_fields.push((name.clone(), val));
                        }
                        if changed {
                            expr = expr.set_make_struct_fields(new_fields);
                        }
                    }
                    StartVisitResult::ReplaceAndRevisit(expr) => {
                        return self.visit_expr(&expr, state).add_changed(true);
                    }
                }
                let res = self
                    .end_visit_make_struct(&expr, state)
                    .add_changed(changed);
                self.revisit_if_changed(res, state)
            }
            Expr::ArrayLit(elems) => {
                let mut changed = false;
                let mut expr = expr.clone();
                let res = self.start_visit_array_lit(&expr, state);
                match res {
                    StartVisitResult::VisitChildren => {
                        let mut new_elems = vec![];
                        for elem in elems {
                            let elem = self.visit_expr(&elem, state).unwrap(&mut changed);
                            new_elems.push(elem);
                        }
                        if changed {
                            expr = expr.set_array_lit_elems(new_elems);
                        }
                    }
                    StartVisitResult::ReplaceAndRevisit(expr) => {
                        return self.visit_expr(&expr, state).add_changed(true);
                    }
                }
                let res = self.end_visit_array_lit(&expr, state).add_changed(changed);
                self.revisit_if_changed(res, state)
            }
            Expr::FFICall(_, _, _, args, _) => {
                let mut changed = false;
                let mut expr = expr.clone();
                let res = self.start_visit_ffi_call(&expr, state);
                match res {
                    StartVisitResult::VisitChildren => {
                        let mut new_args = vec![];
                        for arg in args {
                            let arg = self.visit_expr(&arg, state).unwrap(&mut changed);
                            new_args.push(arg);
                        }
                        if changed {
                            expr = expr.set_ffi_call_args(new_args);
                        }
                    }
                    StartVisitResult::ReplaceAndRevisit(expr) => {
                        return self.visit_expr(&expr, state).add_changed(true);
                    }
                }
                let res = self.end_visit_ffi_call(&expr, state).add_changed(changed);
                self.revisit_if_changed(res, state)
            }
        }
    }
}
