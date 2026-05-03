//! Walks a type-checked expression and turns each `Std::#hole`
//! reference into an `ERR_HOLE` diagnostic.
//!
//! The parser substitutes `Std::#hole` for any expression position that
//! was left empty. `Std::#hole` is registered as the polymorphic
//! builtin `Std::#hole : a`, so type inference completes; this pass
//! must be run after type substitution has been applied to every
//! node, so each hole's type reflects whatever the surrounding
//! context expected.

use std::sync::Arc;
use crate::ast::expr::{hole_full_name, Expr, ExprNode};
use crate::ast::name::FullName;
use crate::constants::ERR_HOLE;
use crate::elaboration::typecheck::TypeCheckContext;
use crate::error::{Error, Errors};

/// Collect ERR_HOLE diagnostics for every hole reference in `expr`.
///
/// `expr` is expected to have already had type substitution applied,
/// so each hole's `type_` carries the substituted expected type. `tc`
/// is used to look up where each free type variable mentioned in the
/// hole's type was first introduced, so the diagnostic can point at
/// the originating expression.
pub fn collect_hole_errors(expr: &Arc<ExprNode>, tc: &TypeCheckContext) -> Errors {
    let hole_name = hole_full_name();
    let mut errors = Errors::empty();
    visit(expr, &hole_name, tc, &mut errors);
    errors
}

/// Recursively walk `expr` and append an ERR_HOLE diagnostic to
/// `errors` for every `Var` node whose name equals `hole_name`.
fn visit(
    expr: &Arc<ExprNode>,
    hole_name: &FullName,
    tc: &TypeCheckContext,
    errors: &mut Errors,
) {
    if let Expr::Var(v) = &*expr.expr {
        if v.name == *hole_name {
            errors.append(report_hole(expr, tc));
            return;
        }
    }
    match &*expr.expr {
        Expr::Var(_) | Expr::LLVM(_) => {}
        Expr::App(f, args) => {
            visit(f, hole_name, tc, errors);
            for a in args {
                visit(a, hole_name, tc, errors);
            }
        }
        Expr::Lam(_, body) => visit(body, hole_name, tc, errors),
        Expr::Let(_, b, v) => {
            visit(b, hole_name, tc, errors);
            visit(v, hole_name, tc, errors);
        }
        Expr::If(c, t, e) => {
            visit(c, hole_name, tc, errors);
            visit(t, hole_name, tc, errors);
            visit(e, hole_name, tc, errors);
        }
        Expr::Match(s, arms) => {
            visit(s, hole_name, tc, errors);
            for (_, a) in arms {
                visit(a, hole_name, tc, errors);
            }
        }
        Expr::TyAnno(e, _) => visit(e, hole_name, tc, errors),
        Expr::MakeStruct(_, fields) => {
            for (_, _, fe) in fields {
                visit(fe, hole_name, tc, errors);
            }
        }
        Expr::ArrayLit(elems) => {
            for e in elems {
                visit(e, hole_name, tc, errors);
            }
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            for a in args {
                visit(a, hole_name, tc, errors);
            }
        }
        Expr::Eval(a, b) => {
            visit(a, hole_name, tc, errors);
            visit(b, hole_name, tc, errors);
        }
    }
}

/// Build an ERR_HOLE diagnostic for the hole reference at `node`.
///
/// When the hole's type was resolved by elaboration the message
/// includes it (`Expected expression of type ...`). When it wasn't,
/// the message degrades to `Expected expression.` and no type-variable
/// origin annotations are attached.
fn report_hole(node: &Arc<ExprNode>, tc: &TypeCheckContext) -> Errors {
    let (msg, free_tvs) = match node.type_.as_ref() {
        Some(ty) => {
            let mut free = vec![];
            ty.free_vars_to_vec(&mut free);
            (
                format!("Expected expression of type `{}`.", ty.to_string()),
                free,
            )
        }
        None => ("Expected expression.".to_string(), vec![]),
    };
    let mut err = Error::from_msg_srcs(msg, &[&node.source]);
    err.code = Some(ERR_HOLE);
    // For each free type variable that survives in the hole's type
    // (typically `#a0` style names when the hole is in an
    // unconstrained context), point at the expression that originally
    // produced that type variable so the user can see where the
    // indeterminacy started.
    err.add_srcs(tc.create_tyvar_location_messages(&free_tvs, None));
    Errors::from_err(err)
}
