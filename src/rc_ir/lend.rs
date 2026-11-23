//! Lending reads: a value read out of a boxed container is read without a retain where the
//! container's object holds it for as long as the code uses it. Such a value is *lent*: it holds no
//! reference of its own, so its reference counting goes, and where the code consumes it, by handing
//! it to a callee or an op that owns it, capturing it, or returning it, a retain right before gives
//! the consumer the reference it takes.
//!
//! A read lends its value `a`, read out of the container `c` whose object is `o`, where
//!
//! 1. each unit that may be `a` is lent whole on every path: no unit of any variable may be `a` on
//!    one path and a value this body does not lend on another, and no op consumes some leaves of
//!    such a unit while passing the others through to its result. So each reference-count node and
//!    each consume, which count a unit at a time, either count a lent value or count none;
//! 2. `o` stays referenced while the code uses `a`: walking forward from the read along every
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
    for_each_node, map_continuation, FieldPath, RcExpr, RcExprNode, RcProgram, RcRhs, RcState,
    RcVar, VarPath,
};
use crate::rc_ir::borrow::{rewrite_bodies, BodyModel};
use crate::rc_ir::leaf_map::boxed_leaf_paths;
use crate::rc_ir::ownership::{
    all_owned_units, origin, rc_units, read_container, units_under, Origin,
};
use std::iter;
use std::slice;
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
            states: read_states(body, &lent),
        }
        .rewrite(body, true)
    })
}

/// The values `body` reads out of a boxed container and can lend: the conditions of the module
/// documentation, checked again until the set is stable, since a value taken out of the set can
/// fail a condition another value passed.
fn lent_values(model: &BodyModel, body: &RcExprNode) -> Set<FullName> {
    let containers = reads(model);
    if containers.is_empty() {
        return Set::default();
    }
    let consumed_in_part = roots_consumed_in_part(model, body);
    let mut lent: Set<FullName> = containers
        .keys()
        .filter(|value| !consumed_in_part.contains(*value))
        .cloned()
        .collect();
    loop {
        let mut failing = lent_beside_unlent(model, &lent);
        let mut anchors: Map<FullName, Option<Set<FullName>>> = Map::default();
        for value in &lent {
            anchors_of(model, value, &containers, &lent, &mut anchors);
        }
        let mut dependents: Map<FullName, Set<FullName>> = Map::default();
        for (value, value_anchors) in &anchors {
            match value_anchors {
                None => {
                    failing.insert(value.clone());
                }
                Some(value_anchors) => {
                    for anchor in value_anchors {
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

/// The roots of the units an op of `body` consumes some leaves of more often than others.
fn roots_consumed_in_part(model: &BodyModel, body: &RcExprNode) -> Set<FullName> {
    let mut roots = Set::default();
    for_each_node(body, &mut |node| {
        let RcExpr::Let(x, rhs, _) = node.expr.as_ref() else {
            return;
        };
        if matches!(rhs, RcRhs::Match(..)) {
            return;
        }
        let positions = positions(model.rhs_consumed_leaves(rhs, &x.ty));
        for v in operands(rhs) {
            for unit in rc_units(&v.ty, model.type_env) {
                if consumed_positions(model, &positions, v, &unit).is_none() {
                    roots.extend(roots_under(model, &v.name, &v.ty, &unit));
                }
            }
        }
    });
    roots
}

/// The values of `lent` that some unit of a variable may be on one path while it may be a value
/// outside `lent` on another.
fn lent_beside_unlent(model: &BodyModel, lent: &Set<FullName>) -> Set<FullName> {
    let mut beside = Set::default();
    for (name, ty) in &model.vars.var_tys {
        for unit in rc_units(ty, model.type_env) {
            let roots = roots_under(model, name, ty, &unit);
            if roots.iter().any(|r| lent.contains(r)) && roots.iter().any(|r| !lent.contains(r)) {
                beside.extend(roots.into_iter().filter(|r| lent.contains(r)));
            }
        }
    }
    beside
}

/// The values a body reads out of a boxed container (`read_container`), each mapped to that
/// container. A fully unboxed value holds no reference, so none of those is among them.
fn reads(model: &BodyModel) -> Map<FullName, RcVar> {
    model
        .vars
        .var_tys
        .iter()
        .filter(|(_, ty)| !ty.is_fully_unboxed(model.type_env))
        .filter_map(|(name, _)| {
            let container = read_container(model.vars, model.type_env, name)?;
            Some((name.clone(), container.clone()))
        })
        .collect()
}

/// The state each read of a value of `lent` retains its value in.
fn read_states(body: &RcExprNode, lent: &Set<FullName>) -> Map<FullName, RcState> {
    let mut states = Map::default();
    for_each_node(body, &mut |node| match node.expr.as_ref() {
        RcExpr::Let(x, RcRhs::Builtin(op, _), _) if lent.contains(&x.name) => {
            let state = if op.assumes_local() {
                RcState::Local
            } else {
                RcState::Unknown
            };
            states.insert(x.name.clone(), state);
        }
        RcExpr::Destructure(_, fields, state, _) => {
            for (_, field) in fields {
                if lent.contains(&field.name) {
                    states.insert(field.name.clone(), *state);
                }
            }
        }
        _ => {}
    });
    states
}

/// How many positions consume each of the leaves `consumed` lists once per position.
fn positions(consumed: Vec<VarPath>) -> Map<VarPath, usize> {
    let mut positions = Map::default();
    for leaf in consumed {
        *positions.entry(leaf).or_default() += 1;
    }
    positions
}

/// How many positions consume the unit `unit` of `v`, given how many consume each leaf
/// (`positions`). `None` where they consume some leaves of the unit more often than others, so no
/// number of retains of the unit matches them.
fn consumed_positions(
    model: &BodyModel,
    positions: &Map<VarPath, usize>,
    v: &RcVar,
    unit: &FieldPath,
) -> Option<usize> {
    let mut counts = boxed_leaves_under(&v.ty, unit, model.type_env)
        .map(|leaf| positions.get(&(v.name.clone(), leaf)).copied().unwrap_or(0));
    let first = counts.next().unwrap_or_else(|| {
        panic!(
            "the unit {:?} of `{}` has no boxed leaf",
            unit,
            v.name.to_string()
        )
    });
    counts.all(|count| count == first).then_some(first)
}

/// The boxed leaves of a value of type `ty` under `path`.
fn boxed_leaves_under<'p>(
    ty: &Arc<TypeNode>,
    path: &'p [usize],
    type_env: &TypeEnv,
) -> impl Iterator<Item = FieldPath> + 'p {
    boxed_leaf_paths(ty, type_env)
        .into_iter()
        .filter(move |leaf| leaf.starts_with(path))
}

/// The variables a right-hand side other than a `Match` reads.
fn operands(rhs: &RcRhs) -> Vec<&RcVar> {
    match rhs {
        RcRhs::Var(y) => vec![y],
        RcRhs::App(callee, args) => iter::once(callee).chain(args).collect(),
        RcRhs::Closure(_, caps) => caps.iter().collect(),
        RcRhs::Builtin(_, args) => args.iter().collect(),
        RcRhs::Match(..) => unreachable!("a match is walked by its arms"),
    }
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
        let mut container_anchors = Set::default();
        let mut held = true;
        for root in &roots {
            match anchors_of(model, root, containers, lent, anchors) {
                Some(more) => container_anchors.extend(more),
                None => held = false,
            }
        }
        held.then_some(container_anchors)
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
/// type `ty`, may belong to. A match binding joining several objects is a name for them and no
/// object itself, so it is not among the roots.
fn roots_under(
    model: &BodyModel,
    name: &FullName,
    ty: &Arc<TypeNode>,
    unit: &FieldPath,
) -> Set<FullName> {
    boxed_leaves_under(ty, unit, model.type_env)
        .flat_map(|leaf| model.leaf_objects(name, &leaf))
        .filter(|(root, path)| {
            matches!(
                origin(model.vars, model.type_env, root, path),
                Origin::Exactly(_)
            )
        })
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
    /// The bound at a read out of the object, which the read itself reads.
    fn at_read() -> Count {
        Count {
            low: 1,
            broken: false,
        }
    }

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

/// The walk of one body checking condition 2 of the module documentation for the values read,
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
                for v in operands(rhs) {
                    self.check_use(v, count);
                }
                let count = if x.name == *self.read {
                    Some(Count::at_read())
                } else {
                    count
                };
                self.walk(k, count)
            }
            RcExpr::Destructure(container, fields, _, k) => {
                self.check_use(container, count);
                let count = if fields.iter().any(|(_, f)| f.name == *self.read) {
                    Some(Count::at_read())
                } else {
                    count
                };
                self.walk(k, count)
            }
            RcExpr::Retain(v, path, _, k) => {
                // A retain of a union retains the leaves of the variant it holds, which may not be
                // the leaf that is the object, so only a unit of one leaf is certain to retain it.
                let leaves: Vec<FieldPath> =
                    boxed_leaves_under(&v.ty, path, self.model.type_env).collect();
                let certain = match leaves.as_slice() {
                    [leaf] => {
                        (self.model.leaf_objects(&v.name, leaf)
                            == slice::from_ref(&self.object)) as i64
                    }
                    _ => 0,
                };
                self.walk(k, count.map(|c| c.moved(certain)))
            }
            RcExpr::Release(v, path, _, k) => {
                let possible = boxed_leaves_under(&v.ty, path, self.model.type_env)
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
        let used = roots_under(self.model, &v.name, &v.ty, &vec![]);
        self.failing
            .extend(used.into_iter().filter(|r| self.dependents.contains(r)));
    }
}

/// The rewrite of one body lending the values of `lent`.
struct Lender<'l, 'm> {
    /// What the body's nodes act on.
    model: &'l BodyModel<'m>,
    /// The values the body lends (`lent_values`).
    lent: &'l Set<FullName>,
    /// The state each lent value's read retained it in (`read_states`).
    states: Map<FullName, RcState>,
}

impl Lender<'_, '_> {
    /// `node` with its lent reads lending, the reference counting of lent values dropped, and a
    /// retain before each consume of a lent value. `returns_from_func` says whether the `Ret` ending
    /// `node` returns from the body, which consumes the value; the `Ret` ending an arm of a match
    /// hands the value to the variable the match binds, an alias.
    fn rewrite(&self, node: &RcExprNode, returns_from_func: bool) -> RcExprNode {
        grow_stack(|| self.rewrite_inner(node, returns_from_func))
    }

    /// One node of `rewrite`, rebuilt over its rewritten continuation.
    fn rewrite_inner(&self, node: &RcExprNode, returns_from_func: bool) -> RcExprNode {
        let source = &node.source;
        match node.expr.as_ref() {
            RcExpr::Let(x, RcRhs::Match(scrutinee, arms), k) => {
                let arms = arms
                    .iter()
                    .map(|arm| arm.with_body(self.rewrite(&arm.body, false)))
                    .collect();
                RcExprNode::new(
                    RcExpr::Let(
                        x.clone(),
                        RcRhs::Match(scrutinee.clone(), arms),
                        self.rewrite(k, returns_from_func),
                    ),
                    source.clone(),
                )
            }
            RcExpr::Let(x, rhs, k) => {
                let new_rhs = if self.lent.contains(&x.name) {
                    let RcRhs::Builtin(op, args) = rhs else {
                        unreachable!(
                            "the lent `{}` is bound by something other than an op",
                            x.name.to_string()
                        )
                    };
                    let op = op
                        .lending()
                        .expect("`lent_values` lends the results of ops that have a lending form");
                    RcRhs::Builtin(op, args.clone())
                } else {
                    rhs.clone()
                };
                let bound = RcExprNode::new(
                    RcExpr::Let(x.clone(), new_rhs, self.rewrite(k, returns_from_func)),
                    source.clone(),
                );
                let consumed = self.model.rhs_consumed_leaves(rhs, &x.ty);
                self.retained_before(bound, &operands(rhs), consumed)
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
                let mut out = self.rewrite(k, returns_from_func);
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
                kept.into_iter().rev().fold(self.rewrite(k, returns_from_func), |k, unit| {
                    let expr = if is_release {
                        RcExpr::Release(v.clone(), unit, *state, k)
                    } else {
                        RcExpr::Retain(v.clone(), unit, *state, k)
                    };
                    RcExprNode::new(expr, source.clone())
                })
            }
            RcExpr::Destructure(..) | RcExpr::Eval(..) => {
                map_continuation(node, |k| self.rewrite(k, returns_from_func))
            }
            RcExpr::Ret(v) if returns_from_func => {
                let consumed = boxed_leaf_paths(&v.ty, self.model.type_env)
                    .into_iter()
                    .map(|leaf| (v.name.clone(), leaf))
                    .collect();
                self.retained_before(node.clone(), &[v], consumed)
            }
            RcExpr::Ret(_) => node.clone(),
        }
    }

    /// `node`, which consumes the leaves `consumed` of `operands`, after a retain of each lent unit
    /// among them: one retain per position consuming the unit, however many leaves it has.
    fn retained_before(
        &self,
        node: RcExprNode,
        operands: &[&RcVar],
        consumed: Vec<VarPath>,
    ) -> RcExprNode {
        let positions = positions(consumed);
        let mut seen: Set<&FullName> = Set::default();
        let mut retains: Vec<(RcVar, FieldPath, RcState)> = vec![];
        for v in operands {
            if !seen.insert(&v.name) {
                continue;
            }
            for unit in rc_units(&v.ty, self.model.type_env) {
                let roots = roots_under(self.model, &v.name, &v.ty, &unit);
                if !roots.iter().any(|root| self.lent.contains(root)) {
                    continue;
                }
                let count = consumed_positions(self.model, &positions, v, &unit)
                    .expect("`lent_values` lends no unit consumed in part");
                // The retain counts the reference the read counted, in the state the read
                // counted it in where every read the unit may be agrees on one.
                let mut states = roots.iter().map(|root| self.states[root]);
                let first = states.next().expect("a lent unit has a root");
                let state = if states.all(|state| state == first) {
                    first
                } else {
                    RcState::Unknown
                };
                retains.extend(iter::repeat_n(((*v).clone(), unit.clone(), state), count));
            }
        }
        let source = node.source.clone();
        retains.into_iter().rev().fold(node, |k, (v, unit, state)| {
            RcExprNode::new(RcExpr::Retain(v, unit, state, k), source.clone())
        })
    }

    /// Whether the unit `unit` of `v` counts a lent value. By condition 1 of the module
    /// documentation, a unit that may be a lent value is one on every path.
    fn counts_lent(&self, v: &RcVar, unit: &FieldPath) -> bool {
        roots_under(self.model, &v.name, &v.ty, unit)
            .iter()
            .any(|root| self.lent.contains(root))
    }
}
