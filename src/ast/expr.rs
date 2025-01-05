use crate::error::Errors;
use misc::{collect_results, Set};
use name::{FullName, Name, NameSpace};
use printer::Text;
use serde::{Deserialize, Serialize};

use super::*;
use core::panic;
use std::{sync::Arc, vec};

// The ways of apply a function to an argument in source code.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum AppSourceCodeOrderType {
    FX,    // `f(x)`
    XDotF, // `x.f`
}

#[derive(Serialize, Deserialize)]
pub struct ExprNode {
    pub expr: Arc<Expr>,
    free_vars: Option<Set<FullName>>,
    pub source: Option<Span>,
    pub app_order: AppSourceCodeOrderType,
    pub ty: Option<Arc<TypeNode>>,
    // When this expression is a function, this field contains indices of parameters which are released exactly once by calling this function (if known).
    pub released_params_indices: Option<Vec<usize>>,
}

impl ExprNode {
    // Clone all fields except the set of free variables.
    fn clone_without_fvs(&self) -> ExprNode {
        ExprNode {
            expr: self.expr.clone(),
            free_vars: None,
            source: self.source.clone(),
            app_order: self.app_order.clone(),
            ty: self.ty.clone(),
            released_params_indices: self.released_params_indices.clone(),
        }
    }

    // Clone all fields.
    fn clone_all(&self) -> Self {
        ExprNode {
            expr: self.expr.clone(),
            free_vars: self.free_vars.clone(),
            source: self.source.clone(),
            app_order: self.app_order.clone(),
            ty: self.ty.clone(),
            released_params_indices: self.released_params_indices.clone(),
        }
    }

    // Set free vars
    fn set_free_vars(&self, free_vars: Set<FullName>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        ret.free_vars = Some(free_vars);
        Arc::new(ret)
    }

    // Set `released_params_indices`.
    pub fn set_released_params_indices(&self, indices: Vec<usize>) -> Arc<Self> {
        let mut ret = self.clone_all();
        ret.released_params_indices = Some(indices);
        Arc::new(ret)
    }

    // Get free vars.
    pub fn free_vars(self: &Self) -> &Set<FullName> {
        self.free_vars.as_ref().unwrap()
    }

    // Get sorted free vars
    pub fn free_vars_sorted(self: &Self) -> Vec<FullName> {
        let mut free_vars = self.free_vars().iter().cloned().collect::<Vec<_>>();
        free_vars.sort();
        free_vars
    }

    // Set source
    pub fn set_source(&self, src: Option<Span>) -> Arc<Self> {
        let mut ret = self.clone_all();
        ret.source = src;
        Arc::new(ret)
    }

    // Set app order
    pub fn set_app_order(&self, app_order: AppSourceCodeOrderType) -> Arc<Self> {
        let mut ret = self.clone_all();
        ret.app_order = app_order;
        Arc::new(ret)
    }

    // Set inferred type.
    pub fn set_inferred_type(&self, ty: Arc<TypeNode>) -> Arc<Self> {
        let mut ret = self.clone_all();
        ret.ty = Some(ty);
        Arc::new(ret)
    }

    // Check if the expression is a function application.
    #[allow(dead_code)]
    pub fn is_app(&self) -> bool {
        matches!(self.expr.as_ref(), Expr::App(_, _))
    }

    pub fn set_var_namespace(&self, ns: NameSpace) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::Var(var) => {
                let var = var.set_namsapce(ns);
                ret.expr = Arc::new(Expr::Var(var))
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_var_var(&self, v: Arc<Var>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::Var(_) => ret.expr = Arc::new(Expr::Var(v)),
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn is_var(&self) -> bool {
        match &*self.expr {
            Expr::Var(_) => true,
            _ => false,
        }
    }

    pub fn get_var(&self) -> Arc<Var> {
        match &*self.expr {
            Expr::Var(v) => v.clone(),
            _ => {
                panic!()
            }
        }
    }

    pub fn set_app_func(&self, func: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::App(_, arg) => {
                ret.expr = Arc::new(Expr::App(func, arg.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_app_args(&self, args: Vec<Arc<ExprNode>>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::App(func, _) => {
                ret.expr = Arc::new(Expr::App(func.clone(), args));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    #[allow(dead_code)]
    pub fn get_app_func(&self) -> Arc<ExprNode> {
        match &*self.expr {
            Expr::App(func, _) => func.clone(),
            _ => {
                panic!()
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_app_args(&self) -> Vec<Arc<ExprNode>> {
        match &*self.expr {
            Expr::App(_, args) => args.clone(),
            _ => {
                panic!()
            }
        }
    }

    // destructure lambda expression to list of variables and body expression
    pub fn destructure_lam(&self) -> (Vec<Arc<Var>>, Arc<ExprNode>) {
        match &*self.expr {
            Expr::Lam(args, body) => (args.clone(), body.clone()),
            _ => panic!("Call destructure_lam for an expression which is not lambda!"),
        }
    }

    // If the expression is a sequence of lambda construction, i.e., |args0| |args1| |args0| {body}, then this function returns (vec![args0, args1, args2], {body}).
    pub fn destructure_lam_sequence(self: &Arc<ExprNode>) -> (Vec<Vec<Arc<Var>>>, Arc<ExprNode>) {
        let mut args = vec![];
        let mut body = self.clone();
        while body.is_lam() {
            let (args_loc, body_loc) = body.destructure_lam();
            args.push(args_loc);
            body = body_loc;
        }
        (args, body)
    }

    #[allow(dead_code)]
    pub fn set_lam_params(&self, params: Vec<Arc<Var>>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::Lam(_, body) => {
                ret.expr = Arc::new(Expr::Lam(params, body.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_lam_body(&self, body: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::Lam(arg, _) => {
                ret.expr = Arc::new(Expr::Lam(arg.clone(), body));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn is_lam(&self) -> bool {
        match &*self.expr {
            Expr::Lam(_, _) => true,
            _ => false,
        }
    }

    pub fn get_lam_body(&self) -> Arc<ExprNode> {
        match &*self.expr {
            Expr::Lam(_, body) => body.clone(),
            _ => {
                panic!()
            }
        }
    }

    pub fn get_lam_params(&self) -> Vec<Arc<Var>> {
        match &*self.expr {
            Expr::Lam(args, _) => args.clone(),
            _ => {
                panic!()
            }
        }
    }

    pub fn get_let_pat(&self) -> Arc<PatternNode> {
        match &*self.expr {
            Expr::Let(pat, _, _) => pat.clone(),
            _ => {
                panic!()
            }
        }
    }

    pub fn set_let_pat(&self, pat: Arc<PatternNode>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::Let(_, bound, val) => {
                ret.expr = Arc::new(Expr::Let(pat, bound.clone(), val.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_let_bound(&self, bound: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::Let(var, _, val) => {
                ret.expr = Arc::new(Expr::Let(var.clone(), bound, val.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_let_value(&self, value: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::Let(var, bound, _) => {
                ret.expr = Arc::new(Expr::Let(var.clone(), bound.clone(), value));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    // Set the value of let expression, and update the type of the let expression to the type of the value.
    #[allow(dead_code)]
    pub fn set_let_value_typed(&self, value: Arc<ExprNode>) -> Arc<Self> {
        let value_ty = value.ty.as_ref().unwrap().clone();
        self.set_let_value(value).set_inferred_type(value_ty)
    }

    #[allow(dead_code)]
    pub fn get_let_value(&self) -> Arc<Self> {
        match &*self.expr {
            Expr::Let(_, _, val) => val.clone(),
            _ => {
                panic!()
            }
        }
    }

    pub fn set_if_cond(&self, cond: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::If(_, then_expr, else_expr) => {
                ret.expr = Arc::new(Expr::If(cond, then_expr.clone(), else_expr.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_if_then(&self, then_expr: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::If(cond, _, else_expr) => {
                ret.expr = Arc::new(Expr::If(cond.clone(), then_expr, else_expr.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_if_else(&self, else_expr: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::If(cond, then_expr, _) => {
                ret.expr = Arc::new(Expr::If(cond.clone(), then_expr.clone(), else_expr));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_if_then_else(&self, then: Arc<ExprNode>, else_: Arc<ExprNode>) -> Arc<ExprNode> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::If(cond, _, _) => {
                ret.expr = Arc::new(Expr::If(cond.clone(), then, else_));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_if_then_else_typed(
        &self,
        then: Arc<ExprNode>,
        else_: Arc<ExprNode>,
    ) -> Arc<ExprNode> {
        let then_ty = then.ty.as_ref().unwrap().clone();
        let else_ty = else_.ty.as_ref().unwrap().clone();
        assert_eq!(then_ty.to_string(), else_ty.to_string());
        self.set_if_then_else(then, else_)
            .set_inferred_type(then_ty)
    }

    pub fn get_match_cond(&self) -> Arc<ExprNode> {
        match &*self.expr {
            Expr::Match(cond, _) => cond.clone(),
            _ => {
                panic!()
            }
        }
    }

    pub fn set_match_cond(&self, cond_expr: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::Match(_, pat_vals) => {
                ret.expr = Arc::new(Expr::Match(cond_expr, pat_vals.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn get_match_pat_vals(&self) -> Vec<(Arc<PatternNode>, Arc<ExprNode>)> {
        match &*self.expr {
            Expr::Match(_, pat_vals) => pat_vals.clone(),
            _ => {
                panic!()
            }
        }
    }

    pub fn set_match_pat_vals(
        &self,
        pat_vals: Vec<(Arc<PatternNode>, Arc<ExprNode>)>,
    ) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::Match(cond, _) => {
                ret.expr = Arc::new(Expr::Match(cond.clone(), pat_vals));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_match_pat_vals_typed(
        &self,
        pat_vals: Vec<(Arc<PatternNode>, Arc<ExprNode>)>,
    ) -> Arc<ExprNode> {
        let vals_tys = pat_vals
            .iter()
            .map(|(_, val)| val.ty.as_ref().unwrap().clone())
            .collect::<Vec<_>>();
        if vals_tys.len() > 0 {
            let ty = vals_tys[0].clone();
            assert!(vals_tys.iter().all(|t| t.to_string() == ty.to_string()));
        }
        self.set_match_pat_vals(pat_vals)
            .set_inferred_type(vals_tys[0].clone())
    }

    pub fn get_tyanno_expr(&self) -> Arc<ExprNode> {
        match &*self.expr {
            Expr::TyAnno(e, _) => e.clone(),
            _ => {
                panic!()
            }
        }
    }

    pub fn set_tyanno_expr(&self, expr: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::TyAnno(_, t) => {
                ret.expr = Arc::new(Expr::TyAnno(expr, t.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_tyanno_ty(&self, ty: Arc<TypeNode>) -> Arc<Self> {
        let mut ret = self.clone_all();
        match &*self.expr {
            Expr::TyAnno(e, _) => {
                ret.expr = Arc::new(Expr::TyAnno(e.clone(), ty));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_make_struct_tycon(&self, tc: Arc<TyCon>) -> Arc<Self> {
        let mut ret = self.clone_all();
        match &*self.expr {
            Expr::MakeStruct(_, fields) => {
                ret.expr = Arc::new(Expr::MakeStruct(tc, fields.clone()));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_make_struct_field(&self, field_name: &Name, field_expr: Arc<ExprNode>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::MakeStruct(tc, fields) => {
                let mut fields = fields.clone();
                for (name, expr) in &mut fields {
                    if name == field_name {
                        *expr = field_expr.clone();
                    }
                }
                ret.expr = Arc::new(Expr::MakeStruct(tc.clone(), fields));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_make_struct_fields(&self, fields: Vec<(Name, Arc<ExprNode>)>) -> Arc<Self> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::MakeStruct(tc, _) => {
                ret.expr = Arc::new(Expr::MakeStruct(tc.clone(), fields));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_array_lit_elem(&self, elem: Arc<ExprNode>, idx: usize) -> Arc<ExprNode> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::ArrayLit(elems) => {
                let mut elems = elems.clone();
                elems[idx] = elem;
                ret.expr = Arc::new(Expr::ArrayLit(elems));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_array_lit_elems(&self, elems: Vec<Arc<ExprNode>>) -> Arc<ExprNode> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::ArrayLit(_) => {
                ret.expr = Arc::new(Expr::ArrayLit(elems));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_ffi_call_arg(&self, arg: Arc<ExprNode>, idx: usize) -> Arc<ExprNode> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::FFICall(fun_name, ret_ty, param_tys, args, is_io) => {
                let mut args = args.clone();
                args[idx] = arg;
                ret.expr = Arc::new(Expr::FFICall(
                    fun_name.clone(),
                    ret_ty.clone(),
                    param_tys.clone(),
                    args,
                    *is_io,
                ));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_ffi_call_args(&self, args: Vec<Arc<ExprNode>>) -> Arc<ExprNode> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::FFICall(fun_name, ret_ty, param_tys, _, is_io) => {
                ret.expr = Arc::new(Expr::FFICall(
                    fun_name.clone(),
                    ret_ty.clone(),
                    param_tys.clone(),
                    args,
                    *is_io,
                ));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn set_llvm(&self, llvm: InlineLLVM) -> Arc<ExprNode> {
        let mut ret = self.clone_without_fvs();
        match &*self.expr {
            Expr::LLVM(_) => {
                ret.expr = Arc::new(Expr::LLVM(Arc::new(llvm)));
            }
            _ => {
                panic!()
            }
        }
        Arc::new(ret)
    }

    pub fn get_llvm(&self) -> Arc<InlineLLVM> {
        match &*self.expr {
            Expr::LLVM(llvm) => llvm.clone(),
            _ => {
                panic!()
            }
        }
    }

    pub fn is_llvm(&self) -> bool {
        match &*self.expr {
            Expr::LLVM(_) => true,
            _ => false,
        }
    }

    pub fn set_llvm_borrowed_vars(&self, vars: Vec<FullName>) -> Arc<ExprNode> {
        let llvm = self.get_llvm();
        let mut llvm: InlineLLVM = llvm.as_ref().clone();
        llvm.borrowed_vars = vars;
        self.set_llvm(llvm)
    }

    // Returns a list of variables which is released by evaluating this expression.
    // None if the expression does not support this interface yet.
    pub fn released_vars(&self) -> Option<Vec<FullName>> {
        match &*self.expr {
            Expr::LLVM(llvm) => llvm.generator.released_vars(),
            _ => None,
        }
    }

    pub fn resolve_namespace(
        self: &Arc<ExprNode>,
        ctx: &NameResolutionContext,
    ) -> Result<Arc<ExprNode>, Errors> {
        match &*self.expr {
            Expr::Var(_) => {
                // Resolution of names of variables will be done in type checking phase.
                Ok(self.clone())
            }
            Expr::LLVM(llvm) => {
                let mut llvm = llvm.as_ref().clone();
                llvm.ty = llvm.ty.resolve_namespace(ctx)?;
                Ok(self.clone().set_llvm(llvm))
            }
            Expr::App(fun, args) => {
                let mut args_res: Vec<Arc<ExprNode>> = vec![];
                for arg in args {
                    args_res.push(arg.resolve_namespace(ctx)?);
                }
                Ok(self
                    .clone()
                    .set_app_func(fun.resolve_namespace(ctx)?)
                    .set_app_args(args_res))
            }
            Expr::Lam(_, body) => Ok(self.clone().set_lam_body(body.resolve_namespace(ctx)?)),
            Expr::Let(pat, bound, value) => Ok(self
                .clone()
                .set_let_pat(pat.resolve_namespace(ctx)?)
                .set_let_bound(bound.resolve_namespace(ctx)?)
                .set_let_value(value.resolve_namespace(ctx)?)),
            Expr::If(cond, then_expr, else_expr) => Ok(self
                .clone()
                .set_if_cond(cond.resolve_namespace(ctx)?)
                .set_if_then(then_expr.resolve_namespace(ctx)?)
                .set_if_else(else_expr.resolve_namespace(ctx)?)),
            Expr::Match(cond, pat_vals) => {
                let mut pat_vals_res = vec![];
                for (pat, val) in pat_vals {
                    pat_vals_res.push((pat.resolve_namespace(ctx)?, val.resolve_namespace(ctx)?));
                }
                Ok(self
                    .clone()
                    .set_match_cond(cond.resolve_namespace(ctx)?)
                    .set_match_pat_vals(pat_vals_res))
            }
            Expr::TyAnno(expr, ty) => Ok(self
                .clone()
                .set_tyanno_expr(expr.resolve_namespace(ctx)?)
                .set_tyanno_ty(ty.resolve_namespace(ctx)?)),
            Expr::MakeStruct(tc, fields) => {
                let mut expr = self.clone();
                let mut tc = tc.as_ref().clone();
                tc.resolve_namespace(ctx, &self.source)?;
                expr = expr.set_make_struct_tycon(Arc::new(tc));
                for (field_name, field_expr) in fields {
                    let field_expr = field_expr.resolve_namespace(ctx)?;
                    expr = expr.set_make_struct_field(field_name, field_expr);
                }
                Ok(expr)
            }
            Expr::ArrayLit(elems) => {
                let mut expr = self.clone();
                for (i, elem) in elems.iter().enumerate() {
                    expr = expr.set_array_lit_elem(elem.resolve_namespace(ctx)?, i);
                }
                Ok(expr)
            }
            Expr::FFICall(_, _, _, args, _) => {
                let mut expr = self.clone();
                for (i, arg) in args.iter().enumerate() {
                    expr = expr.set_ffi_call_arg(arg.resolve_namespace(ctx)?, i);
                }
                Ok(expr)
            }
        }
    }

    pub fn resolve_type_aliases(
        self: &Arc<ExprNode>,
        type_env: &TypeEnv,
    ) -> Result<Arc<ExprNode>, Errors> {
        match &*self.expr {
            Expr::Var(_) => Ok(self.clone()),
            Expr::LLVM(llvm) => {
                let mut llvm = llvm.as_ref().clone();
                llvm.ty = llvm.ty.resolve_type_aliases(type_env)?;
                Ok(self.clone().set_llvm(llvm))
            }
            Expr::App(fun, args) => {
                let args =
                    collect_results(args.iter().map(|arg| arg.resolve_type_aliases(type_env)))?;
                Ok(self
                    .clone()
                    .set_app_func(fun.resolve_type_aliases(type_env)?)
                    .set_app_args(args))
            }
            Expr::Lam(_, body) => Ok(self
                .clone()
                .set_lam_body(body.resolve_type_aliases(type_env)?)),
            Expr::Let(pat, bound, value) => Ok(self
                .clone()
                .set_let_pat(pat.resolve_type_aliases(type_env)?)
                .set_let_bound(bound.resolve_type_aliases(type_env)?)
                .set_let_value(value.resolve_type_aliases(type_env)?)),
            Expr::If(cond, then_expr, else_expr) => Ok(self
                .clone()
                .set_if_cond(cond.resolve_type_aliases(type_env)?)
                .set_if_then(then_expr.resolve_type_aliases(type_env)?)
                .set_if_else(else_expr.resolve_type_aliases(type_env)?)),
            Expr::Match(cond, pat_vals) => {
                let cond = cond.resolve_type_aliases(type_env)?;
                let mut pat_vals_res = vec![];
                for (pat, val) in pat_vals {
                    let pat = pat.resolve_type_aliases(type_env)?;
                    let val = val.resolve_type_aliases(type_env)?;
                    pat_vals_res.push((pat, val));
                }
                Ok(self
                    .clone()
                    .set_match_cond(cond)
                    .set_match_pat_vals(pat_vals_res))
            }
            Expr::TyAnno(expr, ty) => Ok(self
                .clone()
                .set_tyanno_expr(expr.resolve_type_aliases(type_env)?)
                .set_tyanno_ty(ty.resolve_type_aliases(type_env)?)),
            Expr::MakeStruct(tc, fields) => {
                let mut expr = self.clone();
                if type_env.aliases.contains_key(tc) {
                    return Err(Errors::from_msg_srcs(
                        "In struct construction, cannot use type alias instead of struct name."
                            .to_string(),
                        &[&self.source],
                    ));
                }
                for (field_name, field_expr) in fields {
                    let field_expr = field_expr.resolve_type_aliases(type_env)?;
                    expr = expr.set_make_struct_field(field_name, field_expr);
                }
                Ok(expr)
            }
            Expr::ArrayLit(elems) => {
                let mut expr = self.clone();
                for (i, elem) in elems.iter().enumerate() {
                    expr = expr.set_array_lit_elem(elem.resolve_type_aliases(type_env)?, i);
                }
                Ok(expr)
            }
            Expr::FFICall(_, _, _, args, _) => {
                let mut expr = self.clone();
                for (i, arg) in args.iter().enumerate() {
                    expr = expr.set_ffi_call_arg(arg.resolve_type_aliases(type_env)?, i);
                }
                Ok(expr)
            }
        }
    }

    // Find the minimum AST node which includes the specified source code position.
    pub fn find_node_at(self: &Arc<ExprNode>, pos: &SourcePos) -> Option<EndNode> {
        if self.source.is_none() {
            return None;
        }
        let span = self.source.as_ref().unwrap();
        if !span.includes_pos(pos) {
            return None;
        }
        match &*self.expr {
            Expr::Var(v) => Some(EndNode::Expr(v.as_ref().clone(), self.ty.clone())),
            Expr::LLVM(_) => None,
            Expr::App(func, args) => {
                let node = func.find_node_at(pos);
                if node.is_some() {
                    return node;
                }
                for arg in args {
                    let node = arg.find_node_at(pos);
                    if node.is_some() {
                        return node;
                    }
                }
                None
            }
            Expr::Lam(_, body) => body.find_node_at(pos),
            Expr::Let(pat, bound, val) => {
                let node = pat.find_node_at_pos(pos);
                if node.is_some() {
                    return node;
                }
                let node = bound.find_node_at(pos);
                if node.is_some() {
                    return node;
                }
                val.find_node_at(pos)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let node = cond.find_node_at(pos);
                if node.is_some() {
                    return node;
                }
                let node = then_expr.find_node_at(pos);
                if node.is_some() {
                    return node;
                }
                else_expr.find_node_at(pos)
            }
            Expr::Match(cond, pat_vals) => {
                let node = cond.find_node_at(pos);
                if node.is_some() {
                    return node;
                }
                for (pat, val) in pat_vals {
                    let node = pat.find_node_at_pos(pos);
                    if node.is_some() {
                        return node;
                    }
                    let node = val.find_node_at(pos);
                    if node.is_some() {
                        return node;
                    }
                }
                None
            }
            Expr::TyAnno(e, ty) => {
                let node = e.find_node_at(pos);
                if node.is_some() {
                    return node;
                }
                ty.find_node_at(pos)
            }
            Expr::MakeStruct(tc, fields) => {
                for (_, field_expr) in fields {
                    let node = field_expr.find_node_at(pos);
                    if node.is_some() {
                        return node;
                    }
                }
                Some(EndNode::Type(tc.as_ref().clone()))
            }
            Expr::ArrayLit(elems) => {
                for elem in elems {
                    let node = elem.find_node_at(pos);
                    if node.is_some() {
                        return node;
                    }
                }
                None
            }
            Expr::FFICall(_, _, _, args, _) => {
                for (_, e) in args.iter().enumerate() {
                    let node = e.find_node_at(pos);
                    if node.is_some() {
                        return node;
                    }
                }
                None
            }
        }
    }

    pub fn free_vars_shadowed_by(&self, shadowed_vars: &Set<FullName>) -> Set<FullName> {
        let mut free_vars = self.free_vars().clone();
        free_vars.retain(|v| !shadowed_vars.contains(v));
        free_vars
    }

    pub fn calculate_free_vars(self: &Arc<ExprNode>) -> Arc<ExprNode> {
        match &*self.expr {
            Expr::Var(var) => {
                let free_vars = vec![var.name.clone()].into_iter().collect();
                self.set_free_vars(free_vars)
            }
            Expr::LLVM(lit) => {
                let free_vars = lit.free_vars.clone().into_iter().collect();
                self.set_free_vars(free_vars)
            }
            Expr::App(func, args) => {
                let func = func.calculate_free_vars();
                let args = args
                    .iter()
                    .map(|arg| arg.calculate_free_vars())
                    .collect::<Vec<_>>();
                let mut free_vars = func.free_vars.clone().unwrap();
                for arg in &args {
                    free_vars.extend(arg.free_vars.clone().unwrap());
                }
                self.set_app_func(func)
                    .set_app_args(args)
                    .set_free_vars(free_vars)
            }
            Expr::Lam(args, body) => {
                let body = body.calculate_free_vars();
                let mut free_vars = body.free_vars.clone().unwrap();
                for arg in args {
                    free_vars.remove(&arg.name);
                }
                free_vars.remove(&FullName::local(CAP_NAME));
                self.set_lam_body(body).set_free_vars(free_vars)
            }
            Expr::Let(pat, bound, val) => {
                // NOTE: Our let is non-recursive let, i.e.,
                // "let x = f x in g x" is equal to "let y = f x in g y",
                // and x ∈ FreeVars("let x = f x in g x") = (FreeVars(g x) - {x}) + FreeVars(f x) != (FreeVars(g x) + FreeVars(f x)) - {x}.
                let bound = bound.calculate_free_vars();
                let val = val.calculate_free_vars();
                let mut free_vars = val.free_vars.clone().unwrap();
                for v in pat.pattern.vars() {
                    free_vars.remove(&v);
                }
                free_vars.extend(bound.free_vars.clone().unwrap());
                self.set_let_bound(bound)
                    .set_let_value(val)
                    .set_free_vars(free_vars)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = cond.calculate_free_vars();
                let then_expr = then_expr.calculate_free_vars();
                let else_expr = else_expr.calculate_free_vars();
                let mut free_vars = cond.free_vars.clone().unwrap();
                free_vars.extend(then_expr.free_vars.clone().unwrap());
                free_vars.extend(else_expr.free_vars.clone().unwrap());
                self.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
                    .set_free_vars(free_vars)
            }
            Expr::Match(cond, pat_vals) => {
                let mut free_vars = Set::default();
                let cond = cond.calculate_free_vars();
                free_vars.extend(cond.free_vars.clone().unwrap());
                let mut pat_vals_res = vec![];
                for (pat, val) in pat_vals {
                    let val = val.calculate_free_vars();
                    let mut free_vars_shadowed = val.free_vars.clone().unwrap();
                    let pat_vars = pat.pattern.vars();
                    free_vars_shadowed.retain(|v| !pat_vars.contains(v));
                    free_vars.extend(free_vars_shadowed);
                    pat_vals_res.push((pat.clone(), val));
                }
                self.set_match_cond(cond)
                    .set_match_pat_vals(pat_vals_res)
                    .set_free_vars(free_vars)
            }
            Expr::TyAnno(e, _) => {
                let e = e.calculate_free_vars();
                let free_vars = e.free_vars.clone().unwrap();
                self.set_tyanno_expr(e).set_free_vars(free_vars)
            }
            Expr::MakeStruct(_, fields) => {
                let mut free_vars: Set<FullName> = Default::default();
                let mut ei = self.clone();
                for (field_name, field_expr) in fields {
                    let field_expr = field_expr.calculate_free_vars();
                    free_vars.extend(field_expr.free_vars.clone().unwrap());
                    ei = ei.set_make_struct_field(field_name, field_expr);
                }
                ei.set_free_vars(free_vars)
            }
            Expr::ArrayLit(elems) => {
                let mut free_vars: Set<FullName> = Default::default();
                let mut ei = self.clone();
                for (i, e) in elems.iter().enumerate() {
                    let e = e.calculate_free_vars();
                    ei = ei.set_array_lit_elem(e.clone(), i);
                    free_vars.extend(e.free_vars.clone().unwrap());
                }
                ei.set_free_vars(free_vars)
            }
            Expr::FFICall(_, _, _, args, _) => {
                let mut free_vars: Set<FullName> = Default::default();
                let mut ei = self.clone();
                for (i, e) in args.iter().enumerate() {
                    let e = e.calculate_free_vars();
                    ei = ei.set_ffi_call_arg(e.clone(), i);
                    free_vars.extend(e.free_vars.clone().unwrap());
                }
                ei.set_free_vars(free_vars)
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Expr {
    Var(Arc<Var>),
    LLVM(Arc<InlineLLVM>),
    // application of multiple arguments is generated by optimization.
    App(Arc<ExprNode>, Vec<Arc<ExprNode>>),
    // lambda of multiple arguments is generated by optimization.
    Lam(Vec<Arc<Var>>, Arc<ExprNode>),
    Let(Arc<PatternNode>, Arc<ExprNode>, Arc<ExprNode>),
    If(Arc<ExprNode>, Arc<ExprNode>, Arc<ExprNode>),
    Match(Arc<ExprNode>, Vec<(Arc<PatternNode>, Arc<ExprNode>)>),
    TyAnno(Arc<ExprNode>, Arc<TypeNode>),
    ArrayLit(Vec<Arc<ExprNode>>),
    MakeStruct(Arc<TyCon>, Vec<(Name, Arc<ExprNode>)>),
    FFICall(
        Name,               /* function name */
        Arc<TyCon>,         /* Return type */
        Vec<Arc<TyCon>>,    /* Parameter types */
        Vec<Arc<ExprNode>>, /* Arguments */
        bool,               /* is_ios */
    ),
}

impl Expr {
    pub fn into_expr_info(self: &Arc<Self>, src: Option<Span>) -> Arc<ExprNode> {
        Arc::new(ExprNode {
            expr: self.clone(),
            free_vars: Default::default(),
            source: src,
            app_order: AppSourceCodeOrderType::FX,
            ty: None,
            released_params_indices: None,
        })
    }

    // Stringify expression.
    // Returns the lines paired with the indent level.
    pub fn stringify(&self) -> Text {
        match self {
            Expr::Var(v) => Text::from_string(v.name.to_string()),
            Expr::LLVM(l) => Text::from_string(l.name.clone()),
            Expr::App(_, _) => {
                // Stringify the funciton.
                let (fun, args) = collect_app(&Arc::new(self.clone()).into_expr_info(None));
                let brace_fun = match *(fun.expr) {
                    Expr::Var(_) => false,
                    Expr::LLVM(_) => false,
                    Expr::App(_, _) => false,
                    _ => true,
                };
                let mut fun = fun.expr.stringify();
                if brace_fun {
                    fun = fun.brace()
                };

                // Stringify the arguments.
                let mut arg_texts: Vec<Text> = vec![];
                for arg in args {
                    arg_texts.push(arg.expr.stringify());
                }
                let args = Text::join(arg_texts, ", ").brace();

                fun.append_nobreak(args)
            }
            Expr::Lam(xs, fx) => {
                let args = format!(
                    "|{}{}{}| ",
                    if xs.len() > 1 { "{{" } else { "" },
                    xs.iter()
                        .map(|x| x.name.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    if xs.len() > 1 { "}}" } else { "" }
                );
                fx.expr
                    .stringify()
                    .brace_if_multiline()
                    .insert_to_first_line(&args)
            }
            Expr::Let(p, b, v) => Text::from_str("let ")
                .append_to_last_line(&p.pattern.to_string())
                .append_to_last_line(" = ")
                .append_nobreak(b.expr.stringify().brace_if_multiline())
                .append_to_last_line(";")
                .append(v.expr.stringify()),
            Expr::If(c, t, e) => Text::from_str("if ")
                .append_nobreak(c.expr.stringify().brace_if_multiline())
                .append_to_last_line(" {")
                .append(t.expr.stringify().add_indent(1))
                .append(Text::from_str("} else {"))
                .append(e.expr.stringify().add_indent(1))
                .append(Text::from_str("}")),
            Expr::Match(cond, pat_vals) => {
                let mut branches = Text::empty();
                for (pat, val) in pat_vals {
                    let branch = Text::from_str(&pat.pattern.to_string())
                        .append_to_last_line(" => ")
                        .append_nobreak(val.expr.stringify().brace_if_multiline())
                        .append_to_last_line(",");
                    branches = branches.append(branch);
                }
                Text::from_str("match ")
                    .append_nobreak(cond.expr.stringify().brace_if_multiline())
                    .append_nobreak(branches.curly_brace())
            }
            Expr::TyAnno(e, t) => e
                .expr
                .stringify()
                .brace_if_multiline()
                .append_to_last_line(" : ")
                .append_to_last_line(&t.to_string()),
            Expr::MakeStruct(tc, fields) => {
                let mut field_lines: Vec<Text> = vec![];
                for (name, expr) in fields {
                    let field = Text::from_str(&name)
                        .append_to_last_line(" : ")
                        .append_nobreak(expr.expr.stringify().brace_if_multiline());
                    field_lines.push(field);
                }
                let fields = Text::join(field_lines, ", ").curly_brace();
                Text::from_str(&tc.to_string()).append_nobreak(fields)
            }
            Expr::ArrayLit(elems) => {
                let mut elem_lines: Vec<Text> = vec![];
                for elem in elems {
                    let elem = elem.expr.stringify();
                    elem_lines.push(elem);
                }
                Text::join(elem_lines, ", ").square_brace()
            }
            Expr::FFICall(fun_name, _, _, args, is_io) => {
                let mut arg_texts: Vec<Text> = vec![];
                arg_texts.push(Text::from_str(&fun_name));
                for arg in args {
                    arg_texts.push(arg.expr.stringify());
                }
                let args = Text::join(arg_texts, ", ").square_brace();
                Text::from_str(&format!("FFI_CALL{}", if *is_io { "_IO" } else { "" }))
                    .append_nobreak(args)
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Var {
    pub name: FullName,
}

impl Var {
    pub fn set_namsapce(&self, ns: NameSpace) -> Arc<Self> {
        let mut ret = self.clone();
        ret.name.namespace = ns;
        Arc::new(ret)
    }

    pub fn set_name(&self, fullname: FullName) -> Arc<Self> {
        let mut ret = self.clone();
        ret.name = fullname;
        Arc::new(ret)
    }

    pub fn create(name: FullName) -> Self {
        Var { name }
    }
}

pub fn var_var(name: FullName) -> Arc<Var> {
    Arc::new(Var { name })
}

pub fn var_local(var_name: &str) -> Arc<Var> {
    var_var(FullName::local(var_name))
}

pub fn expr_llvm(
    generator: LLVMGenerator,
    free_vars: Vec<FullName>,
    name: String,
    ty: Arc<TypeNode>,
    src: Option<Span>,
) -> Arc<ExprNode> {
    Arc::new(Expr::LLVM(Arc::new(InlineLLVM {
        generator,
        free_vars,
        name,
        ty,
        borrowed_vars: vec![],
    })))
    .into_expr_info(src)
}

pub fn expr_let(
    pat: Arc<PatternNode>,
    bound: Arc<ExprNode>,
    expr: Arc<ExprNode>,
    src: Option<Span>,
) -> Arc<ExprNode> {
    Arc::new(Expr::Let(pat, bound, expr)).into_expr_info(src)
}

pub fn expr_let_typed(
    pat: Arc<PatternNode>,
    bound: Arc<ExprNode>,
    value: Arc<ExprNode>,
) -> Arc<ExprNode> {
    let ty = value.ty.as_ref().unwrap().clone();
    expr_let(pat, bound, value, None).set_inferred_type(ty)
}

pub fn expr_abs(vars: Vec<Arc<Var>>, val: Arc<ExprNode>, src: Option<Span>) -> Arc<ExprNode> {
    Arc::new(Expr::Lam(vars, val)).into_expr_info(src)
}

pub fn expr_abs_many(mut vars: Vec<Arc<Var>>, mut val: Arc<ExprNode>) -> Arc<ExprNode> {
    while let Some(var) = vars.pop() {
        val = expr_abs(vec![var], val, None);
    }
    val
}

// Create abstract expression `|{var}| {val}`, and set the inferred type.
pub fn expr_abs_typed(var: Arc<Var>, var_ty: Arc<TypeNode>, val: Arc<ExprNode>) -> Arc<ExprNode> {
    let expr = expr_abs(vec![var], val.clone(), None);
    let abs_ty = type_fun(var_ty, val.ty.as_ref().unwrap().clone());
    expr.set_inferred_type(abs_ty)
}

pub fn expr_app(lam: Arc<ExprNode>, args: Vec<Arc<ExprNode>>, src: Option<Span>) -> Arc<ExprNode> {
    Arc::new(Expr::App(lam, args)).into_expr_info(src)
}

// Create application expression `{lam}({args})`, and set the inferred type.
pub fn expr_app_typed(lam: Arc<ExprNode>, args: Vec<Arc<ExprNode>>) -> Arc<ExprNode> {
    let dst_ty = lam.ty.as_ref().unwrap().get_lambda_dst();
    expr_app(lam, args, None).set_inferred_type(dst_ty)
}

// Make variable expression.
pub fn expr_var(name: FullName, src: Option<Span>) -> Arc<ExprNode> {
    Arc::new(Expr::Var(var_var(name))).into_expr_info(src)
}

pub fn expr_if(
    cond: Arc<ExprNode>,
    then_expr: Arc<ExprNode>,
    else_expr: Arc<ExprNode>,
    src: Option<Span>,
) -> Arc<ExprNode> {
    Arc::new(Expr::If(cond, then_expr, else_expr)).into_expr_info(src)
}

pub fn expr_if_typed(
    cond: Arc<ExprNode>,
    then_expr: Arc<ExprNode>,
    else_expr: Arc<ExprNode>,
) -> Arc<ExprNode> {
    let then_ty = then_expr.ty.as_ref().unwrap().clone();
    let else_ty = else_expr.ty.as_ref().unwrap().clone();
    assert_eq!(then_ty.to_string(), else_ty.to_string());
    expr_if(cond, then_expr, else_expr, None).set_inferred_type(then_ty)
}

pub fn expr_match(
    cond: Arc<ExprNode>,
    cases: Vec<(Arc<PatternNode>, Arc<ExprNode>)>,
    src: Option<Span>,
) -> Arc<ExprNode> {
    Arc::new(Expr::Match(cond, cases)).into_expr_info(src)
}

pub fn expr_match_typed(
    cond: Arc<ExprNode>,
    cases: Vec<(Arc<PatternNode>, Arc<ExprNode>)>,
) -> Arc<ExprNode> {
    let vals_tys = cases
        .iter()
        .map(|(_, val)| val.ty.as_ref().unwrap().clone())
        .collect::<Vec<_>>();
    if vals_tys.len() > 0 {
        let ty = vals_tys[0].clone();
        assert!(vals_tys.iter().all(|t| t.to_string() == ty.to_string()));
    }
    expr_match(cond, cases, None).set_inferred_type(vals_tys[0].clone())
}

pub fn expr_tyanno(expr: Arc<ExprNode>, ty: Arc<TypeNode>, src: Option<Span>) -> Arc<ExprNode> {
    Arc::new(Expr::TyAnno(expr, ty)).into_expr_info(src)
}

pub fn expr_make_struct(tc: Arc<TyCon>, fields: Vec<(Name, Arc<ExprNode>)>) -> Arc<ExprNode> {
    Arc::new(Expr::MakeStruct(tc, fields)).into_expr_info(None)
}

pub fn expr_array_lit(elems: Vec<Arc<ExprNode>>, src: Option<Span>) -> Arc<ExprNode> {
    Arc::new(Expr::ArrayLit(elems)).into_expr_info(src)
}

pub fn expr_ffi_call(
    fun_name: Name,
    ret_ty: Arc<TyCon>,
    param_tys: Vec<Arc<TyCon>>,
    args: Vec<Arc<ExprNode>>,
    is_ios: bool,
    src: Option<Span>,
) -> Arc<ExprNode> {
    Arc::new(Expr::FFICall(fun_name, ret_ty, param_tys, args, is_ios)).into_expr_info(src)
}

// Convert f(y, z) to (f, [y, z]).
pub fn collect_app(expr: &Arc<ExprNode>) -> (Arc<ExprNode>, Vec<Arc<ExprNode>>) {
    match &*expr.expr {
        Expr::App(fun, arg) => {
            let (fun, mut args) = collect_app(fun);
            args.append(&mut arg.clone());
            (fun, args)
        }
        _ => (expr.clone(), vec![]),
    }
}
