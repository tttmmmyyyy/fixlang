//! The identity a `TypeNode` carries: what `Hash` and `PartialEq` agree on, and what the answers
//! kept on a node are worth once the node holds a different type expression.

use crate::ast::name::FullName;
use crate::ast::types::{tycon, type_tyapp, type_tycon, type_tyvar_star, TypeNode};
use crate::misc::Map;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// The hash a map would take of this type.
fn hash_of(ty: &Arc<TypeNode>) -> u64 {
    let mut hasher = DefaultHasher::new();
    Hash::hash(&**ty, &mut hasher);
    hasher.finish()
}

/// A nullary type constructor to build test types out of.
fn con(name: &str) -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&["Test"], name)))
}

/// A type built twice from separate nodes is one type and hashes alike, so either serves as the
/// other's key in a memo table.
#[test]
fn test_types_built_from_separate_nodes_are_one_key() {
    let build = || type_tyapp(type_tyapp(con("Pair"), con("I64")), con("Bool"));
    let lhs = build();
    let rhs = build();
    assert!(!Arc::ptr_eq(&lhs, &rhs));
    assert_eq!(lhs, rhs);
    assert_eq!(hash_of(&lhs), hash_of(&rhs));

    let mut memo: Map<Arc<TypeNode>, u32> = Map::default();
    memo.insert(lhs.clone(), 1);
    assert_eq!(memo.get(&rhs).copied(), Some(1));
}

/// A node holding one child in both of its positions is the same type as one holding two separate
/// children of that shape, and hashes alike.
#[test]
fn test_a_shared_child_is_the_same_type_as_two_separate_ones() {
    let shared = con("I64");
    let shared_both = type_tyapp(type_tyapp(con("Pair"), shared.clone()), shared);
    let separate = type_tyapp(type_tyapp(con("Pair"), con("I64")), con("I64"));
    assert!(!Arc::ptr_eq(&shared_both, &separate));
    assert_eq!(shared_both, separate);
    assert_eq!(hash_of(&shared_both), hash_of(&separate));
}

/// A node built by replacing the type expression of another answers for the expression it now
/// holds, whatever was asked of the node it was built from.
#[test]
fn test_replacing_a_types_expression_drops_the_answers_kept_on_it() {
    let ty = type_tyapp(con("Pair"), con("I64"));
    // Ask everything that a node keeps, so that a stale answer would be there to find.
    let hash_before = hash_of(&ty);
    assert!(ty.is_ground());
    assert_eq!(ty.depth(), 2);

    let opened = ty.set_tyapp_arg(type_tyvar_star("a"));
    assert_ne!(hash_of(&opened), hash_before);
    assert_eq!(
        hash_of(&opened),
        hash_of(&type_tyapp(con("Pair"), type_tyvar_star("a")))
    );
    assert!(!opened.is_ground());
    assert_eq!(opened.depth(), 2);

    let deepened = ty.set_tyapp_arg(type_tyapp(con("Array"), con("I64")));
    assert_eq!(
        hash_of(&deepened),
        hash_of(&type_tyapp(
            con("Pair"),
            type_tyapp(con("Array"), con("I64"))
        ))
    );
    assert!(deepened.is_ground());
    assert_eq!(deepened.depth(), 3);

    // The node the two were built from still answers for its own expression.
    assert_eq!(hash_of(&ty), hash_before);
    assert!(ty.is_ground());
    assert_eq!(ty.depth(), 2);
}

/// `is_ground` answers what collecting the free type variables answers.
#[test]
fn test_is_ground_agrees_with_free_vars() {
    let ground = type_tyapp(type_tyapp(con("Pair"), con("I64")), con("Bool"));
    assert!(ground.is_ground());
    assert!(ground.free_vars().is_empty());

    let open = type_tyapp(type_tyapp(con("Pair"), con("I64")), type_tyvar_star("a"));
    assert!(!open.is_ground());
    assert_eq!(open.free_vars().len(), 1);
}

/// The depth of a type is the depth of its expression: a chain of applications one level per link,
/// and a shared child counted once however many positions hold it.
#[test]
fn test_depth_measures_the_type_expression() {
    assert_eq!(con("I64").depth(), 1);
    assert_eq!(type_tyapp(con("Array"), con("I64")).depth(), 2);

    let shared = type_tyapp(con("Array"), con("I64"));
    let pair_of_shared = type_tyapp(type_tyapp(con("Pair"), shared.clone()), shared);
    assert_eq!(pair_of_shared.depth(), 4);
}
