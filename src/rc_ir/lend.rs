//! Lending reads: a value read out of a boxed container is read without a retain where the
//! container's object holds it for as long as the code uses it. Such a value is *lent*: it holds no
//! reference of its own, so the retains and releases of it are removed. Where the code consumes it,
//! by handing it to a callee or an op that owns it, capturing it, or returning it, a retain right
//! before gives the consumer the reference it takes.
//!
//! A read lends its value `a`, read out of the container `c` whose object is `o`, where both of
//! these conditions hold:
//!
//! - *Lent whole*: each unit that may be `a` is lent whole on every path. No unit of any variable
//!   may be `a` on one path and a value this body does not lend on another, and no op consumes some
//!   leaves of such a unit while passing the others through to its result. So each reference-count
//!   node and each consume, which count a unit at a time, either count a lent value or count none.
//! - *Held while used*: `o` stays referenced while the code uses `a`. Walking forward from the read
//!   along every path, a lower bound of `o`'s reference count (`CountBound`) stays at least 1 up to and
//!   including each use of `a` and of its aliases. The bound starts at 1, since the read itself
//!   reads `o`, rises by a retain of a value that is `o` on every path, and falls by each consume
//!   and release of a value that may be `o` on some path (`BodyModel`, the model `cancel` reads).
//!
//! A container that is itself lent holds its value only while its own container is referenced, so
//! the walk runs from the outermost read of the chain. A value read out of a global is held
//! throughout, since a global's objects are never freed.
//!
//! The pass runs after every pass that moves or deletes reference counting, so it reads the
//! counting that code generation emits.

use crate::ast::name::FullName;
use crate::ast::type_env::TypeEnv;
use crate::ast::types::TypeNode;
use crate::misc::{grow_stack, Map, Set};
use crate::rc_ir::ast::{
    for_each_node, map_continuation, FieldPath, RcExpr, RcExprNode, RcProgram, RcRhs, RcState,
    RcVar, VarPath,
};
use crate::rc_ir::borrow::{rc_of_units, rewrite_bodies, BodyModel};
use crate::rc_ir::leaf_map::{boxed_leaf_paths, boxed_leaves_under};
use crate::rc_ir::ownership::{all_owned_units, origin, rc_units, read_container, Origin};
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
    let containers = read_containers(model);
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
                anchor,
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

/// The roots of every unit that some op of `body` consumes unevenly, some of its leaves more often
/// than others.
fn roots_consumed_in_part(model: &BodyModel, body: &RcExprNode) -> Set<FullName> {
    let mut roots = Set::default();
    for_each_node(body, &mut |node| {
        let RcExpr::Let(x, rhs, _) = node.expr.as_ref() else {
            return;
        };
        if matches!(rhs, RcRhs::Match(..)) {
            return;
        }
        let position_counts = count_consuming_positions(model.rhs_consumed_leaves(rhs, &x.ty));
        for v in operands(rhs) {
            for unit in rc_units(&v.ty, model.type_env) {
                if consuming_position_count(model, &position_counts, v, &unit).is_none() {
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
/// container. A fully unboxed value holds no reference and is left out.
fn read_containers(model: &BodyModel) -> Map<FullName, RcVar> {
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
    for_each_node(body, &mut |node| {
        let RcExpr::Let(x, RcRhs::Builtin(op, _), _) = node.expr.as_ref() else {
            return;
        };
        if lent.contains(&x.name) {
            states.insert(x.name.clone(), op.assumed_state());
        }
    });
    states
}

/// How many operand positions consume each leaf. `consumed` lists a leaf once for each position
/// that consumes it.
fn count_consuming_positions(consumed: Vec<VarPath>) -> Map<VarPath, usize> {
    let mut counts = Map::default();
    for leaf in consumed {
        *counts.entry(leaf).or_default() += 1;
    }
    counts
}

/// How many positions consume the unit `unit` of `v`, given how many consume each leaf
/// (`position_counts`). `None` where they consume some leaves of the unit more often than others, so no
/// number of retains of the unit matches them.
fn consuming_position_count(
    model: &BodyModel,
    position_counts: &Map<VarPath, usize>,
    v: &RcVar,
    unit: &FieldPath,
) -> Option<usize> {
    let mut counts = boxed_leaves_under(&v.ty, unit, model.type_env).map(|leaf| {
        position_counts
            .get(&(v.name.clone(), leaf))
            .copied()
            .unwrap_or(0)
    });
    let first = counts.next().unwrap_or_else(|| {
        panic!(
            "the unit {:?} of `{}` has no boxed leaf",
            unit,
            v.name.to_string()
        )
    });
    counts.all(|count| count == first).then_some(first)
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
            match grow_stack(|| anchors_of(model, root, containers, lent, anchors)) {
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

/// The one local object that the boxed `container` is on every path.
///
/// # Returns
/// `None` where `container` may be one of several objects, or is a global's object.
fn sole_object(model: &BodyModel, container: &RcVar) -> Option<VarPath> {
    match model.leaf_objects(&container.name, &[]).as_slice() {
        [object] if object.0.is_local() => Some(object.clone()),
        _ => None,
    }
}

/// The roots of the objects the boxed leaves of the unit `unit` of the variable named `name`, of
/// type `ty`, may belong to. A match binding joining several objects only names them, so it is
/// left out of the roots.
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
struct CountBound {
    /// The bound at this point.
    low: i64,
    /// Whether the bound has fallen below 1 since the read: from there on the object may be freed,
    /// or updated in place by an op consuming its last reference.
    broken: bool,
}

impl CountBound {
    /// The bound at a read out of the object, which the read itself reads.
    fn at_read() -> CountBound {
        CountBound {
            low: 1,
            broken: false,
        }
    }

    /// The bound after the reference count changes by `delta`.
    fn moved(self, delta: i64) -> CountBound {
        let low = self.low + delta;
        CountBound {
            low,
            broken: self.broken || low < 1,
        }
    }

    /// The bound where several paths join: the weakest of their bounds.
    fn weakest(paths: impl Iterator<Item = CountBound>) -> Option<CountBound> {
        paths.reduce(|a, b| CountBound {
            low: a.low.min(b.low),
            broken: a.broken || b.broken,
        })
    }
}

/// The walk of one body checking the *held while used* condition of the module documentation for
/// the values read, directly or through lent containers, out of the container the read of
/// `anchor` reads.
struct Walk<'w, 'm> {
    /// What the body's nodes act on.
    model: &'w BodyModel<'m>,
    /// The object of the container `anchor` is read out of.
    object: VarPath,
    /// The value whose read the walk counts from.
    anchor: &'w FullName,
    /// The lent values the object holds: `anchor` and the values read out of it, transitively.
    dependents: &'w Set<FullName>,
    /// Where the walk adds each dependent used where the bound is broken.
    failing: &'w mut Set<FullName>,
}

impl Walk<'_, '_> {
    /// Walks `node` and its continuation from the bound `count`, `None` before the read, and
    /// returns the bound where the body or arm ends.
    fn walk(&mut self, node: &RcExprNode, count: Option<CountBound>) -> Option<CountBound> {
        grow_stack(|| match node.expr.as_ref() {
            RcExpr::Let(_, RcRhs::Match(scrutinee, arms), k) => {
                self.check_use(scrutinee, count);
                let ends: Vec<Option<CountBound>> =
                    arms.iter().map(|arm| self.walk(&arm.body, count)).collect();
                // An arm the read is not on reaches the continuation with no value of the walk
                // bound, so the continuation goes on from the arms the read is on.
                self.walk(k, CountBound::weakest(ends.into_iter().flatten()))
            }
            RcExpr::Let(x, rhs, k) => {
                let consumed = self.model.rhs_consumed_objects(rhs, &x.ty);
                let count = count.map(|c| c.moved(-self.occurrences(&consumed)));
                for v in operands(rhs) {
                    self.check_use(v, count);
                }
                let count = if x.name == *self.anchor {
                    Some(CountBound::at_read())
                } else {
                    count
                };
                self.walk(k, count)
            }
            RcExpr::DestructureUnboxed(container, _, k) => {
                self.check_use(container, count);
                self.walk(k, count)
            }
            RcExpr::Retain(v, path, _, k) => {
                // A retain of a union retains the leaves of the variant it holds, which may not be
                // the leaf that is the object, so only a unit of one leaf is certain to retain it.
                let leaves: Vec<FieldPath> =
                    boxed_leaves_under(&v.ty, path, self.model.type_env).collect();
                let certain = match leaves.as_slice() {
                    [leaf] => {
                        (self.model.leaf_objects(&v.name, leaf) == slice::from_ref(&self.object))
                            as i64
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
    fn check_use(&mut self, v: &RcVar, count: Option<CountBound>) {
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
                    RcRhs::Builtin(op.lending(), args.clone())
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
            RcExpr::Retain(v, path, state, k) | RcExpr::Release(v, path, state, k) => {
                let is_release = matches!(node.expr.as_ref(), RcExpr::Release(..));
                let k = self.rewrite(k, returns_from_func);
                let type_env = self.model.type_env;
                rc_of_units(v, path, *state, is_release, k, source, type_env, |unit| {
                    !self.counts_lent(v, unit)
                })
            }
            RcExpr::DestructureUnboxed(..) | RcExpr::Eval(..) => {
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
        let position_counts = count_consuming_positions(consumed);
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
                let count = consuming_position_count(self.model, &position_counts, v, &unit)
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

    /// Whether the unit `unit` of `v` counts a lent value. By the *lent whole* condition of the
    /// module documentation, a unit that may be a lent value is one on every path.
    fn counts_lent(&self, v: &RcVar, unit: &FieldPath) -> bool {
        roots_under(self.model, &v.name, &v.ty, unit)
            .iter()
            .any(|root| self.lent.contains(root))
    }
}

#[cfg(test)]
mod tests {
    use super::lend_reads;
    use crate::ast::name::FullName;
    use crate::ast::type_env::TypeEnv;
    use crate::ast::typedecl::Field;
    use crate::ast::types::{
        kind_star, tycon, type_arrow, type_tyapp, type_tycon, TyCon, TyConInfo, TyConVariant,
        TypeNode,
    };
    use crate::fixstd::builtin::{builtin_tycons, make_array_ty, make_i64_ty, StructGetOp};
    use crate::misc::{Map, Set};
    use crate::rc_ir::ast::{
        FuncRef, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcState, RcVar,
    };
    use std::sync::Arc;

    /// `Array I64`, a boxed value.
    fn array_ty() -> Arc<TypeNode> {
        type_tyapp(make_array_ty(), make_i64_ty())
    }

    /// `Test::Holder`, the boxed struct of one `Array I64` field that `type_env` declares.
    fn holder_ty() -> Arc<TypeNode> {
        type_tycon(&tycon(FullName::from_strs(&["Test"], "Holder")))
    }

    /// A type environment holding the built-in type constructors and `Test::Holder`.
    fn type_env() -> TypeEnv {
        let mut tycons = builtin_tycons();
        tycons.insert(
            TyCon::new(FullName::from_strs(&["Test"], "Holder")),
            TyConInfo {
                kind: kind_star(),
                variant: TyConVariant::Struct,
                is_unbox: false,
                tyvars: vec![],
                fields: vec![Field::make("held".to_string(), array_ty(), None)],
                source: None,
                document: None,
                punched_from: None,
            },
        );
        TypeEnv::new(tycons, Map::default())
    }

    /// A local variable of the given type.
    fn var_of(name: &str, ty: Arc<TypeNode>) -> RcVar {
        RcVar {
            name: FullName::local(name),
            ty,
            source: None,
            debug_name: None,
            skip_null_check: false,
        }
    }

    /// An expression node carrying no source span.
    fn node(expr: RcExpr) -> RcExprNode {
        RcExprNode::new(expr, None)
    }

    /// `let x = <getter of field 0>(c); k`, where `c` is a `Test::Holder` and `x` its array.
    fn read_then(k: RcExprNode) -> RcExprNode {
        let read = StructGetOp::of_field(&FullName::local("c"), 0);
        node(RcExpr::Let(
            var_of("x", array_ty()),
            RcRhs::Builtin(Box::new(read), vec![var_of("c", holder_ty())]),
            k,
        ))
    }

    /// The body `lend_reads` leaves of the function `f(c : Test::Holder)` whose body is `body`.
    fn lent_body(body: RcExprNode, ret_ty: Arc<TypeNode>) -> RcExprNode {
        let name = FuncRef {
            name: FullName::local("f"),
        };
        let func = RcFunc {
            name: name.clone(),
            fn_ty: type_arrow(holder_ty(), ret_ty.clone()),
            params: vec![var_of("c", holder_ty())],
            capture: None,
            ret_ty,
            body,
            source: None,
            borrowed_units: Set::default(),
            inline_into_callers: false,
        };
        let prog = RcProgram {
            funcs: [(name.clone(), func)].into_iter().collect(),
            globals: vec![],
            roots: [name.name.clone()].into_iter().collect(),
            retains_sunk: false,
        };
        lend_reads(&prog, &type_env()).funcs[&name].body.clone()
    }

    /// Whether the read `read_then` opens `body` with lends its value.
    fn read_lends(body: &RcExprNode) -> bool {
        match body.expr.as_ref() {
            RcExpr::Let(_, RcRhs::Builtin(op, _), _) => op.lends(),
            _ => panic!("the body opens with the read"),
        }
    }

    /// A lent value a closure captures is retained right before the capture, which takes a
    /// reference as a call owning its argument does.
    #[test]
    fn retains_a_lent_value_before_a_closure_captures_it() {
        // let x = c.0; let g = closure target[x]; release c; ret g
        let closure_ty = type_arrow(make_i64_ty(), make_i64_ty());
        let target = FuncRef {
            name: FullName::local("target"),
        };
        let body = read_then(node(RcExpr::Let(
            var_of("g", closure_ty.clone()),
            RcRhs::Closure(target, vec![var_of("x", array_ty())]),
            node(RcExpr::Release(
                var_of("c", holder_ty()),
                vec![],
                RcState::Unknown,
                node(RcExpr::Ret(var_of("g", closure_ty.clone()))),
            )),
        )));
        let lent = lent_body(body, closure_ty);
        assert!(read_lends(&lent), "the read should lend the captured array");
        let RcExpr::Let(_, _, after_read) = lent.expr.as_ref() else {
            unreachable!("`read_lends` saw the read");
        };
        assert!(
            matches!(
                after_read.expr.as_ref(),
                RcExpr::Retain(v, _, _, k)
                    if v.name == FullName::local("x")
                        && matches!(k.expr.as_ref(), RcExpr::Let(_, RcRhs::Closure(..), _))
            ),
            "a retain of the lent array should stand right before the capture"
        );
    }

    /// A value used after its container's count may have reached zero keeps its own reference,
    /// though a retain raises the count again before the use: the retain restores neither the
    /// object nor what it held.
    #[test]
    fn keeps_the_reference_of_a_value_used_after_its_container_may_have_been_freed() {
        // let x = c.0; release c; retain c; ret x
        let c = || var_of("c", holder_ty());
        let body = read_then(node(RcExpr::Release(
            c(),
            vec![],
            RcState::Unknown,
            node(RcExpr::Retain(
                c(),
                vec![],
                RcState::Unknown,
                node(RcExpr::Ret(var_of("x", array_ty()))),
            )),
        )));
        let lent = lent_body(body, array_ty());
        assert!(
            !read_lends(&lent),
            "the read should retain the array returned after the container's count reached zero"
        );
    }
}
