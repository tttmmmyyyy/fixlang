// A struct pattern names every field of its struct, or writes `_` after the fields it names to leave
// out the rest. So `_` alone is not a field or variant name.

#[cfg(test)]
mod tests {
    use crate::{
        configuration::Configuration,
        tests::test_util::{assert_grammar_accepts, assert_grammar_rejects, test_source},
    };

    /// `_` after the fields of a struct pattern leaves out the other fields, in a `let`, a lambda
    /// parameter and a `match` arm alike, and inside another pattern. A trailing comma may follow.
    #[test]
    pub fn test_struct_pattern_rest_leaves_out_the_other_fields() {
        let source = r#"
module Main;

type S = struct { x : I64, y : I64, z : I64 };
type T = struct { s : S, name : String };

main : IO ();
main = (
    let t = T { s : S { x : 1, y : 2, z : 3 }, name : "t" };

    let S { y, _ } = t.@s;
    assert_eq(|_|"let", y, 2);;

    let get_z = |S { z : zz, _, }| zz;
    assert_eq(|_|"lambda", get_z(t.@s), 3);;

    let matched = match t.@s { S { x, y, _ } => x + y };
    assert_eq(|_|"match", matched, 3);;

    let T { s : S { x, _ }, _ } = t;
    assert_eq(|_|"nested", x, 1);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    /// `_` in a struct pattern comes after at least one field, and last.
    #[test]
    pub fn test_struct_pattern_rest_comes_last_after_a_field() {
        let program = |pattern: &str| {
            format!(
                "module Main;\ntype S = struct {{ x : I64, y : I64 }};\nmain : IO ();\nmain = (\n    let {} = S {{ x : 1, y : 2 }};\n    pure()\n);\n",
                pattern
            )
        };
        assert_grammar_accepts(&program("S { x, _ }"));
        assert_grammar_accepts(&program("S { x, _, }"));
        assert_grammar_rejects(&program("S { _ }"));
        assert_grammar_rejects(&program("S { _, x }"));
        assert_grammar_rejects(&program("S { x, _, _ }"));
    }

    /// A struct literal gives every field a value, so `_` has no place among its fields.
    #[test]
    pub fn test_struct_literal_rejects_rest() {
        assert_grammar_rejects(
            "module Main;\ntype S = struct { x : I64, y : I64 };\nmain : IO ();\nmain = (\n    let s = S { x : 1, _ };\n    pure()\n);\n",
        );
    }

    /// `_` alone is not the name of a struct field or of a union variant, while a name that starts
    /// with `_` is.
    #[test]
    pub fn test_underscore_is_not_a_field_or_variant_name() {
        let program =
            |decl: &str| format!("module Main;\n{}\nmain : IO ();\nmain = pure();\n", decl);
        assert_grammar_rejects(&program("type S = struct { _ : I64, y : I64 };"));
        assert_grammar_rejects(&program("type U = union { _ : I64, w : Bool };"));
        assert_grammar_accepts(&program("type S = struct { _x : I64, y : I64 };"));
        assert_grammar_accepts(&program("type U = union { _v : I64, w : Bool };"));
    }
}
