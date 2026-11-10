// A check that the types of a program agree with one another where its expressions meet, for a pass
// rewriting the types of a program to run on what it wrote.

use crate::ast::{
    expr::{Expr, ExprNode},
    name::FullName,
    pattern::{Pattern, PatternNode},
    program::Program,
    type_env::TypeEnv,
    types::{type_arrow, type_func, TypeNode},
};
use crate::misc::Map;
use std::sync::Arc;

/// Panics, naming the symbol and the node, where two types given to meeting places of
/// `prg` disagree: a symbol and its body, a function and the argument it is applied to, a global
/// and a reference to it, a local and a reference to it, a value and the pattern taking it apart,
/// the arms of a choice, a struct and its fields. The types of builtin operations and of calls of C
/// functions are trusted as written.
///
/// A function type is read as arrows, the form it has before `uncurry` gathers parameters into a
/// func type.
pub fn validate_types(prg: &Program) {
    let type_env = prg.type_env();
    let check = Check {
        prg,
        type_env: &type_env,
    };
    let mut names: Vec<_> = prg.symbols.keys().collect();
    names.sort();
    for name in names {
        let symbol = &prg.symbols[name];
        let expr = symbol.expr.as_ref().unwrap();
        let at = name.to_string();
        check.same(
            &symbol.ty,
            ty(expr),
            &at,
            expr,
            "the symbol's type and its body's",
        );
        check.expr(expr, &at, &mut Scope::default());
    }
    for root in prg.root_value_exprs() {
        check.expr(root, "a root of the program", &mut Scope::default());
    }
}

/// What `validate_types` reads the program with.
struct Check<'p> {
    /// The program checked.
    prg: &'p Program,
    /// The type environment of `prg`.
    type_env: &'p TypeEnv,
}

/// The locals in scope at a node, each with the type its binder gives it.
#[derive(Default)]
struct Scope {
    /// The types the binders in scope give each name, the innermost last.
    types: Map<FullName, Vec<Arc<TypeNode>>>,
    /// The names bound, in the order they were bound.
    bound: Vec<FullName>,
}

impl Scope {
    /// The number of bindings in scope, which `unbind_to` returns to.
    fn depth(&self) -> usize {
        self.bound.len()
    }

    /// Brings `name` into scope with the type `ty`, over any binding of it already in scope.
    fn bind(&mut self, name: &FullName, ty: &Arc<TypeNode>) {
        self.types.entry(name.clone()).or_default().push(ty.clone());
        self.bound.push(name.clone());
    }

    /// Takes out of scope the bindings made since the scope had `depth` of them.
    fn unbind_to(&mut self, depth: usize) {
        for name in self.bound.drain(depth..) {
            self.types.get_mut(&name).unwrap().pop();
        }
    }

    /// The type the innermost binder in scope gives `name`.
    fn type_of(&self, name: &FullName) -> Option<&Arc<TypeNode>> {
        self.types.get(name).and_then(|tys| tys.last())
    }
}

/// The type of the node `e`.
fn ty(e: &ExprNode) -> &Arc<TypeNode> {
    e.type_.as_ref().unwrap()
}

impl<'p> Check<'p> {
    /// Panics unless `a` and `b` are one type, the two types `what` names at the node `e` of `at`.
    fn same(&self, a: &Arc<TypeNode>, b: &Arc<TypeNode>, at: &str, e: &ExprNode, what: &str) {
        assert!(
            a == b,
            "in `{}`, {} differ: `{}` and `{}`, at `{}`",
            at,
            what,
            a.to_string(),
            b.to_string(),
            e.expr.stringify().to_string()
        );
    }

    /// Checks the node `e` of `at` and the nodes under it, `scope` holding the locals in scope at
    /// `e`.
    fn expr(&self, e: &Arc<ExprNode>, at: &str, scope: &mut Scope) {
        match &*e.expr {
            Expr::Var(v) => {
                if v.name.is_local() {
                    let bound_ty = scope.type_of(&v.name).unwrap_or_else(|| {
                        panic!(
                            "in `{}`, the local `{}` is read where nothing binds it",
                            at,
                            v.name.to_string()
                        )
                    });
                    self.same(bound_ty, ty(e), at, e, "a local's type and its reference's");
                } else {
                    let symbol = self.prg.symbols.get(&v.name).unwrap_or_else(|| {
                        panic!(
                            "in `{}`, the global `{}` is no symbol",
                            at,
                            v.name.to_string()
                        )
                    });
                    self.same(
                        &symbol.ty,
                        ty(e),
                        at,
                        e,
                        "a global's type and its reference's",
                    );
                }
            }
            Expr::Builtin(_) => {}
            Expr::App(func, args) => {
                self.expr(func, at, scope);
                for arg in args {
                    self.expr(arg, at, scope);
                }
                let expected = if ty(func).is_func() {
                    type_func(args.iter().map(|arg| ty(arg).clone()).collect(), ty(e).clone())
                } else {
                    args.iter()
                        .rev()
                        .fold(ty(e).clone(), |acc, arg| type_arrow(ty(arg).clone(), acc))
                };
                self.same(
                    ty(func),
                    &expected,
                    at,
                    e,
                    "a function's type and its application's",
                );
            }
            Expr::Lam(params, body) => {
                let depth = scope.depth();
                let param_tys = ty(e).get_lambda_srcs();
                assert_eq!(
                    params.len(),
                    param_tys.len(),
                    "in `{}`, a lambda's parameters and its type's differ in number, at `{}`",
                    at,
                    e.expr.stringify().to_string()
                );
                for (param, param_ty) in params.iter().zip(param_tys) {
                    scope.bind(&param.name, &param_ty);
                }
                self.expr(body, at, scope);
                scope.unbind_to(depth);
                self.same(
                    &ty(e).get_lambda_dst(),
                    ty(body),
                    at,
                    e,
                    "a lambda's result and body",
                );
            }
            Expr::Let(pat, bound, val) => {
                self.expr(bound, at, scope);
                let depth = scope.depth();
                self.pattern(pat, ty(bound), at, e, scope);
                self.expr(val, at, scope);
                scope.unbind_to(depth);
                self.same(ty(e), ty(val), at, e, "a let's type and its value's");
            }
            Expr::If(cond, then_expr, else_expr) => {
                self.expr(cond, at, scope);
                self.expr(then_expr, at, scope);
                self.expr(else_expr, at, scope);
                self.same(
                    ty(e),
                    ty(then_expr),
                    at,
                    e,
                    "an if's type and its then arm's",
                );
                self.same(
                    ty(e),
                    ty(else_expr),
                    at,
                    e,
                    "an if's type and its else arm's",
                );
            }
            Expr::Match(cond, arms) => {
                self.expr(cond, at, scope);
                for (pat, val) in arms {
                    let depth = scope.depth();
                    self.pattern(pat, ty(cond), at, e, scope);
                    self.expr(val, at, scope);
                    scope.unbind_to(depth);
                    self.same(ty(e), ty(val), at, e, "a match's type and an arm's");
                }
            }
            Expr::TyAnno(expr, anno) => {
                self.expr(expr, at, scope);
                self.same(
                    ty(e),
                    ty(expr),
                    at,
                    e,
                    "an annotated expression's type and its value's",
                );
                self.same(
                    ty(e),
                    anno,
                    at,
                    e,
                    "an annotated expression's type and its annotation",
                );
            }
            Expr::ArrayLit(elems) => {
                let elem_ty = ty(e).collect_type_arguments()[0].clone();
                for elem in elems {
                    self.expr(elem, at, scope);
                    self.same(
                        &elem_ty,
                        ty(elem),
                        at,
                        e,
                        "an array literal's element types",
                    );
                }
            }
            Expr::MakeStruct(tc, fields) => {
                assert!(
                    ty(e).toplevel_tycon().as_deref() == Some(tc.as_ref()),
                    "in `{}`, a struct built as `{}` has the type `{}`",
                    at,
                    tc.to_string(),
                    ty(e).to_string()
                );
                let field_tys = ty(e).field_types(self.type_env);
                for (name, _, value) in fields {
                    self.expr(value, at, scope);
                    let i = ty(e).field_index(self.type_env, name).unwrap();
                    self.same(
                        &field_tys[i],
                        ty(value),
                        at,
                        e,
                        "a field's type and its value's",
                    );
                }
            }
            Expr::FFICall(_, _, _, _, args, _) => {
                for arg in args {
                    self.expr(arg, at, scope);
                }
            }
            Expr::Eval(side, main) => {
                self.expr(side, at, scope);
                self.expr(main, at, scope);
                self.same(ty(e), ty(main), at, e, "an eval's type and its value's");
            }
        }
    }

    /// Checks the pattern `pat`, which takes apart a value of type `matched` at the node `e` of `at`,
    /// and adds the locals it binds to `scope`.
    fn pattern(
        &self,
        pat: &Arc<PatternNode>,
        matched: &Arc<TypeNode>,
        at: &str,
        e: &ExprNode,
        scope: &mut Scope,
    ) {
        let pat_ty = pat.info.type_.as_ref().unwrap();
        self.same(
            pat_ty,
            matched,
            at,
            e,
            "a pattern's type and the value's it takes apart",
        );
        match &pat.pattern {
            Pattern::Var(var, anno) => {
                scope.bind(&var.name, pat_ty);
                if let Some(anno) = anno {
                    self.same(
                        anno,
                        pat_ty,
                        at,
                        e,
                        "a variable pattern's annotation and type",
                    );
                }
            }
            Pattern::Struct(tc, fields, _) => {
                assert!(
                    pat_ty.toplevel_tycon().as_deref() == Some(tc.as_ref()),
                    "in `{}`, a struct taken apart as `{}` has the type `{}`",
                    at,
                    tc.to_string(),
                    pat_ty.to_string()
                );
                let field_tys = pat_ty.field_types(self.type_env);
                for (name, _, sub) in fields {
                    let i = pat_ty.field_index(self.type_env, name).unwrap();
                    self.pattern(sub, &field_tys[i], at, e, scope);
                }
            }
            Pattern::Union(variant, _, sub) => {
                let union = Pattern::variant_union_tycon(variant).unwrap();
                assert!(
                    pat_ty.toplevel_tycon().as_deref() == Some(&union),
                    "in `{}`, the variant `{}` is matched against the type `{}`",
                    at,
                    variant.to_string(),
                    pat_ty.to_string()
                );
                let i = pat_ty.field_index(self.type_env, &variant.name).unwrap();
                self.pattern(sub, &pat_ty.field_types(self.type_env)[i], at, e, scope);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_types;
    use crate::{elaboration::elaborate_at_max, fixstd::builtin::make_i64_ty};

    /// A read of a local whose type differs from the one its `let` binding gives it is reported.
    /// The read is the argument of a call of a C function, whose argument types are trusted as
    /// written, so this is the only meeting place that sees it.
    #[test]
    #[should_panic(expected = "a local's type and its reference's")]
    fn test_a_local_read_at_a_type_other_than_its_binders_is_reported() {
        let source = r#"
module Main;

to_abs : I32 -> I32;
to_abs = |x| FFI_CALL[I32 abs(I32), x];

main : IO ();
main = eval to_abs(-3_I32); pure();
"#;
        let (mut prg, _) = elaborate_at_max(source, "type_agreement", 0);
        validate_types(&prg);

        let name = prg
            .symbols
            .keys()
            .find(|s| {
                s.namespace.to_string() == "Main" && s.name.split('#').next() == Some("to_abs")
            })
            .unwrap()
            .clone();
        let symbol = prg.symbols.get_mut(&name).unwrap();
        let lam = symbol.expr.clone().unwrap();
        let body = lam.get_lam_body();
        let call = body.get_let_value();
        let read = call.get_ffi_call_args()[0].set_type(make_i64_ty());
        symbol.expr =
            Some(lam.set_lam_body(body.set_let_value(call.set_ffi_call_args(vec![read]))));
        validate_types(&prg);
    }
}
