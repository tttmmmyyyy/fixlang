//! Lowering from the typed AST to the RC IR.
//!
//! This is the structural half of P1 lowering: it produces the RC IR skeleton — an A-normal form
//! with fresh, globally-unique names, every lambda lifted to a top-level function, `If` and union
//! patterns desugared to `Match`, and struct/tuple `let`-patterns desugared to a `Destructure` node.
//! Every argument is owned and there are NO explicit `Retain`/`Release` nodes yet; reference-counting
//! insertion is a separate backward pass. The one reference-counting effect already present is the
//! retain baked into the boxed capture getter, per the retain-getter model.

use std::sync::Arc;

use crate::ast::expr::{Expr, ExprNode, Var};
use crate::ast::inline_llvm::{InlineLLVM, LLVMGenerator};
use crate::ast::name::{FullName, Name};
use crate::ast::pattern::{Pattern, PatternNode};
use crate::ast::program::{Symbol, TypeEnv};
use crate::ast::types::{TyCon, TypeNode};
use crate::constants::CAP_NAME;
use crate::fixstd::builtin::{
    make_dynamic_object_ty, InlineLLVMArrayLitBody, InlineLLVMCaptureProjectBody,
    InlineLLVMFFICallBody, InlineLLVMMakeStructBody,
};
use crate::misc::Map;
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::*;

/// A pending binding accumulated during A-normalization: either a single `let var = rhs`, or a
/// whole struct/tuple destructure binding several fields at once (`Destructure`).
enum Stmt {
    Let(RcVar, RcRhs, Option<Span>),
    Destructure(RcVar, Vec<(usize, RcVar)>, Option<Span>),
}

/// The result of lowering one AST symbol.
enum SymbolLowering {
    Func(RcFunc),
    Global(RcGlobalInit),
}

/// Lower `symbols` to an `RcProgram`. Symbols reference one another by name, so the set need not be
/// closed; passing a subset (e.g. one compilation unit) lowers just those. `all_symbols` is the full
/// program's symbols, used only to type a global referenced as an LLVM operand — it must cover every
/// symbol any lowered function might reference, even from another unit.
pub fn lower_program(type_env: &TypeEnv, symbols: &[Symbol], all_symbols: &[Symbol]) -> RcProgram {
    let mut lw = Lowerer::new(type_env);
    for sym in all_symbols {
        lw.symbol_types.insert(sym.name.clone(), sym.ty.clone());
    }
    let mut globals = vec![];
    for sym in symbols {
        match lw.lower_symbol(sym) {
            SymbolLowering::Func(f) => {
                lw.funcs.insert(f.name.clone(), f);
            }
            SymbolLowering::Global(g) => globals.push(g),
        }
    }
    let funcs = std::mem::take(&mut lw.funcs);
    // The real entry point is the program's `main` IO value, which code generation handles
    // separately; this placeholder names it for the dump.
    let entry = FuncRef {
        name: FullName::local("#entry"),
    };
    RcProgram {
        funcs,
        globals,
        entry,
    }
}

/// The lowering context: a fresh-name counter, the accumulated top-level functions, and the scoped
/// environment mapping each AST local name to the RC IR variable currently bound to it. Because a
/// fresh globally-unique name is minted at every binding, the environment resolves shadowing and
/// the resulting names need no scope tracking downstream.
struct Lowerer<'a> {
    type_env: &'a TypeEnv,
    counter: u64,
    funcs: Map<FuncRef, RcFunc>,
    // A shadow stack per AST name; the last entry is the current binding.
    env: Map<FullName, Vec<RcVar>>,
    // The type of each top-level symbol, to type a global referenced as an LLVM operand.
    symbol_types: Map<FullName, Arc<TypeNode>>,
}

impl<'a> Lowerer<'a> {
    fn new(type_env: &'a TypeEnv) -> Self {
        Lowerer {
            type_env,
            counter: 0,
            funcs: Map::default(),
            env: Map::default(),
            symbol_types: Map::default(),
        }
    }

    // --- fresh names ---

    fn fresh_var(&mut self, hint: &str, ty: Arc<TypeNode>, source: Option<Span>) -> RcVar {
        self.counter += 1;
        let name = FullName::local(&format!("{}#{}", hint, self.counter));
        RcVar {
            name,
            ty,
            source,
            debug_name: None,
        }
    }

    fn fresh_func(&mut self, hint: &str) -> FuncRef {
        self.counter += 1;
        FuncRef {
            name: FullName::local(&format!("{}#{}", hint, self.counter)),
        }
    }

    // --- environment ---

    fn push_env(&mut self, ast_name: &FullName, var: RcVar) {
        self.env.entry(ast_name.clone()).or_insert_with(Vec::new).push(var);
    }

    fn pop_env(&mut self, ast_name: &FullName) {
        if let Some(stack) = self.env.get_mut(ast_name) {
            stack.pop();
        }
    }

    fn lookup_env(&self, ast_name: &FullName) -> Option<RcVar> {
        self.env.get(ast_name).and_then(|stack| stack.last()).cloned()
    }

    // --- building the continuation-nested body ---

    /// Fold accumulated statements into the nested continuation chain ending in `terminal`.
    fn build_let_chain(stmts: Vec<Stmt>, terminal: RcExprNode) -> RcExprNode {
        stmts.into_iter().rev().fold(terminal, |cont, stmt| match stmt {
            Stmt::Let(var, rhs, source) => RcExprNode {
                expr: Box::new(RcExpr::Let(var, rhs, cont)),
                source,
            },
            Stmt::Destructure(container, fields, source) => RcExprNode {
                expr: Box::new(RcExpr::Destructure(container, fields, cont)),
                source,
            },
        })
    }

    fn ret_node(var: RcVar) -> RcExprNode {
        let source = var.source.clone();
        RcExprNode {
            expr: Box::new(RcExpr::Ret(var)),
            source,
        }
    }

    // --- symbols ---

    fn lower_symbol(&mut self, sym: &Symbol) -> SymbolLowering {
        let expr = sym.expr.as_ref().expect("symbol has no expression");
        if sym.ty.is_funptr() {
            // A funptr symbol is a top-level function whose expression is a (possibly multi-param)
            // lambda.
            let expr = expr.set_type(sym.ty.clone());
            let func_ref = FuncRef {
                name: sym.name.clone(),
            };
            SymbolLowering::Func(self.lower_lambda_as_function(&expr, func_ref, vec![]))
        } else {
            // A non-funptr symbol is a global value; lower its initializer.
            let init = self.lower_body(expr);
            SymbolLowering::Global(RcGlobalInit {
                symbol: sym.name.clone(),
                ty: sym.ty.clone(),
                init,
            })
        }
    }

    /// Lower an expression as a complete body: its statements followed by a `Ret` of its value.
    fn lower_body(&mut self, expr: &ExprNode) -> RcExprNode {
        let mut stmts = vec![];
        let v = self.lower_to_var(expr, &mut stmts);
        Self::build_let_chain(stmts, Self::ret_node(v))
    }

    /// Lower a lambda into a top-level function. `captures` are the values captured from the
    /// enclosing scope (already resolved to enclosing RC IR variables), in the order the closure
    /// stores them; for a funptr (no captures) it is empty. The body is lowered under a fresh
    /// environment holding only the parameters and the projected captures.
    fn lower_lambda_as_function(
        &mut self,
        lam: &ExprNode,
        func_ref: FuncRef,
        captures: Vec<(FullName, RcVar)>,
    ) -> RcFunc {
        let lam_ty = lam.type_.clone().unwrap();
        let (params, body) = lam.destructure_lam();
        let src_tys = lam_ty.get_lambda_srcs();
        assert_eq!(params.len(), src_tys.len());

        let saved_env = std::mem::take(&mut self.env);

        let mut param_vars = vec![];
        for (p, ty) in params.iter().zip(src_tys.iter()) {
            let pv = self.fresh_var(&p.name.name, ty.clone(), None);
            self.push_env(&p.name, pv.clone());
            param_vars.push(pv);
        }

        let mut stmts = vec![];
        let cap = if lam_ty.is_closure() {
            let cap_var = self.fresh_var("cap", make_dynamic_object_ty(), None);
            // Bind the capture object under the implicit name `#CAP` too, so a built-in that reads the
            // raw capture object by that name (the `fix` combinator's `FixBody`) resolves to it.
            self.push_env(&FullName::local(CAP_NAME), cap_var.clone());
            let cap_tys: Vec<Arc<TypeNode>> =
                captures.iter().map(|(_, v)| v.ty.clone()).collect();
            for (i, (ast_name, _)) in captures.iter().enumerate() {
                let gen = LLVMGenerator::CaptureProjectBody(InlineLLVMCaptureProjectBody {
                    cap_name: cap_var.name.clone(),
                    cap_idx: i,
                    cap_tys: cap_tys.clone(),
                });
                let mut proj = self.fresh_var(&ast_name.name, cap_tys[i].clone(), None);
                proj.debug_name = Some(ast_name.to_string());
                stmts.push(Stmt::Let(proj.clone(), RcRhs::Llvm(gen, vec![cap_var.clone()]), None));
                self.push_env(ast_name, proj);
            }
            Some(cap_var)
        } else {
            assert!(captures.is_empty(), "a funptr function cannot have captures");
            None
        };

        let ret_var = self.lower_to_var(&body, &mut stmts);
        let body_expr = Self::build_let_chain(stmts, Self::ret_node(ret_var));

        self.env = saved_env;

        RcFunc {
            name: func_ref,
            fn_ty: lam_ty.clone(),
            params: param_vars,
            cap,
            ret_ty: lam_ty.get_lambda_dst(),
            body: body_expr,
            source: lam.source.clone(),
        }
    }

    // --- expressions (A-normalization: lower to an atom, appending statements) ---

    fn lower_to_var(&mut self, expr: &ExprNode, stmts: &mut Vec<Stmt>) -> RcVar {
        let ty = expr.type_.clone().unwrap();
        let source = expr.source.clone();
        match expr.expr.as_ref() {
            Expr::Var(v) => self.lower_var(v, &ty, &source),
            Expr::LLVM(inline) => self.lower_llvm(inline, ty, source, stmts),
            Expr::App(fun, args) => self.lower_app(fun, args, ty, source, stmts),
            Expr::Lam(_, _) => self.lower_lam(expr, ty, source, stmts),
            Expr::Let(pat, bound, val) => self.lower_let(pat, bound, val, stmts),
            Expr::If(c, t, e) => self.lower_if(c, t, e, ty, source, stmts),
            Expr::Match(cond, arms) => self.lower_match(cond, arms, ty, source, stmts),
            Expr::TyAnno(e, _) => self.lower_to_var(e, stmts),
            Expr::MakeStruct(_, fields) => self.lower_make_struct(fields, ty, source, stmts),
            Expr::ArrayLit(elems) => self.lower_array_lit(elems, ty, source, stmts),
            Expr::FFICall(fun_name, ret_ty, param_tys, is_va, args, is_io) => self.lower_ffi_call(
                fun_name, ret_ty, param_tys, *is_va, args, *is_io, ty, source, stmts,
            ),
            Expr::Eval(side, main) => self.lower_eval(side, main, stmts),
        }
    }

    fn lower_var(&mut self, v: &Arc<Var>, ty: &Arc<TypeNode>, source: &Option<Span>) -> RcVar {
        match self.lookup_env(&v.name) {
            // A local: reuse the variable already bound (it is already an atom).
            Some(var) => var,
            // A global: an atom naming the global, materialized by code generation.
            None => RcVar {
                name: v.name.clone(),
                ty: ty.clone(),
                source: source.clone(),
                debug_name: None,
            },
        }
    }

    fn lower_llvm(
        &mut self,
        inline: &Arc<InlineLLVM>,
        ty: Arc<TypeNode>,
        source: Option<Span>,
        stmts: &mut Vec<Stmt>,
    ) -> RcVar {
        let mut gen = inline.generator.clone();
        // The generator's free variables are its operands, in a fixed order. A local operand reuses
        // the variable already bound to it; an operand that is not a local is a reference to a global
        // value or function, materialized by code generation from its (unchanged) name.
        let operand_vars: Vec<RcVar> = gen
            .free_vars()
            .iter()
            .map(|name| match self.lookup_env(name) {
                Some(var) => var,
                None => {
                    let ty = self.symbol_types.get(name).cloned().unwrap_or_else(|| {
                        panic!(
                            "LLVM operand `{}` is not bound in scope during RC IR lowering",
                            name.to_string()
                        )
                    });
                    RcVar {
                        name: name.clone(),
                        ty,
                        source: None,
                        debug_name: None,
                    }
                }
            })
            .collect();
        // Rewrite the generator's embedded operand names to the fresh local names, so code
        // generation resolves them from scope.
        for (slot, var) in gen.free_vars_mut().into_iter().zip(operand_vars.iter()) {
            *slot = var.name.clone();
        }
        let result = self.fresh_var("v", ty, source.clone());
        stmts.push(Stmt::Let(result.clone(), RcRhs::Llvm(gen, operand_vars), source));
        result
    }

    fn lower_app(
        &mut self,
        fun: &ExprNode,
        args: &[Arc<ExprNode>],
        ty: Arc<TypeNode>,
        source: Option<Span>,
        stmts: &mut Vec<Stmt>,
    ) -> RcVar {
        // Evaluation order (matching the current generator): the callee first, then the arguments
        // left to right.
        let callee = self.lower_to_var(fun, stmts);
        let arg_vars: Vec<RcVar> = args.iter().map(|arg| self.lower_to_var(arg, stmts)).collect();
        let result = self.fresh_var("app", ty, source.clone());
        stmts.push(Stmt::Let(result.clone(), RcRhs::App(callee, arg_vars), source));
        result
    }

    fn lower_lam(
        &mut self,
        expr: &ExprNode,
        ty: Arc<TypeNode>,
        source: Option<Span>,
        stmts: &mut Vec<Stmt>,
    ) -> RcVar {
        // Resolve the captured values from the enclosing scope, in the closure's storage order.
        let cap_names = expr.lambda_cap_names();
        let cap_vals: Vec<RcVar> = cap_names
            .iter()
            .map(|n| self.lookup_env(n).expect("captured variable not bound"))
            .collect();
        let captures: Vec<(FullName, RcVar)> =
            cap_names.iter().cloned().zip(cap_vals.iter().cloned()).collect();

        let func_ref = self.fresh_func("lambda");
        let rc_func = self.lower_lambda_as_function(expr, func_ref.clone(), captures);
        self.funcs.insert(func_ref.clone(), rc_func);

        let result = self.fresh_var("closure", ty, source.clone());
        stmts.push(Stmt::Let(result.clone(), RcRhs::Closure(func_ref, cap_vals), source));
        result
    }

    fn lower_let(
        &mut self,
        pat: &PatternNode,
        bound: &ExprNode,
        val: &ExprNode,
        stmts: &mut Vec<Stmt>,
    ) -> RcVar {
        let bound_var = self.lower_to_var(bound, stmts);
        let pushed = self.destructure_pattern(pat, &bound_var, stmts);
        let result = self.lower_to_var(val, stmts);
        for name in &pushed {
            self.pop_env(name);
        }
        result
    }

    fn lower_if(
        &mut self,
        cond: &ExprNode,
        then_expr: &ExprNode,
        else_expr: &ExprNode,
        ty: Arc<TypeNode>,
        source: Option<Span>,
        stmts: &mut Vec<Stmt>,
    ) -> RcVar {
        // Desugar to a match on the Bool union: `_true` (tag 1) -> then, `_false` (tag 0) -> else.
        let cond_var = self.lower_to_var(cond, stmts);
        let payload_tys = cond_var.ty.field_types(self.type_env);
        let then_arm = MatchArm {
            variant: Some(1),
            payload: self.fresh_var("unit", payload_tys[1].clone(), None),
            body: self.lower_body(then_expr),
        };
        let else_arm = MatchArm {
            variant: Some(0),
            payload: self.fresh_var("unit", payload_tys[0].clone(), None),
            body: self.lower_body(else_expr),
        };
        let result = self.fresh_var("if", ty, source.clone());
        stmts.push(Stmt::Let(result.clone(), RcRhs::Match(cond_var, vec![then_arm, else_arm]), source));
        result
    }

    fn lower_match(
        &mut self,
        cond: &ExprNode,
        arms: &[(Arc<PatternNode>, Arc<ExprNode>)],
        ty: Arc<TypeNode>,
        source: Option<Span>,
        stmts: &mut Vec<Stmt>,
    ) -> RcVar {
        let cond_var = self.lower_to_var(cond, stmts);
        let rc_arms: Vec<MatchArm> = arms
            .iter()
            .map(|(pat, body)| self.lower_match_arm(&cond_var, pat, body))
            .collect();
        let result = self.fresh_var("match", ty, source.clone());
        stmts.push(Stmt::Let(result.clone(), RcRhs::Match(cond_var, rc_arms), source));
        result
    }

    fn lower_match_arm(&mut self, scrutinee: &RcVar, pat: &PatternNode, body: &ExprNode) -> MatchArm {
        match &pat.pattern {
            Pattern::Union(variant_name, _, subpat) => {
                let (variant_idx, _, _) = Pattern::get_variant_info(variant_name, self.type_env);
                let payload_ty = scrutinee.ty.field_types(self.type_env)[variant_idx].clone();
                let mut payload = self.fresh_var("payload", payload_ty, pat.info.source.clone());
                // When the payload is bound whole to a source variable (e.g. `Some(x)`), that name is
                // its debug name; a destructuring sub-pattern names its leaves instead.
                if let Pattern::Var(v, _) = &subpat.pattern {
                    payload.debug_name = Some(v.name.to_string());
                }
                // Destructure the payload's subpattern, then lower the arm body under those bindings.
                let mut arm_stmts = vec![];
                let pushed = self.destructure_pattern(subpat, &payload, &mut arm_stmts);
                let ret_var = self.lower_to_var(body, &mut arm_stmts);
                for name in &pushed {
                    self.pop_env(name);
                }
                MatchArm {
                    variant: Some(variant_idx),
                    payload,
                    body: Self::build_let_chain(arm_stmts, Self::ret_node(ret_var)),
                }
            }
            Pattern::Var(v, _) => {
                // A catch-all arm binds the whole scrutinee to its source variable.
                let mut payload = self.fresh_var(&v.name.name, scrutinee.ty.clone(), pat.info.source.clone());
                payload.debug_name = Some(v.name.to_string());
                self.push_env(&v.name, payload.clone());
                let body = self.lower_body(body);
                self.pop_env(&v.name);
                MatchArm {
                    variant: None,
                    payload,
                    body,
                }
            }
            Pattern::Struct(_, _) => {
                // A struct/tuple pattern in a `match` is a single non-variant (default) arm that
                // binds the whole scrutinee and destructures it.
                let payload = self.fresh_var("scrut", scrutinee.ty.clone(), pat.info.source.clone());
                let mut arm_stmts = vec![];
                let pushed = self.destructure_pattern(pat, &payload, &mut arm_stmts);
                let ret_var = self.lower_to_var(body, &mut arm_stmts);
                for name in &pushed {
                    self.pop_env(name);
                }
                MatchArm {
                    variant: None,
                    payload,
                    body: Self::build_let_chain(arm_stmts, Self::ret_node(ret_var)),
                }
            }
        }
    }

    fn lower_make_struct(
        &mut self,
        fields: &[(Name, Option<Span>, Arc<ExprNode>)],
        ty: Arc<TypeNode>,
        source: Option<Span>,
        stmts: &mut Vec<Stmt>,
    ) -> RcVar {
        // The AST fields are already in declaration order.
        let field_vars: Vec<RcVar> = fields
            .iter()
            .map(|(_, _, e)| self.lower_to_var(e, stmts))
            .collect();
        let gen = LLVMGenerator::MakeStructBody(InlineLLVMMakeStructBody {
            field_names: field_vars.iter().map(|v| v.name.clone()).collect(),
        });
        let result = self.fresh_var("struct", ty, source.clone());
        stmts.push(Stmt::Let(result.clone(), RcRhs::Llvm(gen, field_vars), source));
        result
    }

    fn lower_array_lit(
        &mut self,
        elems: &[Arc<ExprNode>],
        ty: Arc<TypeNode>,
        source: Option<Span>,
        stmts: &mut Vec<Stmt>,
    ) -> RcVar {
        let elem_vars: Vec<RcVar> = elems.iter().map(|e| self.lower_to_var(e, stmts)).collect();
        let gen = LLVMGenerator::ArrayLitBody(InlineLLVMArrayLitBody {
            elem_names: elem_vars.iter().map(|v| v.name.clone()).collect(),
        });
        let result = self.fresh_var("array", ty, source.clone());
        stmts.push(Stmt::Let(result.clone(), RcRhs::Llvm(gen, elem_vars), source));
        result
    }

    fn lower_ffi_call(
        &mut self,
        fun_name: &Name,
        ret_tycon: &Arc<TyCon>,
        param_tycons: &[Arc<TyCon>],
        is_var_args: bool,
        args: &[Arc<ExprNode>],
        is_io: bool,
        ty: Arc<TypeNode>,
        source: Option<Span>,
        stmts: &mut Vec<Stmt>,
    ) -> RcVar {
        // All arguments are operands; when `is_io` the last is the input IOState token, kept for the
        // ordering dependency but not passed to C.
        let arg_vars: Vec<RcVar> = args.iter().map(|arg| self.lower_to_var(arg, stmts)).collect();
        let gen = LLVMGenerator::FFICallBody(InlineLLVMFFICallBody {
            fun_name: fun_name.clone(),
            ret_tycon: ret_tycon.clone(),
            param_tycons: param_tycons.to_vec(),
            is_var_args,
            is_io,
            arg_names: arg_vars.iter().map(|v| v.name.clone()).collect(),
        });
        let result = self.fresh_var("ffi", ty, source.clone());
        stmts.push(Stmt::Let(result.clone(), RcRhs::Llvm(gen, arg_vars), source));
        result
    }

    fn lower_eval(&mut self, side: &ExprNode, main: &ExprNode, stmts: &mut Vec<Stmt>) -> RcVar {
        // The side value is evaluated for its effect and discarded; RC insertion releases it.
        let _side = self.lower_to_var(side, stmts);
        self.lower_to_var(main, stmts)
    }

    // --- pattern destructuring ---

    /// Destructure `pat` against the value `obj`, binding each pattern variable in the environment
    /// (a struct/tuple pattern emits a `Destructure` statement extracting all its fields at once) and
    /// returning the AST names pushed, for the caller to pop after the scope closes.
    fn destructure_pattern(
        &mut self,
        pat: &PatternNode,
        obj: &RcVar,
        stmts: &mut Vec<Stmt>,
    ) -> Vec<FullName> {
        match &pat.pattern {
            Pattern::Var(v, _) => {
                // Bind the source variable to the value, carrying its source name for debug info.
                let named_here = name_definition(&v.name, obj, stmts);
                let is_own_binding = obj.debug_name == Some(v.name.to_string());
                if named_here || is_own_binding {
                    // The value is the fresh result just produced for this binding (named on its
                    // defining statement), or `obj` was already created as this variable's binding (a
                    // match-arm payload). Either way it carries the name; bind directly.
                    self.push_env(&v.name, obj.clone());
                } else {
                    // `obj` is a pre-existing variable (a rename such as `let j = i`, or a
                    // parameter), so it has no defining statement to name. Represent the rename
                    // faithfully with a move binding, which carries the source name. The move is
                    // reference-count-neutral: RC insertion retains `obj` before it exactly when it
                    // is used after, matching the current back end's `let j = i`.
                    let mut renamed = self.fresh_var(&v.name.name, obj.ty.clone(), pat.info.source.clone());
                    renamed.debug_name = Some(v.name.to_string());
                    stmts.push(Stmt::Let(renamed.clone(), RcRhs::Var(obj.clone()), pat.info.source.clone()));
                    self.push_env(&v.name, renamed);
                }
                vec![v.name.clone()]
            }
            Pattern::Struct(tc, field_pats) => {
                let field_tys = obj.ty.field_types(self.type_env);
                let mut fields = vec![]; // (field index, field variable) for the whole destructure
                let mut nested = vec![]; // (field variable, sub-pattern) lowered after the extraction
                let mut pushed = vec![];
                for (field_name, _, subpat) in field_pats {
                    let field_idx = self.struct_field_index(tc, field_name);
                    let hint = field_var_hint(subpat, field_name);
                    let mut field_var =
                        self.fresh_var(&hint, field_tys[field_idx].clone(), subpat.info.source.clone());
                    if let Pattern::Var(v, _) = &subpat.pattern {
                        // The field binds a source variable directly: carry its name for debug info
                        // and bind it. The field variable is always freshly produced by the
                        // destructure, so no rename move is needed (unlike the top-level `Var` case).
                        field_var.debug_name = Some(v.name.to_string());
                        self.push_env(&v.name, field_var.clone());
                        pushed.push(v.name.clone());
                    } else {
                        // A nested sub-pattern destructures this field further, after it is extracted.
                        nested.push((field_var.clone(), subpat));
                    }
                    fields.push((field_idx, field_var));
                }
                // Extract all fields in one step (mirroring the back end's `get_struct_fields`); RC
                // insertion retains the container beforehand iff it is used after this destructure.
                stmts.push(Stmt::Destructure(obj.clone(), fields, pat.info.source.clone()));
                for (field_var, subpat) in nested {
                    pushed.extend(self.destructure_pattern(subpat, &field_var, stmts));
                }
                pushed
            }
            Pattern::Union(_, _, _) => {
                panic!("a union pattern in a let-binding is not handled in RC IR lowering")
            }
        }
    }

    fn struct_field_index(&self, tc: &Arc<TyCon>, field_name: &str) -> usize {
        let ti = self
            .type_env
            .tycons
            .get(tc.as_ref())
            .expect("unknown type constructor in struct pattern");
        ti.fields
            .iter()
            .position(|f| f.name == *field_name)
            .expect("unknown field in struct pattern")
    }
}

/// Record `dbg` as the source-level debug name of the statement that just produced `var`, so code
/// generation can emit a debug local variable under it, and return whether it did. This applies only
/// when that statement is the immediately preceding one and is not already named — i.e. `var` is the
/// fresh result of a compound expression bound to a source `let`/pattern variable. It does not apply
/// to an alias of a pre-existing variable (a rename, a global, or a parameter), which has no such
/// statement.
fn name_definition(dbg: &FullName, var: &RcVar, stmts: &mut Vec<Stmt>) -> bool {
    if let Some(Stmt::Let(bound, _, _)) = stmts.last_mut() {
        if bound.name == var.name && bound.debug_name.is_none() {
            bound.debug_name = Some(dbg.to_string());
            return true;
        }
    }
    false
}

/// A readable name hint for a field variable: the sub-pattern's variable name, or the field name.
fn field_var_hint(subpat: &PatternNode, field_name: &str) -> String {
    match &subpat.pattern {
        Pattern::Var(v, _) => v.name.name.clone(),
        _ => field_name.to_string(),
    }
}
