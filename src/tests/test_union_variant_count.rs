// A union value says which variant it was created as by carrying that variant's index in a tag of
// `UNION_TAG_BITS` bits, so a union declares at most as many variants as that tag tells apart.
// `Program::validate_type_defns` rejects a declaration of more; below that boundary every variant
// keeps its own tag, in the methods a union defines, in the switch a `match` compiles to, and in the
// walk the reference counter makes over a union to find the payload it has to release.

use crate::configuration::Configuration;
use crate::constants::MAX_UNION_VARIANTS;
use crate::elaboration::elaborate_via_config;
use crate::error::panic_if_err;
use crate::misc::save_temporary_source;
use crate::tests::test_util::{test_source, test_source_fail};

/// A program declaring a union `Many` of `variants` variants named `v0` onward, whose `main` is
/// `main_body`. Variant 0 is boxed, so the reference counter walks the tag to decide whether the
/// value holds it.
fn union_of_variants_with_main(variants: usize, main_body: &str) -> String {
    let mut source = "module Main;\n\ntype Many = union {\n    v0 : Box I64".to_string();
    for i in 1..variants {
        source += &format!(",\n    v{} : I64", i);
    }
    source += "\n};\n\n";
    source += main_body;
    source
}

/// A `main` that binds `x` to a value of the last of `variants` variants, carrying `7`, and then
/// runs `body`. The variant is chosen by a condition read from the program's arguments, which the
/// compiler cannot fold away, so the tag survives into the running program.
fn main_on_a_value_of_the_last_variant(variants: usize, body: &str) -> String {
    let last = variants - 1;
    format!(
        "main : IO ();\n\
         main = (\n\
         \x20   let args = *IO::get_args;\n\
         \x20   let x = if args.@size > 0 {{ Many::v{last}(7) }} else {{ Many::v0(Box {{ value : 1 }}) }};\n\
         {body}\
         \x20   pure()\n\
         );\n"
    )
}

/// A program that shares a value of the last variant and asks it which variant it is.
fn union_of_variants(variants: usize) -> String {
    let last = variants - 1;
    let body = format!(
        "\x20   let y = x;\n\
         \x20   assert_eq(|_|\"the variant the value was created as\", y.is_v{last}, true);;\n\
         \x20   assert_eq(|_|\"a variant the value was not created as\", y.is_v0, false);;\n\
         \x20   assert_eq(|_|\"the payload of the variant the value was created as\", x.as_v{last}, 7);;\n"
    );
    union_of_variants_with_main(variants, &main_on_a_value_of_the_last_variant(variants, &body))
}

/// The error a declaration of a union of `variants` variants is reported with, where `variants`
/// exceeds the number of variants a tag tells apart.
fn too_many_variants_error(variants: usize) -> String {
    format!(
        "Union `Main::Many` has {} variants, but a union can have at most {} variants.",
        variants, MAX_UNION_VARIANTS
    )
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

/// A `match` takes the arm of the variant the value was created as, at the last variant of a union
/// of as many variants as the tag tells apart.
///
/// A `match` chooses its arm by a switch over the tag, which the methods of a union do not build.
/// The scrutinee is read out of an array, at an index the program computes from its arguments,
/// because a `match` on a value the compiler traces back to a construction is replaced by the arm it
/// knows is taken, and then no tag is compared at all.
#[test]
pub fn test_match_takes_the_arm_of_the_last_variant() {
    let last = MAX_UNION_VARIANTS - 1;
    let main_body = format!(
        "main : IO ();\n\
         main = (\n\
         \x20   let args = *IO::get_args;\n\
         \x20   let values = [Many::v{last}(7), Many::v0(Box {{ value : 1 }})];\n\
         \x20   let x = values.@(if args.@size > 0 {{ 0 }} else {{ 1 }});\n\
         \x20   let taken = match x {{ v{last}(v) => v, _ => -1 }};\n\
         \x20   assert_eq(|_|\"the payload the arm of the variant bound\", taken, 7);;\n\
         \x20   pure()\n\
         );\n"
    );
    test_source(
        &union_of_variants_with_main(MAX_UNION_VARIANTS, &main_body),
        Configuration::develop_mode(),
    );
}

/// `mod_v` of the variant a value was created as applies the function to the payload and returns a
/// value that answers as that variant, at the last variant of a union of as many variants as the tag
/// tells apart. `mod_v` is the one method that writes a tag into the value it returns.
#[test]
pub fn test_mod_of_the_last_variant_modifies_its_payload() {
    let last = MAX_UNION_VARIANTS - 1;
    let body = format!(
        "\x20   let m = x.mod_v{last}(|v| v + 1);\n\
         \x20   assert_eq(|_|\"the variant the modified value answers as\", m.is_v{last}, true);;\n\
         \x20   assert_eq(|_|\"the payload the function returned\", m.as_v{last}, 8);;\n"
    );
    test_source(
        &union_of_variants_with_main(
            MAX_UNION_VARIANTS,
            &main_on_a_value_of_the_last_variant(MAX_UNION_VARIANTS, &body),
        ),
        Configuration::develop_mode(),
    );
}

/// A union of more variants than the tag tells apart is reported as an error.
#[test]
pub fn test_union_of_more_variants_than_the_tag_holds_is_rejected() {
    test_source_fail(
        &union_of_variants(MAX_UNION_VARIANTS + 1),
        Configuration::develop_mode(),
        &too_many_variants_error(MAX_UNION_VARIANTS + 1),
    );
}

/// A run that only checks the program, as `fix check` and the editor make, reports the union of more
/// variants than the tag tells apart. Such a run type-checks the modules it is asked about and
/// returns before the checks that need the whole program, so where the declaration is checked
/// decides whether the editor ever shows this error.
#[test]
pub fn test_union_of_more_variants_than_the_tag_holds_is_reported_by_a_check() {
    let source = union_of_variants(MAX_UNION_VARIANTS + 1);
    let saved = panic_if_err(save_temporary_source(&source, "union_variant_count_check"));
    let mut config = panic_if_err(Configuration::check_mode());
    config.add_user_source_file(saved.file_path);

    let errors = elaborate_via_config(&config)
        .err()
        .expect("a union of more variants than the tag tells apart is reported by a check");
    let expected = too_many_variants_error(MAX_UNION_VARIANTS + 1);
    assert!(
        errors.to_string().contains(&expected),
        "the check reports `{}`, and it reported {}",
        expected,
        errors
    );
}

/// The limit is the number of variants the manual promises a union may have.
#[test]
pub fn test_the_limit_is_the_number_of_variants_the_manual_promises() {
    assert_eq!(
        MAX_UNION_VARIANTS, 256,
        "`Document.md` tells the programmer a union may have at most 256 variants."
    );
}
