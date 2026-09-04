//! The container the symbolic analyses of the RC IR are built on: one fact per boxed leaf of a
//! value, keyed by the leaf's path.
//!
//! Reference counting reaches a value's boxed leaves and nothing else, so an analysis of it states
//! one fact per leaf — where the reference came from, or whether the object is local. The value's
//! type is the sole authority on which paths are leaves, so a `LeafMap` stores no aggregate
//! structure that could disagree with the type, and a value with no boxed leaf is the empty map,
//! whatever its unboxed structure.
//!
//! `LeafKey` is the same content in a canonical order. A specialization key is a function's inputs'
//! facts, so it has to hash and compare by content; the ordered map gives equal shapes an identical
//! hash whatever order they were built in.

use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::constants::CLOSURE_CAPTURE_IDX;
use crate::misc::Map;
use crate::rc_ir::ast::FieldPath;
use std::collections::BTreeMap;
use std::sync::Arc;

/// The paths of the boxed leaves of a value of type `ty` — the field indices from the root of `ty`
/// down to each boxed leaf. A closure lowers to `{funptr, capture-pointer}`, so its one boxed leaf is
/// the capture; a boxed value is a single leaf at the current path; an unboxed aggregate (struct,
/// tuple, or union) recurses into the fields that hold a value (a union's variants' payloads); a
/// fully unboxed value has none. It is the single source of truth for which of a type's paths are
/// boxed leaves.
// PROOF: D/A (dev-docs/proof/rc_ir/borrow-cancel)
pub fn boxed_leaf_paths(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Vec<FieldPath> {
    /// Descend a type, pushing onto `out` the path of each boxed leaf reached. `path` is the field
    /// path from the value's root down to `ty`, which each pushed leaf is named relative to.
    fn go(ty: &Arc<TypeNode>, type_env: &TypeEnv, path: &mut FieldPath, out: &mut Vec<FieldPath>) {
        if ty.is_fully_unboxed(type_env) {
            return;
        }
        if ty.is_closure() {
            path.push(CLOSURE_CAPTURE_IDX as usize);
            out.push(path.clone());
            path.pop();
            return;
        }
        if ty.is_box(type_env) {
            out.push(path.clone());
            return;
        }
        // `Array` is one indivisible boxed leaf at its own path: its `field_types` is the element
        // type, not the storage, so descending would miss the leaf (like `is_box`, stop here).
        if ty.is_array() {
            out.push(path.clone());
            return;
        }
        for (i, fty) in ty.unpunched_field_types(type_env) {
            path.push(i);
            go(&fty, type_env, path, out);
            path.pop();
        }
    }
    let mut out = Vec::new();
    go(ty, type_env, &mut Vec::new(), &mut out);
    out
}

/// One fact of type `T` per boxed leaf of a value, keyed by the leaf's path.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LeafMap<T>(Map<FieldPath, T>);

/// The empty map — a value with no boxed leaf. Available for every fact type `T`, whatever defaults
/// `T` itself has.
impl<T> Default for LeafMap<T> {
    fn default() -> LeafMap<T> {
        LeafMap(Map::default())
    }
}

/// Collect the given leaves, where the caller knows the shape by other means than a type.
impl<T> FromIterator<(FieldPath, T)> for LeafMap<T> {
    fn from_iter<I: IntoIterator<Item = (FieldPath, T)>>(leaves: I) -> LeafMap<T> {
        LeafMap(leaves.into_iter().collect())
    }
}

impl<T: Clone> LeafMap<T> {
    /// A value with no boxed leaf (a scalar or a fieldless aggregate).
    pub fn empty() -> LeafMap<T> {
        LeafMap(Map::default())
    }

    /// The fact of each boxed leaf of a value of type `ty`. `leaf` is called once per boxed leaf,
    /// with that path, so no leaf of the type can be left out.
    pub fn build_shape(
        ty: &Arc<TypeNode>,
        type_env: &TypeEnv,
        leaf: &dyn Fn(&FieldPath) -> T,
    ) -> LeafMap<T> {
        LeafMap(
            boxed_leaf_paths(ty, type_env)
                .into_iter()
                .map(|path| {
                    let fact = leaf(&path);
                    (path, fact)
                })
                .collect(),
        )
    }

    /// The map whose every boxed leaf carries `fact`.
    pub fn uniform(ty: &Arc<TypeNode>, type_env: &TypeEnv, fact: T) -> LeafMap<T> {
        LeafMap::build_shape(ty, type_env, &|_| fact.clone())
    }

    /// The fact recorded at `path`, or `None` where `path` is not a boxed leaf of this value — a
    /// scalar, or an aggregate queried at a non-leaf path such as its root.
    pub fn get(&self, path: &[usize]) -> Option<&T> {
        self.0.get(path)
    }

    /// The fact at `path`, where the caller knows the path names a boxed leaf of the value's type,
    /// which is the sole authority on the shape.
    pub fn leaf_at(&self, path: &[usize]) -> &T {
        self.0
            .get(path)
            .unwrap_or_else(|| unreachable!("{:?} is not a boxed leaf of this value's type", path))
    }

    /// Every boxed leaf's fact, in no particular order.
    pub fn leaves(&self) -> impl Iterator<Item = &T> {
        self.0.values()
    }

    /// The facts of the boxed leaves under `path` — the leaves one reference-counting operation on
    /// that subtree touches. The empty path covers the whole value.
    pub fn leaves_under<'a>(&'a self, path: &'a [usize]) -> impl Iterator<Item = &'a T> {
        self.0
            .iter()
            .filter(move |(leaf_path, _)| leaf_path.starts_with(path))
            .map(|(_, fact)| fact)
    }

    /// The facts of the boxed leaves that descend through none of `fields` — what a `Destructure` of
    /// an unboxed container drops rather than hands out.
    pub fn leaves_outside_fields<'a>(&'a self, fields: &'a [usize]) -> impl Iterator<Item = &'a T> {
        self.0
            .iter()
            .filter(move |(path, _)| match path.split_first() {
                Some((head, _)) => !fields.contains(head),
                None => true,
            })
            .map(|(_, fact)| fact)
    }

    /// Every leaf's path and fact, in no particular order.
    pub fn iter(&self) -> impl Iterator<Item = (&FieldPath, &T)> {
        self.0.iter()
    }

    /// Whether the value has no boxed leaf.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// How many boxed leaves the value has.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The map of field `i` of an unboxed aggregate: the leaves whose path descends through field
    /// `i`, with that head index stripped. A boxed value or a scalar has no such leaf, so it projects
    /// to the empty map.
    pub fn project(&self, i: usize) -> LeafMap<T> {
        LeafMap(
            self.0
                .iter()
                .filter_map(|(path, fact)| match path.split_first() {
                    Some((head, rest)) if *head == i => Some((rest.to_vec(), fact.clone())),
                    _ => None,
                })
                .collect(),
        )
    }

    /// The map carrying `f` of each leaf's path and fact.
    pub fn map_leaves<U>(&self, f: impl Fn(&FieldPath, &T) -> U) -> LeafMap<U> {
        LeafMap(
            self.0
                .iter()
                .map(|(path, fact)| (path.clone(), f(path, fact)))
                .collect(),
        )
    }

    /// The map carrying `f` of each leaf under `path`, and the rest unchanged. The empty path covers
    /// the whole value.
    pub fn map_leaves_under(&self, path: &[usize], f: impl Fn(&T) -> T) -> LeafMap<T> {
        LeafMap(
            self.0
                .iter()
                .map(|(leaf_path, fact)| {
                    let fact = if leaf_path.starts_with(path) {
                        f(fact)
                    } else {
                        fact.clone()
                    };
                    (leaf_path.clone(), fact)
                })
                .collect(),
        )
    }

    /// The pointwise `combine` of two maps of the same shape — a branch merge. `what` names the fact
    /// in the message a shape mismatch aborts with.
    pub fn zip_with(
        &self,
        other: &LeafMap<T>,
        what: &str,
        combine: impl Fn(&T, &T) -> T,
    ) -> LeafMap<T> {
        // Differing paths would leave the result shaped like neither operand's type, which every
        // reader of a leaf takes for granted.
        assert!(
            self.0.len() == other.0.len() && self.0.keys().all(|path| other.0.contains_key(path)),
            "merging the {} of differently shaped values",
            what
        );
        LeafMap(
            self.0
                .iter()
                .map(|(path, fact)| (path.clone(), combine(fact, &other.0[path])))
                .collect(),
        )
    }

    /// The same facts under `f`, in canonical path order, so that the result hashes and compares by
    /// content — what a specialization key needs.
    pub fn to_key<U>(&self, f: impl Fn(&T) -> U) -> LeafKey<U> {
        LeafKey(
            self.0
                .iter()
                .map(|(path, fact)| (path.clone(), f(fact)))
                .collect(),
        )
    }
}

/// One fact of type `T` per boxed leaf, in canonical path order, so that two values of the same
/// shape hash and compare alike whatever order they were built in.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LeafKey<T>(BTreeMap<FieldPath, T>);

/// Collect the given leaves, where the caller knows the shape by other means than a type.
impl<T> FromIterator<(FieldPath, T)> for LeafKey<T> {
    fn from_iter<I: IntoIterator<Item = (FieldPath, T)>>(leaves: I) -> LeafKey<T> {
        LeafKey(leaves.into_iter().collect())
    }
}

impl<T: Copy> LeafKey<T> {
    /// The fact at `path`, where the caller knows the path names a boxed leaf of the value's type.
    pub fn at(&self, path: &[usize]) -> T {
        self.0
            .get(path)
            .copied()
            .unwrap_or_else(|| unreachable!("{:?} is not a boxed leaf of this key", path))
    }

    /// The key of a value of type `ty` whose every boxed leaf carries `fact`.
    pub fn uniform(ty: &Arc<TypeNode>, type_env: &TypeEnv, fact: T) -> LeafKey<T> {
        LeafKey(
            boxed_leaf_paths(ty, type_env)
                .into_iter()
                .map(|path| (path, fact))
                .collect(),
        )
    }
}
