/*
# Decapturing the functions given to scope builtins

The scope builtins — `Std::with_retained`, `Std::Array::borrow_elements`, `Std::FFI::borrow_boxed`
and the mutating ones behind `mutate_elements` and `mutate_boxed` — are builtin ops that declare
`BuiltinOp::env_operand`. Such an op applies one of its operands, a function, to a tuple whose first
field is another of its operands, the environment.

`closure_specialization` lifts every lambda to a global function taking its capture list as the
first argument, so the function such an op is given is that global function applied to a capture
list: a closure holding the capture list, built on the heap on every call. This pass moves the
capture list into the environment instead, so that the function the op is given captures nothing.

```
let h = F(cap);
...
op(.., h, env, ..)
```

becomes

```
let c = cap;
let h = |q| (
    let ((env0, c0), r_1, ..., r_k) = q;
    F(c0, (env0, r_1, ..., r_k))
);
...
let env1 = (env, c);
op(.., h, env1, ..)
```

where `(env, r_1, ..., r_k)` is the tuple the op applies `h` to, and `F` is a global function.

A binding is rewritten when the name it binds is used once, as the function operand of such an op.
Rewriting the binding changes the type of the name, which a use anywhere else would not accept. The
op applies its function once, so `F(cap)`, which the binding evaluated once, is evaluated once inside
`h` as well; a second op applying `h` would evaluate it a second time.

Where the symbol holding the op is the only one naming `F`, and it names `F` in this call alone, the
body of `F` is put in place of the call (`move_bodies_into_callers`). That is the common case, since
`F` is the lambda the op was given, lifted: the body then takes apart the tuple it is applied to right
where it is built, and `collapse_constructions`, run after this pass, reads the one into the other.

## Relations to other optimizations

* It runs after `closure_specialization`, which is what turns the function every such op is given
  into a global function applied to its capture list. Running after it also leaves a lambda given to
  such an op capturing what it captures while specialization reads it, so a function whose parameter
  the lambda calls is specialized on the closure that parameter receives, as it would be for any
  other lambda calling it.
* It runs above `collapse_constructions`, which reads the tuple built for `F` into the pattern that
  takes it apart in `F`'s body.
*/

use super::{
    find_usage_of_name::{self, UsageType},
    move_bodies::move_bodies_into_callers,
};
use crate::{
    ast::{
        builtin_op::BuiltinOpExpr,
        expr::{
            expr_abs_typed, expr_app_typed, expr_let_typed, expr_make_struct, expr_var, var_local,
            ExprNode,
        },
        name::FullName,
        pattern::PatternNode,
        program::{Program, TypeEnv},
        traverse::{ExprVisitor, StartVisitResult, VisitState},
        types::{tycon, TypeNode},
    },
    constants::DECAPTURE_PREFIX,
    fixstd::builtin::{make_tuple_name_abs, make_tuple_ty},
    misc::{Map, Set},
};
use std::sync::Arc;

/// Moves the capture list of every function given to an op declaring `BuiltinOp::env_operand` into the
/// op's environment operand, where the function is a global function applied to its capture list,
/// bound by `let` and used nowhere else.
pub fn run(prg: &mut Program) {
    // For each symbol, the global functions the functions it gives to ops now call.
    let mut callees_by_caller: Map<FullName, Vec<FullName>> = Map::default();
    for (name, sym) in &mut prg.symbols {
        let expr = sym.expr.as_ref().unwrap();
        let mut collector = EnvFunctionCollector {
            names: Set::default(),
        };
        collector.traverse(expr);
        if collector.names.is_empty() {
            continue;
        }
        let mut mover = CaptureListMover {
            env_functions: collector.names,
            type_env: &prg.type_env,
            counter: 0,
            callees: Vec::new(),
        };
        let res = mover.traverse(expr);
        if res.changed {
            sym.expr = Some(res.expr);
            callees_by_caller.insert(name.clone(), mover.callees);
        }
    }
    move_bodies_into_callers(&mut prg.symbols, &callees_by_caller);
}

/// The name an op declaring `BuiltinOp::env_operand` applies as a function, if `builtin` is such an
/// op.
fn env_function_operand(builtin: &BuiltinOpExpr) -> Option<FullName> {
    let env = builtin.op.env_operand()?;
    Some(builtin.op.free_vars()[env.function].clone())
}

/// Collects the names an expression passes as the function operand of an op declaring
/// `BuiltinOp::env_operand`, wherever they are bound. Only a binding of one of these names can be
/// rewritten, so `CaptureListMover` checks the uses of these names alone.
struct EnvFunctionCollector {
    /// The names collected so far.
    names: Set<FullName>,
}

impl ExprVisitor for EnvFunctionCollector {
    /// Collects the function operand of an op declaring `BuiltinOp::env_operand`.
    fn start_visit_builtin(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if let Some(name) = env_function_operand(&expr.get_builtin()) {
            self.names.insert(name);
        }
        StartVisitResult::VisitChildren
    }
}

/// Rewrites the bindings of one symbol, as the module comment describes.
struct CaptureListMover<'a> {
    /// The names the symbol passes as the function operand of an op declaring
    /// `BuiltinOp::env_operand`.
    env_functions: Set<FullName>,
    /// The type environment, which knows the fields of the tuples the ops apply their functions to.
    type_env: &'a TypeEnv,
    /// The number of bindings rewritten so far, which tells apart the names each rewrite introduces.
    counter: u32,
    /// The global functions the rewritten bindings call, in the order they were rewritten.
    callees: Vec<FullName>,
}

impl CaptureListMover<'_> {
    /// `expr` rewritten as the module comment describes, where it is `let h = F(cap); body`, `F` is
    /// a global function, and `h` is used once in `body`, as the function operand of an op declaring
    /// `BuiltinOp::env_operand`.
    fn rewrite(&mut self, expr: &Arc<ExprNode>) -> Option<Arc<ExprNode>> {
        let pat = expr.get_let_pat();
        if !pat.is_var() {
            return None;
        }
        let name = pat.get_var().name.clone();
        if !self.env_functions.contains(&name) {
            return None;
        }
        let bound = expr.get_let_bound();
        if !bound.is_app() {
            return None;
        }
        let (func, args) = bound.destructure_app();
        if !func.is_var() || !func.get_var().name.is_global() || args.len() != 1 {
            return None;
        }
        let body = expr.get_let_value();
        if !used_once_as_env_function(&body, &name) {
            return None;
        }

        let id = self.counter;
        self.counter += 1;
        let local =
            |suffix: &str| FullName::local(&format!("{}{}{}", DECAPTURE_PREFIX, id, suffix));
        self.callees.push(func.get_var().name.clone());

        // The ops apply `h` to the tuple `(env, r_1, ..., r_k)`, where `r_1`, ..., `r_k` are the
        // arguments besides the environment. They are given `(env, cap)` in place of `env`.
        let cap = args[0].clone();
        let cap_ty = cap.type_.as_ref().unwrap().clone();
        let cap_name = local("_cap");
        let param_tys = bound.type_.as_ref().unwrap().get_lambda_srcs();
        assert_eq!(
            param_tys.len(),
            1,
            "a function given to an op declaring `env_operand` takes one tuple: {}",
            name.to_string()
        );
        let param_ty = param_tys[0].clone();
        let param_field_tys = param_ty.field_types(self.type_env);
        let env_ty = param_field_tys[0].clone();
        let new_env_ty = make_tuple_ty(vec![env_ty.clone(), cap_ty.clone()]);
        let mut op_rewriter = OpRewriter {
            function: &name,
            cap: var_expr(&cap_name, &cap_ty),
            new_env: local("_op_env"),
            new_env_ty: new_env_ty.clone(),
            env_ty: env_ty.clone(),
        };
        let body = op_rewriter.traverse(&body).expr;

        // `|q| (let ((env0, c0), r_1, ..., r_k) = q; F(c0, (env0, r_1, ..., r_k)))`
        let mut new_param_field_tys = param_field_tys.clone();
        new_param_field_tys[0] = new_env_ty.clone();
        let new_param_ty = make_tuple_ty(new_param_field_tys);
        let param_tycon = tycon(make_tuple_name_abs(param_field_tys.len() as u32));
        let env0 = local("_env");
        let cap0 = local("_cap0");
        let env_pat = PatternNode::make_struct(
            tycon(make_tuple_name_abs(2)),
            vec![
                ("0".to_string(), var_pattern(&env0, &env_ty)),
                ("1".to_string(), var_pattern(&cap0, &cap_ty)),
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
        let call = expr_app_typed(
            expr_app_typed(func, vec![var_expr(&cap0, &cap_ty)]),
            vec![expr_make_struct(param_tycon.clone(), param_field_exprs).set_type(param_ty)],
        );
        let new_func = expr_abs_typed(
            var_local(&new_param.name),
            new_param_ty.clone(),
            expr_let_typed(
                PatternNode::make_struct(param_tycon, param_field_pats)
                    .set_type(new_param_ty.clone()),
                var_expr(&new_param, &new_param_ty),
                call,
            ),
        );
        assert!(
            new_func.lambda_cap_names().is_empty(),
            "the function bound to `{}` still captures {:?} once its capture list is moved into \
             the environment",
            name.to_string(),
            new_func
                .lambda_cap_names()
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<_>>()
        );

        // `let c = cap; let h = |q| ..; body`, where `cap` is read where `F(cap)` read it.
        let new_func_ty = new_func.type_.as_ref().unwrap().clone();
        Some(expr_let_typed(
            var_pattern(&cap_name, &cap_ty),
            cap,
            expr_let_typed(pat.set_type(new_func_ty), new_func, body),
        ))
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

impl ExprVisitor for CaptureListMover<'_> {
    /// Rewrites a `let` binding a function given to ops, and visits what the rewrite left.
    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        match self.rewrite(expr) {
            Some(expr) => StartVisitResult::ReplaceAndRevisit(expr),
            None => StartVisitResult::VisitChildren,
        }
    }
}

/// Whether `name` is used once in `expr`, as the function operand of an op declaring
/// `BuiltinOp::env_operand`.
fn used_once_as_env_function(expr: &Arc<ExprNode>, name: &FullName) -> bool {
    matches!(
        find_usage_of_name::run(expr, name).as_slice(),
        [UsageType::EnvFunctionOperand]
    )
}

/// Whether `name` is bound again inside the expression a walk started from, so that an occurrence
/// of the name here refers to that inner binding.
fn shadowed(name: &FullName, state: &VisitState) -> bool {
    state.scope.has_value(&name.name)
}

/// Gives each op that applies one function the environment `(env, cap)` in place of `env`.
struct OpRewriter<'a> {
    /// The name the function is bound to.
    function: &'a FullName,
    /// The capture list of the function, read by name.
    cap: Arc<ExprNode>,
    /// The name the new environment is bound to ahead of an op.
    new_env: FullName,
    /// The type of the new environment, `(env, cap)`.
    new_env_ty: Arc<TypeNode>,
    /// The type of the environment an op is given.
    env_ty: Arc<TypeNode>,
}

impl ExprVisitor for OpRewriter<'_> {
    /// Binds `(env, cap)` ahead of an op applying the function, and gives the op that name as its
    /// environment.
    fn start_visit_builtin(
        &mut self,
        expr: &Arc<ExprNode>,
        state: &mut VisitState,
    ) -> StartVisitResult {
        if env_function_operand(&expr.get_builtin()).as_ref() != Some(self.function)
            || shadowed(self.function, state)
        {
            return StartVisitResult::VisitChildren;
        }

        // `let env1 = (env, cap); op(.., env1, ..)`
        let mut builtin = expr.get_builtin().as_ref().clone();
        let env_index = builtin.op.env_operand().unwrap().env;
        let mut operands = builtin.op.free_vars_mut();
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
            expr.set_builtin(builtin),
        ))
    }
}
