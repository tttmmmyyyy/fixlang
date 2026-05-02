use crate::{
    ast::{
        expr::{expr_var, ExprNode},
        name::FullName,
        predicate::Predicate,
        program::Program,
        qual_pred::QualPred,
        traits::{TraitId, TraitImpl},
        typedecl::{TypeAlias, TypeDeclValue, TypeDefn},
        types::{type_fun, type_tyapp, type_tycon, type_tyvar_star, Scheme, TyCon, TypeNode},
    },
    fixstd::builtin::{
        add_trait_instance_float, add_trait_instance_int, array_check_range, array_check_size,
        array_get_capacity, array_get_size, array_unsafe_empty, array_unsafe_fill,
        array_unsafe_get_bounds_unchecked, array_unsafe_get_linear_bounds_unchecked_unretained,
        array_unsafe_set_bounds_uniqueness_unchecked_unreleased, bit_not_function,
        bitwise_operation_function, boxed_from_retained_ptr_ios, boxed_to_retained_ptr_ios,
        boxed_trait_instance, cast_between_float_function, cast_between_integral_function,
        cast_float_to_int_function, cast_int_to_float_function, destructor_make,
        divide_trait_instance_float, divide_trait_instance_int, eq_trait_instance_float,
        eq_trait_instance_int, eq_trait_instance_ptr, fix, floating_types, force_unique_array,
        get_get_boxed_ptr, get_mutate_boxed_internal, get_mutate_boxed_ios_internal, get_ptr_array,
        get_release_function_of_boxed_value, get_retain_function_of_boxed_value, infinity_value,
        integral_types, is_unique_function, less_than_or_equal_to_trait_instance_float,
        less_than_or_equal_to_trait_instance_int, less_than_trait_instance_float,
        less_than_trait_instance_int, make_array_ty, make_bool_ty, make_dynamic_object_ty,
        make_floating_ty, make_integral_ty, make_iostate_unsafe_create, make_ptr_ty,
        mark_threaded_function, multiply_trait_instance_float, multiply_trait_instance_int,
        negate_trait_instance_float, negate_trait_instance_int, not_trait_instance_bool,
        quiet_nan_value, remainder_trait_instance_int, set_array, shift_function,
        hole_function, subtract_trait_instance_float, subtract_trait_instance_int,
        undefined_internal_function, unsafe_set_size_array, with_retained_function,
        BitOperationType,
    },
    configuration::Configuration,
    constants::{
        ARRAY_CHECK_RANGE, ARRAY_CHECK_SIZE, ARRAY_NAME,
        ARRAY_UNSAFE_EMPTY_NAME, ARRAY_UNSAFE_FILL_NAME, ARRAY_UNSAFE_GET_BOUNDS_UNCHECKED,
        ARRAY_UNSAFE_GET_LINEAR_BOUNDS_UNCHECKED_UNRETAINED,
        ARRAY_UNSAFE_SET_BOUNDS_UNIQUENESS_UNCHECKED_UNRELEASED, DESTRUCTOR_NAME, F32_NAME,
        F64_NAME, FFI_NAME, HOLE_NAME, IOSTATE_NAME, IO_NAME, STD_NAME, WITH_RETAINED_NAME,
    },
    error::Errors,
    misc::{make_map, upper_camel_to_lower_snake, Map},
    parse::parser::parse_and_save_to_temporary_file,
};
use std::sync::Arc;

pub const FIX_NAME: &str = "fix";

const STD_SOURCE: &str = include_str!("std.fix");

// Body and scheme for a deprecated `Std::<From>::to_<To>` global that delegates
// to its canonical replacement, the trait method `Std::To<To>::<method>`.
//
// `return_ty` is the value type as it should appear in the global's scheme:
// the Fix type itself for Fix-target casts, or the C alias (e.g. `CInt`) for
// FFI-target casts. The trait's own method already has type `[a : To<To>] a -> <To>`,
// so the scheme is just `from -> return_ty` with no constraints (the constraint is
// satisfied by an instance registered in `make_numeric_cast_traits_mod`).
fn cast_delegation(
    from: &Arc<TypeNode>,
    to_name: &str,
    return_ty: Arc<TypeNode>,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    let mut trait_method = FullName::from_strs(
        &[STD_NAME, &format!("To{}", to_name)],
        &upper_camel_to_lower_snake(to_name),
    );
    trait_method.set_absolute();
    let body = expr_var(trait_method, None);
    let scm = Scheme::generalize(&[], vec![], vec![], type_fun(from.clone(), return_ty));
    (body, scm)
}

// Register a deprecated `Std::<From>::to_<To>` cast: it gets a body that
// delegates to the `To<To>` trait method and a `DEPRECATED[...]` entry
// pointing users at that trait method.
fn register_deprecated_cast(
    fix_module: &mut Program,
    errors: &mut Errors,
    from: &Arc<TypeNode>,
    from_name: &str,
    to_name: &str,
    return_ty: Arc<TypeNode>,
) {
    let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
    let target = FullName::new(&from_namespace, &format!("to_{}", to_name));
    errors.eat_err(fix_module.add_global_value(
        target.clone(),
        cast_delegation(from, to_name, return_ty),
        None,
        None,
        Some(format!(
            "Casts a value of `{}` into a value of `{}`.",
            from_name, to_name
        )),
    ));
    fix_module.add_deprecation(
        target,
        format!(
            "Use the trait member `To{}::{}` instead.",
            to_name,
            upper_camel_to_lower_snake(to_name),
        ),
    );
}

pub fn make_std_mod(config: &Configuration) -> Result<Program, Errors> {
    let mut fix_module = parse_and_save_to_temporary_file(STD_SOURCE, "std", config)?;

    let mut errors = Errors::empty();

    // Add C types type aliases.
    let c_types = config.c_type_sizes.get_c_types();
    for (name, sign, size) in &c_types {
        let fix_type = if *sign == "F" {
            make_floating_ty(&format!("{}{}", sign, size))
        } else {
            make_integral_ty(&format!("{}{}", sign, size))
        };
        let fix_type = fix_type.expect("Type alias `{}` is not supported in this system.");
        fix_module.add_type_defns(vec![TypeDefn {
            name: FullName::from_strs(&[STD_NAME, FFI_NAME], name),
            value: TypeDeclValue::Alias(TypeAlias { value: fix_type }),
            tyvars: vec![],
            source: None,
            name_src: None,
        }]);
    }

    let integral_types = &integral_types();
    let float_types = &floating_types();

    // Trait instances

    // Eq
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(eq_trait_instance_int(ty.clone()))?;
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(eq_trait_instance_float(ty.clone()))?;
    }
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_int(make_bool_ty()))?;
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_ptr(make_ptr_ty()))?;

    // Add
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(add_trait_instance_int(ty.clone()))?;
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(add_trait_instance_float(ty.clone()))?;
    }

    // Sub
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(subtract_trait_instance_int(ty.clone()))?;
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(subtract_trait_instance_float(ty.clone()))?;
    }

    // Neg
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(negate_trait_instance_int(ty.clone()))?;
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(negate_trait_instance_float(ty.clone()))?;
    }

    // Mul
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(multiply_trait_instance_int(ty.clone()))?;
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(multiply_trait_instance_float(ty.clone()))?;
    }

    // Div
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(divide_trait_instance_int(ty.clone()))?;
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(divide_trait_instance_float(ty.clone()))?;
    }

    // Rem
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(remainder_trait_instance_int(ty.clone()))?;
    }

    // LessThan
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(less_than_trait_instance_int(ty.clone()))?;
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(less_than_trait_instance_float(ty.clone()))?;
    }

    // LessThanOrEq
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(less_than_or_equal_to_trait_instance_int(ty.clone()))?;
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(less_than_or_equal_to_trait_instance_float(ty.clone()))?;
    }

    // Not
    fix_module
        .trait_env
        .add_instance(not_trait_instance_bool())?;

    // Boxed
    let builtin_boxed = vec![
        type_tyapp(make_array_ty(), type_tyvar_star("a")), // Array a
        make_dynamic_object_ty(),                          // #DynamicObject
    ];
    for ty in builtin_boxed {
        fix_module
            .trait_env
            .add_instance(boxed_trait_instance(&ty))?;
    }

    // Cast functions

    // Helper: build the C-alias `TypeNode` for a (sign, size) when the host
    // has a matching Fix counterpart, or `None` to skip.
    let c_alias_ty = |to_name_c: &str, sign: &str, size: usize| -> Option<Arc<TypeNode>> {
        let has_fix_counterpart = if sign == "F" {
            make_floating_ty(&format!("{}{}", sign, size)).is_some()
        } else {
            make_integral_ty(&format!("{}{}", sign, size)).is_some()
        };
        if !has_fix_counterpart {
            return None;
        }
        Some(type_tycon(&Arc::new(TyCon::new(FullName::from_strs(
            &[STD_NAME, FFI_NAME],
            to_name_c,
        )))))
    };

    // Fix → Fix: integer/float to integer/float.
    for from in integral_types.iter().chain(float_types.iter()) {
        let from_name = from.toplevel_tycon().unwrap().name.name.clone();
        for to in integral_types.iter().chain(float_types.iter()) {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            register_deprecated_cast(
                &mut fix_module,
                &mut errors,
                from,
                &from_name,
                &to_name,
                to.clone(),
            );
        }
    }
    // Fix → C alias: integer/float to any C numeric type.
    for from in integral_types.iter().chain(float_types.iter()) {
        let from_name = from.toplevel_tycon().unwrap().name.name.clone();
        for (to_name_c, sign, size) in &c_types {
            let Some(to_type_c) = c_alias_ty(to_name_c, sign, *size) else {
                continue;
            };
            register_deprecated_cast(
                &mut fix_module,
                &mut errors,
                from,
                &from_name,
                to_name_c,
                to_type_c,
            );
        }
    }
    // Bit operations
    for int_ty in integral_types {
        let ty_name = int_ty.toplevel_tycon().unwrap().name.name.clone();
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "bit_not"),
            bit_not_function(int_ty.clone()),
            None,
            None,
            Some(include_str!("../docs/std_bit_not.md").to_string()),
        ));
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "shift_left"),
            shift_function(int_ty.clone(), true),
            None,
            None,
            Some(include_str!("../docs/std_shift_left.md").to_string()),
        ));
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "shift_right"),
            shift_function(int_ty.clone(), false),
            None,
            None,
            Some(include_str!("../docs/std_shift_right.md").to_string()),
        ));
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "bit_xor"),
            bitwise_operation_function(int_ty.clone(), BitOperationType::Xor),
            None,
            None,
            Some(include_str!("../docs/std_bit_xor.md").to_string()),
        ));
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "bit_and"),
            bitwise_operation_function(int_ty.clone(), BitOperationType::And),
            None,
            None,
            Some(include_str!("../docs/std_bit_and.md").to_string()),
        ));
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "bit_or"),
            bitwise_operation_function(int_ty.clone(), BitOperationType::Or),
            None,
            None,
            Some(include_str!("../docs/std_bit_or.md").to_string()),
        ));
    }

    // Basic functions
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME], FIX_NAME),
        fix(),
        None,
        None,
        Some(include_str!("../docs/std_fix.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME], "unsafe_is_unique"),
        is_unique_function(),
        None,
        None,
        Some(include_str!("../docs/std_unsafe_is_unique.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME], "mark_threaded"),
        mark_threaded_function(),
        None,
        None,
        Some(include_str!("../docs/std_mark_threaded.md").to_string()),
    ));

    // Array
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "_unsafe_set_size"),
        unsafe_set_size_array(),
        None,
        None,
        Some(include_str!("../docs/std_array_unsafe_set_size.md").to_string()),
    ));
    errors.eat_err(
        fix_module.add_global_value(
            FullName::from_strs(
                &[STD_NAME, ARRAY_NAME],
                ARRAY_UNSAFE_SET_BOUNDS_UNIQUENESS_UNCHECKED_UNRELEASED,
            ),
            array_unsafe_set_bounds_uniqueness_unchecked_unreleased(),
            None,
            None,
            Some(
                include_str!(
                    "../docs/std_array_unsafe_set_bounds_uniqueness_unchecked_unreleased.md"
                )
                .to_string(),
            ),
        ),
    );
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_UNSAFE_GET_BOUNDS_UNCHECKED),
        array_unsafe_get_bounds_unchecked(),
        None,
        None,
        Some(include_str!("../docs/std_array_unsafe_get_bounds_unchecked.md").to_string()),
    ));
    errors.eat_err(
        fix_module.add_global_value(
            FullName::from_strs(
                &[STD_NAME, ARRAY_NAME],
                ARRAY_UNSAFE_GET_LINEAR_BOUNDS_UNCHECKED_UNRETAINED,
            ),
            array_unsafe_get_linear_bounds_unchecked_unretained(false),
            None,
            None,
            Some(
                include_str!("../docs/std_array_unsafe_get_linear_bounds_unchecked_unretained.md")
                    .to_string(),
            ),
        ),
    );
    errors.eat_err(
        fix_module.add_global_value(
            FullName::from_strs(
                &[STD_NAME, ARRAY_NAME],
                &format!(
                    "{}_forceunique",
                    ARRAY_UNSAFE_GET_LINEAR_BOUNDS_UNCHECKED_UNRETAINED
                ),
            ),
            array_unsafe_get_linear_bounds_unchecked_unretained(true),
            None,
            None,
            Some(
                include_str!(
                    "../docs/std_array_unsafe_get_linear_bounds_unchecked_unretained_fu.md"
                )
                .to_string(),
            ),
        ),
    );
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "_unsafe_force_unique"),
        force_unique_array(),
        None,
        None,
        Some(include_str!("../docs/std_array_force_unique.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_CHECK_RANGE),
        array_check_range(),
        None,
        None,
        Some(include_str!("../docs/std_array_check_range.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_CHECK_SIZE),
        array_check_size(),
        None,
        None,
        Some(include_str!("../docs/std_array_check_size.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "set"),
        set_array(),
        None,
        None,
        Some(include_str!("../docs/std_array_set.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "get_capacity"),
        array_get_capacity(),
        None,
        None,
        Some(include_str!("../docs/std_array_get_capacity.md").to_string()),
    ));
    // The canonical name of the array length accessor is `Array::@size`;
    // it gets the LLVM-builtin implementation. `Array::get_size` is then
    // a thin Fix-level alias defined in std.fix and marked deprecated.
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "@size"),
        array_get_size(),
        None,
        None,
        Some(include_str!("../docs/std_array_get_size.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "_get_ptr"),
        get_ptr_array(),
        None,
        None,
        Some(include_str!("../docs/std_array_get_ptr.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_UNSAFE_EMPTY_NAME),
        array_unsafe_empty(),
        None,
        None,
        Some(include_str!("../docs/std_array_unsafe_empty.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_UNSAFE_FILL_NAME),
        array_unsafe_fill(),
        None,
        None,
        Some(include_str!("../docs/std_array_unsafe_fill.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME], "_undefined_internal"),
        undefined_internal_function(),
        None,
        None,
        Some(include_str!("../docs/std_undefined_internal.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME], HOLE_NAME),
        hole_function(),
        None,
        None,
        None,
    ));
    errors.eat_err(fix_module.add_global_value(
        make_with_retained_name(),
        with_retained_function(),
        None,
        None,
        Some(include_str!("../docs/std_with_retained.md").to_string()),
    ));

    // Numeric constants
    for type_name in [F32_NAME, F64_NAME] {
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, type_name], "infinity"),
            infinity_value(type_name),
            None,
            None,
            Some(include_str!("../docs/std_float_infinity.md").to_string()),
        ));
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, type_name], "quiet_nan"),
            quiet_nan_value(type_name),
            None,
            None,
            Some(include_str!("../docs/std_float_quiet_nan.md").to_string()),
        ));
    }

    // FFI
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "_boxed_to_retained_ptr_ios"),
        boxed_to_retained_ptr_ios(),
        None,
        None,
        Some(include_str!("../docs/std_ffi_boxed_to_retained_ptr_ios.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "_boxed_from_retained_ptr_ios"),
        boxed_from_retained_ptr_ios(),
        None,
        None,
        Some(include_str!("../docs/std_ffi_boxed_from_retained_ptr_ios.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "get_funptr_release"),
        get_release_function_of_boxed_value(),
        None,
        None,
        Some(include_str!("../docs/std_ffi_get_funptr_release.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "get_funptr_retain"),
        get_retain_function_of_boxed_value(),
        None,
        None,
        Some(include_str!("../docs/std_ffi_get_funptr_retain.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "_get_boxed_ptr"),
        get_get_boxed_ptr(),
        None,
        None,
        Some(include_str!("../docs/std_ffi_get_boxed_ptr.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "_mutate_boxed_internal"),
        get_mutate_boxed_internal(),
        None,
        None,
        Some(include_str!("../docs/std_ffi_mutate_boxed_internal.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "_mutate_boxed_ios_internal"),
        get_mutate_boxed_ios_internal(),
        None,
        None,
        Some(include_str!("../docs/std_ffi_mutate_boxed_ios_internal.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, IO_NAME, IOSTATE_NAME], "_unsafe_create"),
        make_iostate_unsafe_create(),
        None,
        None,
        Some(include_str!("../docs/std_iostate_unsafe_create.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME, DESTRUCTOR_NAME], "_make"),
        destructor_make(),
        None,
        None,
        Some(include_str!("../docs/std_ffi_destructor_make.md").to_string()),
    ));

    // Add numeric cast traits
    let cast_traits_mod = make_numeric_cast_traits_mod(config)?;
    fix_module.link(cast_traits_mod, true)?;

    errors.to_result()?;
    Ok(fix_module)
}

// Create source code to define traits such as ToString or Eq for tuples.
fn make_tuple_traits_source(sizes: &[u32]) -> String {
    let mut src = "module Std; \n\n".to_string();
    for size in sizes {
        // For unit type, we define necessary traits in "std.fix".
        if *size == 0 {
            continue;
        }
        let tuple_close = if *size == 1 { ",)" } else { ")" };
        // Implement `ToString` trait.
        src += "impl [";
        src += &(0..*size)
            .into_iter()
            .map(|i| format!("t{} : ToString", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += "] ";
        src += "(";
        src += &(0..*size)
            .map(|i| format!("t{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += tuple_close;
        src += " : ToString { \n";
        src += "    to_string = |(";
        src += &(0..*size)
            .map(|i| format!("x{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += tuple_close;
        src += "| \"(\" + ";
        src += &(0..*size)
            .into_iter()
            .map(|i| format!("x{}.to_string", i))
            .collect::<Vec<_>>()
            .join(" + \", \" + ");
        src += " + \"";
        src += tuple_close;
        src += "\";\n";
        src += "}\n\n";

        // Implement `Eq` trait.
        src += "impl [";
        src += &(0..*size)
            .into_iter()
            .map(|i| format!("t{} : Eq", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += "] ";
        src += "(";
        src += &(0..*size)
            .map(|i| format!("t{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += tuple_close;
        src += " : Eq { \n";
        src += "    eq = |(";
        src += &(0..*size)
            .map(|i| format!("x{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += tuple_close;
        src += ", (";
        src += &(0..*size)
            .map(|i| format!("y{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += tuple_close;
        src += "| ";
        src += &(0..*size)
            .into_iter()
            .map(|i| format!("x{} == y{}", i, i))
            .collect::<Vec<_>>()
            .join(" && ");
        src += ";\n";
        src += "}\n\n";

        // Implement `LessThan` trait.
        src += "impl [";
        src += &(0..*size)
            .into_iter()
            .map(|i| format!("t{} : Eq, t{} : LessThan", i, i))
            .collect::<Vec<_>>()
            .join(", ");
        src += "] ";
        src += "(";
        src += &(0..*size)
            .map(|i| format!("t{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += tuple_close;
        src += " : LessThan { \n";
        src += "    less_than = |(";
        src += &(0..*size)
            .map(|i| format!("x{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += tuple_close;
        src += ", (";
        src += &(0..*size)
            .map(|i| format!("y{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += tuple_close;
        src += "| (\n";
        for i in 0..*size {
            src += &format!("        if x{} != y{} {{ x{} < y{} }};\n", i, i, i, i);
        }
        src += "        false\n";
        src += "    );\n";
        src += "}\n\n";

        // Implement `LessThanOrEq` trait.
        src += "impl [";
        src += &(0..*size)
            .into_iter()
            .map(|i| format!("t{} : Eq, t{} : LessThanOrEq", i, i))
            .collect::<Vec<_>>()
            .join(", ");
        src += "] ";
        src += "(";
        src += &(0..*size)
            .map(|i| format!("t{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += tuple_close;
        src += " : LessThanOrEq { \n";
        src += "    less_than_or_eq = |(";
        src += &(0..*size)
            .map(|i| format!("x{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += tuple_close;
        src += ", (";
        src += &(0..*size)
            .map(|i| format!("y{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        src += tuple_close;
        src += "| (\n";
        for i in 0..*size {
            src += &format!("        if x{} != y{} {{ x{} <= y{} }};\n", i, i, i, i);
        }
        src += "        true\n";
        src += "    );\n";
        src += "}\n\n";

        // Impl `TuepleN t0 ... t(N-1) : Functor`:
        // For example, if N = 2,
        // impl Tuple2 t0 : Functor {
        //     map = |f, (x0, x1)| (x0, f(x1));
        // }
        src += "impl ";
        src += format!("Tuple{} ", size).as_str();
        src += &(0..*size - 1)
            .map(|i| format!("t{}", i))
            .collect::<Vec<_>>()
            .join(" ");
        src += ": Functor { \n";
        src += "    map = |f, (";
        src += &(0..*size)
            .map(|i| format!("x{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        if *size == 1 {
            src += ",";
        }
        src += ")| (";
        src += &(0..*size)
            .map(|i| {
                if i != *size - 1 {
                    format!("x{}", i)
                } else {
                    format!("f(x{})", i)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        if *size == 1 {
            src += ",";
        }
        src += ");\n";
        src += "}\n\n";
    }

    src
}

// Build the module that defines traits which convert between numeric types.
//
// The trait declarations (`trait a : ToF64 { f64 : a -> F64; }` etc.) are
// emitted as source so that they participate in the usual parsing pipeline,
// but the per-type instances are added programmatically below: each
// `impl <From> : To<To> { <method> = ... }` is built directly as a `TraitImpl`
// whose body is the LLVM cast lambda returned by `cast_*_function`.
//
// This avoids a synthetic source line per instance, which would otherwise
// surface as deprecation warnings (the trivial `<method> = to_<To>;` body
// referenced the deprecated `to_<To>` global).
pub fn make_numeric_cast_traits_mod(config: &Configuration) -> Result<Program, Errors> {
    let int_types = integral_types();
    let float_types = floating_types();
    let c_types = config.c_type_sizes.get_c_types();

    // Source: trait declarations only.
    let mut to_type_names: Vec<String> = vec![];
    for ty in int_types.iter().chain(float_types.iter()) {
        to_type_names.push(ty.toplevel_tycon().unwrap().name.name.clone());
    }
    for (c_ty_name, _, _) in &c_types {
        to_type_names.push(c_ty_name.to_string());
    }
    let mut src = "module Std; \n\n".to_string();
    for to_name in &to_type_names {
        src += &format!(
            "trait a : To{} {{ \n\
            // Casts a value into `{}` type.\n\
            {} : a -> {};\n\
            }}\n",
            to_name,
            to_name,
            upper_camel_to_lower_snake(to_name),
            to_name,
        );
    }
    let mut prog = parse_and_save_to_temporary_file(&src, "std_numeric_cast_traits", config)?;

    // Programmatic impls.
    let make_impl = |from: &Arc<TypeNode>, to_name: &str, body: Arc<ExprNode>| -> TraitImpl {
        let trait_id = TraitId::from_fullname(FullName::from_strs(
            &[STD_NAME],
            &format!("To{}", to_name),
        ));
        let method = upper_camel_to_lower_snake(to_name);
        TraitImpl {
            qual_pred: QualPred {
                pred_constraints: vec![],
                eq_constraints: vec![],
                kind_constraints: vec![],
                predicate: Predicate::make(trait_id, from.clone()),
            },
            members: make_map([(method, body)]),
            member_lhs_srcs: Map::default(),
            member_sigs: Map::default(),
            assoc_types: Map::default(),
            define_module: STD_NAME.to_string(),
            source: None,
            is_user_defined: false,
        }
    };

    // Build the cast body for `from -> to`, dispatched on the (int/float)
    // shapes of both ends. `to_alias` is the C alias type when present
    // (Fix → C target); for Fix → Fix targets it is `None`.
    let cast_body =
        |from: &Arc<TypeNode>,
         from_is_int: bool,
         to_fix: Arc<TypeNode>,
         to_is_int: bool,
         to_alias: Option<Arc<TypeNode>>|
         -> Arc<ExprNode> {
            let (body, _) = match (from_is_int, to_is_int) {
                (true, true) => cast_between_integral_function(from.clone(), to_fix, to_alias),
                (false, false) => cast_between_float_function(from.clone(), to_fix, to_alias),
                (true, false) => cast_int_to_float_function(from.clone(), to_fix),
                (false, true) => cast_float_to_int_function(from.clone(), to_fix),
            };
            body
        };

    // Fix → Fix: every (int|float) → (int|float) combination.
    for (from, from_is_int) in int_types
        .iter()
        .map(|t| (t, true))
        .chain(float_types.iter().map(|t| (t, false)))
    {
        for (to, to_is_int) in int_types
            .iter()
            .map(|t| (t, true))
            .chain(float_types.iter().map(|t| (t, false)))
        {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let body = cast_body(from, from_is_int, to.clone(), to_is_int, None);
            prog.trait_env.add_instance(make_impl(from, &to_name, body))?;
        }
    }
    // Fix → C type. The C type name (e.g. `CInt`) is an alias for one of the
    // Fix integral/floating types, identified by `(sign, size)`.
    for (from, from_is_int) in int_types
        .iter()
        .map(|t| (t, true))
        .chain(float_types.iter().map(|t| (t, false)))
    {
        for (to_name_c, sign, size) in &c_types {
            let to_is_int = *sign != "F";
            let to_fix = if to_is_int {
                make_integral_ty(&format!("{}{}", sign, size))
            } else {
                make_floating_ty(&format!("{}{}", sign, size))
            };
            let Some(to_fix) = to_fix else { continue };
            let to_alias = type_tycon(&Arc::new(TyCon::new(FullName::from_strs(
                &[STD_NAME, FFI_NAME],
                to_name_c,
            ))));
            let body = cast_body(from, from_is_int, to_fix, to_is_int, Some(to_alias));
            prog.trait_env.add_instance(make_impl(from, to_name_c, body))?;
        }
    }

    Ok(prog)
}

// Create module which defines traits such as ToString or Eq for tuples.
pub fn make_tuple_traits_mod(sizes: &[u32], config: &Configuration) -> Result<Program, Errors> {
    let src = make_tuple_traits_source(sizes);
    parse_and_save_to_temporary_file(&src, "std_tuple_traits", config)
}

// Make full name of `Std::with_retained` function.
pub fn make_with_retained_name() -> FullName {
    FullName::from_strs(&[STD_NAME], WITH_RETAINED_NAME)
}
