// Tests for updating a field of an unboxed struct: `set_x`, `mod_x`, and the punch/plug builtins
// they are built on. An unboxed struct is taken apart and put back together in registers, so an
// update carries the fields it does not touch straight through — including a boxed one, which must
// therefore still be cloned when something else holds it, and released exactly once when it is
// dropped. The memory-safety test checks the release side under valgrind.

#[cfg(test)]
mod struct_rmw_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    #[test]
    pub fn test_unboxed_struct_field_update_correctness() {
        let source = r#"
module Main;

type Rec = unbox struct { arr : Array I64, tag : I64 };

main : IO () = (
    // The field's array is held outside the struct as well, so an update through the struct clones.
    let arr = [1, 2, 3];
    let r = Rec { arr : arr, tag : 0 };
    let r = r.mod_arr(|a| a.set(0, 99));
    assert_eq(|_|"outside holder intact", arr.@(0), 1);;
    assert_eq(|_|"field updated", r.@arr.@(0), 99);;

    // Two struct values holding the same array.
    let r1 = Rec { arr : [1, 2, 3], tag : 0 };
    let r2 = r1;
    let r1 = r1.mod_arr(|a| a.set(0, 99));
    assert_eq(|_|"other struct intact", r2.@arr.@(0), 1);;
    assert_eq(|_|"modified struct updated", r1.@arr.@(0), 99);;

    // `set` of the field with a shared array, with a sibling update in between: the sibling update
    // carries the array field through, and the array is still shared.
    let shared = [7, 8];
    let r3 = Rec { arr : [0], tag : 0 };
    let r3 = r3.set_arr(shared);
    let r3 = r3.set_tag(5);
    let r3 = r3.mod_arr(|a| a.set(1, 77));
    assert_eq(|_|"shared array intact", shared.@(1), 8);;
    assert_eq(|_|"set field updated", r3.@arr.@(1), 77);;
    assert_eq(|_|"sibling updated", r3.@tag, 5);;

    // A loop threading the struct, which is where the field update is worth carrying through: the
    // array is unique the whole way, so every write lands in place.
    let s = Rec { arr : Array::fill(4, 0), tag : 0 };
    let s = loop((0, s), |(i, s)|
        if i == 4 { break $ s };
        continue $ (i + 1, s.mod_arr(|a| a.set(i, i * 2)))
    );
    assert_eq(|_|"loop result", s.@arr, [0, 2, 4, 6]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_unboxed_struct_field_update_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let source = r#"
module Main;

type Boxes = unbox struct { xs : Array (Array I64), ys : Array I64 };

main : IO () = (
    // Updating one boxed field and replacing the other: the replaced field's old value is released
    // once, and the field carried through is not.
    let b = Boxes { xs : [[1], [2]], ys : [3] };
    let b = b.mod_xs(|xs| xs.mod(0, |x| x.push_back(9)));
    let b = b.set_ys([4, 5]);
    assert_eq(|_|"boxed field", b.@xs.@(0), [1, 9]);;
    assert_eq(|_|"boxed sibling", b.@ys, [4, 5]);;

    // The whole struct dropped after an update releases both fields exactly once.
    let dropped = Boxes { xs : [[1]], ys : [2] };
    eval dropped.mod_ys(|ys| ys.push_back(3));

    // An update through a struct whose boxed field is shared clones the field, leaving both the
    // holder and the result to be released independently.
    let shared = [[1], [2]];
    let b2 = Boxes { xs : shared, ys : [] };
    let b2 = b2.mod_xs(|xs| xs.push_back([3]));
    assert_eq(|_|"holder intact", shared.@size, 2);;
    assert_eq(|_|"result grown", b2.@xs.@size, 3);;
    pure()
);
"#;
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(source, config);
    }
}
