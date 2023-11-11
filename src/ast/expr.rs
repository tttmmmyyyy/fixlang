use serde::{Deserialize, Serialize};

use super::*;
use core::panic;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum AppSourceCodeOrderType {
    FunctionIsFormer,
    ArgumentIsFormer,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExprNode {
    pub expr: Rc<Expr>,
    pub free_vars: Option<HashSet<FullName>>,
    pub source: Option<Span>,
    pub app_order: AppSourceCodeOrderType,
    pub ty: Option<Rc<TypeNode>>,
}

impl ExprNode {
    // Set free vars
    fn set_free_vars(&self, free_vars: HashSet<FullName>) -> Rc<Self> {
        let mut ret = self.clone();
        ret.free_vars = Some(free_vars);
        Rc::new(ret)
    }

    // Get free vars
    pub fn free_vars(self: &Self) -> &HashSet<FullName> {
        self.free_vars.as_ref().unwrap()
    }

    // Set source
    pub fn set_source(&self, src: Option<Span>) -> Rc<Self> {
        let mut ret = self.clone();
        ret.source = src;
        Rc::new(ret)
    }

    // Set app order
    pub fn set_app_order(&self, app_order: AppSourceCodeOrderType) -> Rc<Self> {
        let mut ret = self.clone();
        ret.app_order = app_order;
        Rc::new(ret)
    }

    // Set inferred type.
    pub fn set_inferred_type(&self, ty: Rc<TypeNode>) -> Rc<Self> {
        let mut ret = self.clone();
        ret.ty = Some(ty);
        Rc::new(ret)
    }

    pub fn set_var_namespace(&self, ns: NameSpace) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Var(var) => {
                let var = var.set_namsapce(ns);
                ret.expr = Rc::new(Expr::Var(var))
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_var_var(&self, v: Rc<Var>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Var(_) => ret.expr = Rc::new(Expr::Var(v)),
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_app_func(&self, func: Rc<ExprNode>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::App(_, arg) => {
                ret.expr = Rc::new(Expr::App(func, arg.clone()));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_app_args(&self, args: Vec<Rc<ExprNode>>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::App(func, _) => {
                ret.expr = Rc::new(Expr::App(func.clone(), args));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    // destructure lambda expression to list of variables and body expression
    pub fn destructure_lam(&self) -> (Vec<Rc<Var>>, Rc<ExprNode>) {
        match &*self.expr {
            Expr::Lam(args, body) => (args.clone(), body.clone()),
            _ => panic!(""),
        }
    }

    #[allow(dead_code)]
    pub fn set_lam_params(&self, params: Vec<Rc<Var>>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Lam(_, body) => {
                ret.expr = Rc::new(Expr::Lam(params, body.clone()));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_lam_body(&self, body: Rc<ExprNode>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Lam(arg, _) => {
                ret.expr = Rc::new(Expr::Lam(arg.clone(), body));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    #[allow(dead_code)]
    pub fn set_let_pat(&self, pat: Rc<PatternNode>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Let(_, bound, val) => {
                ret.expr = Rc::new(Expr::Let(pat, bound.clone(), val.clone()));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_let_bound(&self, bound: Rc<ExprNode>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Let(var, _, val) => {
                ret.expr = Rc::new(Expr::Let(var.clone(), bound, val.clone()));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_let_value(&self, value: Rc<ExprNode>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::Let(var, bound, _) => {
                ret.expr = Rc::new(Expr::Let(var.clone(), bound.clone(), value));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    #[allow(dead_code)]
    pub fn get_let_value(&self) -> Rc<Self> {
        match &*self.expr {
            Expr::Let(_, _, val) => val.clone(),
            _ => {
                panic!()
            }
        }
    }

    pub fn set_if_cond(&self, cond: Rc<ExprNode>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::If(_, then_expr, else_expr) => {
                ret.expr = Rc::new(Expr::If(cond, then_expr.clone(), else_expr.clone()));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_if_then(&self, then_expr: Rc<ExprNode>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::If(cond, _, else_expr) => {
                ret.expr = Rc::new(Expr::If(cond.clone(), then_expr, else_expr.clone()));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_if_else(&self, else_expr: Rc<ExprNode>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::If(cond, then_expr, _) => {
                ret.expr = Rc::new(Expr::If(cond.clone(), then_expr.clone(), else_expr));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_tyanno_expr(&self, expr: Rc<ExprNode>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::TyAnno(_, t) => {
                ret.expr = Rc::new(Expr::TyAnno(expr, t.clone()));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_tyanno_ty(&self, ty: Rc<TypeNode>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::TyAnno(e, _) => {
                ret.expr = Rc::new(Expr::TyAnno(e.clone(), ty));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_lit_lit(&self, lit: Rc<InlineLLVM>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::LLVM(_) => {
                ret.expr = Rc::new(Expr::LLVM(lit));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_make_struct_tycon(&self, tc: Rc<TyCon>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::MakeStruct(_, fields) => {
                ret.expr = Rc::new(Expr::MakeStruct(tc, fields.clone()));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_make_struct_field(&self, field_name: &Name, field_expr: Rc<ExprNode>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::MakeStruct(tc, fields) => {
                let mut fields = fields.clone();
                for (name, expr) in &mut fields {
                    if name == field_name {
                        *expr = field_expr.clone();
                    }
                }
                ret.expr = Rc::new(Expr::MakeStruct(tc.clone(), fields));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_make_struct_fields(&self, fields: Vec<(Name, Rc<ExprNode>)>) -> Rc<Self> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::MakeStruct(tc, _) => {
                ret.expr = Rc::new(Expr::MakeStruct(tc.clone(), fields));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_array_lit_elem(&self, elem: Rc<ExprNode>, idx: usize) -> Rc<ExprNode> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::ArrayLit(elems) => {
                let mut elems = elems.clone();
                elems[idx] = elem;
                ret.expr = Rc::new(Expr::ArrayLit(elems));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn set_call_c_arg(&self, arg: Rc<ExprNode>, idx: usize) -> Rc<ExprNode> {
        let mut ret = self.clone();
        match &*self.expr {
            Expr::CallC(fun_name, ret_ty, param_tys, is_var_args, args) => {
                let mut args = args.clone();
                args[idx] = arg;
                ret.expr = Rc::new(Expr::CallC(
                    fun_name.clone(),
                    ret_ty.clone(),
                    param_tys.clone(),
                    *is_var_args,
                    args,
                ));
            }
            _ => {
                panic!()
            }
        }
        Rc::new(ret)
    }

    pub fn resolve_namespace(self: &Rc<ExprNode>, ctx: &NameResolutionContext) -> Rc<ExprNode> {
        match &*self.expr {
            Expr::Var(_) => {
                // Resolution of names of variables will be done in type checking phase.
                self.clone()
            }
            Expr::LLVM(lit) => {
                let mut lit = lit.as_ref().clone();
                lit.ty = lit.ty.resolve_namespace(ctx);
                self.clone().set_lit_lit(Rc::new(lit))
            }
            Expr::App(fun, args) => {
                let args = args.iter().map(|arg| arg.resolve_namespace(ctx)).collect();
                self.clone()
                    .set_app_func(fun.resolve_namespace(ctx))
                    .set_app_args(args)
            }
            Expr::Lam(_, body) => self.clone().set_lam_body(body.resolve_namespace(ctx)),
            Expr::Let(pat, bound, value) => self
                .clone()
                .set_let_pat(pat.resolve_namespace(ctx))
                .set_let_bound(bound.resolve_namespace(ctx))
                .set_let_value(value.resolve_namespace(ctx)),
            Expr::If(cond, then_expr, else_expr) => self
                .clone()
                .set_if_cond(cond.resolve_namespace(ctx))
                .set_if_then(then_expr.resolve_namespace(ctx))
                .set_if_else(else_expr.resolve_namespace(ctx)),
            Expr::TyAnno(expr, ty) => self
                .clone()
                .set_tyanno_expr(expr.resolve_namespace(ctx))
                .set_tyanno_ty(ty.resolve_namespace(ctx)),
            Expr::MakeStruct(tc, fields) => {
                let mut expr = self.clone();
                let mut tc = tc.as_ref().clone();
                let resolve_result = tc.resolve_namespace(ctx);
                if resolve_result.is_err() {
                    let msg = resolve_result.unwrap_err();
                    error_exit_with_src(&msg, &self.source)
                }
                expr = expr.set_make_struct_tycon(Rc::new(tc));
                for (field_name, field_expr) in fields {
                    let field_expr = field_expr.resolve_namespace(ctx);
                    expr = expr.set_make_struct_field(field_name, field_expr);
                }
                expr
            }
            Expr::ArrayLit(elems) => {
                let mut expr = self.clone();
                for (i, elem) in elems.iter().enumerate() {
                    expr = expr.set_array_lit_elem(elem.resolve_namespace(ctx), i);
                }
                expr
            }
            Expr::CallC(_, _, _, _, args) => {
                let mut expr = self.clone();
                for (i, arg) in args.iter().enumerate() {
                    expr = expr.set_call_c_arg(arg.resolve_namespace(ctx), i);
                }
                expr
            }
        }
    }

    pub fn resolve_type_aliases(self: &Rc<ExprNode>, type_env: &TypeEnv) -> Rc<ExprNode> {
        match &*self.expr {
            Expr::Var(_) => self.clone(),
            Expr::LLVM(lit) => {
                let mut lit = lit.as_ref().clone();
                lit.ty = lit.ty.resolve_type_aliases(type_env);
                self.clone().set_lit_lit(Rc::new(lit))
            }
            Expr::App(fun, args) => {
                let args = args
                    .iter()
                    .map(|arg| arg.resolve_type_aliases(type_env))
                    .collect();
                self.clone()
                    .set_app_func(fun.resolve_type_aliases(type_env))
                    .set_app_args(args)
            }
            Expr::Lam(_, body) => self
                .clone()
                .set_lam_body(body.resolve_type_aliases(type_env)),
            Expr::Let(pat, bound, value) => self
                .clone()
                .set_let_pat(pat.resolve_type_aliases(type_env))
                .set_let_bound(bound.resolve_type_aliases(type_env))
                .set_let_value(value.resolve_type_aliases(type_env)),
            Expr::If(cond, then_expr, else_expr) => self
                .clone()
                .set_if_cond(cond.resolve_type_aliases(type_env))
                .set_if_then(then_expr.resolve_type_aliases(type_env))
                .set_if_else(else_expr.resolve_type_aliases(type_env)),
            Expr::TyAnno(expr, ty) => self
                .clone()
                .set_tyanno_expr(expr.resolve_type_aliases(type_env))
                .set_tyanno_ty(ty.resolve_type_aliases(type_env)),
            Expr::MakeStruct(tc, fields) => {
                let mut expr = self.clone();
                if type_env.aliases.contains_key(tc) {
                    error_exit_with_src(
                        "In struct construction, cannot use type alias instead of struct name.",
                        &self.source,
                    );
                }
                for (field_name, field_expr) in fields {
                    let field_expr = field_expr.resolve_type_aliases(type_env);
                    expr = expr.set_make_struct_field(field_name, field_expr);
                }
                expr
            }
            Expr::ArrayLit(elems) => {
                let mut expr = self.clone();
                for (i, elem) in elems.iter().enumerate() {
                    expr = expr.set_array_lit_elem(elem.resolve_type_aliases(type_env), i);
                }
                expr
            }
            Expr::CallC(_, _, _, _, args) => {
                let mut expr = self.clone();
                for (i, arg) in args.iter().enumerate() {
                    expr = expr.set_call_c_arg(arg.resolve_type_aliases(type_env), i);
                }
                expr
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Expr {
    Var(Rc<Var>),
    LLVM(Rc<InlineLLVM>),
    // application of multiple arguments is generated by optimization.
    App(Rc<ExprNode>, Vec<Rc<ExprNode>>),
    // lambda of multiple arguments is generated by optimization.
    Lam(Vec<Rc<Var>>, Rc<ExprNode>),
    Let(Rc<PatternNode>, Rc<ExprNode>, Rc<ExprNode>),
    If(Rc<ExprNode>, Rc<ExprNode>, Rc<ExprNode>),
    TyAnno(Rc<ExprNode>, Rc<TypeNode>),
    ArrayLit(Vec<Rc<ExprNode>>),
    // Expresison `(x, y)` is not parsed to `Tuple2.new x y`, but to `MakeStruct x y`.
    // `MakeStruct x y` is compiled to a more performant code than function call (currently).
    MakeStruct(Rc<TyCon>, Vec<(Name, Rc<ExprNode>)>),
    CallC(
        Name,              /* function name */
        Rc<TyCon>,         /* Return type */
        Vec<Rc<TyCon>>,    /* Parameter types */
        bool,              /* Is va_args? */
        Vec<Rc<ExprNode>>, /* Arguments */
    ),
}

impl Expr {
    pub fn into_expr_info(self: &Rc<Self>, src: Option<Span>) -> Rc<ExprNode> {
        Rc::new(ExprNode {
            expr: self.clone(),
            free_vars: Default::default(),
            source: src,
            app_order: AppSourceCodeOrderType::FunctionIsFormer,
            ty: None,
        })
    }
    pub fn to_string(&self) -> String {
        match self {
            Expr::Var(v) => v.name.to_string(),
            Expr::LLVM(l) => l.name.clone(),
            Expr::App(_, _) => {
                let (fun, args) = collect_app(&Rc::new(self.clone()).into_expr_info(None));
                let mut omit_brace_around_fun = false;
                match *(fun.expr) {
                    Expr::Var(_) => omit_brace_around_fun = true,
                    Expr::LLVM(_) => omit_brace_around_fun = true,
                    Expr::App(_, _) => omit_brace_around_fun = true,
                    _ => {}
                }
                let fun_str = fun.expr.to_string();

                let args_str = args
                    .iter()
                    .map(|arg| arg.expr.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                if omit_brace_around_fun {
                    format!("{}({})", fun_str, args_str)
                } else {
                    format!("({})({})", fun_str, args_str)
                }
            }
            Expr::Lam(xs, fx) => {
                format!(
                    "|{}| {}",
                    xs.iter()
                        .map(|x| x.name.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    fx.expr.to_string()
                )
            }
            Expr::Let(p, b, v) => format!(
                "let {}={} in {}",
                p.pattern.to_string(),
                b.expr.to_string(),
                v.expr.to_string()
            ),
            Expr::If(c, t, e) => format!(
                "if {} then {} else {}",
                c.expr.to_string(),
                t.expr.to_string(),
                e.expr.to_string()
            ),
            Expr::TyAnno(e, t) => format!("{}: {}", e.expr.to_string(), t.to_string()),
            Expr::MakeStruct(tc, fields) => {
                format!(
                    "{} {{{}}}",
                    tc.to_string(),
                    fields
                        .iter()
                        .map(|(name, expr)| format!("{}: {}", name, expr.expr.to_string()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expr::ArrayLit(elems) => {
                format!(
                    "[{}]",
                    elems
                        .iter()
                        .map(|e| e.expr.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expr::CallC(fun_name, _, _, _, args) => {
                format!(
                    "CALL_C[{}{}]",
                    fun_name,
                    args.iter()
                        .map(|e| ", ".to_string() + &e.expr.to_string())
                        .collect::<Vec<_>>()
                        .join("")
                )
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LLVMGenerator {
    IntLit(InlineLLVMIntLit),
    FloatLit(InlineLLVMFloatLit),
    NullPtrLit(InlineLLVMNullPtrLit),
    BoolLit(InlineLLVMBoolLit),
    StringLit(InlineLLVMStringLit),
    FixBody(InlineLLVMFixBody),
    CastIntegralBody(InlineLLVMCastIntegralBody),
    CastFloatBody(InlineLLVMCastFloatBody),
    CastIntToFloatBody(InlineLLVMCastIntToFloatBody),
    CastFloatToIntBody(InlineLLVMCastFloatToIntBody),
    ShiftBody(InlineLLVMShiftBody),
    BitwiseOperationBody(InlineLLVMBitwiseOperationBody),
    FillArrayBody(InlineLLVMFillArrayBody),
    MakeEmptyArrayBody(InlineLLVMMakeEmptyArrayBody),
    UnsafeSetArrayBody(InlineLLVMUnsafeSetArrayBody),
    UnsafeGetArrayBody(InlineLLVMUnsafeGetArrayBody),
    UnsafeSetSizeArrayBody(InlineLLVMUnsafeSetSizeArrayBody),
    ArrayGetBody(InlineLLVMArrayGetBody),
    ArraySetBody(InlineLLVMArraySetBody),
    ArrayModBody(InlineLLVMArrayModBody),
    ArrayForceUniqueBody(InlineLLVMArrayForceUniqueBody),
    ArrayGetPtrBody(InlineLLVMArrayGetPtrBody),
    ArrayGetSizeBody(InlineLLVMArrayGetSizeBody),
    ArrayGetCapacityBody(InlineLLVMArrayGetCapacityBody),
    StructGetBody(InlineLLVMStructGetBody),
    StructModBody(InlineLLVMStructModBody),
    StructSetBody(InlineLLVMStructSetBody),
    MakeUnionBody(InlineLLVMMakeUnionBody),
    UnionAsBody(InlineLLVMUnionAsBody),
    UnionIsBody(InlineLLVMUnionIsBody),
    UnionModBody(InlineLLVMUnionModBody),
    LoopFunctionBody(InlineLLVMLoopFunctionBody),
    AbortFunctionBody(InlineLLVMAbortFunctionBody),
    IsUniqueFunctionBody(InlineLLVMIsUniqueFunctionBody),
    IntNegBody(InlineLLVMIntNegBody),
    FloatNegBody(InlineLLVMFloatNegBody),
    BoolNegBody(InlineLLVMBoolNegBody),
    IntEqBody(InlineLLVMIntEqBody),
    PtrEqBody(InlineLLVMPtrEqBody),
    FloatEqBody(InlineLLVMFloatEqBody),
    IntLessThanBody(InlineLLVMIntLessThanBody),
    FloatLessThanBody(InlineLLVMFloatLessThanBody),
    IntLessThanOrEqBody(InlineLLVMIntLessThanOrEqBody),
    FloatLessThanOrEqBody(InlineLLVMFloatLessThanOrEqBody),
    IntAddBody(InlineLLVMIntAddBody),
    FloatAddBody(InlineLLVMFloatAddBody),
    IntSubBody(InlineLLVMIntSubBody),
    FloatSubBody(InlineLLVMFloatSubBody),
    IntMulBody(InlineLLVMIntMulBody),
    FloatMulBody(InlineLLVMFloatMulBody),
    IntDivBody(InlineLLVMIntDivBody),
    FloatDivBody(InlineLLVMFloatDivBody),
    IntRemBody(InlineLLVMIntRemBody),
}

impl LLVMGenerator {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Rc<TypeNode>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        match self {
            LLVMGenerator::IntLit(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::FloatLit(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::NullPtrLit(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::BoolLit(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::StringLit(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::FixBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::CastIntegralBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::CastFloatBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::CastIntToFloatBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::CastFloatToIntBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::ShiftBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::BitwiseOperationBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::FillArrayBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::MakeEmptyArrayBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::UnsafeSetArrayBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::UnsafeGetArrayBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::UnsafeSetSizeArrayBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::ArrayGetBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::ArraySetBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::ArrayModBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::ArrayForceUniqueBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::ArrayGetPtrBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::ArrayGetSizeBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::ArrayGetCapacityBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::StructGetBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::StructModBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::StructSetBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::MakeUnionBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::UnionAsBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::UnionIsBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::UnionModBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::LoopFunctionBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::AbortFunctionBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::IsUniqueFunctionBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::IntNegBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::FloatNegBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::BoolNegBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::IntEqBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::PtrEqBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::FloatEqBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::IntLessThanBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::FloatLessThanBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::IntLessThanOrEqBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::FloatLessThanOrEqBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::IntAddBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::FloatAddBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::IntSubBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::FloatSubBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::IntMulBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::FloatMulBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::IntDivBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::FloatDivBody(x) => x.generate(gc, ty, rvo),
            LLVMGenerator::IntRemBody(x) => x.generate(gc, ty, rvo),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVM {
    pub generator: LLVMGenerator,
    pub free_vars: Vec<FullName>, // e.g. "+" literal has two free variables.
    name: String,
    pub ty: Rc<TypeNode>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Var {
    pub name: FullName,
}

impl Var {
    pub fn set_namsapce(&self, ns: NameSpace) -> Rc<Self> {
        let mut ret = self.clone();
        ret.name.namespace = ns;
        Rc::new(ret)
    }

    pub fn set_name(&self, nsn: FullName) -> Rc<Self> {
        let mut ret = self.clone();
        ret.name = nsn;
        Rc::new(ret)
    }
}

pub fn var_var(name: FullName) -> Rc<Var> {
    Rc::new(Var { name })
}

pub fn var_local(var_name: &str) -> Rc<Var> {
    var_var(FullName::local(var_name))
}

pub fn expr_llvm(
    generator: LLVMGenerator,
    free_vars: Vec<FullName>,
    name: String,
    ty: Rc<TypeNode>,
    src: Option<Span>,
) -> Rc<ExprNode> {
    Rc::new(Expr::LLVM(Rc::new(InlineLLVM {
        generator,
        free_vars,
        name,
        ty,
    })))
    .into_expr_info(src)
}

pub fn expr_let(
    pat: Rc<PatternNode>,
    bound: Rc<ExprNode>,
    expr: Rc<ExprNode>,
    src: Option<Span>,
) -> Rc<ExprNode> {
    Rc::new(Expr::Let(pat, bound, expr)).into_expr_info(src)
}

pub fn expr_abs(vars: Vec<Rc<Var>>, val: Rc<ExprNode>, src: Option<Span>) -> Rc<ExprNode> {
    Rc::new(Expr::Lam(vars, val)).into_expr_info(src)
}

pub fn expr_app(lam: Rc<ExprNode>, args: Vec<Rc<ExprNode>>, src: Option<Span>) -> Rc<ExprNode> {
    Rc::new(Expr::App(lam, args)).into_expr_info(src)
}

// Make variable expression.
pub fn expr_var(name: FullName, src: Option<Span>) -> Rc<ExprNode> {
    Rc::new(Expr::Var(var_var(name))).into_expr_info(src)
}

pub fn expr_if(
    cond: Rc<ExprNode>,
    then_expr: Rc<ExprNode>,
    else_expr: Rc<ExprNode>,
    src: Option<Span>,
) -> Rc<ExprNode> {
    Rc::new(Expr::If(cond, then_expr, else_expr)).into_expr_info(src)
}

pub fn expr_tyanno(expr: Rc<ExprNode>, ty: Rc<TypeNode>, src: Option<Span>) -> Rc<ExprNode> {
    Rc::new(Expr::TyAnno(expr, ty)).into_expr_info(src)
}

pub fn expr_make_struct(tc: Rc<TyCon>, fields: Vec<(Name, Rc<ExprNode>)>) -> Rc<ExprNode> {
    Rc::new(Expr::MakeStruct(tc, fields)).into_expr_info(None)
}

pub fn expr_array_lit(elems: Vec<Rc<ExprNode>>, src: Option<Span>) -> Rc<ExprNode> {
    Rc::new(Expr::ArrayLit(elems)).into_expr_info(src)
}

pub fn expr_call_c(
    fun_name: Name,
    ret_ty: Rc<TyCon>,
    param_tys: Vec<Rc<TyCon>>,
    is_va_args: bool,
    args: Vec<Rc<ExprNode>>,
    src: Option<Span>,
) -> Rc<ExprNode> {
    Rc::new(Expr::CallC(fun_name, ret_ty, param_tys, is_va_args, args)).into_expr_info(src)
}

// TODO: Use persistent binary search tree avoid O(n^2) complexity of calculate_free_vars?
pub fn calculate_free_vars(ei: Rc<ExprNode>) -> Rc<ExprNode> {
    match &*ei.expr {
        Expr::Var(var) => {
            let free_vars = vec![var.name.clone()].into_iter().collect();
            ei.set_free_vars(free_vars)
        }
        Expr::LLVM(lit) => {
            let free_vars = lit.free_vars.clone().into_iter().collect();
            ei.set_free_vars(free_vars)
        }
        Expr::App(func, args) => {
            let func = calculate_free_vars(func.clone());
            let args = args
                .iter()
                .map(|arg| calculate_free_vars(arg.clone()))
                .collect::<Vec<_>>();
            let mut free_vars = func.free_vars.clone().unwrap();
            for arg in &args {
                free_vars.extend(arg.free_vars.clone().unwrap());
            }
            ei.set_app_func(func)
                .set_app_args(args)
                .set_free_vars(free_vars)
        }
        Expr::Lam(args, body) => {
            let body = calculate_free_vars(body.clone());
            let mut free_vars = body.free_vars.clone().unwrap();
            for arg in args {
                free_vars.remove(&arg.name);
            }
            free_vars.remove(&FullName::local(CAP_NAME));
            ei.set_lam_body(body).set_free_vars(free_vars)
        }
        Expr::Let(pat, bound, val) => {
            // NOTE: Our let is non-recursive let, i.e.,
            // "let x = f x in g x" is equal to "let y = f x in g y",
            // and x ∈ FreeVars("let x = f x in g x") = (FreeVars(g x) - {x}) + FreeVars(f x) != (FreeVars(g x) + FreeVars(f x)) - {x}.
            let bound = calculate_free_vars(bound.clone());
            let val = calculate_free_vars(val.clone());
            let mut free_vars = val.free_vars.clone().unwrap();
            for v in pat.pattern.vars() {
                free_vars.remove(&v);
            }
            free_vars.extend(bound.free_vars.clone().unwrap());
            ei.set_let_bound(bound)
                .set_let_value(val)
                .set_free_vars(free_vars)
        }
        Expr::If(cond, then_expr, else_expr) => {
            let cond = calculate_free_vars(cond.clone());
            let then_expr = calculate_free_vars(then_expr.clone());
            let else_expr = calculate_free_vars(else_expr.clone());
            let mut free_vars = cond.free_vars.clone().unwrap();
            free_vars.extend(then_expr.free_vars.clone().unwrap());
            free_vars.extend(else_expr.free_vars.clone().unwrap());
            ei.set_if_cond(cond)
                .set_if_then(then_expr)
                .set_if_else(else_expr)
                .set_free_vars(free_vars)
        }
        Expr::TyAnno(e, _) => {
            let e = calculate_free_vars(e.clone());
            let free_vars = e.free_vars.clone().unwrap();
            ei.set_tyanno_expr(e).set_free_vars(free_vars)
        }
        Expr::MakeStruct(_, fields) => {
            let mut free_vars: HashSet<FullName> = Default::default();
            let mut ei = ei.clone();
            for (field_name, field_expr) in fields {
                let field_expr = calculate_free_vars(field_expr.clone());
                free_vars.extend(field_expr.free_vars.clone().unwrap());
                ei = ei.set_make_struct_field(field_name, field_expr);
            }
            ei.set_free_vars(free_vars)
        }
        Expr::ArrayLit(elems) => {
            let mut free_vars: HashSet<FullName> = Default::default();
            let mut ei = ei.clone();
            for (i, e) in elems.iter().enumerate() {
                let e = calculate_free_vars(e.clone());
                ei = ei.set_array_lit_elem(e.clone(), i);
                free_vars.extend(e.free_vars.clone().unwrap());
            }
            ei.set_free_vars(free_vars)
        }
        Expr::CallC(_, _, _, _, args) => {
            let mut free_vars: HashSet<FullName> = Default::default();
            let mut ei = ei.clone();
            for (i, e) in args.iter().enumerate() {
                let e = calculate_free_vars(e.clone());
                ei = ei.set_call_c_arg(e.clone(), i);
                free_vars.extend(e.free_vars.clone().unwrap());
            }
            ei.set_free_vars(free_vars)
        }
    }
}

// Convert f(y, z) to (f, [y, z]).
pub fn collect_app(expr: &Rc<ExprNode>) -> (Rc<ExprNode>, Vec<Rc<ExprNode>>) {
    match &*expr.expr {
        Expr::App(fun, arg) => {
            let (fun, mut args) = collect_app(fun);
            args.append(&mut arg.clone());
            (fun, args)
        }
        _ => (expr.clone(), vec![]),
    }
}
