//! Whether a type has a size, and what to report when it has none.
//!
//! A value's size is the size of the object the code generator lays out for it, so the walk here
//! follows the objects (`ty_to_object_ty`) rather than the declarations. It reports the two ways a
//! layout can fail to exist: a type reaching itself with no pointer in between, and a type reaching
//! the same type constructor at an ever larger argument.
//!
//! `Program::validate_layouts` runs this over an instantiated program, before code generation walks
//! the fields of any type.

use std::sync::Arc;

use crate::ast::program::TypeEnv;
use crate::ast::types::{Type, TypeNode};
use crate::misc::Set;
use crate::object::{ty_to_object_ty, ObjectFieldType};

/// Why a value of `ty` has no size, and `None` where the code generator can lay one out.
///
/// The walk follows what `ty_to_object_ty` puts in the object: the fields of a struct, the payloads
/// of a union, and the elements an array storage holds are laid out in place, so it descends into
/// them. A field of a boxed type is a pointer, so what it points at is an object of its own: the
/// walk goes on there with the in-place chain started afresh.
///
/// # Arguments
/// * `checked` - the types walked so far. Whether a type has a size is a property of that type, so
///   one it holds is passed over; the ways down to it are compared against it first, which is what
///   catches a type reaching itself.
pub fn no_size_reason(
    ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    checked: &mut Set<Arc<TypeNode>>,
) -> Option<String> {
    /// Walk the layout of `ty` and of the types it holds. `in_place` is the types `ty` sits inside
    /// with no pointer in between, `across_pointers` every type the walk reached `ty` through, both
    /// outermost first.
    fn walk(
        ty: &Arc<TypeNode>,
        type_env: &TypeEnv,
        in_place: &mut Vec<Arc<TypeNode>>,
        across_pointers: &mut Vec<Arc<TypeNode>>,
        checked: &mut Set<Arc<TypeNode>>,
    ) -> Option<String> {
        if let Some(msg) = no_size_cause(ty, in_place, across_pointers) {
            return Some(msg);
        }
        if !checked.insert(ty.clone()) {
            return None;
        }
        // The types this object holds.
        let mut held: Vec<Arc<TypeNode>> = vec![];
        for field in ty_to_object_ty(ty, &vec![], type_env).field_types {
            match field {
                ObjectFieldType::SubObject(field_ty, _is_punched) => held.push(field_ty),
                ObjectFieldType::UnionBuf(payload_tys) => held.extend(payload_tys),
                ObjectFieldType::ArrayStorageBuf(elem_ty) => held.push(elem_ty),
                _ => {}
            }
        }

        in_place.push(ty.clone());
        across_pointers.push(ty.clone());
        let reason = held.iter().find_map(|held_ty| {
            if held_ty.is_unbox(type_env) {
                walk(held_ty, type_env, in_place, across_pointers, checked)
            } else {
                walk(held_ty, type_env, &mut vec![], across_pointers, checked)
            }
        });
        across_pointers.pop();
        in_place.pop();
        reason
    }
    // A function value is a pair of pointers, but the types it takes and returns are laid out where
    // the function is compiled. A function type reaching here is one the program has a value of, so
    // that function is compiled and its signature laid out.
    if ty.is_closure() || ty.is_funptr() {
        return ty
            .get_lambda_srcs()
            .into_iter()
            .chain([ty.get_lambda_dst()])
            .find_map(|signature_ty| {
                walk(
                    &signature_ty,
                    type_env,
                    &mut vec![],
                    &mut vec![ty.clone()],
                    checked,
                )
            });
    }
    walk(ty, type_env, &mut vec![], &mut vec![], checked)
}

/// Why a value of `ty` has no size, given the types its layout came through, and `None` where it
/// has one.
///
/// # Arguments
/// * `in_place` - the types `ty` sits inside with no pointer in between, outermost first.
///   Reaching one of them again is a value that contains itself.
/// * `across_pointers` - every type the layout came through, the ones behind a pointer included.
///   Reaching a larger type of the same type constructor there has no end either: the same
///   fields lead from that one to a larger one again.
fn no_size_cause(
    ty: &Arc<TypeNode>,
    in_place: &[Arc<TypeNode>],
    across_pointers: &[Arc<TypeNode>],
) -> Option<String> {
    if let Some(i) = in_place.iter().position(|ancestor| ancestor == ty) {
        let cause = format!("its unboxed fields reach `{}` itself", ty.to_string());
        return Some(format_no_size_error(
            ty,
            &in_place[i..],
            cause,
            NoSizeRemedy::Box,
        ));
    }
    // A function value is a pair of pointers whatever it takes and returns, so its size is
    // settled. Every function type shares the `->` constructor, so the growth of one function's
    // argument would otherwise be read off another's.
    if ty.is_closure() || ty.is_funptr() {
        return None;
    }
    // The same type constructor with arguments that have grown: the fields that led from that
    // one here lead on to a larger one again. A type merely appearing inside another (`Tree`
    // inside `(Tree, Tree)`) is how an ordinary recursive type is written, and the walk ends
    // there by meeting `Tree` a second time.
    let my_app_seq = ty.flatten_type_application();
    let grows_from = |ancestor: &Arc<TypeNode>| {
        if ancestor == ty {
            return false;
        }
        let their_app_seq = ancestor.flatten_type_application();
        their_app_seq.len() == my_app_seq.len()
            && their_app_seq[0] == my_app_seq[0]
            && their_app_seq[1..]
                .iter()
                .zip(my_app_seq[1..].iter())
                .all(|(their_arg, my_arg)| embeds_in(their_arg, my_arg))
    };
    if let Some(i) = across_pointers.iter().position(grows_from) {
        let cause = "its fields reach ever larger types".to_string();
        return Some(format_no_size_error(
            ty,
            &across_pointers[i..],
            cause,
            NoSizeRemedy::SameTypeArguments,
        ));
    }
    None
}

/// Whether `inner` is embedded in `outer`: it appears there with its own shape intact, with more
/// type around it or inside its arguments. An argument grown this way is what tells a type reached
/// again at a larger argument from one reached at a smaller or unrelated one.
fn embeds_in(inner: &Arc<TypeNode>, outer: &Arc<TypeNode>) -> bool {
    // Inside one of `outer`'s parts.
    let inside = match &outer.ty {
        Type::TyApp(fun, arg) => embeds_in(inner, fun) || embeds_in(inner, arg),
        Type::AssocTy(_, args) => args.iter().any(|arg| embeds_in(inner, arg)),
        Type::TyVar(_) | Type::TyCon(_) => false,
    };
    if inside {
        return true;
    }
    // The same shape at the top, each part embedded in the part facing it.
    match (&inner.ty, &outer.ty) {
        (Type::TyVar(inner_var), Type::TyVar(outer_var)) => inner_var.name == outer_var.name,
        (Type::TyCon(inner_tycon), Type::TyCon(outer_tycon)) => inner_tycon == outer_tycon,
        (Type::TyApp(inner_fun, inner_arg), Type::TyApp(outer_fun, outer_arg)) => {
            embeds_in(inner_fun, outer_fun) && embeds_in(inner_arg, outer_arg)
        }
        (Type::AssocTy(inner_assoc, inner_args), Type::AssocTy(outer_assoc, outer_args)) => {
            inner_assoc == outer_assoc
                && inner_args.len() == outer_args.len()
                && inner_args
                    .iter()
                    .zip(outer_args.iter())
                    .all(|(inner_arg, outer_arg)| embeds_in(inner_arg, outer_arg))
        }
        _ => false,
    }
}

/// The change that gives a type with no size a size, chosen by what makes its layout endless.
enum NoSizeRemedy {
    /// The fields lead back to the type: a pointer anywhere on the way round ends the descent.
    Box,
    /// The fields lead to the same type constructor at a larger argument: the descent ends once the
    /// recursion stops enlarging the argument, whatever is boxed.
    SameTypeArguments,
}

/// The report for a type with no size: what its fields do, the way down to it from the type that
/// shows it, and the change that gives it a size.
fn format_no_size_error(
    ty: &Arc<TypeNode>,
    ancestors: &[Arc<TypeNode>],
    cause: String,
    remedy: NoSizeRemedy,
) -> String {
    let descent = ancestors
        .iter()
        .chain([ty])
        .map(|ty| ty.to_string())
        .collect::<Vec<_>>();
    // A type holding itself directly is the whole story already, so the way down is spelled
    // out only where it passes through another type.
    let holds_itself = ancestors.iter().all(|ancestor| ancestor == ty);
    let way_down = if holds_itself {
        String::new()
    } else {
        format!(
            " ({})",
            descent
                .iter()
                .map(|ty| format!("`{}`", ty))
                .collect::<Vec<_>>()
                .join(" -> ")
        )
    };
    let remedy = match remedy {
        NoSizeRemedy::Box if holds_itself => format!("Make `{}` boxed.", descent[0]),
        NoSizeRemedy::Box => "Make one of these types boxed.".to_string(),
        NoSizeRemedy::SameTypeArguments => {
            "Give the recursive occurrence the same type arguments.".to_string()
        }
    };
    format!(
        "`{}` has no size: {}{}. {}",
        descent[0], cause, way_down, remedy,
    )
}
