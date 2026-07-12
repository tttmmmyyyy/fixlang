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
enum Binding {
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
    // `entry` labels the program in the dump only; it has no role in code generation or in
    // entry-point selection. The actual entry point — `main` for a build, `test` for `fix test`,
    // or an FFI-exported function — is chosen by the build driver, independently of this field.
    // It is a placeholder, NOT a reachability root: RC-IR dead-function elimination, when added,
    // must take its roots from the real entry points (there can be several — every FFI-exported
    // function is one), not from this field.
    let entry = FuncRef {
        name: FullName::local("#entry"),
    };
    RcProgram {
        funcs,
        globals,
        entry,
    }
}

/// The lowering context: a fresh-name counter, the accumulated top-level functions, and the scope
/// mapping each AST local name to the RC IR variable currently bound to it. Because a fresh
/// globally-unique name is minted at every binding, the scope resolves shadowing and the resulting
/// names need no scope tracking downstream.
struct Lowerer<'a> {
    type_env: &'a TypeEnv,
    fresh_counter: u64,
    funcs: Map<FuncRef, RcFunc>,
    // A shadow stack per AST name; the last entry is the current binding.
    scope: Map<FullName, Vec<RcVar>>,
    // The type of each top-level symbol, to type a global referenced as an LLVM operand.
    symbol_types: Map<FullName, Arc<TypeNode>>,
}

impl<'a> Lowerer<'a> {
    fn new(type_env: &'a TypeEnv) -> Self {
        Lowerer {
            type_env,
            fresh_counter: 0,
            funcs: Map::default(),
            scope: Map::default(),
            symbol_types: Map::default(),
        }
    }

    // --- fresh names ---

    fn fresh_var(&mut self, hint: &str, ty: Arc<TypeNode>, source: Option<Span>) -> RcVar {
        self.fresh_counter += 1;
        let name = FullName::local(&format!("{}#{}", hint, self.fresh_counter));
        RcVar {
            name,
            ty,
            source,
            debug_name: None,
        }
    }

    fn fresh_func(&mut self, hint: &str) -> FuncRef {
        self.fresh_counter += 1;
        FuncRef {
            name: FullName::local(&format!("{}#{}", hint, self.fresh_counter)),
        }
    }

    // --- environment ---

    fn bind(&mut self, ast_name: &FullName, var: RcVar) {
        self.scope.entry(ast_name.clone()).or_insert_with(Vec::new).push(var);
    }

    fn unbind(&mut self, ast_name: &FullName) {
        if let Some(stack) = self.scope.get_mut(ast_name) {
            stack.pop();
        }
    }

    fn resolve(&self, ast_name: &FullName) -> Option<RcVar> {
        self.scope.get(ast_name).and_then(|stack| stack.last()).cloned()
    }

    // --- building the continuation-nested body ---

    /// Fold accumulated bindings into the nested continuation chain ending in `terminal`.
    fn fold_bindings(bindings: Vec<Binding>, terminal: RcExprNode) -> RcExprNode {
        bindings.into_iter().rev().fold(terminal, |cont, binding| match binding {
            Binding::Let(var, rhs, source) => RcExprNode {
                expr: Box::new(RcExpr::Let(var, rhs, cont)),
                source,
            },
            Binding::Destructure(container, fields, source) => RcExprNode {
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

    /// Lower an expression as a complete body: its bindings followed by a `Ret` of its value.
    fn lower_body(&mut self, expr: &ExprNode) -> RcExprNode {
        let mut bindings = vec![];
        let v = self.lower_to_var(expr, &mut bindings);
        Self::fold_bindings(bindings, Self::ret_node(v))
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

        let saved_env = std::mem::take(&mut self.scope);

        let mut param_vars = vec![];
        for (p, ty) in params.iter().zip(src_tys.iter()) {
            let pv = self.fresh_var(&p.name.name, ty.clone(), None);
            self.bind(&p.name, pv.clone());
            param_vars.push(pv);
        }

        let mut bindings = vec![];
        let cap = if lam_ty.is_closure() {
            let cap_var = self.fresh_var("cap", make_dynamic_object_ty(), None);
            // Bind the capture object under the implicit name `#CAP` too, so a built-in that reads the
            // raw capture object by that name (the `fix` combinator's `FixBody`) resolves to it.
            self.bind(&FullName::local(CAP_NAME), cap_var.clone());
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
                bindings.push(Binding::Let(proj.clone(), RcRhs::Llvm(gen, vec![cap_var.clone()]), None));
                self.bind(ast_name, proj);
            }
            Some(cap_var)
        } else {
            assert!(captures.is_empty(), "a funptr function cannot have captures");
            None
        };

        let ret_var = self.lower_to_var(&body, &mut bindings);
        let body_expr = Self::fold_bindings(bindings, Self::ret_node(ret_var));

        self.scope = saved_env;

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

    // --- expressions (A-normalization: lower to an atom, appending bindings) ---

    fn lower_to_var(&mut self, expr: &ExprNode, bindings: &mut Vec<Binding>) -> RcVar {
        // A deeply nested expression recurses deeply here (as it does in RC insertion and code
        // generation); grow the stack on demand so a large program does not overflow it.
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || self.lower_to_var_inner(expr, bindings))
    }

    fn lower_to_var_inner(&mut self, expr: &ExprNode, bindings: &mut Vec<Binding>) -> RcVar {
        let ty = expr.type_.clone().unwrap();
        let source = expr.source.clone();
        match expr.expr.as_ref() {
            Expr::Var(v) => self.lower_var(v, &ty, &source),
            Expr::LLVM(inline) => self.lower_llvm(inline, ty, source, bindings),
            Expr::App(fun, args) => self.lower_app(fun, args, ty, source, bindings),
            Expr::Lam(_, _) => self.lower_lam(expr, ty, source, bindings),
            Expr::Let(pat, bound, val) => self.lower_let(pat, bound, val, bindings),
            Expr::If(c, t, e) => self.lower_if(c, t, e, ty, source, bindings),
            Expr::Match(cond, arms) => self.lower_match(cond, arms, ty, source, bindings),
            Expr::TyAnno(e, _) => self.lower_to_var(e, bindings),
            Expr::MakeStruct(_, fields) => self.lower_make_struct(fields, ty, source, bindings),
            Expr::ArrayLit(elems) => self.lower_array_lit(elems, ty, source, bindings),
            Expr::FFICall(fun_name, ret_ty, param_tys, is_va, args, is_io) => self.lower_ffi_call(
                fun_name, ret_ty, param_tys, *is_va, args, *is_io, ty, source, bindings,
            ),
            Expr::Eval(side, main) => self.lower_eval(side, main, bindings),
        }
    }

    fn lower_var(&mut self, v: &Arc<Var>, ty: &Arc<TypeNode>, source: &Option<Span>) -> RcVar {
        match self.resolve(&v.name) {
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
        bindings: &mut Vec<Binding>,
    ) -> RcVar {
        let mut gen = inline.generator.clone();
        // The generator's free variables are its operands, in a fixed order. A local operand reuses
        // the variable already bound to it; an operand that is not a local is a reference to a global
        // value or function, materialized by code generation from its (unchanged) name.
        let operand_vars: Vec<RcVar> = gen
            .free_vars()
            .iter()
            .map(|name| match self.resolve(name) {
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
        bindings.push(Binding::Let(result.clone(), RcRhs::Llvm(gen, operand_vars), source));
        result
    }

    fn lower_app(
        &mut self,
        fun: &ExprNode,
        args: &[Arc<ExprNode>],
        ty: Arc<TypeNode>,
        source: Option<Span>,
        bindings: &mut Vec<Binding>,
    ) -> RcVar {
        // Evaluation order (matching the current generator): the callee first, then the arguments
        // left to right.
        let callee = self.lower_to_var(fun, bindings);
        let arg_vars: Vec<RcVar> = args.iter().map(|arg| self.lower_to_var(arg, bindings)).collect();
        let result = self.fresh_var("app", ty, source.clone());
        bindings.push(Binding::Let(result.clone(), RcRhs::App(callee, arg_vars), source));
        result
    }

    fn lower_lam(
        &mut self,
        expr: &ExprNode,
        ty: Arc<TypeNode>,
        source: Option<Span>,
        bindings: &mut Vec<Binding>,
    ) -> RcVar {
        // Resolve the captured values from the enclosing scope, in the closure's storage order.
        let cap_names = expr.lambda_cap_names();
        let cap_vals: Vec<RcVar> = cap_names
            .iter()
            .map(|n| self.resolve(n).expect("captured variable not bound"))
            .collect();
        let captures: Vec<(FullName, RcVar)> =
            cap_names.iter().cloned().zip(cap_vals.iter().cloned()).collect();

        let func_ref = self.fresh_func("lambda");
        let rc_func = self.lower_lambda_as_function(expr, func_ref.clone(), captures);
        self.funcs.insert(func_ref.clone(), rc_func);

        let result = self.fresh_var("closure", ty, source.clone());
        bindings.push(Binding::Let(result.clone(), RcRhs::Closure(func_ref, cap_vals), source));
        result
    }

    fn lower_let(
        &mut self,
        pat: &PatternNode,
        bound: &ExprNode,
        val: &ExprNode,
        bindings: &mut Vec<Binding>,
    ) -> RcVar {
        let bound_var = self.lower_to_var(bound, bindings);
        let pushed = self.destructure_pattern(pat, &bound_var, bindings);
        let result = self.lower_to_var(val, bindings);
        for name in &pushed {
            self.unbind(name);
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
        bindings: &mut Vec<Binding>,
    ) -> RcVar {
        // Desugar to a match on the Bool union: `_true` (tag 1) -> then, `_false` (tag 0) -> else.
        let cond_var = self.lower_to_var(cond, bindings);
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
        bindings.push(Binding::Let(result.clone(), RcRhs::Match(cond_var, vec![then_arm, else_arm]), source));
        result
    }

    fn lower_match(
        &mut self,
        cond: &ExprNode,
        arms: &[(Arc<PatternNode>, Arc<ExprNode>)],
        ty: Arc<TypeNode>,
        source: Option<Span>,
        bindings: &mut Vec<Binding>,
    ) -> RcVar {
        let cond_var = self.lower_to_var(cond, bindings);
        let rc_arms: Vec<MatchArm> = arms
            .iter()
            .map(|(pat, body)| self.lower_match_arm(&cond_var, pat, body))
            .collect();
        let result = self.fresh_var("match", ty, source.clone());
        bindings.push(Binding::Let(result.clone(), RcRhs::Match(cond_var, rc_arms), source));
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
                let mut arm_bindings = vec![];
                let pushed = self.destructure_pattern(subpat, &payload, &mut arm_bindings);
                let ret_var = self.lower_to_var(body, &mut arm_bindings);
                for name in &pushed {
                    self.unbind(name);
                }
                MatchArm {
                    variant: Some(variant_idx),
                    payload,
                    body: Self::fold_bindings(arm_bindings, Self::ret_node(ret_var)),
                }
            }
            Pattern::Var(v, _) => {
                // A catch-all arm binds the whole scrutinee to its source variable.
                let mut payload = self.fresh_var(&v.name.name, scrutinee.ty.clone(), pat.info.source.clone());
                payload.debug_name = Some(v.name.to_string());
                self.bind(&v.name, payload.clone());
                let body = self.lower_body(body);
                self.unbind(&v.name);
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
                let mut arm_bindings = vec![];
                let pushed = self.destructure_pattern(pat, &payload, &mut arm_bindings);
                let ret_var = self.lower_to_var(body, &mut arm_bindings);
                for name in &pushed {
                    self.unbind(name);
                }
                MatchArm {
                    variant: None,
                    payload,
                    body: Self::fold_bindings(arm_bindings, Self::ret_node(ret_var)),
                }
            }
        }
    }

    fn lower_make_struct(
        &mut self,
        fields: &[(Name, Option<Span>, Arc<ExprNode>)],
        ty: Arc<TypeNode>,
        source: Option<Span>,
        bindings: &mut Vec<Binding>,
    ) -> RcVar {
        // The AST fields are already in declaration order.
        let field_vars: Vec<RcVar> = fields
            .iter()
            .map(|(_, _, e)| self.lower_to_var(e, bindings))
            .collect();
        let gen = LLVMGenerator::MakeStructBody(InlineLLVMMakeStructBody {
            field_names: field_vars.iter().map(|v| v.name.clone()).collect(),
        });
        let result = self.fresh_var("struct", ty, source.clone());
        bindings.push(Binding::Let(result.clone(), RcRhs::Llvm(gen, field_vars), source));
        result
    }

    fn lower_array_lit(
        &mut self,
        elems: &[Arc<ExprNode>],
        ty: Arc<TypeNode>,
        source: Option<Span>,
        bindings: &mut Vec<Binding>,
    ) -> RcVar {
        let elem_vars: Vec<RcVar> = elems.iter().map(|e| self.lower_to_var(e, bindings)).collect();
        let gen = LLVMGenerator::ArrayLitBody(InlineLLVMArrayLitBody {
            elem_names: elem_vars.iter().map(|v| v.name.clone()).collect(),
        });
        let result = self.fresh_var("array", ty, source.clone());
        bindings.push(Binding::Let(result.clone(), RcRhs::Llvm(gen, elem_vars), source));
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
        bindings: &mut Vec<Binding>,
    ) -> RcVar {
        // All arguments are operands; when `is_io` the last is the input IOState token, kept for the
        // ordering dependency but not passed to C.
        let arg_vars: Vec<RcVar> = args.iter().map(|arg| self.lower_to_var(arg, bindings)).collect();
        let gen = LLVMGenerator::FFICallBody(InlineLLVMFFICallBody {
            fun_name: fun_name.clone(),
            ret_tycon: ret_tycon.clone(),
            param_tycons: param_tycons.to_vec(),
            is_var_args,
            is_io,
            arg_names: arg_vars.iter().map(|v| v.name.clone()).collect(),
        });
        let result = self.fresh_var("ffi", ty, source.clone());
        bindings.push(Binding::Let(result.clone(), RcRhs::Llvm(gen, arg_vars), source));
        result
    }

    fn lower_eval(&mut self, side: &ExprNode, main: &ExprNode, bindings: &mut Vec<Binding>) -> RcVar {
        // The side value is evaluated for its effect and discarded. A compound side is forced by the
        // bindings it emits, and a local variable is already evaluated; but a reference to a global
        // value lowers to a bare atom that emits nothing, so materialize it here to force the global's
        // initializer to run (which may have effects — e.g. an `undefined`-valued global).
        let side_var = self.lower_to_var(side, bindings);
        if !side_var.name.is_local() {
            let forced = self.fresh_var("eval", side_var.ty.clone(), None);
            bindings.push(Binding::Let(forced, RcRhs::Var(side_var), None));
        }
        self.lower_to_var(main, bindings)
    }

    // --- pattern destructuring ---

    /// Destructure `pat` against the value `obj`, binding each pattern variable in the environment
    /// (a struct/tuple pattern emits a `Destructure` binding extracting all its fields at once) and
    /// returning the AST names pushed, for the caller to pop after the scope closes.
    fn destructure_pattern(
        &mut self,
        pat: &PatternNode,
        obj: &RcVar,
        bindings: &mut Vec<Binding>,
    ) -> Vec<FullName> {
        match &pat.pattern {
            Pattern::Var(v, _) => {
                // Bind the source variable to the value, carrying its source name for debug info.
                let debug_named = try_attach_debug_name(&v.name, obj, bindings);
                let already_debug_named = obj.debug_name == Some(v.name.to_string());
                if debug_named || already_debug_named {
                    // The value is the fresh result just produced for this binding (named on its
                    // defining binding), or `obj` was already created as this variable's binding (a
                    // match-arm payload). Either way it carries the name; bind directly.
                    self.bind(&v.name, obj.clone());
                } else {
                    // `obj` is a pre-existing variable (a rename such as `let j = i`, or a
                    // parameter), so it has no defining binding to name. Represent the rename
                    // faithfully with a move binding, which carries the source name. The move is
                    // reference-count-neutral: RC insertion retains `obj` before it exactly when it
                    // is used after, matching the current back end's `let j = i`.
                    let mut renamed = self.fresh_var(&v.name.name, obj.ty.clone(), pat.info.source.clone());
                    renamed.debug_name = Some(v.name.to_string());
                    bindings.push(Binding::Let(renamed.clone(), RcRhs::Var(obj.clone()), pat.info.source.clone()));
                    self.bind(&v.name, renamed);
                }
                vec![v.name.clone()]
            }
            Pattern::Struct(_tc, field_pats) => {
                let field_tys = obj.ty.field_types(self.type_env);
                let mut fields = vec![]; // (field index, field variable) for the whole destructure
                let mut nested = vec![]; // (field variable, sub-pattern) lowered after the extraction
                let mut pushed = vec![];
                for (field_name, _, subpat) in field_pats {
                    let field_idx = obj
                        .ty
                        .field_index(self.type_env, field_name)
                        .expect("unknown field in struct pattern");
                    let hint = field_var_hint(subpat, field_name);
                    let mut field_var =
                        self.fresh_var(&hint, field_tys[field_idx].clone(), subpat.info.source.clone());
                    if let Pattern::Var(v, _) = &subpat.pattern {
                        // The field binds a source variable directly: carry its name for debug info
                        // and bind it. The field variable is always freshly produced by the
                        // destructure, so no rename move is needed (unlike the top-level `Var` case).
                        field_var.debug_name = Some(v.name.to_string());
                        self.bind(&v.name, field_var.clone());
                        pushed.push(v.name.clone());
                    } else {
                        // A nested sub-pattern destructures this field further, after it is extracted.
                        nested.push((field_var.clone(), subpat));
                    }
                    fields.push((field_idx, field_var));
                }
                // Extract all fields in one step (mirroring the back end's `get_struct_fields`); RC
                // insertion retains the container beforehand iff it is used after this destructure.
                bindings.push(Binding::Destructure(obj.clone(), fields, pat.info.source.clone()));
                for (field_var, subpat) in nested {
                    pushed.extend(self.destructure_pattern(subpat, &field_var, bindings));
                }
                pushed
            }
            Pattern::Union(_, _, _) => {
                panic!("a union pattern in a let-binding is not handled in RC IR lowering")
            }
        }
    }

}

/// Record `dbg` as the source-level debug name of the binding that just produced `var`, so code
/// generation can emit a debug local variable under it, and return whether it did. This applies only
/// when that binding is the immediately preceding one and is not already named — i.e. `var` is the
/// fresh result of a compound expression bound to a source `let`/pattern variable. It does not apply
/// to an alias of a pre-existing variable (a rename, a global, or a parameter), which has no such
/// binding.
fn try_attach_debug_name(dbg: &FullName, var: &RcVar, bindings: &mut Vec<Binding>) -> bool {
    if let Some(Binding::Let(bound, _, _)) = bindings.last_mut() {
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
