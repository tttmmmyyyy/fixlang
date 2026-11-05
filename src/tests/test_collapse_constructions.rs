//! Reading a construction where the code taking it apart can see it, as the program answers and as
//! the `--emit-rc-ir` dump shows it.
//!
//! The chains whose `advance` walked past elements by calling itself are asserted the same way: an
//! `advance` that calls itself is a body `inline` never moves, so a chain that keeps one leaves it
//! standing as a function of its own in the dump.
//!
//! The answers are asserted at the levels above and below the one the pass runs at, which is what
//! says the rewrites preserve them.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{build_project_run_and_read_rc_ir, setup_case_projects};
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// What `nested_iterators` prints: the sum of `3 * i + 1` over `0..9`.
    const NESTED_ITERATORS_OUTPUT: &str = "145";

    /// What `read_constructions` prints: `sum_pair(4)` 17, `choose(3)` 12, `choose(-2)` -2,
    /// `magnitude(-7)` 7, `defaulted(2)` 12 and `defaulted(-3)` -3.
    const READ_CONSTRUCTIONS_OUTPUT: &str = "43";

    /// What `shadowed_field_name` prints: `f(2, ..)` reads the value bound before the pattern, so
    /// the two rounds contribute 4000 + 10 + 3 and 2000 + 50 + 6.
    const SHADOWED_FIELD_NAME_OUTPUT: &str = "6069";

    /// What `field_in_another_argument` prints: `f` adds the field it was given on each of three
    /// rounds, 1 + 2 + 4.
    const FIELD_IN_ANOTHER_ARGUMENT_OUTPUT: &str = "7";

    /// What `filter_chain` prints: the sum of the even values among `i % 251` over `0..999`.
    const FILTER_CHAIN_OUTPUT: &str = "62502";

    /// What `filter_map_chain` prints: twice `FILTER_CHAIN_OUTPUT`.
    const FILTER_MAP_CHAIN_OUTPUT: &str = "125004";

    /// What `flatten_chain` prints: the sum of `r + i` over 50 rows of 20.
    const FLATTEN_CHAIN_OUTPUT: &str = "34000";

    /// What `dyn_flatten_chain` prints: each of `0..2` twice, times ten.
    const DYN_FLATTEN_CHAIN_OUTPUT: &str = "0,0,10,10,20,20";

    /// What `field_name_reused_by_later_run` prints: each of the three rounds reads the `some` its
    /// triple holds, `3i + (i + 7) + 7` over `i` in `1..3`.
    const FIELD_NAME_REUSED_BY_LATER_RUN_OUTPUT: &str = "66";

    /// What `split_field_order` prints: `run` folds `high` into `low` three times to reach 7321, and
    /// adds the weight of the value it was handed on each round, 3007 + 2073 + 1732.
    const SPLIT_FIELD_ORDER_OUTPUT: &str = "14133";

    /// The optimization levels the answers are asserted at: one below the level the passes run at,
    /// the level they run at, and the one above it.
    const OPT_LEVELS: [&str; 3] = ["basic", "max", "experimental"];

    /// Copies the cases into a temporary directory and gives back that directory together with the
    /// path of `case` inside it, so that cases run in parallel without sharing build output.
    fn setup_test_env(case: &str) -> (TempDir, PathBuf) {
        setup_case_projects("src/tests/test_collapse_constructions/cases", case)
    }

    /// The functions of `dump` whose name says they are an iterator's `advance`.
    ///
    /// An `advance` that walks past elements by calling itself is a body `inline` never moves, so the
    /// loop consuming the iterator calls it once per element. Handing that walk to a body of its own
    /// leaves `advance` small enough to go into the consumer, and then none stands as a function.
    fn advance_functions(dump: &str) -> Vec<&str> {
        dump.lines()
            .filter(|line| line.starts_with("fn Std::Iterator::advance"))
            .collect()
    }

    /// Builds `case`, checks that it answers `expected_output`, and asserts that `standing` finds
    /// nothing of `subject` left as a function of its own in the RC IR dump.
    fn assert_leaves_no_function_of_its_own(
        case: &str,
        expected_output: &str,
        standing: impl Fn(&str) -> Vec<&str>,
        subject: &str,
    ) {
        let (_temp_dir, project_dir) = setup_test_env(case);
        let dump = build_project_run_and_read_rc_ir(&project_dir, "max", expected_output);

        let left = standing(&dump);
        assert!(
            left.is_empty(),
            "no {} should stand as a function of its own, but these do:\n{}",
            subject,
            left.join("\n")
        );
    }

    /// `filter` walks past the elements its predicate rejects, and that walk is a body of its own,
    /// so the `advance` handing back an accepted element is small enough to go into the loop
    /// consuming it, and none is left standing.
    #[test]
    pub fn test_the_filter_chain_leaves_no_advance_of_its_own() {
        assert_leaves_no_function_of_its_own(
            "filter_chain",
            FILTER_CHAIN_OUTPUT,
            advance_functions,
            "advance of the chain",
        );
    }

    /// The same for `filter_map`, whose walk passes the elements its function declines.
    #[test]
    pub fn test_the_filter_map_chain_leaves_no_advance_of_its_own() {
        assert_leaves_no_function_of_its_own(
            "filter_map_chain",
            FILTER_MAP_CHAIN_OUTPUT,
            advance_functions,
            "advance of the chain",
        );
    }

    /// The same for `flatten`, whose walk passes the inner iterators that yield nothing.
    #[test]
    pub fn test_the_flatten_chain_leaves_no_advance_of_its_own() {
        assert_leaves_no_function_of_its_own(
            "flatten_chain",
            FLATTEN_CHAIN_OUTPUT,
            advance_functions,
            "advance of the chain",
        );
    }

    /// A chain of two iterators of one type constructor answers the same at every level, which is
    /// what says the rewrites that flatten it keep the two apart: they hand the fold the value the
    /// outer one carries and the value the inner one carries, both named after the same field.
    #[test]
    pub fn test_a_chain_of_two_iterators_of_one_type_answers_the_same() {
        let (_temp_dir, project_dir) = setup_test_env("nested_iterators");
        for opt_level in OPT_LEVELS {
            build_project_run_and_read_rc_ir(&project_dir, opt_level, NESTED_ITERATORS_OUTPUT);
        }
    }

    /// A struct built out of expressions and taken apart, a union each branch of an `if` builds a
    /// different variant of, one both branches build the same variant of, and one whose reader has
    /// no arm for the variant either branch builds, all answer the same as they do at the level the
    /// rewrites do not run at.
    #[test]
    pub fn test_a_construction_read_where_it_is_built_answers_the_same() {
        let (_temp_dir, project_dir) = setup_test_env("read_constructions");
        for opt_level in OPT_LEVELS {
            build_project_run_and_read_rc_ir(&project_dir, opt_level, READ_CONSTRUCTIONS_OUTPUT);
        }
    }

    /// A struct argument whose pattern writes its fields in the reverse of the order they are
    /// declared, read whole besides, answers the same at every level. Both fields hold an `I64`, so
    /// handing the fields over in the order the pattern writes them type checks, and the value is
    /// what says the two orders were brought together — both where the fields are handed to the
    /// function taking them one by one, and where the body that still names the struct rebuilds it.
    #[test]
    pub fn test_a_struct_argument_taken_apart_out_of_declaration_order_answers_the_same() {
        let (_temp_dir, project_dir) = setup_test_env("split_field_order");
        for opt_level in OPT_LEVELS {
            build_project_run_and_read_rc_ir(&project_dir, opt_level, SPLIT_FIELD_ORDER_OUTPUT);
        }
    }

    /// A field handed over as an argument of its own carries its name over the whole body, so a
    /// value bound to that name before the pattern is read as the field unless every local has a
    /// name of its own.
    #[test]
    pub fn test_a_value_named_like_a_field_before_the_pattern_stays_itself() {
        let (_temp_dir, project_dir) = setup_test_env("shadowed_field_name");
        for opt_level in OPT_LEVELS {
            build_project_run_and_read_rc_ir(&project_dir, opt_level, SHADOWED_FIELD_NAME_OUTPUT);
        }
    }

    /// An argument written beside the struct can name a field of the struct the call was given,
    /// while the struct argument builds the value the call passes on. The first reaches the call
    /// under a name of its own, so it keeps reading the value it was written against.
    #[test]
    pub fn test_an_argument_naming_a_field_reads_the_value_it_was_written_against() {
        let (_temp_dir, project_dir) = setup_test_env("field_in_another_argument");
        for opt_level in OPT_LEVELS {
            build_project_run_and_read_rc_ir(&project_dir, opt_level, FIELD_IN_ANOTHER_ARGUMENT_OUTPUT);
        }
    }

    /// A struct built from a field the program names and a field it computes, where the name was
    /// bound by an earlier run of the pass: the name bound for the computed field is one the struct
    /// does not already read, so each field keeps the value it was written with.
    #[test]
    pub fn test_a_field_named_by_an_earlier_run_keeps_its_value() {
        let (_temp_dir, project_dir) = setup_test_env("dyn_flatten_chain");
        for opt_level in OPT_LEVELS {
            build_project_run_and_read_rc_ir(&project_dir, opt_level, DYN_FLATTEN_CHAIN_OUTPUT);
        }
    }

    /// A name an earlier run of the pass bound to a union, read after a later run binds a field of
    /// another struct in the same global: the name the later run chooses is one the global does not
    /// bind yet, so the reader is given the union the name was bound to.
    #[test]
    pub fn test_a_field_name_from_an_earlier_run_is_read_as_it_was_bound() {
        let (_temp_dir, project_dir) = setup_test_env("field_name_reused_by_later_run");
        for opt_level in OPT_LEVELS {
            build_project_run_and_read_rc_ir(
                &project_dir,
                opt_level,
                FIELD_NAME_REUSED_BY_LATER_RUN_OUTPUT,
            );
        }
    }
}
