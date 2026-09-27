/*
# Moving captures into an environment operand

An inline-LLVM op that declares `LLVMGen::env_operand` applies one of its operands, a function, to a
tuple whose first field is another of its operands, the environment. A lambda given to such an op
needs no closure object for what it captures: the captured values can travel in the environment
instead. This pass moves them there, so that the lambda captures nothing and building it allocates
nothing.

```
let f = |p| e;              // `e` reads the locals `x_1`, ..., `x_n`
...
op(f, env, ...)
```

becomes

```
let f = |q| (
    let ((env0, Cap { x_1 : x_1, ..., x_n : x_n }), r_1, ..., r_k) = q;
    let p = (env0, r_1, ..., r_k);
    e
);
let cap = Cap { x_1 : x_1, ..., x_n : x_n };
...
let env1 = (env, cap);
op(f, env1, ...)
```

where `p` is the tuple `(env, r_1, ..., r_k)` the op applies `f` to, and `Cap` is an unboxed struct
of the captured values. The captured values are read where `f` is bound, which is where the lambda
read them.

A lambda is rewritten when it is bound to a name by `let` and every use of the name is the function
operand of such an op. Rewriting the lambda changes its type, which a use anywhere else would not
accept.

The bindings are rewritten outermost first. Rewriting one gives the ops using its name the name of
its capture struct, and a lambda nested in its scope that holds such an op captures that name; it is
rewritten afterwards, with that capture among the others.

## Relations to other optimizations

* Inlining puts the lambda a caller writes and the op of the function it calls into one expression,
  which is what this pass needs to see them together.
* It runs above `collapse_constructions`, which reads the tuple `p` built here into the pattern that
  takes it apart in `e`.
* `closure_specialization` lifts the lambda this pass leaves to a global function, and the empty
  capture list the lambda now has costs no allocation.
*/

use super::{
    capture_struct::{captured_fields, CaptureStruct},
    find_usage_of_name::{self, UsageType},
};
use crate::{
    ast::{
        expr::{expr_abs_typed, expr_let_typed, expr_make_struct, expr_var, var_local, ExprNode},
        inline_llvm::InlineLLVM,
        name::FullName,
        pattern::PatternNode,
        program::{Program, TypeEnv},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
        types::{tycon, TyCon, TyConInfo, TypeNode},
    },
    constants::CAPTURE_INTO_ENV_PREFIX,
    fixstd::builtin::{make_tuple_name_abs, make_tuple_ty},
    misc::{Map, Set},
};
use std::sync::Arc;

/// Moves the captures of every lambda given to an op declaring `LLVMGen::env_operand` into the op's
/// environment operand, where the lambda is bound by `let` and used nowhere else.
pub fn run(prg: &mut Program) {
    let mut new_tycons: Map<TyCon, TyConInfo> = Map::default();
    for (name, sym) in &mut prg.symbols {
        let expr = sym.expr.as_ref().unwrap();
        let mut collector = EnvFunctionCollector {
            names: Set::default(),
        };
        collector.traverse(expr);
        if collector.names.is_empty() {
            continue;
        }
        let mut mover = CaptureMover {
            symbol: name.clone(),
            env_functions: collector.names,
            type_env: &prg.type_env,
            counter: 0,
            new_tycons: &mut new_tycons,
        };
        let res = mover.traverse(expr);
        if res.changed {
            sym.expr = Some(res.expr);
        }
    }
    prg.type_env.add_tycons(new_tycons);
}

/// The name an op declaring `LLVMGen::env_operand` applies as a function, if `llvm` is such an op.
fn env_function_operand(llvm: &InlineLLVM) -> Option<FullName> {
    let env = llvm.generator.env_operand()?;
    Some(llvm.generator.free_vars()[env.function].clone())
}

/// Collects the names an expression passes as the function operand of an op declaring
/// `LLVMGen::env_operand`, wherever they are bound. Only a lambda bound to one of these names can be
/// rewritten, so `CaptureMover` checks the uses of these names alone.
struct EnvFunctionCollector {
    /// The names collected so far.
    names: Set<FullName>,
}

impl ExprVisitor for EnvFunctionCollector {
    /// Collects the function operand of an op declaring `LLVMGen::env_operand`.
    fn start_visit_llvm(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if let Some(name) = env_function_operand(&expr.get_llvm()) {
            self.names.insert(name);
        }
        StartVisitResult::VisitChildren
    }

    // The rest of the expression kinds are passed through: their children are visited, and the
    // expression itself is left as it is.

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
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

    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
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

/// Rewrites the lambdas of one symbol, as the module comment describes.
struct CaptureMover<'a> {
    /// The symbol being rewritten, which, with the counter, names the capture structs built for it.
    symbol: FullName,
    /// The names the symbol passes as the function operand of an op declaring
    /// `LLVMGen::env_operand`.
    env_functions: Set<FullName>,
    /// The type environment, which knows the fields of the tuples the ops apply their functions to.
    type_env: &'a TypeEnv,
    /// The number of lambdas rewritten so far, which tells apart the names each rewrite introduces.
    counter: u32,
    /// The capture structs built so far, to be registered in the type environment.
    new_tycons: &'a mut Map<TyCon, TyConInfo>,
}

impl CaptureMover<'_> {
    /// `expr` rewritten as the module comment describes, where it is `let v = lambda; body`, the
    /// lambda captures something, and every use of `v` in `body` is the function operand of an op
    /// declaring `LLVMGen::env_operand`.
    fn rewrite(&mut self, expr: &Arc<ExprNode>, state: &VisitState) -> Option<Arc<ExprNode>> {
        let pat = expr.get_let_pat();
        if !pat.is_var() {
            return None;
        }
        let name = pat.get_var().name.clone();
        if !self.env_functions.contains(&name) {
            return None;
        }
        let lam = expr.get_let_bound();
        if !lam.is_lam() {
            return None;
        }
        // The captured variables, with the types they have where the lambda is bound.
        let fields = captured_fields(&lam, state);
        if fields.is_empty() {
            return None;
        }
        let body = expr.get_let_value();
        if !used_only_as_env_function(&body, &name) {
            return None;
        }

        let id = self.counter;
        let local =
            |suffix: &str| FullName::local(&format!("{}{}{}", CAPTURE_INTO_ENV_PREFIX, id, suffix));

        let cap = CaptureStruct::new(
            &format!("{}{}", CAPTURE_INTO_ENV_PREFIX, id),
            &self.symbol,
            &fields,
        );
        let cap_name = local("");

        // The lambda is applied to the tuple `(env, r_1, ..., r_k)`, as the ops it is given to
        // declare, where `r_1`, ..., `r_k` are the arguments besides the environment. The ops are
        // given `(env, cap)` in place of `env`.
        let param_ty = lam.type_.as_ref().unwrap().get_lambda_srcs()[0].clone();
        let param_field_tys = param_ty.field_types(self.type_env);
        let env_ty = param_field_tys[0].clone();
        let new_env_ty = make_tuple_ty(vec![env_ty.clone(), cap.ty.clone()]);
        let mut op_rewriter = OpRewriter {
            function: &name,
            cap: var_expr(&cap_name, &cap.ty),
            new_env: local("_op_env"),
            new_env_ty: new_env_ty.clone(),
            env_ty: env_ty.clone(),
        };
        let body = op_rewriter.traverse(&body).expr;
        self.counter += 1;
        self.new_tycons
            .insert(cap.tycon.as_ref().clone(), cap.tycon_info.clone());

        // `|q| (let ((env0, Cap { .. }), r_1, ..., r_k) = q; let p = (env0, r_1, ..., r_k); e)`
        let mut new_param_field_tys = param_field_tys.clone();
        new_param_field_tys[0] = new_env_ty.clone();
        let new_param_ty = make_tuple_ty(new_param_field_tys);
        let param_tycon = tycon(make_tuple_name_abs(param_field_tys.len() as u32));
        let env0 = local("_env");
        let env_pat = PatternNode::make_struct(
            tycon(make_tuple_name_abs(2)),
            vec![
                ("0".to_string(), var_pattern(&env0, &env_ty)),
                ("1".to_string(), cap.pattern()),
            ],
        )
        .set_type(new_env_ty);
        let mut param_field_pats = vec![("0".to_string(), env_pat)];
        let mut param_field_exprs = vec![("0".to_string(), var_expr(&env0, &env_ty))];
        for (i, ty) in param_field_tys.iter().enumerate().skip(1) {
            let field_name = local(&format!("_{}", i));
            param_field_pats.push((i.to_string(), var_pattern(&field_name, ty)));
            param_field_exprs.push((i.to_string(), var_expr(&field_name, ty)));
        }
        let new_param = local("_arg");
        let old_params = lam.get_lam_params();
        assert_eq!(
            old_params.len(),
            1,
            "a function given to an op declaring `env_operand` takes one tuple: {}",
            name.to_string()
        );
        let old_param = old_params[0].name.clone();
        let new_lam = expr_abs_typed(
            var_local(&new_param.name),
            new_param_ty.clone(),
            expr_let_typed(
                PatternNode::make_struct(param_tycon.clone(), param_field_pats)
                    .set_type(new_param_ty.clone()),
                var_expr(&new_param, &new_param_ty),
                expr_let_typed(
                    var_pattern(&old_param, &param_ty),
                    expr_make_struct(param_tycon, param_field_exprs).set_type(param_ty.clone()),
                    lam.get_lam_body(),
                ),
            ),
        );

        // The walk revisits the binding it rewrote, and ends there because the new lambda captures
        // nothing.
        assert!(
            new_lam.lambda_cap_names().is_empty(),
            "the lambda bound to `{}` still captures {:?} once its captures are moved into the \
             environment",
            name.to_string(),
            new_lam
                .lambda_cap_names()
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<_>>()
        );

        // `let v = |q| ..; let cap = Cap { .. }; body`
        let body = expr_let_typed(var_pattern(&cap_name, &cap.ty), cap.struct_expr(), body);
        let new_lam_ty = new_lam.type_.as_ref().unwrap().clone();
        Some(expr_let_typed(pat.set_type(new_lam_ty), new_lam, body))
    }
}

/// A pattern binding the local `name` at type `ty`.
fn var_pattern(name: &FullName, ty: &Arc<TypeNode>) -> Arc<PatternNode> {
    PatternNode::make_var(var_local(&name.name), None).set_type(ty.clone())
}

/// The name `name` read at type `ty`.
fn var_expr(name: &FullName, ty: &Arc<TypeNode>) -> Arc<ExprNode> {
    expr_var(name.clone(), None).set_type(ty.clone())
}

impl ExprVisitor for CaptureMover<'_> {
    /// Rewrites a `let` binding a lambda, and revisits the result so that the lambdas nested in it
    /// are rewritten with the name of the new capture struct among their captures.
    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        state: &mut VisitState,
    ) -> StartVisitResult {
        match self.rewrite(expr, state) {
            Some(expr) => StartVisitResult::ReplaceAndRevisit(expr),
            None => StartVisitResult::VisitChildren,
        }
    }

    // The rest of the expression kinds are passed through: their children are visited, and the
    // expression itself is left as it is.

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
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

    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
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

/// Whether every use of `name` in `expr` is the function operand of an op declaring
/// `LLVMGen::env_operand`, and there is one.
fn used_only_as_env_function(expr: &Arc<ExprNode>, name: &FullName) -> bool {
    let usages = find_usage_of_name::run(expr, name);
    !usages.is_empty()
        && usages
            .iter()
            .all(|usage| matches!(usage, UsageType::EnvFunctionOperand))
}

/// Whether `name` is bound again inside the expression a walk started from, so that an occurrence
/// of the name here refers to that inner binding.
fn shadowed(name: &FullName, state: &VisitState) -> bool {
    state.scope.has_value(&name.name)
}

/// Gives each op that applies one lambda as its function the environment `(env, cap)` in place of
/// `env`.
struct OpRewriter<'a> {
    /// The name the lambda is bound to.
    function: &'a FullName,
    /// The capture struct of the lambda, read by name.
    cap: Arc<ExprNode>,
    /// The name the new environment is bound to ahead of an op.
    new_env: FullName,
    /// The type of the new environment, `(env, cap)`.
    new_env_ty: Arc<TypeNode>,
    /// The type of the environment an op is given.
    env_ty: Arc<TypeNode>,
}

impl ExprVisitor for OpRewriter<'_> {
    /// Binds `(env, cap)` ahead of an op applying the lambda, and gives the op that name as its
    /// environment.
    fn start_visit_llvm(
        &mut self,
        expr: &Arc<ExprNode>,
        state: &mut VisitState,
    ) -> StartVisitResult {
        if env_function_operand(&expr.get_llvm()).as_ref() != Some(self.function)
            || shadowed(self.function, state)
        {
            return StartVisitResult::VisitChildren;
        }

        // `let env1 = (env, cap); op(.., env1, ..)`
        let mut llvm = expr.get_llvm().as_ref().clone();
        let env_index = llvm.generator.env_operand().unwrap().env;
        let mut operands = llvm.generator.free_vars_mut();
        let env = &mut *operands[env_index];
        let old_env = env.clone();
        *env = self.new_env.clone();
        let env_and_cap = expr_make_struct(
            tycon(make_tuple_name_abs(2)),
            vec![
                ("0".to_string(), var_expr(&old_env, &self.env_ty)),
                ("1".to_string(), self.cap.clone()),
            ],
        )
        .set_type(self.new_env_ty.clone());
        StartVisitResult::ReplaceAndReturn(expr_let_typed(
            var_pattern(&self.new_env, &self.new_env_ty),
            env_and_cap,
            expr.set_llvm(llvm),
        ))
    }

    // The rest of the expression kinds are passed through: their children are visited, and the
    // expression itself is left as it is.

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
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

    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
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
