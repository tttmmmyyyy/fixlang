//! Lending reads: a value read out of a boxed container is read without a retain where the
//! container's object holds it for as long as the code uses it. Such a value is *lent*: it holds no
//! reference of its own, so its reference counting goes, and where the code hands it to a callee
//! that owns it, a retain right before the call gives the callee the reference it takes.
//!
//! A read lends its value `a`, read out of the container `c` whose object is `o`, where
//!
//! 1. the body consumes `a`, through its aliases, only as an argument of a call;
//! 2. no unit of any variable may be `a` on one path and a value this body does not lend on
//!    another, so each reference-count node and each call argument either counts a lent value or
//!    counts none;
//! 3. `o` stays referenced while the code uses `a`: walking forward from the read along every
//!    path, a lower bound of `o`'s reference count (`Count`) stays at least 1 up to and including
//!    each use of `a` and of its aliases. The bound starts at 1, since the read itself reads `o`,
//!    rises by a retain of a value that is `o` on every path, and falls by each consume and release
//!    of a value that may be `o` on some path (`BodyModel`, the model `cancel` reads).
//!
//! A container that is itself lent holds its value only while its own container is referenced, so
//! the walk runs from the outermost read of the chain, and a value read out of a global is held
//! throughout, since a global's objects are never freed.
//!
//! The pass runs once every pass moving or deleting reference counting has run, on the counting
//! code generation emits.

use crate::ast::name::FullName;
use crate::ast::type_env::TypeEnv;
use crate::ast::types::TypeNode;
use crate::fixstd::builtin::StructGetOp;
use crate::misc::{grow_stack, Map, Set};
use crate::rc_ir::ast::{
    for_each_node, map_children, FieldPath, RcExpr, RcExprNode, RcProgram, RcRhs, RcState, RcVar,
    VarPath,
};
use crate::rc_ir::borrow::{rewrite_bodies, BodyModel};
use crate::rc_ir::leaf_map::boxed_leaf_paths;
use crate::rc_ir::ownership::{
    all_owned_units, collect_consumes_outside_call_arguments, rc_units, units_under,
};
use std::sync::Arc;

/// `prog` with every read that can lend its value (see the module documentation) lending it.
pub(crate) fn lend_reads(prog: &RcProgram, type_env: &TypeEnv) -> RcProgram {
    let owned_units = all_owned_units(prog, type_env);
    rewrite_bodies(prog, |vars, body| {
        let model = BodyModel {
            vars,
            prog,
            owned_units: &owned_units,
            type_env,
        };
        let lent = lent_values(&model, body);
        if lent.is_empty() {
            return body.clone();
        }
        Lender {
            model: &model,
            lent: &lent,
        }
        .rewrite(body)
    })
}

/// The values `body` reads out of a boxed container and can lend: the conditions of the module
/// documentation, checked again until the set is stable, since a value taken out of the set can
/// fail a condition another value passed.
fn lent_values(model: &BodyModel, body: &RcExprNode) -> Set<FullName> {
    let type_env = model.type_env;
    let containers = reads(body, type_env);
    if containers.is_empty() {
        return Set::default();
    }
    let mut lent: Set<FullName> = containers.keys().cloned().collect();
    let mut consumed = vec![];
    collect_consumes_outside_call_arguments(body, model.vars, model.prog, type_env, &mut consumed);
    for (var, leaf) in &consumed {
        for (root, _) in model.leaf_objects(var, leaf) {
            lent.remove(&root);
        }
    }
    loop {
        let mut failing = Set::default();
        for (name, ty) in &model.vars.var_tys {
            for unit in rc_units(ty, type_env) {
                let roots = roots_under(model, name, ty, &unit);
                if roots.iter().any(|r| lent.contains(r)) && roots.iter().any(|r| !lent.contains(r))
                {
                    failing.extend(roots.into_iter().filter(|r| lent.contains(r)));
                }
            }
        }
        let mut anchors: Map<FullName, Option<Set<FullName>>> = Map::default();
        for value in &lent {
            anchors_of(model, value, &containers, &lent, &mut anchors);
        }
        let mut dependents: Map<FullName, Set<FullName>> = Map::default();
        for (value, of) in &anchors {
            match of {
                None => {
                    failing.insert(value.clone());
                }
                Some(of) => {
                    for anchor in of {
                        dependents
                            .entry(anchor.clone())
                            .or_default()
                            .insert(value.clone());
                    }
                }
            }
        }
        for (anchor, dependents) in &dependents {
            let object = sole_object(model, &containers[anchor])
                .expect("an anchor's container is one object");
            let mut walk = Walk {
                model,
                object,
                read: anchor,
                dependents,
                failing: &mut failing,
            };
            walk.walk(body, None);
        }
        if failing.is_empty() {
            return lent;
        }
        for value in &failing {
            lent.remove(value);
        }
    }
}

/// The values `body` reads out of a boxed container, each mapped to that container: the fields a
/// `Destructure` binds out of a boxed struct, and the results of the ops reading them out of an
/// operand they borrow (`BuiltinOp::result_held_by_operand`) that have a lending form
/// (`BuiltinOp::lending`). A fully unboxed value holds no reference, so none of those is among them.
fn reads(body: &RcExprNode, type_env: &TypeEnv) -> Map<FullName, RcVar> {
    let mut containers = Map::default();
    for_each_node(body, &mut |node| match node.expr.as_ref() {
        RcExpr::Destructure(container, fields, _, _) if container.ty.is_box(type_env) => {
            for (_, field) in fields {
                if !field.ty.is_fully_unboxed(type_env) {
                    containers.insert(field.name.clone(), container.clone());
                }
            }
        }
        RcExpr::Let(x, RcRhs::Builtin(op, args), _) if !x.ty.is_fully_unboxed(type_env) => {
            let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
            if let Some(j) = op.result_held_by_operand(&arg_tys, type_env) {
                if op.borrows_operand(j, &arg_tys, type_env) && op.lending().is_some() {
                    containers.insert(x.name.clone(), args[j].clone());
                }
            }
        }
        RcExpr::Let(..)
        | RcExpr::Destructure(..)
        | RcExpr::Retain(..)
        | RcExpr::Release(..)
        | RcExpr::Eval(..)
        | RcExpr::Ret(..) => {}
    });
    containers
}

/// Records in `anchors` the reads whose container's object holds the lent value named `value`:
/// the read of `value` itself where its container is one object this body does not lend, the
/// anchors of its container where the container is lent, and none where the container is a global.
/// `None` where the container is none of those, so `value` is not lent.
fn anchors_of(
    model: &BodyModel,
    value: &FullName,
    containers: &Map<FullName, RcVar>,
    lent: &Set<FullName>,
    anchors: &mut Map<FullName, Option<Set<FullName>>>,
) -> Option<Set<FullName>> {
    if let Some(known) = anchors.get(value) {
        return known.clone();
    }
    let container = &containers[value];
    let roots = roots_under(model, &container.name, &container.ty, &vec![]);
    let answer = if roots.iter().all(|root| !root.is_local()) {
        Some(Set::default())
    } else if roots.iter().all(|root| lent.contains(root)) {
        let mut of = Set::default();
        let mut held = true;
        for root in &roots {
            match anchors_of(model, root, containers, lent, anchors) {
                Some(more) => of.extend(more),
                None => held = false,
            }
        }
        held.then_some(of)
    } else if sole_object(model, container).is_some_and(|(root, _)| !lent.contains(&root)) {
        Some(Set::from_iter([value.clone()]))
    } else {
        None
    };
    anchors.insert(value.clone(), answer.clone());
    answer
}

/// The object `container`, a boxed value, is on every path, if it is one.
fn sole_object(model: &BodyModel, container: &RcVar) -> Option<VarPath> {
    match model.leaf_objects(&container.name, &[]).as_slice() {
        [object] if object.0.is_local() => Some(object.clone()),
        _ => None,
    }
}

/// The roots of the objects the boxed leaves of the unit `unit` of the variable named `name`, of
/// type `ty`, may belong to.
fn roots_under(
    model: &BodyModel,
    name: &FullName,
    ty: &Arc<TypeNode>,
    unit: &FieldPath,
) -> Set<FullName> {
    boxed_leaf_paths(ty, model.type_env)
        .into_iter()
        .filter(|leaf| leaf.starts_with(unit))
        .flat_map(|leaf| model.leaf_objects(name, &leaf))
        .map(|(root, _)| root)
        .collect()
}

/// A lower bound of the reference count of an object, along a path forward from a read out of it.
#[derive(Clone, Copy)]
struct Count {
    /// The bound at this point.
    low: i64,
    /// Whether the bound has fallen below 1 since the read: from there on the object may be freed,
    /// or updated in place by an op consuming its last reference.
    broken: bool,
}

impl Count {
    /// The bound after `delta` references come and go.
    fn moved(self, delta: i64) -> Count {
        let low = self.low + delta;
        Count {
            low,
            broken: self.broken || low < 1,
        }
    }

    /// The bound after one of several paths, the weakest of them.
    fn weakest(paths: impl Iterator<Item = Count>) -> Option<Count> {
        paths.reduce(|a, b| Count {
            low: a.low.min(b.low),
            broken: a.broken || b.broken,
        })
    }
}

/// The walk of one body checking condition 3 of the module documentation for the values read,
/// directly or through lent containers, out of the container the read of `read` reads.
struct Walk<'w, 'm> {
    /// What the body's nodes act on.
    model: &'w BodyModel<'m>,
    /// The object of the container `read` is read out of.
    object: VarPath,
    /// The value whose read the walk counts from.
    read: &'w FullName,
    /// The lent values the object holds: `read` and the values read out of it, transitively.
    dependents: &'w Set<FullName>,
    /// Where the walk adds each dependent used where the bound is broken.
    failing: &'w mut Set<FullName>,
}

impl Walk<'_, '_> {
    /// Walks `node` and its continuation from the bound `count`, `None` before the read, and
    /// returns the bound where the body or arm ends.
    fn walk(&mut self, node: &RcExprNode, count: Option<Count>) -> Option<Count> {
        grow_stack(|| match node.expr.as_ref() {
            RcExpr::Let(_, RcRhs::Match(scrutinee, arms), k) => {
                self.check_use(scrutinee, count);
                let ends: Vec<Option<Count>> =
                    arms.iter().map(|arm| self.walk(&arm.body, count)).collect();
                // An arm the read is not on reaches the continuation with no value of the walk
                // bound, so the continuation goes on from the arms the read is on.
                self.walk(k, Count::weakest(ends.into_iter().flatten()))
            }
            RcExpr::Let(x, rhs, k) => {
                let consumed = self.model.rhs_consumed_objects(rhs, &x.ty);
                let count = count.map(|c| c.moved(-self.occurrences(&consumed)));
                let operands: Vec<&RcVar> = match rhs {
                    RcRhs::Var(y) => vec![y],
                    RcRhs::App(callee, args) => std::iter::once(callee).chain(args).collect(),
                    RcRhs::Closure(_, caps) => caps.iter().collect(),
                    RcRhs::Builtin(_, args) => args.iter().collect(),
                    RcRhs::Match(..) => unreachable!("a match is walked by its own arm"),
                };
                for v in operands {
                    self.check_use(v, count);
                }
                let count = if x.name == *self.read {
                    Some(Count {
                        low: 1,
                        broken: false,
                    })
                } else {
                    count
                };
                self.walk(k, count)
            }
            RcExpr::Destructure(container, fields, _, k) => {
                self.check_use(container, count);
                let count = if fields.iter().any(|(_, f)| f.name == *self.read) {
                    Some(Count {
                        low: 1,
                        broken: false,
                    })
                } else {
                    count
                };
                self.walk(k, count)
            }
            RcExpr::Retain(v, path, _, k) => {
                // A retain of a union retains the leaves of the variant it holds, which may not be
                // the leaf that is the object, so only a unit of one leaf is certain to retain it.
                let leaves: Vec<FieldPath> = self.leaves_under(v, path).collect();
                let certain = match leaves.as_slice() {
                    [leaf] => {
                        (self.model.leaf_objects(&v.name, leaf)
                            == std::slice::from_ref(&self.object)) as i64
                    }
                    _ => 0,
                };
                self.walk(k, count.map(|c| c.moved(certain)))
            }
            RcExpr::Release(v, path, _, k) => {
                let possible = self
                    .leaves_under(v, path)
                    .filter(|leaf| {
                        self.model
                            .leaf_objects(&v.name, leaf)
                            .contains(&self.object)
                    })
                    .count() as i64;
                self.walk(k, count.map(|c| c.moved(-possible)))
            }
            RcExpr::Eval(v, k) => {
                self.check_use(v, count);
                self.walk(k, count)
            }
            RcExpr::Ret(v) => {
                self.check_use(v, count);
                count
            }
        })
    }

    /// The boxed leaves of `v` under `path`.
    fn leaves_under<'p>(
        &self,
        v: &RcVar,
        path: &'p FieldPath,
    ) -> impl Iterator<Item = FieldPath> + 'p {
        boxed_leaf_paths(&v.ty, self.model.type_env)
            .into_iter()
            .filter(move |leaf| leaf.starts_with(path))
    }

    /// How many of `objects` are the walk's object.
    fn occurrences(&self, objects: &[VarPath]) -> i64 {
        objects.iter().filter(|o| **o == self.object).count() as i64
    }

    /// Adds to `failing` each dependent a use of `v` reads where the bound is `count`, unless the
    /// bound holds there.
    fn check_use(&mut self, v: &RcVar, count: Option<Count>) {
        if count.is_some_and(|c| !c.broken) {
            return;
        }
        let read = roots_under(self.model, &v.name, &v.ty, &vec![]);
        self.failing
            .extend(read.into_iter().filter(|r| self.dependents.contains(r)));
    }
}

/// The rewrite of one body lending the values of `lent`.
struct Lender<'l, 'm> {
    /// What the body's nodes act on.
    model: &'l BodyModel<'m>,
    /// The values the body lends (`lent_values`).
    lent: &'l Set<FullName>,
}

impl Lender<'_, '_> {
    /// `node` with its lent reads lending, the reference counting of lent values dropped, and a
    /// retain before each call taking a lent value at an owning position.
    fn rewrite(&self, node: &RcExprNode) -> RcExprNode {
        grow_stack(|| self.rewrite_inner(node))
    }

    /// One node of `rewrite`, rebuilt over its rewritten continuation.
    fn rewrite_inner(&self, node: &RcExprNode) -> RcExprNode {
        let source = &node.source;
        match node.expr.as_ref() {
            RcExpr::Let(x, RcRhs::Builtin(op, args), k) if self.lent.contains(&x.name) => {
                let op = op
                    .lending()
                    .expect("`lent_values` lends the results of ops that have a lending form");
                RcExprNode::new(
                    RcExpr::Let(x.clone(), RcRhs::Builtin(op, args.clone()), self.rewrite(k)),
                    source.clone(),
                )
            }
            RcExpr::Let(x, rhs @ RcRhs::App(callee, args), k) => {
                // A unit takes one retain per owning position it is passed at, however many leaves
                // it has, so each consumed leaf is counted once per position it is consumed at.
                let mut consumed: Map<VarPath, usize> = Map::default();
                for leaf in self.model.rhs_consumed_leaves(rhs, &x.ty) {
                    *consumed.entry(leaf).or_default() += 1;
                }
                let mut operands: Vec<&RcVar> = vec![];
                for v in std::iter::once(callee).chain(args) {
                    if !operands.iter().any(|seen| seen.name == v.name) {
                        operands.push(v);
                    }
                }
                let mut retains: Vec<(RcVar, FieldPath)> = vec![];
                for v in operands {
                    for unit in rc_units(&v.ty, self.model.type_env) {
                        if !self.counts_lent(v, &unit) {
                            continue;
                        }
                        let positions = boxed_leaf_paths(&v.ty, self.model.type_env)
                            .into_iter()
                            .filter(|leaf| leaf.starts_with(&unit))
                            .map(|leaf| consumed.get(&(v.name.clone(), leaf)).copied().unwrap_or(0))
                            .max()
                            .unwrap_or(0);
                        retains.extend(std::iter::repeat_n((v.clone(), unit.clone()), positions));
                    }
                }
                let call = RcExprNode::new(
                    RcExpr::Let(x.clone(), rhs.clone(), self.rewrite(k)),
                    source.clone(),
                );
                retains.into_iter().rev().fold(call, |k, (arg, unit)| {
                    RcExprNode::new(
                        RcExpr::Retain(arg, unit, RcState::Unknown, k),
                        source.clone(),
                    )
                })
            }
            RcExpr::Destructure(container, fields, state, k)
                if fields.iter().any(|(_, f)| self.lent.contains(&f.name)) =>
            {
                // A lent field is read by the getter lending it, and the destructure retains the
                // rest out of the container.
                let (lent, kept): (Vec<(usize, RcVar)>, Vec<(usize, RcVar)>) = fields
                    .iter()
                    .cloned()
                    .partition(|(_, f)| self.lent.contains(&f.name));
                let mut out = self.rewrite(k);
                if !kept.is_empty() {
                    out = RcExprNode::new(
                        RcExpr::Destructure(container.clone(), kept, *state, out),
                        source.clone(),
                    );
                }
                for (idx, field) in lent.into_iter().rev() {
                    let read = StructGetOp::lending_field(&container.name, idx);
                    out = RcExprNode::new(
                        RcExpr::Let(
                            field,
                            RcRhs::Builtin(Box::new(read), vec![container.clone()]),
                            out,
                        ),
                        source.clone(),
                    );
                }
                out
            }
            RcExpr::Retain(v, path, state, k) | RcExpr::Release(v, path, state, k) => {
                let is_release = matches!(node.expr.as_ref(), RcExpr::Release(..));
                let kept: Vec<FieldPath> = units_under(&v.ty, path, self.model.type_env)
                    .into_iter()
                    .filter(|unit| !self.counts_lent(v, unit))
                    .collect();
                kept.into_iter().rev().fold(self.rewrite(k), |k, unit| {
                    let expr = if is_release {
                        RcExpr::Release(v.clone(), unit, *state, k)
                    } else {
                        RcExpr::Retain(v.clone(), unit, *state, k)
                    };
                    RcExprNode::new(expr, source.clone())
                })
            }
            RcExpr::Let(..) | RcExpr::Destructure(..) | RcExpr::Eval(..) | RcExpr::Ret(..) => {
                map_children(node, |child| self.rewrite(child))
            }
        }
    }

    /// Whether the unit `unit` of `v` counts a lent value. By condition 2 of the module
    /// documentation, a unit that may be a lent value is one on every path.
    fn counts_lent(&self, v: &RcVar, unit: &FieldPath) -> bool {
        roots_under(self.model, &v.name, &v.ty, unit)
            .iter()
            .any(|root| self.lent.contains(root))
    }
}
