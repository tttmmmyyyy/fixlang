// This module provides a way to traverse the AST of a program.

use std::sync::Arc;

use super::{Expr, ExprNode};

pub enum StartVisitResult {
    VisitChildren,
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
    expr: Arc<ExprNode>,
    replaced: bool,
}

impl EndVisitResult {
    pub fn unwrap(self, replaced: &mut bool) -> Arc<ExprNode> {
        *replaced = self.replaced;
        self.expr
    }

    pub fn noreplace(expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult {
            expr: expr.clone(),
            replaced: false,
        }
    }

    pub fn add_replaced(mut self: EndVisitResult, replaced: bool) -> EndVisitResult {
        self.replaced |= replaced;
        self
    }
}

pub trait ExprVisitor {
    fn start_visit_var(&mut self, _expr: &Arc<ExprNode>) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_var(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult::noreplace(expr)
    }

    fn start_visit_llvm(&mut self, _expr: &Arc<ExprNode>) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult::noreplace(expr)
    }

    fn start_visit_app(&mut self, _expr: &Arc<ExprNode>) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_app(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult::noreplace(expr)
    }

    fn start_visit_lam(&mut self, _expr: &Arc<ExprNode>) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult::noreplace(expr)
    }

    fn start_visit_let(&mut self, _expr: &Arc<ExprNode>) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_let(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult::noreplace(expr)
    }

    fn start_visit_if(&mut self, _expr: &Arc<ExprNode>) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_if(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult::noreplace(expr)
    }

    fn start_visit_match(&mut self, _expr: &Arc<ExprNode>) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_match(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult::noreplace(expr)
    }

    fn start_visit_tyanno(&mut self, _expr: &Arc<ExprNode>) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_tyanno(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult::noreplace(expr)
    }

    fn start_visit_make_struct(&mut self, _expr: &Arc<ExprNode>) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_make_struct(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult::noreplace(expr)
    }

    fn start_visit_array_lit(&mut self, _expr: &Arc<ExprNode>) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_array_lit(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult::noreplace(expr)
    }

    fn start_visit_ffi_call(&mut self, _expr: &Arc<ExprNode>) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_ffi_call(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        EndVisitResult::noreplace(expr)
    }

    fn visit_expr(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        match &*expr.expr {
            Expr::Var(_var) => {
                let res = self.start_visit_var(expr);
                match res {
                    StartVisitResult::VisitChildren => {
                        // Has no children
                    }
                }
                self.end_visit_var(expr)
            }
            Expr::LLVM(_lit) => {
                let res = self.start_visit_llvm(expr);
                match res {
                    StartVisitResult::VisitChildren => {
                        // Has no children
                    }
                }
                self.end_visit_llvm(expr)
            }
            Expr::App(func, args) => {
                let mut replaced = false;
                let mut expr = expr.clone();

                let res = self.start_visit_app(&expr);
                match res {
                    StartVisitResult::VisitChildren => {
                        let func = self.visit_expr(func).unwrap(&mut replaced);
                        let mut args_new = vec![];
                        for arg in args {
                            let arg = self.visit_expr(&arg).unwrap(&mut replaced);
                            args_new.push(arg);
                        }
                        if replaced {
                            expr = expr.set_app_func(func).set_app_args(args_new);
                        }
                    }
                }
                self.end_visit_app(&expr).add_replaced(replaced)
            }
            Expr::Lam(_args, body) => {
                let mut replaced = false;
                let mut expr = expr.clone();
                let res = self.start_visit_lam(&expr);
                match res {
                    StartVisitResult::VisitChildren => {
                        let body = self.visit_expr(body).unwrap(&mut replaced);
                        if replaced {
                            expr = expr.set_lam_body(body);
                        }
                    }
                }
                self.end_visit_lam(&expr).add_replaced(replaced)
            }
            Expr::Let(_pat, bound, val) => {
                let mut replaced = false;
                let mut expr = expr.clone();
                let res = self.start_visit_let(&expr);
                match res {
                    StartVisitResult::VisitChildren => {
                        let bound = self.visit_expr(bound).unwrap(&mut replaced);
                        let val = self.visit_expr(val).unwrap(&mut replaced);
                        if replaced {
                            expr = expr.set_let_bound(bound).set_let_value(val);
                        }
                    }
                }
                self.end_visit_let(&expr).add_replaced(replaced)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let mut replaced = false;
                let mut expr = expr.clone();
                let res = self.start_visit_if(&expr);
                match res {
                    StartVisitResult::VisitChildren => {
                        let cond = self.visit_expr(cond).unwrap(&mut replaced);
                        let then_expr = self.visit_expr(then_expr).unwrap(&mut replaced);
                        let else_expr = self.visit_expr(else_expr).unwrap(&mut replaced);
                        if replaced {
                            expr = expr
                                .set_if_cond(cond)
                                .set_if_then(then_expr)
                                .set_if_else(else_expr);
                        }
                    }
                }
                self.end_visit_if(&expr).add_replaced(replaced)
            }
            Expr::Match(cond, pat_vals) => {
                let mut replaced = false;
                let mut expr = expr.clone();
                let res = self.start_visit_match(&expr);
                match res {
                    StartVisitResult::VisitChildren => {
                        let cond = self.visit_expr(cond).unwrap(&mut replaced);
                        let mut new_pat_vals = vec![];
                        for (pat, val) in pat_vals {
                            let val = self.visit_expr(&val).unwrap(&mut replaced);
                            new_pat_vals.push((pat.clone(), val));
                        }
                        if replaced {
                            expr = expr.set_match_cond(cond).set_match_pat_vals(new_pat_vals);
                        }
                    }
                }
                self.end_visit_match(&expr).add_replaced(replaced)
            }
            Expr::TyAnno(e, _) => {
                let mut replaced = false;
                let mut expr = expr.clone();
                let res = self.start_visit_tyanno(&expr);
                match res {
                    StartVisitResult::VisitChildren => {
                        let e = self.visit_expr(e).unwrap(&mut replaced);
                        if replaced {
                            expr = expr.set_tyanno_expr(e);
                        }
                    }
                }
                self.end_visit_tyanno(&expr).add_replaced(replaced)
            }
            Expr::MakeStruct(_, fields) => {
                let mut replaced = false;
                let mut expr = expr.clone();
                let res = self.start_visit_make_struct(&expr);
                match res {
                    StartVisitResult::VisitChildren => {
                        let mut new_fields = vec![];
                        for (name, val) in fields {
                            let val = self.visit_expr(&val).unwrap(&mut replaced);
                            new_fields.push((name.clone(), val));
                        }
                        if replaced {
                            expr = expr.set_make_struct_fields(new_fields);
                        }
                    }
                }
                self.end_visit_make_struct(&expr).add_replaced(replaced)
            }
            Expr::ArrayLit(elems) => {
                let mut replaced = false;
                let mut expr = expr.clone();
                let res = self.start_visit_array_lit(&expr);
                match res {
                    StartVisitResult::VisitChildren => {
                        let mut new_elems = vec![];
                        for elem in elems {
                            let elem = self.visit_expr(&elem).unwrap(&mut replaced);
                            new_elems.push(elem);
                        }
                        if replaced {
                            expr = expr.set_array_lit_elems(new_elems);
                        }
                    }
                }
                self.end_visit_array_lit(&expr).add_replaced(replaced)
            }
            Expr::FFICall(_, _, _, args, _) => {
                let mut replaced = false;
                let mut expr = expr.clone();
                let res = self.start_visit_ffi_call(&expr);
                match res {
                    StartVisitResult::VisitChildren => {
                        let mut new_args = vec![];
                        for arg in args {
                            let arg = self.visit_expr(&arg).unwrap(&mut replaced);
                            new_args.push(arg);
                        }
                        if replaced {
                            expr = expr.set_ffi_call_args(new_args);
                        }
                    }
                }
                self.end_visit_ffi_call(&expr).add_replaced(replaced)
            }
        }
    }
}
