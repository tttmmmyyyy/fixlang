//! Whether the compiler can build the types a program needs, and what to report when it cannot.
//!
//! Two things can go wrong, and the walks here report both.
//!
//! **A value that contains itself.** The size of an unboxed value is the sum of its fields' sizes,
//! so deciding it descends into every field laid out in place. A pointer is a pointer whatever it
//! points at, so the descent stops at a boxed field. A type the descent is already inside would
//! have to be larger than itself.
//!
//! **A type reached from itself at a larger type argument.** `P (a, a)` reached from `P a` leads to
//! a type that is larger again at every step and never repeats, so the program needs endlessly many
//! types even though every one of them has a size. Two relations lead from a type to another one,
//! and a family growing along either of them is endless:
//!
//! - **the types a value holds**, since the code generator lays out an object per type;
//! - **the types a declaration names** — its type arguments, and the types of its fields at those
//!   arguments — since the passes that specialize types before code generation rewrite a copy of a
//!   declaration per list of type arguments it is used at. This relation is the wider one: it
//!   reaches a type argument the declaration discards and a type that appears only as what a
//!   function takes or returns, neither of which is laid out anywhere.
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
//! The bound also settles termination, of the walks here and of the passes alike: over a finite set
//! of type constructors there are finitely many types of bounded depth, so a walk over either
//! relation runs out of new types to visit.
//!
//! The checks run over an instantiated program — every type they see has its type arguments given,
//! so every application they walk is headed by a type constructor — and before any pass rewrites a
//! type or code generation lays one out.

use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::misc::{grow_stack, Set};
use crate::object::{ty_to_object_ty, ObjectFieldType};
use std::sync::Arc;

/// How deeply a single type may nest before it is called endless.
///
/// This is a property of one type, not of the program: a chain of a thousand types that each hold
/// the next is a thousand types of depth one, and a project keeps compiling however many such types
/// it gains. A type reached from itself at a larger argument, on the other hand, gains a level at
/// every step and passes any bound.
///
/// Over the benchmark corpus and the examples the deepest type reached is 10; a type written with
/// 25 nested tuples reaches 27. The bound also caps how deep the walks over a type go — hashing it,
/// substituting into it, printing it — so raising it costs stack on the programs it exists to
/// reject.
const MAX_TYPE_DEPTH: usize = 500;

/// What the walks carry from one root to the next, so that a type is answered once however many
/// values carry it.
#[derive(Default)]
pub struct TypeWalk {
    /// Types whose in-place descent completed. Recorded on the way out, so that a type the descent
    /// is still inside is not mistaken for one already answered.
    settled: Set<Arc<TypeNode>>,
    /// Types the program needs an object for. Recorded on arrival: meeting one again needs no
    /// second answer.
    reached: Set<Arc<TypeNode>>,
    /// Types whose declarations have been unfolded. A type laid out nowhere is here as well, since
    /// a declaration names more types than a value holds.
    unfolded: Set<Arc<TypeNode>>,
}

/// Why the compiler cannot build the types a value of `ty` needs, and `None` where it can: every
/// type the program lays out has a size, and unfolding the declarations it names reaches finitely
/// many types.
pub fn no_build_reason(
    ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    walk: &mut TypeWalk,
) -> Option<String> {
    no_size_reason(ty, type_env, walk).or_else(|| endless_types_reason(ty, type_env, walk))
}

/// Why the compiler needs endlessly many types to compile a value of `ty`, and `None` where
/// unfolding the declarations `ty` names reaches finitely many.
///
/// Before code generation, the passes that specialize types build a type per declaration they
/// unfold: `remove_hktvs` copies a declaration once per list of type arguments it is used at, and
/// `unwrap_newtype` replaces a one-field declaration by that field's type at each of them. What
/// they unfold is wider than what a value holds — a type argument a declaration discards, and a
/// type that appears only as what a function takes or returns, are laid out nowhere and are still
/// types these passes rewrite.
fn endless_types_reason(
    ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    walk: &mut TypeWalk,
) -> Option<String> {
    root_types(ty).iter().find_map(|root_ty| {
        endless_types_reachable(root_ty, root_ty, type_env, walk, &mut vec![])
    })
}

/// The types a root stands for. A function type stands for the types it takes and returns: the
/// function is compiled where a value of it appears, and naming the arrow in a report would name a
/// type that is not the one at fault. Every other type stands for itself.
fn root_types(ty: &Arc<TypeNode>) -> Vec<Arc<TypeNode>> {
    if ty.is_closure() || ty.is_funptr() {
        return ty
            .get_lambda_srcs()
            .into_iter()
            .chain([ty.get_lambda_dst()])
            .collect();
    }
    vec![ty.clone()]
}

/// Walk the types unfolding `ty` names, bounding how deeply one of them nests.
///
/// `root` is the type the walk started from, which the report names where the type at fault is one
/// the walk built on the way: printing that one would print a term as deep as the bound.
fn endless_types_reachable(
    root: &Arc<TypeNode>,
    ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    walk: &mut TypeWalk,
    unfolded_from: &mut Vec<Arc<TypeNode>>,
) -> Option<String> {
    if !walk.unfolded.insert(ty.clone()) {
        return None;
    }
    // A function type nests once per argument it takes, and how many types the program needs is not
    // what that measures: the types the function takes and returns are answered as themselves.
    if !ty.is_closure() && !ty.is_funptr() && ty.depth() > MAX_TYPE_DEPTH {
        return Some(endless_types_message(root, unfolded_from));
    }
    unfolded_from.push(ty.clone());
    let reason = grow_stack(|| {
        named_types(ty, type_env)
            .iter()
            .find_map(|named_ty| {
                endless_types_reachable(root, named_ty, type_env, walk, unfolded_from)
            })
    });
    unfolded_from.pop();
    reason
}

/// The types unfolding `ty` names: the type arguments it is applied to, and the types its
/// declaration gives its fields at those arguments.
fn named_types(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Vec<Arc<TypeNode>> {
    // The program's types are instantiated by now, so an application here is headed by a type
    // constructor. A type headed by anything else names nothing this walk can follow.
    let Some(tycon) = ty.toplevel_tycon() else {
        return vec![];
    };
    let mut named_tys = ty.collect_type_argments();
    // A function type is made of the types it takes and returns and declares no field of its own.
    if ty.is_closure() {
        return named_tys;
    }
    let Some(tycon_info) = type_env.tycons.get(tycon.as_ref()) else {
        return named_tys;
    };
    // A type constructor given fewer arguments than its declaration takes stands at a higher-kinded
    // argument position, where what its fields hold is settled only once the arguments arrive.
    if tycon_info.tyvars.len() == named_tys.len() {
        named_tys.extend(ty.field_types(type_env));
        // A struct with one field punched out is a type of its own, and the declaration it is
        // punched from is one it names: the passes rewrite the two beside each other, at the same
        // type arguments.
        if let Some(struct_tycon) = &tycon_info.punched_from {
            named_tys.push(ty.set_toplevel_tycon(Arc::new(struct_tycon.clone())));
        }
    }
    named_tys
}

/// Why a value of `ty` has no size, and `None` where the code generator can lay one out.
fn no_size_reason(
    ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    walk: &mut TypeWalk,
) -> Option<String> {
    // A function type at a root is one the program has a value of, so that function is compiled and
    // the types it takes and returns are laid out with it. A function type reached as a field is
    // not: a value of it may never be built, and its own layout is two pointers either way.
    root_types(ty)
        .iter()
        .find_map(|root_ty| no_size_reachable(root_ty, root_ty, type_env, walk, &mut vec![]))
}

/// Walk the types the program needs an object for, deciding at each one whether its size settles.
///
/// `root` is the type the walk started from, which the report names where the type at fault is one
/// the walk built on the way: printing that one would print a term as deep as the bound.
fn no_size_reachable(
    root: &Arc<TypeNode>,
    ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    walk: &mut TypeWalk,
    asked_for: &mut Vec<Arc<TypeNode>>,
) -> Option<String> {
    if !walk.reached.insert(ty.clone()) {
        return None;
    }
    // A function value is a pair of pointers whatever it takes and returns, so how deeply the
    // function type itself nests says nothing about a layout: a function of five hundred arguments
    // nests five hundred deep and is still two pointers.
    if !ty.is_closure() && !ty.is_funptr() && ty.depth() > MAX_TYPE_DEPTH {
        return Some(endless_types_message(root, asked_for));
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
    walk: &mut TypeWalk,
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
        return Some(endless_types_message(root, path));
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

/// The report for a program that reaches types deeper than the bound, which both walks give.
///
/// It names the type the walk started from and the first few types that one led to, which is where
/// a type reached from itself at a larger argument shows itself. The type actually at fault is one
/// the walk built on the way, and printing that one would print a term as deep as the bound.
///
/// Which walk found it is left out: both say that a type grows without end, and both ask the
/// program for the same repair.
fn endless_types_message(root: &Arc<TypeNode>, walked: &[Arc<TypeNode>]) -> String {
    format!(
        "`{}` needs endlessly many types: it reaches types nested more than {} deep{}. A type \
         reached from itself at a larger type argument does this; give the recursive occurrence \
         the same type arguments.",
        shorten(root),
        MAX_TYPE_DEPTH,
        way_through(walked)
    )
}

/// A type as the report prints it, cut short past the point a reader takes it in. A type that trips
/// the bound is a term of any size, and the whole of one says no more than its beginning does.
fn shorten(ty: &Arc<TypeNode>) -> String {
    /// How much of a type to print before cutting it short.
    const MAX_SHOWN_CHARS: usize = 200;

    let text = ty.to_string();
    match text.char_indices().nth(MAX_SHOWN_CHARS) {
        Some((cut, _)) => format!("{}...", &text[..cut]),
        None => text,
    }
}

/// The first few types a walk passed through on its way to the one at fault, which is where a
/// growing family shows itself.
fn way_through(walked: &[Arc<TypeNode>]) -> String {
    /// How many steps to show. Enough for one turn of a growing family to be visible, and short
    /// enough that the types printed are ones the reader can read.
    const SHOWN_STEPS: usize = 3;

    let shown = walked
        .iter()
        .take(SHOWN_STEPS)
        .map(|ty| format!("`{}`", shorten(ty)))
        .collect::<Vec<_>>();
    if shown.is_empty() {
        return String::new();
    }
    format!(" ({} -> ...)", shown.join(" -> "))
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
