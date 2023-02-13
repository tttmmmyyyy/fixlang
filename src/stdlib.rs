use super::*;

pub const FIX_NAME: &str = "fix";
pub const VECTOR_DATA_IDX: u32 = 0;
pub const VECTOR_RESERVED_LEN_IDX: u32 = 1;

const STD_SOURCE: &str = r#"
module Std;

namespace Array {

    from_map : Int -> (Int -> a) -> Array a;
    from_map = |size, map| (
        let arr = Array.__new_uninitialized(size);
        loop((0, arr), |(idx, arr)|(
            if idx == size {
                break $ arr
            } else {
                let arr = arr.__set_uninitialized_unique_array(idx, map(idx));
                continue $ (idx + 1, arr)
            }
        ))
    );

}

namespace Debug {

    assert_eq : [a: Eq] String -> a -> a -> ();
    assert_eq = |msg, lhs, rhs| assert(msg, lhs == rhs);

    assert : String -> Bool -> ();
    assert = |msg, b| (
        if !b {
            let u = debug_print("assertion failed!: ");
            let u = debug_print(msg);
            abort()
        } else {
            ()
        }
    );
}

impl Int : ToString {
    to_string = Int._int_to_string;
}

namespace IOState {

    pure : a -> IOState -> (a, IOState);
    pure = |val, io| (val, io);

    println! : String -> IOState -> ((), IOState);
    println! = |msg, io| (
        let (_, io) = io.print!(msg);
        io.print!("\n")
    );

}

// Iterator (a.k.a lazy list)
type Iterator a = unbox struct { _data: () -> Option (a, Iterator a) };

namespace Iterator {

    // Push an elemnt to an iterator.
    push_head : a -> Iterator a -> Iterator a;
    push_head = |elem, iter| (
        let data = |_| (
            some $ (elem, iter)
        );
        Iterator { _data: data }     
    );

    // Counts the length of an iterator.
    get_length : Iterator a -> Int;
    get_length = fold(0, |acm, _| acm + 1);

    // Creates an iterator that counts up from a number.
    // count_up(n) = [n, n+1, n+2, ...]
    count_up : Int -> Iterator Int;
    count_up = |i| (
        let data = |_| (
            some $ (i, Iterator.count_up(i+1))
        );
        Iterator { _data: data }
    );

    // Create an empty iterator.
    make_empty : Iterator a;
    make_empty = (
        let data = |_| (none());
        Iterator { _data: data }
    );

    // Filter elements by a condition function
    filter : (a -> Bool) -> Iterator a -> Iterator a;
    filter = |cond, iter| (
        let data = |_| (
            loop(iter, |iter| (
                let next = iter.next;
                if next.is_none { break $ none() };
                let (v, iter) = next.unwrap;
                if !cond(v) { continue $ iter };
                let iter = filter(cond, iter);
                break $ some((v, iter))
            ))
        );
        Iterator { _data: data }
    );

    // Folds iterator from left.
    // fold(init, op, [a0, a1, a2, ...]) = ...op(op(op(init, a0), a1), a2)...
    fold : b -> (b -> a -> b) -> Iterator a -> b;
    fold = |init, op, iter| (
        loop((init, iter), |(accum, iter)|
            let next = iter.next;
            if next.is_none {
                break $ accum
            } else {
                let (next, iter) = next.unwrap;
                continue $ (op(accum, next), iter)
            }
        )
    );

    // Create iterator from an array.
    from_array : Array a -> Iterator a;
    from_array = |arr| count_up(0).take(arr.get_length).map(|i| arr.get(i));

    // Creates iterator from mapping function.
    // from_map(f) = [f(0), f(1), f(2), ...]
    from_map : (Int -> a) -> Iterator a;
    from_map = |f| count_up(0).map(f);

    // Takes the last element of an iterator.
    take_last : Iterator a -> Option a;
    take_last = |iter| (
        if iter.is_empty { none() };
        let (elem, iter) = iter.next.unwrap;
        if iter.is_empty { 
            some(elem)
        } else {
            iter.take_last
        }
    );

    // Checks if an iterator is empty.
    is_empty : Iterator a -> Bool;
    is_empty = |iter| iter.next.is_none;

    // Apply a function to each value of iterator.
    // map(f, [a0, a1, a2, ...]) = [f(a0), f(a1), f(a2), ...]
    map : (a -> b) -> Iterator a -> Iterator b;
    map = |f, a_iter| (
        let data = |_| (
            a_iter.next.map(
                |(a_val, a_iter)| (f(a_val), a_iter.map(f))
            )
        );
        Iterator { _data: data }
    );

    // Get next value and next iterator.
    next : Iterator a -> Option (a, Iterator a);
    next = |iter| (iter.@_data)();

    // Reverse an iterator.
    reverse : Iterator a -> Iterator a;
    reverse = |iter| (
        loop((Iterator.make_empty, iter), |(out_iter, in_iter)|(
            if in_iter.is_empty {
                break $ out_iter
            } else {
                let (elem, in_iter) = in_iter.next.unwrap;
                let out_iter = out_iter.push_head(elem);
                continue $ (out_iter, in_iter)
            }
        ))
    );

    // Take at most n elements from an iterator.
    take : Int -> Iterator a -> Iterator a;
    take = |n, iter| (
        let data = |_| (
            if n == 0 { none() };
            let iter_next = iter.next;
            if iter_next.is_none { none() };
            let (v, iter) = iter_next.unwrap;
            some $ (v, iter.take(n-1))
        );
        Iterator { _data: data }
    );

    // Zip two iterators.
    zip : Iterator a -> Iterator b -> Iterator (a, b);
    zip = |iter0, iter1| (
        let data = |_| (
            let iter0_next = iter0.next;
            if iter0_next.is_none { none() };
            let iter1_next = iter1.next;
            if iter1_next.is_none { none() };
            let (v0, iter0) = iter0_next.unwrap;
            let (v1, iter1) = iter1_next.unwrap;
            some $ ((v0, v1), zip(iter0, iter1))
        );
        Iterator { _data: data }
    );
}

type Option a = union { none: (), some: a };

namespace Option {

    map : (a -> b) -> Option a -> Option b;
    map = |f, opt| (
        if opt.is_none {
            none()
        } else {
            some $ f $ opt.unwrap
        }
    );

    unwrap : Option a -> a;
    unwrap = |opt| ( 
        let Option.some(val) = opt;
        val
    );

}

type String = unbox struct { _data : Vector Byte };

namespace String {
    get_length : String -> Int;
    get_length = |s| s.@_data.get_length - 1; // exclude null terminator
}

type Vector a = unbox struct { _data : Array a, _reserved_length : Int };

namespace Vector {
    get_length : Vector a -> Int;
    get_length = |v| v.@_data.get_length;
}

trait a : ToString {
    to_string : a -> String;
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
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "__new_uninitialized"),
        new_uninitialized(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "__set_uninitialized_unique_array"),
        set_uninitialized_unique_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "__set_array_length"),
        set_array_length(),
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
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "mod"),
        mod_array(false),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "mod!"),
        mod_array(true),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, ARRAY_NAME], "get_length"),
        length_array(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, IOSTATE_NAME], "print!"),
        print_io_func(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, DEBUG_NAME], "debug_print"),
        debug_print_function(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, DEBUG_NAME], "abort"),
        abort_function(),
    );
    fix_module.add_global_value(
        FullName::from_strs(&[STD_NAME, INT_NAME], "_int_to_string"),
        int_to_string_function(),
    );

    fix_module
}
