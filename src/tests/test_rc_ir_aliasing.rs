// A value read out of a union, a struct, a closure or a tuple keeps the reference its container
// holds, so mutating that value copies it and leaves the container's own copy as it was. Each case
// keeps the container alive across the mutation and asserts that what it holds still reads as it
// did.

#[cfg(test)]
mod rc_ir_aliasing_tests {
    use crate::{configuration::Configuration, tests::test_util::test_source};

    /// Reading a value out of a container and mutating it leaves the container's own copy as it was.
    /// The containers covered are an `Option::some` built on the spot, a boxed and an unboxed union,
    /// a boxed and an unboxed struct, a closure capture, and one array stored in both fields of a
    /// tuple.
    #[test]
    pub fn test_container_keeps_its_reference_across_a_read_out() {
        let source = r#"
            module Main;

            type BU = box union { a : Array I64, b : I64 };
            type UU = unbox union { p : Array I64, q : I64 };
            type S = struct { xs : Array I64, n : I64 };
            type BS = box struct { xs : Array I64, n : I64 };

            main : IO ();
            main = (
                // A union built on the spot, whose payload the enclosing scope also holds.
                let base = Array::fill(4, 0);
                let taken = match Option::some(base) { some(x) => x.set(0, 111), none() => base };
                assert_eq(|_|"payload mutated", taken.@(0), 111);;
                assert_eq(|_|"payload unchanged", base.@(0), 0);;

                // A boxed union still alive after its payload is read out.
                let boxed = BU::a(Array::fill(4, 0));
                let out_of_boxed = match boxed { a(x) => x, b(_) => Array::fill(1, 0) };
                let mutated_boxed = out_of_boxed.set(1, 222);
                assert_eq(|_|"boxed union payload mutated", mutated_boxed.@(1), 222);;
                assert_eq(
                    |_|"boxed union payload unchanged",
                    match boxed { a(x) => x.@(1), b(_) => -1 }, 0
                );;

                // An unboxed union still alive after its payload is read out.
                let unboxed = UU::p(Array::fill(4, 0));
                let out_of_unboxed = match unboxed { p(x) => x, q(_) => Array::fill(1, 0) };
                let mutated_unboxed = out_of_unboxed.set(2, 333);
                assert_eq(|_|"unboxed union payload mutated", mutated_unboxed.@(2), 333);;
                assert_eq(
                    |_|"unboxed union payload unchanged",
                    match unboxed { p(x) => x.@(2), q(_) => -1 }, 0
                );;

                // An unboxed struct destructured while the struct is still alive.
                let plain = S { xs : Array::fill(4, 0), n : 1 };
                let S { xs : plain_field, n : _ } = plain;
                assert_eq(|_|"unboxed struct field mutated", plain_field.set(3, 444).@(3), 444);;
                assert_eq(|_|"unboxed struct field unchanged", plain.@xs.@(3), 0);;

                // A boxed struct destructured while the struct is still alive.
                let boxed_struct = BS { xs : Array::fill(4, 0), n : 1 };
                let BS { xs : boxed_field, n : _ } = boxed_struct;
                assert_eq(|_|"boxed struct field mutated", boxed_field.set(3, 555).@(3), 555);;
                assert_eq(|_|"boxed struct field unchanged", boxed_struct.@xs.@(3), 0);;

                // A closure capturing an array the enclosing scope also holds.
                let captured = Array::fill(4, 0);
                let writer = |k| captured.set(0, k);
                assert_eq(|_|"capture mutated", writer(666).@(0), 666);;
                assert_eq(|_|"capture unchanged", captured.@(0), 0);;

                // One array stored in both fields of a tuple.
                let twice = Array::fill(4, 0);
                let pair = (twice, twice);
                assert_eq(|_|"tuple field mutated", pair.@0.set(0, 777).@(0), 777);;
                assert_eq(|_|"tuple field unchanged", pair.@1.@(0), 0);;

                pure()
            );
        "#;
        test_source(&source, Configuration::develop_mode());
    }
}
