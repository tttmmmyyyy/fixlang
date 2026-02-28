use crate::{
    ast::{
        name::FullName,
        program::Program,
        typedecl::{TypeAlias, TypeDeclValue, TypeDefn},
        types::{type_tyapp, type_tycon, type_tyvar_star, TyCon},
    },
    builtin::{
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
        subtract_trait_instance_float, subtract_trait_instance_int, undefined_internal_function,
        unsafe_set_size_array, with_retained_function, BitOperationType,
    },
    configuration::Configuration,
    constants::{
        ARRAY_CHECK_RANGE, ARRAY_CHECK_SIZE, ARRAY_GET_SIZE_NAME, ARRAY_NAME,
        ARRAY_UNSAFE_EMPTY_NAME, ARRAY_UNSAFE_FILL_NAME, ARRAY_UNSAFE_GET_BOUNDS_UNCHECKED,
        ARRAY_UNSAFE_GET_LINEAR_BOUNDS_UNCHECKED_UNRETAINED,
        ARRAY_UNSAFE_SET_BOUNDS_UNIQUENESS_UNCHECKED_UNRELEASED, DESTRUCTOR_NAME, F32_NAME,
        F64_NAME, FFI_NAME, IOSTATE_NAME, IO_NAME, STD_NAME, WITH_RETAINED_NAME,
    },
    error::Errors,
    misc::upper_camel_to_lower_snake,
    parser::parse_and_save_to_temporary_file,
};
use std::sync::Arc;

pub const FIX_NAME: &str = "fix";

const STD_SOURCE: &str = include_str!("fix/std.fix");

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

    // Cast function between integers.
    for from in integral_types {
        let from_name = from.toplevel_tycon().unwrap().name.name.clone();
        for to in integral_types {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            errors.eat_err(fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_between_integral_function(from.clone(), to.clone(), None),
                None,
                None,
                Some(format!(
                    "(Deprecated) The function `Std::{}::to_{}` has been deprecated in favor of the trait member `To{}::{}`. The old function name will remain available for the foreseeable future to maintain backward compatibility.\n\nCasts a value of `{}` into a value of `{}`.",
                    from_name, to_name, to_name, upper_camel_to_lower_snake(&to_name),
                    from_name, to_name
                )),
            ));
        }
    }
    // Cast function from integer to C integers.
    for from in integral_types {
        let from_name = from.toplevel_tycon().unwrap().name.name.clone();
        for (to_name_c, sign, size) in &c_types {
            if *sign == "F" {
                continue;
            }
            // The type as is in C, e.g., "CInt".
            let to_type_c = FullName::from_strs(&[STD_NAME, FFI_NAME], to_name_c);
            let to_type_c = type_tycon(&Arc::new(TyCon::new(to_type_c)));

            // The type in Fix, e.g., "I32".
            let to_type_fix = make_integral_ty(&format!("{}{}", sign, size));
            if to_type_fix.is_none() {
                continue;
            }
            let to_type_fix = to_type_fix.unwrap();

            // The namespace of the conversion function is the same as the namespace of the source type.
            let namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            errors.eat_err(fix_module.add_global_value(
                FullName::new(&namespace, &format!("to_{}", to_name_c)),
                cast_between_integral_function(from.clone(), to_type_fix, Some(to_type_c)),
                None,
                None,
                Some(format!(
                    "(Deprecated) The function `Std::{}::to_{}` has been deprecated in favor of the trait member `To{}::{}`. The old function name will remain available for the foreseeable future to maintain backward compatibility.\n\nCasts a value of `{}` into a value of `{}`.",
                    from_name, to_name_c, to_name_c, upper_camel_to_lower_snake(&to_name_c),
                    from_name, to_name_c
                )),
            ));
        }
    }
    // Cast function between floats.
    for from in float_types {
        let from_name = from.toplevel_tycon().unwrap().name.name.clone();
        for to in float_types {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            errors.eat_err(fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_between_float_function(from.clone(), to.clone(), None),
                None,
                None,
                Some(format!(
                    "(Deprecated) The function `Std::{}::to_{}` has been deprecated in favor of the trait member `To{}::{}`. The old function name will remain available for the foreseeable future to maintain backward compatibility.\n\nCasts a value of `{}` into a value of `{}`.",
                    from_name, to_name, to_name, upper_camel_to_lower_snake(&to_name),
                    from_name, to_name
                )),
            ));
        }
    }
    // Cast function from floats to C floats.
    for from in float_types {
        let from_name = from.toplevel_tycon().unwrap().name.name.clone();
        for (to_name, sign, size) in &c_types {
            if *sign == "I" {
                continue;
            }
            // The type as is in C, e.g., "CFloat".
            let to_type_c = FullName::from_strs(&[STD_NAME, FFI_NAME], to_name);
            let to_type_c = type_tycon(&Arc::new(TyCon::new(to_type_c)));

            // The type as is in Fix, e.g., "F32".
            let to_type_fix = make_floating_ty(&format!("{}{}", sign, size));
            if to_type_fix.is_none() {
                continue;
            }
            let to_type_fix = to_type_fix.unwrap();

            // The namespace of the conversion function is the same as the namespace of the source type.
            let namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            errors.eat_err(fix_module.add_global_value(
                FullName::new(&namespace, &format!("to_{}", to_name)),
                cast_between_float_function(from.clone(), to_type_fix.clone(), Some(to_type_c)),
                None,
                None,
                Some(format!(
                    "(Deprecated) The function `Std::{}::to_{}` has been deprecated in favor of the trait member `To{}::{}`. The old function name will remain available for the foreseeable future to maintain backward compatibility.\n\nCasts a value of `{}` into a value of `{}`.",
                    from_name, to_name, to_name, upper_camel_to_lower_snake(&to_name),
                    from_name, to_name
                )),
            ));
        }
    }
    // Cast from integers to floats.
    for from in integral_types {
        let from_name = from.toplevel_tycon().unwrap().name.name.clone();
        for to in float_types {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            errors.eat_err(fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_int_to_float_function(from.clone(), to.clone()),
                None,
                None,
                Some(format!(
                    "(Deprecated) The function `Std::{}::to_{}` has been deprecated in favor of the trait member `To{}::{}`. The old function name will remain available for the foreseeable future to maintain backward compatibility.\n\nCasts a value of `{}` into a value of `{}`.",
                    from_name, to_name, to_name, upper_camel_to_lower_snake(&to_name),
                    from_name, to_name
                )),
            ));
        }
    }
    // Cast from integers to C floats.
    for from in integral_types {
        let from_name = from.toplevel_tycon().unwrap().name.name.clone();
        for (to_name, sign, size) in &c_types {
            if *sign == "I" {
                continue;
            }
            let to_type = make_floating_ty(&format!("{}{}", sign, size));
            if to_type.is_none() {
                continue;
            }
            let to_type = to_type.unwrap();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            errors.eat_err(fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_int_to_float_function(from.clone(), to_type),
                None,
                None,
                Some(format!(
                    "(Deprecated) The function `Std::{}::to_{}` has been deprecated in favor of the trait member `To{}::{}`. The old function name will remain available for the foreseeable future to maintain backward compatibility.\n\nCasts a value of `{}` into a value of `{}`.",
                    from_name, to_name, to_name, upper_camel_to_lower_snake(&to_name),
                    from_name, to_name
                )),
            ));
        }
    }
    // Cast from floats to integers.
    for from in float_types {
        let from_name = from.toplevel_tycon().unwrap().name.name.clone();
        for to in integral_types {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            errors.eat_err(fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_float_to_int_function(from.clone(), to.clone()),
                None,
                None,
                Some(format!(
                    "(Deprecated) The function `Std::{}::to_{}` has been deprecated in favor of the trait member `To{}::{}`. The old function name will remain available for the foreseeable future to maintain backward compatibility.\n\nCasts a value of `{}` into a value of `{}`.",
                    from_name, to_name, to_name, upper_camel_to_lower_snake(&to_name),
                    from_name, to_name
                )),
            ));
        }
    }
    // Cast from floats to C integers.
    for from in float_types {
        let from_name = from.toplevel_tycon().unwrap().name.name.clone();
        for (to_name, sign, size) in &c_types {
            if *sign == "F" {
                continue;
            }
            let to_type = make_integral_ty(&format!("{}{}", sign, size));
            if to_type.is_none() {
                continue;
            }
            let to_type = to_type.unwrap();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            errors.eat_err(fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_float_to_int_function(from.clone(), to_type),
                None,
                None,
                Some(format!(
                    "(Deprecated) The function `Std::{}::to_{}` has been deprecated in favor of the trait member `To{}::{}`. The old function name will remain available for the foreseeable future to maintain backward compatibility.\n\nCasts a value of `{}` into a value of `{}`.",
                    from_name, to_name, to_name, upper_camel_to_lower_snake(&to_name),
                    from_name, to_name
                )),
            ));
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
            Some(include_str!("./docs/std_bit_not.md").to_string()),
        ));
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "shift_left"),
            shift_function(int_ty.clone(), true),
            None,
            None,
            Some(include_str!("./docs/std_shift_left.md").to_string()),
        ));
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "shift_right"),
            shift_function(int_ty.clone(), false),
            None,
            None,
            Some(include_str!("./docs/std_shift_right.md").to_string()),
        ));
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "bit_xor"),
            bitwise_operation_function(int_ty.clone(), BitOperationType::Xor),
            None,
            None,
            Some(include_str!("./docs/std_bit_xor.md").to_string()),
        ));
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "bit_and"),
            bitwise_operation_function(int_ty.clone(), BitOperationType::And),
            None,
            None,
            Some(include_str!("./docs/std_bit_and.md").to_string()),
        ));
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "bit_or"),
            bitwise_operation_function(int_ty.clone(), BitOperationType::Or),
            None,
            None,
            Some(include_str!("./docs/std_bit_or.md").to_string()),
        ));
    }

    // Basic functions
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME], FIX_NAME),
        fix(),
        None,
        None,
        Some(include_str!("./docs/std_fix.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME], "unsafe_is_unique"),
        is_unique_function(),
        None,
        None,
        Some(include_str!("./docs/std_unsafe_is_unique.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME], "mark_threaded"),
        mark_threaded_function(),
        None,
        None,
        Some(include_str!("./docs/std_mark_threaded.md").to_string()),
    ));

    // Array
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "_unsafe_set_size"),
        unsafe_set_size_array(),
        None,
        None,
        Some(include_str!("./docs/std_array_unsafe_set_size.md").to_string()),
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
                    "./docs/std_array_unsafe_set_bounds_uniqueness_unchecked_unreleased.md"
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
        Some(include_str!("./docs/std_array_unsafe_get_bounds_unchecked.md").to_string()),
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
                include_str!("./docs/std_array_unsafe_get_linear_bounds_unchecked_unretained.md")
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
                    "./docs/std_array_unsafe_get_linear_bounds_unchecked_unretained_fu.md"
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
        Some(include_str!("./docs/std_array_force_unique.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_CHECK_RANGE),
        array_check_range(),
        None,
        None,
        Some(include_str!("./docs/std_array_check_range.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_CHECK_SIZE),
        array_check_size(),
        None,
        None,
        Some(include_str!("./docs/std_array_check_size.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "set"),
        set_array(),
        None,
        None,
        Some(include_str!("./docs/std_array_set.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "get_capacity"),
        array_get_capacity(),
        None,
        None,
        Some(include_str!("./docs/std_array_get_capacity.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_GET_SIZE_NAME),
        array_get_size(),
        None,
        None,
        Some(include_str!("./docs/std_array_get_size.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "_get_ptr"),
        get_ptr_array(),
        None,
        None,
        Some(include_str!("./docs/std_array_get_ptr.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_UNSAFE_EMPTY_NAME),
        array_unsafe_empty(),
        None,
        None,
        Some(include_str!("./docs/std_array_unsafe_empty.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_UNSAFE_FILL_NAME),
        array_unsafe_fill(),
        None,
        None,
        Some(include_str!("./docs/std_array_unsafe_fill.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME], "_undefined_internal"),
        undefined_internal_function(),
        None,
        None,
        Some(include_str!("./docs/std_undefined_internal.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        make_with_retained_name(),
        with_retained_function(),
        None,
        None,
        Some(include_str!("./docs/std_with_retained.md").to_string()),
    ));

    // Numeric constants
    for type_name in [F32_NAME, F64_NAME] {
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, type_name], "infinity"),
            infinity_value(type_name),
            None,
            None,
            Some(include_str!("./docs/std_float_infinity.md").to_string()),
        ));
        errors.eat_err(fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, type_name], "quiet_nan"),
            quiet_nan_value(type_name),
            None,
            None,
            Some(include_str!("./docs/std_float_quiet_nan.md").to_string()),
        ));
    }

    // FFI
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "_boxed_to_retained_ptr_ios"),
        boxed_to_retained_ptr_ios(),
        None,
        None,
        Some(include_str!("./docs/std_ffi_boxed_to_retained_ptr_ios.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "_boxed_from_retained_ptr_ios"),
        boxed_from_retained_ptr_ios(),
        None,
        None,
        Some(include_str!("./docs/std_ffi_boxed_from_retained_ptr_ios.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "get_funptr_release"),
        get_release_function_of_boxed_value(),
        None,
        None,
        Some(include_str!("./docs/std_ffi_get_funptr_release.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "get_funptr_retain"),
        get_retain_function_of_boxed_value(),
        None,
        None,
        Some(include_str!("./docs/std_ffi_get_funptr_retain.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "_get_boxed_ptr"),
        get_get_boxed_ptr(),
        None,
        None,
        Some(include_str!("./docs/std_ffi_get_boxed_ptr.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "_mutate_boxed_internal"),
        get_mutate_boxed_internal(),
        None,
        None,
        Some(include_str!("./docs/std_ffi_mutate_boxed_internal.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "_mutate_boxed_ios_internal"),
        get_mutate_boxed_ios_internal(),
        None,
        None,
        Some(include_str!("./docs/std_ffi_mutate_boxed_ios_internal.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, IO_NAME, IOSTATE_NAME], "_unsafe_create"),
        make_iostate_unsafe_create(),
        None,
        None,
        Some(include_str!("./docs/std_iostate_unsafe_create.md").to_string()),
    ));
    errors.eat_err(fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME, DESTRUCTOR_NAME], "_make"),
        destructor_make(),
        None,
        None,
        Some(include_str!("./docs/std_ffi_destructor_make.md").to_string()),
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

// Create source code to define traits which convert between numeric types.
pub fn make_numeric_cast_traits_mod(config: &Configuration) -> Result<Program, Errors> {
    let mut fix_type_names = vec![];
    for int_ty in integral_types() {
        let ty_name = int_ty.toplevel_tycon().unwrap().name.name.clone();
        fix_type_names.push(ty_name);
    }
    for float_ty in floating_types() {
        let ty_name = float_ty.toplevel_tycon().unwrap().name.name.clone();
        fix_type_names.push(ty_name);
    }
    let mut type_names = fix_type_names.clone();
    for (c_ty_name, _, _e) in config.c_type_sizes.get_c_types() {
        type_names.push(c_ty_name.to_string());
    }
    let mut src = "module Std; \n\n".to_string();
    for to_name in &type_names {
        // Add trait definition.
        src += &format!(
            "trait a : To{} {{ \n\
            // Casts a value into `{}` type.\n\
            {} : a -> {};\n\
            }}\n",
            to_name,
            to_name,
            upper_camel_to_lower_snake(&to_name),
            to_name,
        );

        // Add trait implementations.
        for from_name in &fix_type_names {
            src += &format!(
                "impl {} : To{} {{ {} = to_{}; }}\n",
                from_name,
                to_name,
                upper_camel_to_lower_snake(&to_name),
                to_name,
            );
        }
    }
    parse_and_save_to_temporary_file(&src, "std_numeric_cast_traits", config)
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
