use super::*;

pub const FIX_NAME: &str = "fix";

const STD_SOURCE: &str = include_str!("std.fix");

pub fn make_std_mod() -> FixModule {
    let mut fix_module = parse_source(STD_SOURCE);

    // Types
    fix_module.type_defns.push(loop_result_defn());
    for i in 0..=TUPLE_SIZE_MAX {
        if i != 1 {
            fix_module.type_defns.push(tuple_defn(i));
        }
    }

    // Traits
    fix_module.trait_env.add_trait(eq_trait());
    fix_module.trait_env.add_trait(add_trait());
    fix_module.trait_env.add_trait(subtract_trait());
    fix_module.trait_env.add_trait(negate_trait());
    fix_module.trait_env.add_trait(not_trait());
    fix_module.trait_env.add_trait(multiply_trait());
    fix_module.trait_env.add_trait(divide_trait());
    fix_module.trait_env.add_trait(remainder_trait());
    fix_module.trait_env.add_trait(less_than_trait());
    fix_module
        .trait_env
        .add_trait(less_than_or_equal_to_trait());

    // Trait instances

    // Eq
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_ptr(make_ptr_ty()));
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_int(make_u8_ty()));
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_int(make_i32_ty()));
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_int(make_u32_ty()));
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_int(make_i64_ty()));
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_int(make_u64_ty()));
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_float(make_f32_ty()));
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_float(make_f64_ty()));
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_int(make_bool_ty()));

    // Add
    fix_module
        .trait_env
        .add_instance(add_trait_instance_int(make_u8_ty()));
    fix_module
        .trait_env
        .add_instance(add_trait_instance_int(make_i32_ty()));
    fix_module
        .trait_env
        .add_instance(add_trait_instance_int(make_u32_ty()));
    fix_module
        .trait_env
        .add_instance(add_trait_instance_int(make_i64_ty()));
    fix_module
        .trait_env
        .add_instance(add_trait_instance_int(make_u64_ty()));
    fix_module
        .trait_env
        .add_instance(add_trait_instance_float(make_f32_ty()));
    fix_module
        .trait_env
        .add_instance(add_trait_instance_float(make_f64_ty()));

    // Sub
    fix_module
        .trait_env
        .add_instance(subtract_trait_instance_int(make_u8_ty()));
    fix_module
        .trait_env
        .add_instance(subtract_trait_instance_int(make_i32_ty()));
    fix_module
        .trait_env
        .add_instance(subtract_trait_instance_int(make_u32_ty()));
    fix_module
        .trait_env
        .add_instance(subtract_trait_instance_int(make_i64_ty()));
    fix_module
        .trait_env
        .add_instance(subtract_trait_instance_int(make_u64_ty()));
    fix_module
        .trait_env
        .add_instance(subtract_trait_instance_float(make_f32_ty()));
    fix_module
        .trait_env
        .add_instance(subtract_trait_instance_float(make_f64_ty()));

    // Neg
    fix_module
        .trait_env
        .add_instance(negate_trait_instance_int(make_u8_ty()));
    fix_module
        .trait_env
        .add_instance(negate_trait_instance_int(make_i32_ty()));
    fix_module
        .trait_env
        .add_instance(negate_trait_instance_int(make_u32_ty()));
    fix_module
        .trait_env
        .add_instance(negate_trait_instance_int(make_i64_ty()));
    fix_module
        .trait_env
        .add_instance(negate_trait_instance_int(make_u64_ty()));
    fix_module
        .trait_env
        .add_instance(negate_trait_instance_float(make_f32_ty()));
    fix_module
        .trait_env
        .add_instance(negate_trait_instance_float(make_f64_ty()));

    // Mul
    fix_module
        .trait_env
        .add_instance(multiply_trait_instance_int(make_u8_ty()));
    fix_module
        .trait_env
        .add_instance(multiply_trait_instance_int(make_i32_ty()));
    fix_module
        .trait_env
        .add_instance(multiply_trait_instance_int(make_u32_ty()));
    fix_module
        .trait_env
        .add_instance(multiply_trait_instance_int(make_i64_ty()));
    fix_module
        .trait_env
        .add_instance(multiply_trait_instance_int(make_u64_ty()));
    fix_module
        .trait_env
        .add_instance(multiply_trait_instance_float(make_f32_ty()));
    fix_module
        .trait_env
        .add_instance(multiply_trait_instance_float(make_f64_ty()));

    // Div
    fix_module
        .trait_env
        .add_instance(divide_trait_instance_int(make_u8_ty()));
    fix_module
        .trait_env
        .add_instance(divide_trait_instance_int(make_i32_ty()));
    fix_module
        .trait_env
        .add_instance(divide_trait_instance_int(make_u32_ty()));
    fix_module
        .trait_env
        .add_instance(divide_trait_instance_int(make_i64_ty()));
    fix_module
        .trait_env
        .add_instance(divide_trait_instance_int(make_u64_ty()));
    fix_module
        .trait_env
        .add_instance(divide_trait_instance_float(make_f32_ty()));
    fix_module
        .trait_env
        .add_instance(divide_trait_instance_float(make_f64_ty()));

    // Rem
    fix_module
        .trait_env
        .add_instance(remainder_trait_instance_int(make_u8_ty()));
    fix_module
        .trait_env
        .add_instance(remainder_trait_instance_int(make_i32_ty()));
    fix_module
        .trait_env
        .add_instance(remainder_trait_instance_int(make_u32_ty()));
    fix_module
        .trait_env
        .add_instance(remainder_trait_instance_int(make_i64_ty()));
    fix_module
        .trait_env
        .add_instance(remainder_trait_instance_int(make_u64_ty()));

    // LessThan
    fix_module
        .trait_env
        .add_instance(less_than_trait_instance_int(make_u8_ty()));
    fix_module
        .trait_env
        .add_instance(less_than_trait_instance_int(make_i32_ty()));
    fix_module
        .trait_env
        .add_instance(less_than_trait_instance_int(make_u32_ty()));
    fix_module
        .trait_env
        .add_instance(less_than_trait_instance_int(make_i64_ty()));
    fix_module
        .trait_env
        .add_instance(less_than_trait_instance_int(make_u64_ty()));
    fix_module
        .trait_env
        .add_instance(less_than_trait_instance_float(make_f32_ty()));
    fix_module
        .trait_env
        .add_instance(less_than_trait_instance_float(make_f64_ty()));

    // LessThanOrEq
    fix_module
        .trait_env
        .add_instance(less_than_or_equal_to_trait_instance_int(make_u8_ty()));
    fix_module
        .trait_env
        .add_instance(less_than_or_equal_to_trait_instance_int(make_i32_ty()));
    fix_module
        .trait_env
        .add_instance(less_than_or_equal_to_trait_instance_int(make_u32_ty()));
    fix_module
        .trait_env
        .add_instance(less_than_or_equal_to_trait_instance_int(make_i64_ty()));
    fix_module
        .trait_env
        .add_instance(less_than_or_equal_to_trait_instance_int(make_u64_ty()));
    fix_module
        .trait_env
        .add_instance(less_than_or_equal_to_trait_instance_float(make_f32_ty()));
    fix_module
        .trait_env
        .add_instance(less_than_or_equal_to_trait_instance_float(make_f64_ty()));

    // Not
    fix_module.trait_env.add_instance(not_trait_instance_bool());

    // Internal function of ToString for integral types.
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, U8_NAME], "_U8_to_string"),
        number_to_string_function(make_u8_ty()),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, I32_NAME], "_I32_to_string"),
        number_to_string_function(make_i32_ty()),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, U32_NAME], "_U32_to_string"),
        number_to_string_function(make_u32_ty()),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, I64_NAME], "_I64_to_string"),
        number_to_string_function(make_i64_ty()),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, U64_NAME], "_U64_to_string"),
        number_to_string_function(make_u64_ty()),
    );
    // The following does not work correctly for some reason.
    // fix_module.add_global_value(
    //     FullName::from_strs(&[STD_NAME, F32_NAME], "_F32_to_string"),
    //     number_to_string_function(make_f32_ty()),
    // );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, F64_NAME], "_F64_to_string"),
        number_to_string_function(make_f64_ty()),
    );

    // Cast functions
    let integral_tys: &[Rc<TypeNode>] = &[
        make_u8_ty(),
        make_i32_ty(),
        make_u32_ty(),
        make_i64_ty(),
        make_u64_ty(),
    ];
    let float_tys: &[Rc<TypeNode>] = &[make_f32_ty(), make_f64_ty()];
    // Cast function between integral types.
    for from in integral_tys {
        for to in integral_tys {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_between_integral_function(from.clone(), to.clone()),
            );
        }
    }
    // Cast function between float types.
    for from in float_tys {
        for to in float_tys {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_between_float_function(from.clone(), to.clone()),
            );
        }
    }
    // Cast from integers to float types.
    for from in integral_tys {
        for to in float_tys {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_int_to_float_function(from.clone(), to.clone()),
            );
        }
    }
    // Cast from float types to integers.
    for from in float_tys {
        for to in integral_tys {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_float_to_int_function(from.clone(), to.clone()),
            );
        }
    }
    // Bit operations
    for int_ty in integral_tys {
        let ty_name = int_ty.toplevel_tycon().unwrap().name.name.clone();
        fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "shift_left"),
            shift_function(int_ty.clone(), true),
        );
        fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "shift_right"),
            shift_function(int_ty.clone(), false),
        );
        fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "bit_xor"),
            bitwise_operation_function(int_ty.clone(), BitOperationType::Xor),
        );
        fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "bit_and"),
            bitwise_operation_function(int_ty.clone(), BitOperationType::And),
        );
        fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, &ty_name], "bit_or"),
            bitwise_operation_function(int_ty.clone(), BitOperationType::Or),
        );
    }

    // Basic functions
    fix_module.add_global_value(FullName::from_strs(&[STD_NAME], FIX_NAME), fix());
    fix_module.add_global_value(FullName::from_strs(&[STD_NAME], "loop"), state_loop());
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME], "is_unique"),
        is_unique_function(),
    );

    // Array
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "fill"),
        fill_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "__unsafe_set"),
        unsafe_set_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "__unsafe_set_length"),
        unsafe_set_length_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "__unsafe_get"),
        unsafe_get_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "force_unique!"),
        force_unique_array(true),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "force_unique"),
        force_unique_array(false),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "get"),
        read_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "set"),
        write_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "set!"),
        write_array_unique(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "empty"),
        make_empty(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "mod"),
        mod_array(false),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "mod!"),
        mod_array(true),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "get_capacity"),
        get_capacity_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "get_size"),
        get_size_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "_get_ptr"),
        get_ptr_array(),
    );
    // Debug
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, DEBUG_NAME], "debug_print"),
        debug_print_function(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, DEBUG_NAME], "abort"),
        abort_function(),
    );

    fix_module
}
