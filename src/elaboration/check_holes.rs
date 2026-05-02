// Post-elaboration scan that turns each surviving `Std::#hole`
// reference into an `ERR_HOLE` diagnostic.
//
// The parser substitutes `Std::#hole` for any expression position that
// was left empty (see `expr_hole` / `parse_expr_or_hole` in
// `parse/parser.rs`). `Std::#hole` is registered as the polymorphic
// builtin `Std::#hole : a`, so type inference completes; this pass
// then walks every type-checked global value and reports each hole
// occurrence with the type the surrounding context expected.

use std::sync::Arc;

use crate::ast::expr::{Expr, ExprNode};
use crate::ast::name::FullName;
use crate::ast::program::{Program, SymbolExpr};
use crate::constants::{ERR_HOLE, HOLE_NAME, STD_NAME};
use crate::error::{Error, Errors};

pub fn collect_hole_diagnostics(program: &Program) -> Errors {
    let hole_name = absolute_hole_name();
    let mut errors = Errors::empty();
    for (_name, gv) in program.global_values.iter() {
        match &gv.expr {
            SymbolExpr::Simple(te) => {
                visit(&te.expr, &hole_name, &mut errors);
            }
            SymbolExpr::Method(impls) => {
                for impl_ in impls {
                    visit(&impl_.expr.expr, &hole_name, &mut errors);
                }
            }
        }
    }
    errors
}

fn absolute_hole_name() -> FullName {
    let mut name = FullName::from_strs(&[STD_NAME], HOLE_NAME);
    name.global_to_absolute();
    name
}

fn visit(expr: &Arc<ExprNode>, hole_name: &FullName, errors: &mut Errors) {
    if let Expr::Var(v) = &*expr.expr {
        if v.name == *hole_name {
            errors.append(report_hole(expr));
            return;
        }
    }
    match &*expr.expr {
        Expr::Var(_) | Expr::LLVM(_) => {}
        Expr::App(f, args) => {
            visit(f, hole_name, errors);
            for a in args {
                visit(a, hole_name, errors);
            }
        }
        Expr::Lam(_, body) => visit(body, hole_name, errors),
        Expr::Let(_, b, v) => {
            visit(b, hole_name, errors);
            visit(v, hole_name, errors);
        }
        Expr::If(c, t, e) => {
            visit(c, hole_name, errors);
            visit(t, hole_name, errors);
            visit(e, hole_name, errors);
        }
        Expr::Match(s, arms) => {
            visit(s, hole_name, errors);
            for (_, a) in arms {
                visit(a, hole_name, errors);
            }
        }
        Expr::TyAnno(e, _) => visit(e, hole_name, errors),
        Expr::MakeStruct(_, fields) => {
            for (_, _, fe) in fields {
                visit(fe, hole_name, errors);
            }
        }
        Expr::ArrayLit(elems) => {
            for e in elems {
                visit(e, hole_name, errors);
            }
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            for a in args {
                visit(a, hole_name, errors);
            }
        }
        Expr::Eval(a, b) => {
            visit(a, hole_name, errors);
            visit(b, hole_name, errors);
        }
    }
}

fn report_hole(node: &Arc<ExprNode>) -> Errors {
    // When the hole's type was resolved by elaboration, include it.
    // When it wasn't (typically because typecheck failed earlier and
    // never substituted this node's type), drop the type clause —
    // saying nothing is more honest than printing `?`.
    let msg = match node.type_.as_ref() {
        Some(ty) => format!("Expected expression of type `{}`.", ty.to_string()),
        None => "Expected expression.".to_string(),
    };
    let mut err = Error::from_msg_srcs(msg, &[&node.source]);
    err.code = Some(ERR_HOLE);
    Errors::from_err(err)
}
