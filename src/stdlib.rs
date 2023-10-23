use build_time::build_time_utc;
use chrono::{DateTime, Utc};

use super::*;

pub const FIX_NAME: &str = "fix";

const STD_SOURCE: &str = include_str!("fix/std.fix");

pub fn make_std_mod() -> Program {
    let mut fix_module = parse_source_temporary_file(
        STD_SOURCE,
        "std",
        &format!("{:x}", md5::compute(build_time_utc!())),
    );
    fix_module.set_last_update(
        STD_NAME.to_string(),
        UpdateDate(
            DateTime::parse_from_rfc3339(build_time_utc!())
                .unwrap()
                .with_timezone(&Utc),
        ),
    );

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

    let integral_types = &[
        make_i8_ty(),
        make_u8_ty(),
        make_i16_ty(),
        make_u16_ty(),
        make_i32_ty(),
        make_u32_ty(),
        make_i64_ty(),
        make_u64_ty(),
    ];
    let float_types = &[make_f32_ty(), make_f64_ty()];

    // Eq
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(eq_trait_instance_int(ty.clone()));
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(eq_trait_instance_float(ty.clone()));
    }
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_int(make_bool_ty()));
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_ptr(make_ptr_ty()));

    // Add
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(add_trait_instance_int(ty.clone()));
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(add_trait_instance_float(ty.clone()));
    }

    // Sub
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(subtract_trait_instance_int(ty.clone()));
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(subtract_trait_instance_float(ty.clone()));
    }

    // Neg
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(negate_trait_instance_int(ty.clone()));
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(negate_trait_instance_float(ty.clone()));
    }

    // Mul
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(multiply_trait_instance_int(ty.clone()));
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(multiply_trait_instance_float(ty.clone()));
    }

    // Div
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(divide_trait_instance_int(ty.clone()));
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(divide_trait_instance_float(ty.clone()));
    }

    // Rem
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(remainder_trait_instance_int(ty.clone()));
    }

    // LessThan
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(less_than_trait_instance_int(ty.clone()));
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(less_than_trait_instance_float(ty.clone()));
    }

    // LessThanOrEq
    for ty in integral_types {
        fix_module
            .trait_env
            .add_instance(less_than_or_equal_to_trait_instance_int(ty.clone()));
    }
    for ty in float_types {
        fix_module
            .trait_env
            .add_instance(less_than_or_equal_to_trait_instance_float(ty.clone()));
    }

    // Not
    fix_module.trait_env.add_instance(not_trait_instance_bool());

    // Cast functions

    // Cast function between integral types.
    for from in integral_types {
        for to in integral_types {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_between_integral_function(from.clone(), to.clone()),
            );
        }
    }
    // Cast function between float types.
    for from in float_types {
        for to in float_types {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_between_float_function(from.clone(), to.clone()),
            );
        }
    }
    // Cast from integers to float types.
    for from in integral_types {
        for to in float_types {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_int_to_float_function(from.clone(), to.clone()),
            );
        }
    }
    // Cast from float types to integers.
    for from in float_types {
        for to in integral_types {
            let to_name = to.toplevel_tycon().unwrap().name.name.clone();
            let from_namespace = from.toplevel_tycon().unwrap().name.to_namespace();
            fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_float_to_int_function(from.clone(), to.clone()),
            );
        }
    }
    // Bit operations
    for int_ty in integral_types {
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
        FullName::from_strs(&[STD_NAME], "unsafe_is_unique"),
        is_unique_function(),
    );
    // NOTE: Maybe can we define i64 fixruntime_sub_ptr(void* lhs, void* rhs) by LLVM and implement `subtract_ptr` by CALL_C?
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, PTR_NAME], "subtract_ptr"),
        subtract_ptr_function(),
    );

    // Array
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "fill"),
        fill_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "_unsafe_set"),
        unsafe_set_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "_unsafe_set_size"),
        unsafe_set_size_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "_unsafe_get"),
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
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "@"),
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
    fix_module.add_global_value(FullName::from_strs(&[STD_NAME], "abort"), abort_function());

    fix_module
}
