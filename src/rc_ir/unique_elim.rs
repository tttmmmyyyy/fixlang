//! Unique-check elimination on the RC IR.
//!
//! An operation that force-uniques its container before mutating it in place (`Array::set`, `swap`,
//! `mod`/`act` via punch/plug, struct `set`/`mod`) carries a runtime check that clones the container
//! when it is shared. Where the provenance analysis proves the container statically `Unique` at the
//! operation, that check is redundant: this pass drops it, leaving the operation to write in place
//! unconditionally.
//!
//! Without cross-function specialization a function's parameters are of unknown sharing, so only its
//! locally produced (`Fresh`) values resolve to `Unique`; a container that flows in as an argument
//! stays `Dynamic` and keeps its check. Specialization supplies known input uniqueness and unlocks
//! the in-loop cases.

use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::misc::Map;
use crate::rc_ir::ast::{MatchArm, RcExpr, RcExprNode, RcProgram, RcRhs};
use crate::rc_ir::provenance::{analyze_program, leaf_is_unique, Provenance};

/// Drop the force-unique check from every mutation whose container the provenance analysis proves
/// statically unique at the operation.
pub fn elim_unique_checks(prog: &mut RcProgram, type_env: &TypeEnv) {
    // The container provenance is taken at each operation's program point, so a container demoted by
    // a preceding `Retain` is correctly seen as shared.
    let op_containers = analyze_program(prog, type_env).op_containers;
    for func in prog.funcs.values_mut() {
        func.body = rewrite(&func.body, &op_containers);
    }
    for glob in prog.globals.iter_mut() {
        glob.init = rewrite(&glob.init, &op_containers);
    }
}

fn rewrite(node: &RcExprNode, op_containers: &Map<FullName, Provenance>) -> RcExprNode {
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || rewrite_inner(node, op_containers))
}

fn rewrite_inner(node: &RcExprNode, op_containers: &Map<FullName, Provenance>) -> RcExprNode {
    let expr = match node.expr.as_ref() {
        RcExpr::Let(x, RcRhs::Llvm(gen, args), k) => {
            let container_unique = match gen.force_unique_target() {
                Some((_, path)) => op_containers.get(&x.name).map_or(false, |prov| {
                    // No specialization here, so the container's function inputs are unknown: resolve
                    // against all-`Dynamic` inputs, which drops the check only on a locally `Fresh`
                    // container.
                    leaf_is_unique(prov, &path, &[])
                }),
                None => false,
            };
            let gen = if container_unique {
                gen.without_force_unique()
            } else {
                gen.clone()
            };
            RcExpr::Let(
                x.clone(),
                RcRhs::Llvm(gen, args.clone()),
                rewrite(k, op_containers),
            )
        }
        RcExpr::Let(x, RcRhs::Match(scrutinee, arms), k) => {
            // A `match` holds arm bodies that may themselves contain mutations.
            let arms = arms
                .iter()
                .map(|arm| MatchArm {
                    variant: arm.variant,
                    payload: arm.payload.clone(),
                    body: rewrite(&arm.body, op_containers),
                })
                .collect();
            RcExpr::Let(
                x.clone(),
                RcRhs::Match(scrutinee.clone(), arms),
                rewrite(k, op_containers),
            )
        }
        RcExpr::Let(x, rhs, k) => {
            RcExpr::Let(x.clone(), rhs.clone(), rewrite(k, op_containers))
        }
        RcExpr::Retain(v, path, state, k) => {
            RcExpr::Retain(v.clone(), path.clone(), *state, rewrite(k, op_containers))
        }
        RcExpr::Release(v, path, state, k) => {
            RcExpr::Release(v.clone(), path.clone(), *state, rewrite(k, op_containers))
        }
        RcExpr::Destructure(container, fields, k) => {
            RcExpr::Destructure(container.clone(), fields.clone(), rewrite(k, op_containers))
        }
        RcExpr::Ret(v) => RcExpr::Ret(v.clone()),
    };
    RcExprNode {
        expr: Box::new(expr),
        source: node.source.clone(),
    }
}
