use super::*;

pub const FIX_NAME: &str = "fix";

const STD_SOURCE: &str = include_str!("fix/std.fix");

pub fn make_std_mod(config: &Configuration) -> Program {
    let mut fix_module = parse_and_save_to_temporary_file(STD_SOURCE, "std", config);

    // Add C types type aliases.
    let c_types = config.c_type_sizes.get_c_types();
    for (name, sign, size) in &c_types {
        let fix_type = if *sign == "F" {
            make_floating_ty(&format!("{}{}", sign, size))
        } else {
            make_integral_ty(&format!("{}{}", sign, size))
        };
        if fix_type.is_none() {
            println!(
                "Warning: Type alias `{}` is not supported in this system.",
                name
            );
            continue;
        }
        let fix_type = fix_type.unwrap();
        fix_module.add_type_defns(vec![TypeDefn {
            name: FullName::from_strs(&[STD_NAME, FFI_NAME], name),
            value: TypeDeclValue::Alias(TypeAlias { value: fix_type }),
            tyvars: vec![],
            source: None,
        }]);
    }

    // `LoopResult` type.
    fix_module.type_defns.push(loop_result_defn());

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

    // Cast function between integers.
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
    // Cast function from integer to C integers.
    for from in integral_types {
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
            fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_between_integral_function(from.clone(), to_type),
            );
        }
    }
    // Cast function between floats.
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
    // Cast function from floats to C floats.
    for from in float_types {
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
            fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_between_float_function(from.clone(), to_type.clone()),
            );
        }
    }
    // Cast from integers to floats.
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
    // Cast from integers to C floats.
    for from in integral_types {
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
            fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_int_to_float_function(from.clone(), to_type),
            );
        }
    }
    // Cast from floats to integers.
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
    // Cast from floats to C integers.
    for from in float_types {
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
            fix_module.add_global_value(
                FullName::new(&from_namespace, &format!("to_{}", to_name)),
                cast_float_to_int_function(from.clone(), to_type),
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
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME], "mark_threaded"),
        mark_threaded_function(),
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
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "force_unique"),
        force_unique_array(),
    );
    fix_module.add_global_value(array_getter_function_name(), read_array());
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "set"),
        set_array(),
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

    // Numeric constants
    for type_name in [F32_NAME, F64_NAME] {
        fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, type_name], "infinity"),
            infinity_value(type_name),
        );
        fix_module.add_global_value(
            FullName::from_strs(&[STD_NAME, type_name], "quiet_nan"),
            quiet_nan_value(type_name),
        );
    }

    // FFI
    fix_module.add_global_value(
        FullName::from_strs(
            &[STD_NAME, FFI_NAME],
            "unsafe_get_retained_ptr_of_boxed_value",
        ),
        get_retained_ptr_of_boxed_value_function(),
    );
    fix_module.add_global_value(
        FullName::from_strs(
            &[STD_NAME, FFI_NAME],
            "unsafe_get_boxed_value_from_retained_ptr",
        ),
        get_boxed_value_from_retained_ptr_function(),
    );
    fix_module.add_global_value(
        FullName::from_strs(
            &[STD_NAME, FFI_NAME],
            "unsafe_get_release_function_of_boxed_value",
        ),
        get_release_function_of_boxed_value(),
    );
    fix_module.add_global_value(
        FullName::from_strs(
            &[STD_NAME, FFI_NAME],
            "unsafe_get_retain_function_of_boxed_value",
        ),
        get_retain_function_of_boxed_value(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, FFI_NAME], "_unsafe_get_boxed_data_ptr"),
        get_unsafe_get_boxed_ptr(),
    );

    fix_module
}

pub fn array_getter_function_name() -> FullName {
    FullName::from_strs(&[STD_NAME, ARRAY_NAME], ARRAY_GETTER_FUNCTION_NAME)
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

// Create module which defines traits such as ToString or Eq for tuples.
pub fn make_tuple_traits_mod(sizes: &[u32], config: &Configuration) -> Program {
    let src = make_tuple_traits_source(sizes);
    parse_and_save_to_temporary_file(&src, "std_tuple_traits", config)
}
