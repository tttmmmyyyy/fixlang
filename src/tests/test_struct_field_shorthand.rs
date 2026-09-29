// A field of a struct literal or of a struct pattern can be written as its name alone: `S { x }`
// stands for `S { x : x }`. In a literal the name is then an expression naming a value, local or
// global; in a pattern it binds a variable of that name.

#[cfg(test)]
mod tests {
    use crate::{
        configuration::Configuration,
        tests::test_util::{test_source, test_source_fail},
    };

    /// A literal gives a field written as its name alone the local variable of that name, beside
    /// fields written with their values, in any order.
    #[test]
    pub fn test_struct_literal_shorthand_takes_local_variable() {
        let source = r#"
module Main;

type S = struct { x : I64, y : I64, z : I64 };

main : IO ();
main = (
    let x = 1;
    let z = 3;
    let s = S { z, y : 2, x };
    assert_eq(|_|"x", s.@x, 1);;
    assert_eq(|_|"y", s.@y, 2);;
    assert_eq(|_|"z", s.@z, 3);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    /// A field written as its name alone names whatever value that name resolves to: a global
    /// value where no local variable has the name, and the local variable where one shadows it.
    #[test]
    pub fn test_struct_literal_shorthand_resolves_name_as_expression() {
        let source = r#"
module Main;

type S = struct { x : I64, y : I64 };

x : I64;
x = 10;

main : IO ();
main = (
    let s = S { x, y : 0 };
    assert_eq(|_|"global", s.@x, 10);;
    let x = 20;
    let t = S { x, y : 0 };
    assert_eq(|_|"local", t.@x, 20);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    /// A field written as its name alone in a literal is an expression like any other, so a name
    /// that resolves to nothing is reported as an unknown name.
    #[test]
    pub fn test_struct_literal_shorthand_unknown_name_rejected() {
        let source = r#"
module Main;

type S = struct { x : I64 };

main : IO ();
main = (
    let s = S { x };
    println(s.@x.to_string)
);
"#;
        test_source_fail(source, Configuration::develop_mode(), "Unknown name `x`.");
    }

    /// A field written as its name alone counts as giving that field, so writing it again is a
    /// repeat.
    #[test]
    pub fn test_struct_literal_shorthand_counts_as_giving_the_field() {
        let source = r#"
module Main;

type S = struct { x : I64 };

main : IO ();
main = (
    let x = 1;
    let s = S { x, x : 2 };
    println(s.@x.to_string)
);
"#;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "Duplicate field `x` of struct `Main::S`.",
        );
    }

    /// A pattern binds a field written as its name alone to a variable of that name, beside fields
    /// written with their sub-patterns, in a `let`, a lambda parameter and a `match` arm alike, and
    /// inside another pattern.
    #[test]
    pub fn test_struct_pattern_shorthand_binds_variable() {
        let source = r#"
module Main;

type S = struct { x : I64, y : I64 };
type T = struct { s : S, name : String };

main : IO ();
main = (
    let t = T { s : S { x : 1, y : 2 }, name : "t" };

    let S { x, y : y2 } = t.@s;
    assert_eq(|_|"let x", x, 1);;
    assert_eq(|_|"let y", y2, 2);;

    let sum = |S { x, y }| x + y;
    assert_eq(|_|"lambda", sum(t.@s), 3);;

    let matched = match t.@s { S { y, x : _ } => y };
    assert_eq(|_|"match", matched, 2);;

    let T { s : S { x, _ }, name } = t;
    assert_eq(|_|"nested x", x, 1);;
    assert_eq(|_|"nested name", name, "t");;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    /// A variable a field written as its name alone binds is a binder of the pattern like any
    /// other, so binding its name again in the same pattern is a duplicate.
    #[test]
    pub fn test_struct_pattern_shorthand_binder_duplicate_rejected() {
        let source = r#"
module Main;

type S = struct { x : I64 };

main : IO ();
main = (
    let (S { x }, x) = (S { x : 1 }, 2);
    println(x.to_string)
);
"#;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "Duplicate name defined by pattern.",
        );
    }

    /// A pattern names a field written as its name alone like one written with its sub-pattern, so
    /// a name the struct does not declare is an unknown field.
    #[test]
    pub fn test_struct_pattern_shorthand_unknown_field_rejected() {
        let source = r#"
module Main;

type S = struct { x : I64 };

main : IO ();
main = (
    let S { z } = S { x : 1 };
    println(z.to_string)
);
"#;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "Unknown field `z` for struct `Main::S`.",
        );
    }
}
