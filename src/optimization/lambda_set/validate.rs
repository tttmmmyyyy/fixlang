// A check that the types of a rewritten program agree with one another where its expressions meet.

use crate::ast::{
    expr::{Expr, ExprNode},
    pattern::{Pattern, PatternNode},
    program::Program,
    type_env::TypeEnv,
    types::{type_arrow, TypeNode},
};
use std::sync::Arc;

/// Panics, naming the symbol and the node, where two types the rewrite gives meeting places of
/// `prg` disagree: a symbol and its body, a function and the argument it is applied to, a global
/// and a reference to it, a value and the pattern taking it apart, the arms of a choice, a struct
/// and its fields. The types of builtin operations and of calls of C functions are trusted as
/// written.
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
        check.expr(expr, &at);
    }
    for root in prg.root_value_exprs() {
        check.expr(root, "a root of the program");
    }
}

/// What `validate_types` reads the program with.
struct Check<'p> {
    /// The program checked.
    prg: &'p Program,
    /// The type environment of `prg`.
    type_env: &'p TypeEnv,
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

    /// Checks the node `e` of `at` and the nodes under it.
    fn expr(&self, e: &Arc<ExprNode>, at: &str) {
        match &*e.expr {
            Expr::Var(v) => {
                if !v.name.is_local() {
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
                self.expr(func, at);
                for arg in args {
                    self.expr(arg, at);
                }
                let expected = args
                    .iter()
                    .rev()
                    .fold(ty(e).clone(), |acc, arg| type_arrow(ty(arg).clone(), acc));
                self.same(
                    ty(func),
                    &expected,
                    at,
                    e,
                    "a function's type and its application's",
                );
            }
            Expr::Lam(_, body) => {
                self.expr(body, at);
                self.same(
                    &ty(e).get_lambda_dst(),
                    ty(body),
                    at,
                    e,
                    "a lambda's result and body",
                );
            }
            Expr::Let(pat, bound, val) => {
                self.expr(bound, at);
                self.pattern(pat, ty(bound), at, e);
                self.expr(val, at);
                self.same(ty(e), ty(val), at, e, "a let's type and its value's");
            }
            Expr::If(cond, then_expr, else_expr) => {
                self.expr(cond, at);
                self.expr(then_expr, at);
                self.expr(else_expr, at);
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
                self.expr(cond, at);
                for (pat, val) in arms {
                    self.pattern(pat, ty(cond), at, e);
                    self.expr(val, at);
                    self.same(ty(e), ty(val), at, e, "a match's type and an arm's");
                }
            }
            Expr::TyAnno(expr, anno) => {
                self.expr(expr, at);
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
                    self.expr(elem, at);
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
                    self.expr(value, at);
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
                    self.expr(arg, at);
                }
            }
            Expr::Eval(side, main) => {
                self.expr(side, at);
                self.expr(main, at);
                self.same(ty(e), ty(main), at, e, "an eval's type and its value's");
            }
        }
    }

    /// Checks the pattern `pat`, which takes apart a value of type `matched` at the node `e` of `at`.
    fn pattern(&self, pat: &Arc<PatternNode>, matched: &Arc<TypeNode>, at: &str, e: &ExprNode) {
        let pat_ty = pat.info.type_.as_ref().unwrap();
        self.same(
            pat_ty,
            matched,
            at,
            e,
            "a pattern's type and the value's it takes apart",
        );
        match &pat.pattern {
            Pattern::Var(_, anno) => {
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
                    self.pattern(sub, &field_tys[i], at, e);
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
                self.pattern(sub, &pat_ty.field_types(self.type_env)[i], at, e);
            }
        }
    }
}
