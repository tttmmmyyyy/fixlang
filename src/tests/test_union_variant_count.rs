// A union value says which variant it was created as by carrying that variant's index in a tag of
// `UNION_TAG_BITS` bits, so a union declares at most as many variants as that tag tells apart.
// `Program::validate_type_defns` rejects a declaration of more; below that boundary every variant
// keeps its own tag, including in the walk the reference counter makes over a union to find the
// payload it has to release.

use crate::configuration::Configuration;
use crate::constants::MAX_UNION_VARIANTS;
use crate::tests::test_util::{test_source, test_source_fail};

/// A program declaring a union `Many` of `variants` variants named `v0` onward. Variant 0 is boxed,
/// so the reference counter walks the tag to decide whether the value holds it. `main` builds a
/// value of the last variant, under a condition it reads from the program's arguments so that the
/// compiler cannot fold the tag away, shares it, and asks it which variant it is.
fn union_of_variants(variants: usize) -> String {
    let mut source = "module Main;\n\ntype Many = union {\n    v0 : Box I64".to_string();
    for i in 1..variants {
        source += &format!(",\n    v{} : I64", i);
    }
    source += "\n};\n";
    let last = variants - 1;
    source += &format!(
        "\n\
         main : IO ();\n\
         main = (\n\
         \x20   let args = *IO::get_args;\n\
         \x20   let x = if args.@size > 0 {{ Many::v{last}(7) }} else {{ Many::v0(Box {{ value : 1 }}) }};\n\
         \x20   let y = x;\n\
         \x20   assert_eq(|_|\"the variant the value was created as\", y.is_v{last}, true);;\n\
         \x20   assert_eq(|_|\"a variant the value was not created as\", y.is_v0, false);;\n\
         \x20   assert_eq(|_|\"the payload of the variant the value was created as\", x.as_v{last}, 7);;\n\
         \x20   pure()\n\
         );\n"
    );
    source
}

/// A union of as many variants as the tag tells apart compiles, and each of its values answers for
/// the one variant it was created as.
#[test]
pub fn test_union_of_the_greatest_number_of_variants_tells_its_variants_apart() {
    test_source(
        &union_of_variants(MAX_UNION_VARIANTS),
        Configuration::develop_mode(),
    );
}

/// A union of more variants than the tag tells apart is reported as an error.
#[test]
pub fn test_union_of_more_variants_than_the_tag_holds_is_rejected() {
    test_source_fail(
        &union_of_variants(MAX_UNION_VARIANTS + 1),
        Configuration::develop_mode(),
        &format!(
            "Union `Main::Many` has {} variants, but a union can have at most {} variants.",
            MAX_UNION_VARIANTS + 1,
            MAX_UNION_VARIANTS
        ),
    );
}
