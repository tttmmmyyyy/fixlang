//! Whether a type has a size, and what to report when it has none.
//!
//! Two things can go wrong, and the walk here reports both.
//!
//! **A value that contains itself.** The size of an unboxed value is the sum of its fields' sizes,
//! so deciding it descends into every field laid out in place. A pointer is a pointer whatever it
//! points at, so the descent stops at a boxed field. A type the descent is already inside would
//! have to be larger than itself.
//!
//! **A type reached from itself at a larger type argument.** The code generator lays out one object
//! per type, so a type behind a pointer needs a layout of its own. `P (a, a)` reached from `P a`
//! leads to a type that is larger again at every step and never repeats, so the program needs
//! endlessly many objects even though every one of them has a size.
//!
//! The first is decided exactly. The second is bounded rather than decided: no exact criterion is
//! available — the question is undecidable in general, and the decidable fragments in the
//! literature (Kennedy and Pierce's expansive recursion, which .NET applies to generics) exclude the
//! higher-kinded type parameters Fix has. What is bounded is the depth of a single type
//! (`MAX_TYPE_DEPTH`), which is what such a family grows.
//!
//! Bounding the depth of one type, rather than the number of types or the length of the descent,
//! is what keeps the bound off ordinary programs: adding modules to a project adds types and
//! lengthens descents, but the types a program writes stay as deep as they were. A bound on a
//! program-sized quantity would make an ordinary source file the thing that stops a project from
//! compiling.
//!
//! The bound also settles termination: over a finite set of type constructors there are finitely
//! many types of bounded depth, so the walk runs out of new types to visit.
//!
//! The check runs over an instantiated program, before code generation walks the fields of any
//! type.

use crate::ast::program::TypeEnv;
use crate::ast::types::{TypeNode, MAX_TYPE_DEPTH};
use crate::misc::{grow_stack, shorten_for_report, Set};
use crate::object::{ty_to_object_ty, ObjectFieldType};
use std::sync::Arc;

/// What the walk carries from one root to the next, so that a type is answered once however many
/// values carry it.
#[derive(Default)]
pub struct LayoutWalk {
    /// Types whose in-place descent completed. Recorded on the way out, so that a type the descent
    /// is still inside is not mistaken for one already answered.
    settled: Set<Arc<TypeNode>>,
    /// Types the program needs an object for. Recorded on arrival: meeting one again needs no
    /// second answer.
    reached: Set<Arc<TypeNode>>,
}

/// Why a value of `ty` has no size, and `None` where the code generator can lay one out.
pub fn no_size_reason(
    ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    walk: &mut LayoutWalk,
) -> Option<String> {
    // A function type at a root is one the program has a value of, so that function is compiled and
    // the types it takes and returns are laid out with it. A function type reached as a field is
    // not: a value of it may never be built, and its own layout is two pointers either way.
    if ty.is_closure() || ty.is_funptr() {
        return ty
            .get_lambda_srcs()
            .into_iter()
            .chain([ty.get_lambda_dst()])
            .find_map(|signature_ty| {
                no_size_reachable(&signature_ty, &signature_ty, type_env, walk, &mut vec![])
            });
    }
    no_size_reachable(ty, ty, type_env, walk, &mut vec![])
}

/// Walk the types the program needs an object for, deciding at each one whether its size settles.
///
/// `root` is the type the walk started from, which the report names where the type at fault is one
/// the walk built on the way: printing that one would print a term as deep as the bound.
fn no_size_reachable(
    root: &Arc<TypeNode>,
    ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    walk: &mut LayoutWalk,
    asked_for: &mut Vec<Arc<TypeNode>>,
) -> Option<String> {
    if !walk.reached.insert(ty.clone()) {
        return None;
    }
    // A function value is a pair of pointers whatever it takes and returns, so how deeply the
    // function type itself nests says nothing about a layout: a function of five hundred arguments
    // nests five hundred deep and is still two pointers.
    if !ty.is_closure() && !ty.is_funptr() && ty.depth() > MAX_TYPE_DEPTH {
        return Some(depth_message(root, asked_for));
    }
    if let Some(msg) = no_size_in_place(root, ty, type_env, &mut vec![], &mut Set::default(), walk)
    {
        return Some(msg);
    }
    // The descent is as deep as the types the program holds, which is deeper than a thread's stack.
    asked_for.push(ty.clone());
    let reason = grow_stack(|| {
        held_types(ty, type_env)
            .iter()
            .find_map(|held_ty| no_size_reachable(root, held_ty, type_env, walk, asked_for))
    });
    asked_for.pop();
    reason
}

/// Whether the size of `ty` settles, descending into the fields laid out in place.
///
/// `path` is the types the descent is inside, outermost first, and `on_path` is the same types as a
/// set. A type met twice on one path is a value that contains itself. `root` names the report where
/// the descent reaches a type too deep to be one the program wrote.
fn no_size_in_place(
    root: &Arc<TypeNode>,
    ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    path: &mut Vec<Arc<TypeNode>>,
    on_path: &mut Set<Arc<TypeNode>>,
    walk: &mut LayoutWalk,
) -> Option<String> {
    if on_path.contains(ty) {
        return Some(format!(
            "`{}` has no size: its unboxed fields reach `{}` itself{}. Make one of these types \
             boxed.",
            ty.to_string(),
            ty.to_string(),
            way_down(path, ty)
        ));
    }
    if walk.settled.contains(ty) {
        return None;
    }
    // Two pointers, settled, whatever the function takes and returns.
    if ty.is_closure() || ty.is_funptr() {
        return None;
    }
    if ty.depth() > MAX_TYPE_DEPTH {
        return Some(depth_message(root, path));
    }
    let in_place_tys: Vec<Arc<TypeNode>> = held_types(ty, type_env)
        .into_iter()
        .filter(|held_ty| held_ty.is_unbox(type_env))
        .collect();

    path.push(ty.clone());
    on_path.insert(ty.clone());
    let reason = grow_stack(|| {
        in_place_tys
            .iter()
            .find_map(|held_ty| no_size_in_place(root, held_ty, type_env, path, on_path, walk))
    });
    on_path.remove(ty);
    path.pop();

    if reason.is_none() {
        walk.settled.insert(ty.clone());
    }
    reason
}

/// The report for a type whose layout reaches types deeper than the bound.
///
/// It names the type the walk started from and the first few types that one asked for, which is
/// where a type reached from itself at a larger argument shows itself. The type actually at fault
/// is one the walk built on the way, and printing that one would print a term as deep as the bound.
fn depth_message(root: &Arc<TypeNode>, asked_for: &[Arc<TypeNode>]) -> String {
    fn shorten(ty: &Arc<TypeNode>) -> String {
        shorten_for_report(ty.to_string())
    }

    /// How many steps of the way down to show. Enough for one turn of a growing family to be
    /// visible, and short enough that the types printed are ones the reader can read.
    const SHOWN_STEPS: usize = 3;

    let steps = asked_for
        .iter()
        .take(SHOWN_STEPS)
        .map(|ty| format!("`{}`", shorten(ty)))
        .collect::<Vec<_>>();
    let way_down = if steps.is_empty() {
        String::new()
    } else {
        format!(" ({} -> ...)", steps.join(" -> "))
    };
    format!(
        "`{}` has no size: laying it out asks for types nested more than {} deep{}, so it needs \
         endlessly many. A type reached from itself at a larger type argument does this; give the \
         recursive occurrence the same type arguments.",
        shorten(root),
        MAX_TYPE_DEPTH,
        way_down
    )
}

/// The types a value of `ty` holds: the fields of a struct, the payloads of a union, and the
/// elements an array storage holds. Read from the object the code generator builds, so that the
/// walk asks for a layout of exactly what the code generator asks for one of.
fn held_types(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Vec<Arc<TypeNode>> {
    let mut held = vec![];
    for field in ty_to_object_ty(ty, &vec![], type_env).field_types {
        match field {
            ObjectFieldType::SubObject(field_ty, _is_punched) => held.push(field_ty),
            ObjectFieldType::UnionBuf(payload_tys) => held.extend(payload_tys),
            ObjectFieldType::ArrayStorageBuf(elem_ty) => held.push(elem_ty),
            // A lambda field is a pointer to compiled code, and the types that function takes and
            // returns are laid out where it is compiled.
            ObjectFieldType::LambdaFunction(_) => {}
            ObjectFieldType::Array(_) => {
                unreachable!("an object holds its elements as an `ArrayStorageBuf` field")
            }
            // The rest carry no Fix type. Listing them keeps this match exhaustive, so a field kind
            // added later is answered here as well.
            ObjectFieldType::ControlBlock
            | ObjectFieldType::TraverseFunction
            | ObjectFieldType::Ptr
            | ObjectFieldType::I8
            | ObjectFieldType::U8
            | ObjectFieldType::I16
            | ObjectFieldType::U16
            | ObjectFieldType::I32
            | ObjectFieldType::U32
            | ObjectFieldType::I64
            | ObjectFieldType::U64
            | ObjectFieldType::F32
            | ObjectFieldType::F64
            | ObjectFieldType::UnionTag => {}
        }
    }
    held
}

/// The way down from the type that shows the defect to the type that has none, spelled out only
/// where it passes through another type — a type holding itself directly is the whole story.
fn way_down(path: &[Arc<TypeNode>], repeated_ty: &Arc<TypeNode>) -> String {
    // The caller reports a way down only for a type the descent is inside, so `repeated_ty` is on
    // `path`.
    let start = path
        .iter()
        .position(|ancestor| ancestor == repeated_ty)
        .unwrap_or_else(|| {
            panic!(
                "`{}` is not on the descent it was found on",
                repeated_ty.to_string()
            )
        });
    if path.len() - start <= 1 {
        return String::new();
    }
    let descent = path[start..]
        .iter()
        .chain([repeated_ty])
        .map(|step_ty| format!("`{}`", step_ty.to_string()))
        .collect::<Vec<_>>();
    format!(" ({})", descent.join(" -> "))
}
