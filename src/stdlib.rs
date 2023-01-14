use super::*;

pub const FIX_NAME: &str = "fix";

const STD_SOURCE: &str = r###"
module Std;
"###;

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
    fix_module.trait_env.add_trait(and_trait());
    fix_module.trait_env.add_trait(or_trait());
    fix_module.trait_env.add_trait(less_than_trait());
    fix_module
        .trait_env
        .add_trait(less_than_or_equal_to_trait());

    // Trait instances
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_primitive(int_lit_ty()));
    fix_module
        .trait_env
        .add_instance(eq_trait_instance_primitive(bool_lit_ty()));
    fix_module.trait_env.add_instance(add_trait_instance_int());
    fix_module
        .trait_env
        .add_instance(subtract_trait_instance_int());
    fix_module
        .trait_env
        .add_instance(negate_trait_instance_int());
    fix_module.trait_env.add_instance(not_trait_instance_bool());
    fix_module
        .trait_env
        .add_instance(multiply_trait_instance_int());
    fix_module
        .trait_env
        .add_instance(divide_trait_instance_int());
    fix_module
        .trait_env
        .add_instance(remainder_trait_instance_int());
    fix_module.trait_env.add_instance(and_trait_instance_bool());
    fix_module.trait_env.add_instance(or_trait_instance_bool());
    fix_module
        .trait_env
        .add_instance(less_than_trait_instance_int());
    fix_module
        .trait_env
        .add_instance(less_than_or_equal_to_trait_instance_int());

    // Functions and values
    fix_module.add_global_value(FullName::from_strs(&[STD_NAME], FIX_NAME), fix());
    fix_module.add_global_value(FullName::from_strs(&[STD_NAME], "loop"), state_loop());
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "new"),
        new_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "from_map"),
        from_map_array(),
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
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "len"),
        length_array(),
    );

    fix_module
}
