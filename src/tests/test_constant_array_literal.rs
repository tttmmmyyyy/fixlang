// An array literal whose elements are all number literals evaluates to an array whose storage is a
// constant in the program's data: no evaluation allocates it or writes its elements. The storage is
// shared by every evaluation, in memory the program may not write, so each way of changing an array
// has to copy it first.

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::configuration::{Configuration, FixOptimizationLevel};
    use crate::tests::test_util::{build_run_and_read_rc_ir, build_within_and_run, test_source};

    /// The levels a program writing into a constant is run at. A write into read-only memory faults
    /// at `-O none` and `-O basic`; at `-O max`, LLVM may delete the store instead, so that level
    /// alone would let a missing copy pass.
    const OPT_LEVELS: [FixOptimizationLevel; 3] = [
        FixOptimizationLevel::None,
        FixOptimizationLevel::Basic,
        FixOptimizationLevel::Max,
    ];

    /// A literal of each numeric type holds the numbers written in it, the extremes of the type
    /// among them.
    #[test]
    fn test_an_array_literal_of_numbers_holds_them() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let i8s = [-128_I8, -1_I8, 0_I8, 127_I8];
            assert_eq(|_|"I8", i8s.to_iter.map(to_string).join(","), "-128,-1,0,127");;
            let u8s = [0_U8, 'a', 255_U8];
            assert_eq(|_|"U8", u8s.to_iter.map(to_string).join(","), "0,97,255");;
            let i16s = [-32768_I16, 32767_I16];
            assert_eq(|_|"I16", i16s.to_iter.map(to_string).join(","), "-32768,32767");;
            let u16s = [0_U16, 65535_U16];
            assert_eq(|_|"U16", u16s.to_iter.map(to_string).join(","), "0,65535");;
            let i32s = [-2147483648_I32, 2147483647_I32];
            assert_eq(|_|"I32", i32s.to_iter.map(to_string).join(","), "-2147483648,2147483647");;
            let u32s = [0_U32, 4294967295_U32];
            assert_eq(|_|"U32", u32s.to_iter.map(to_string).join(","), "0,4294967295");;
            let i64s = [-9223372036854775808, -1, 9223372036854775807];
            assert_eq(|_|"I64", i64s.to_iter.map(to_string).join(","), "-9223372036854775808,-1,9223372036854775807");;
            let u64s = [0_U64, 0xFFFFFFFFFFFFFFFF_U64];
            assert_eq(|_|"U64", u64s.to_iter.map(to_string).join(","), "0,18446744073709551615");;
            let i128s = [-170141183460469231731687303715884105728_I128, 170141183460469231731687303715884105727_I128];
            assert_eq(|_|"I128", i128s.to_iter.map(to_string).join(","), "-170141183460469231731687303715884105728,170141183460469231731687303715884105727");;
            let u128s = [1_U128, 340282366920938463463374607431768211455_U128];
            assert_eq(|_|"U128", u128s.to_iter.map(to_string).join(","), "1,340282366920938463463374607431768211455");;
            let f32s = [-1.5_F32, 0.0_F32, 3.0e38_F32];
            assert_eq(|_|"F32", f32s.to_iter.map(to_string).join(","), "-1.5,0.0,3.0e38");;
            let f64s = [-0.0, 0.1, 1.7976931348623157e308];
            assert_eq(|_|"F64", f64s.to_iter.map(to_string).join(","), "-0.0,0.1,1.7976931348623157e308");;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// Each way of changing an array changes a copy of a literal's storage, and the literal goes on
    /// holding what it was written with.
    #[test]
    fn test_changing_an_array_literal_of_numbers_leaves_the_literal_alone() {
        let source = r#"
        module Main;

        // A literal evaluated afresh at each call.
        fresh : () -> Array I64;
        fresh = |_| [1, 2, 3];

        // A literal held by a global value.
        table : Array I64;
        table = [10, 20, 30];

        main : IO ();
        main = (
            assert_eq(|_|"set", fresh().set(0, 9), [9, 2, 3]);;
            assert_eq(|_|"mod", fresh().mod(1, |x| x + 5), [1, 7, 3]);;
            assert_eq(|_|"push_back", fresh().push_back(4), [1, 2, 3, 4]);;
            assert_eq(|_|"pop_back", fresh().pop_back, [1, 2]);;
            assert_eq(|_|"append", fresh().append([4, 5]), [1, 2, 3, 4, 5]);;
            assert_eq(|_|"truncate", fresh().truncate(1), [1]);;
            assert_eq(|_|"sort_by", fresh().sort_by(|(a, b)| a > b), [3, 2, 1]);;
            assert_eq(|_|"the literal after the changes", fresh(), [1, 2, 3]);;
            assert_eq(|_|"set on a global", table.set(1, 0), [10, 0, 30]);;
            assert_eq(|_|"the global after the change", table, [10, 20, 30]);;
            pure()
        );
        "#;
        for opt_level in OPT_LEVELS {
            let mut config = Configuration::develop_mode();
            config.set_fix_opt_level(opt_level);
            test_source(source, config);
        }
    }

    /// A literal of number literals is not built at run time, and a literal holding anything else, or
    /// nothing, is.
    #[test]
    fn test_only_an_array_literal_of_number_literals_is_a_constant() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let args = *IO::get_args;
            let n = args.@size;
            let numbers = [5, 6, 7];
            let computed = [n, 6, 7];
            let empty = [] : Array I64;
            println((numbers.@(0) + computed.@(0) + empty.@size).to_string)
        );
        "#;
        let dump = build_run_and_read_rc_ir(
            source,
            "none",
            "6",
            "a literal of numbers beside one holding a computed element",
        );
        assert!(
            dump.contains("constant_array_lit(int(5), int(6), int(7))"),
            "the literal of number literals should be a constant; the dump is:\n{}",
            dump
        );
        assert_eq!(
            dump.matches("constant_array_lit(").count(),
            1,
            "only the literal of number literals should be a constant; the dump is:\n{}",
            dump
        );
    }

    /// The number of elements of the literal `test_a_long_array_literal_compiles_in_reasonable_time`
    /// compiles. A literal this long builds in a few seconds where the compiler's work is linear in
    /// the elements, and in minutes where any one pass over the literal rebuilds the list of elements
    /// once per element.
    const LONG_LITERAL_LENGTH: usize = 120000;

    /// Generous next to the few seconds the build takes, and well short of the minutes it takes
    /// once the work of one pass is quadratic in the elements.
    const LONG_LITERAL_TIMEOUT: Duration = Duration::from_secs(60);

    /// A literal of over a hundred thousand numbers compiles in time linear in its length.
    #[test]
    fn test_a_long_array_literal_compiles_in_reasonable_time() {
        let elements = (0..LONG_LITERAL_LENGTH)
            .map(|i| format!("{}_U64", i))
            .collect::<Vec<_>>()
            .join(", ");
        let source = format!(
            "module Main;\n\
             \n\
             table : Array U64;\n\
             table = [{elements}];\n\
             \n\
             main : IO ();\n\
             main = println((table.@size.u64 + table.@({last})).to_string);\n",
            elements = elements,
            last = LONG_LITERAL_LENGTH - 1,
        );
        let printed = build_within_and_run(
            &source,
            "max",
            LONG_LITERAL_TIMEOUT,
            &format!("an array literal of {} numbers", LONG_LITERAL_LENGTH),
        );
        assert_eq!(printed, (2 * LONG_LITERAL_LENGTH - 1).to_string());
    }
}
