use super::*;

pub const FIX_NAME: &str = "fix";
pub const VECTOR_LEN_IDX: u32 = 0;
pub const VECTOR_DATA_IDX: u32 = 1;

const STD_SOURCE: &str = r#"
module Std;

type Vector a = unbox struct { len : Int, data : Array a };

type String = unbox struct { data : Vector Byte };

namespace String {
    @len : String -> Int;
    @len = |s| s.@data.@len - 1; // exclude null terminator
}

trait a : ToString {
    to_string : a -> String;
}

impl Int : ToString {
    to_string = int_to_string;
}

namespace IO {

    pure : a -> IOState -> (a, IOState);
    pure = |val, io| (val, io);

}

namespace Debug {

    assert_eq : [a: Eq] String -> a -> a -> ();
    assert_eq = |msg, lhs, rhs| assert(msg, lhs == rhs);

    assert : String -> Bool -> ();
    assert = |msg, b| (
        if !b then (
            let u = debug_print("assertion failed!: ");
            let u = debug_print(msg);
            abort()
        ) else (
            ()
        )
    );
}

type Option a = union { none: (), some: a };

namespace Option {

    unwrap : Option a -> a;
    unwrap = |opt| ( 
        let Option.some(val) = opt;
        val
    );

    map : (a -> b) -> Option a -> Option b;
    map = |f, opt| (
        if opt.is_none then (
            none()
        ) else (
            some $ f $ opt.unwrap
        )
    );

}

// Iterator (a.k.a lazy list)
type Iterator a = struct { data: () -> Option (a, Iterator a) };

namespace Iterator {

    // Get next value and iterator.
    next : Iterator a -> Option (a, Iterator a);
    next = |iter| (iter.@data)();

    // Create iterator that counts up from a number.
    // count_up(n) = [n, n+1, n+2, ...]
    count_up : Int -> Iterator Int;
    count_up = |i| (
        let data = |_| (
            some $ (i, Iterator.count_up(i+1))
        );
        Iterator { data: data }
    );

    // Apply a function to each value of iterator.
    // map(f, [a0, a1, a2, ...]) = [f(a0), f(a1), f(a2), ...]
    map : (a -> b) -> Iterator a -> Iterator b;
    map = |f, a_iter| (
        let data = |_| (
            a_iter.next.map(
                |(a_val, a_iter)| (f(a_val), a_iter.map(f))
            )
        );
        Iterator { data: data }
    );

    // Create iterator from mapping function.
    // from_map(f) = [f(0), f(1), f(2), ...]
    from_map : (Int -> a) -> Iterator a;
    from_map = |f| count_up(0).map(f);
}
"#;

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
    fix_module.add_global_value(FullName::from_strs(&[STD_NAME], "print"), print_io_func());
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, DEBUG_NAME], "debug_print"),
        debug_print_function(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, DEBUG_NAME], "abort"),
        abort_function(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME], "int_to_string"),
        int_to_string_function(),
    );

    fix_module
}
