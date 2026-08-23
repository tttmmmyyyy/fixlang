//! Reading a construction where the code taking it apart can see it, as the program answers and as
//! the `--emit-rc-ir` dump shows it.
//!
//! The chains whose `advance` walked past elements by calling itself are asserted the same way: an
//! `advance` that calls itself is a body `inline` never moves, so a chain that keeps one leaves it
//! standing as a function of its own in the dump.
//!
//! What the rewrites are for cannot be observed by running the program — they leave the answer
//! unchanged — so the chain that motivates them is asserted on the dump: a function an iterator
//! carries in a field is called through a pointer, and the same function reached as an argument is
//! called by name. The answers are asserted at the levels above and below the one the pass runs at,
//! which is what says the rewrites preserve them.

#[cfg(test)]
mod integration_tests {
    use crate::constants::CAP_LIST_PREFIX;
    use crate::tests::test_util::{copy_dir_recursive, fix_command_at_opt_level};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
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

    /// What `split_field_order` prints: `run` folds `high` into `low` three times to reach 7321, and
    /// adds the weight of the value it was handed on each round, 3007 + 2073 + 1732.
    const SPLIT_FIELD_ORDER_OUTPUT: &str = "14133";

    /// The optimization levels the answers are asserted at: one below the level the passes run at,
    /// the level they run at, and the one above it.
    const OPT_LEVELS: [&str; 3] = ["basic", "max", "experimental"];

    /// Copies the cases into a temporary directory and gives back that directory together with the
    /// path of `case` inside it, so that cases run in parallel without sharing build output.
    fn setup_test_env(case: &str) -> (TempDir, PathBuf) {
        let mut cases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cases_dir.push("src/tests/test_collapse_constructions/cases");
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        copy_dir_recursive(&cases_dir, &temp_dir.path().to_path_buf())
            .expect("Failed to copy test cases");
        let project_dir = temp_dir.path().join(case);
        (temp_dir, project_dir)
    }

    /// Builds the case at `opt_level`, runs it, and returns the RC IR of every module.
    ///
    /// Running the program is what keeps the dump honest: the names asserted on come off a build
    /// that is known to answer correctly.
    fn build_run_and_read_rc_ir(
        project_dir: &Path,
        opt_level: &str,
        expected_output: &str,
    ) -> String {
        let build = fix_command_at_opt_level("build", opt_level)
            .args(["--emit-rc-ir", "all"])
            .current_dir(project_dir)
            .output()
            .expect("Failed to execute fix build");
        assert!(
            build.status.success(),
            "the build should succeed at -O {}.\nstdout: {}\nstderr: {}",
            opt_level,
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr),
        );

        let run = Command::new(project_dir.join("a.out"))
            .current_dir(project_dir)
            .output()
            .expect("Failed to execute the built program");
        assert!(
            run.status.success(),
            "the built program should run cleanly at -O {}, but exited with {}.\nstderr: {}",
            opt_level,
            run.status,
            String::from_utf8_lossy(&run.stderr),
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout).trim(),
            expected_output,
            "the program should answer the same at -O {}",
            opt_level
        );

        let dump_path = project_dir.join(".fixlang/rc_ir.post.txt");
        fs::read_to_string(&dump_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", dump_path.display(), e))
    }

    /// The largest number of parameters of capture-list type that any one function in `dump` takes.
    ///
    /// A function reached as an argument arrives as its capture list, which is a type of its own, and
    /// the body holding it calls that function by name. A function an iterator carries in a field
    /// arrives as a closure and is called through a pointer. So this says how many of a chain's
    /// functions one body knows the identity of.
    fn most_capture_lists_taken_by_one_function(dump: &str) -> usize {
        dump.lines()
            .filter_map(|line| line.strip_prefix("fn "))
            .map(|header| parameter_list(header).matches(CAP_LIST_PREFIX).count())
            .max()
            .unwrap_or(0)
    }

    /// The parameter list of a dumped function header, taken from the parenthesis that opens it to
    /// the one that closes it.
    ///
    /// A parameter's own type carries parentheses — a function type is written `(a) -> b` — so the
    /// list runs to the parenthesis that brings the nesting back to where it started.
    fn parameter_list(header: &str) -> &str {
        let Some(open) = header.find('(') else {
            return "";
        };
        let mut depth = 0;
        for (offset, character) in header[open..].char_indices() {
            match character {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return &header[open + 1..open + offset];
                    }
                }
                _ => {}
            }
        }
        ""
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

    /// A chain of two `map`s hands the fold two functions, each in a field of a struct that is in a
    /// field of the next, and a third as the fold's own operation. Reading each construction where
    /// the fold takes it apart is what makes all three arguments, so the fold receives three capture
    /// lists; the chain's two functions stay closures without it.
    #[test]
    pub fn test_the_functions_a_chain_of_iterators_carries_are_called_by_name() {
        let (_temp_dir, project_dir) = setup_test_env("nested_iterators");
        let dump = build_run_and_read_rc_ir(&project_dir, "max", NESTED_ITERATORS_OUTPUT);

        let known_functions = most_capture_lists_taken_by_one_function(&dump);
        assert!(
            known_functions >= 3,
            "the fold should receive the chain's two functions and its own operation as capture \
             lists, but the function taking the most of them takes {}",
            known_functions
        );
    }

    /// A chain of two iterators of one type constructor answers the same at every level, so the
    /// rewrites that flatten it — which hand the fold the value the outer one carries and the value
    /// the inner one carries, both named after the same field — keep the two apart.
    /// `filter` walks past the elements its predicate rejects, and that walk is a body of its own, so
    /// the `advance` handing back an accepted element goes into the loop consuming it.
    #[test]
    pub fn test_the_filter_chain_leaves_no_advance_of_its_own() {
        let (_temp_dir, project_dir) = setup_test_env("filter_chain");
        let dump = build_run_and_read_rc_ir(&project_dir, "max", FILTER_CHAIN_OUTPUT);

        let advances = advance_functions(&dump);
        assert!(
            advances.is_empty(),
            "no advance of the chain should stand as a function of its own, but these do:\n{}",
            advances.join("\n")
        );
    }

    /// The same for `filter_map`, whose walk passes the elements its function declines.
    #[test]
    pub fn test_the_filter_map_chain_leaves_no_advance_of_its_own() {
        let (_temp_dir, project_dir) = setup_test_env("filter_map_chain");
        let dump = build_run_and_read_rc_ir(&project_dir, "max", FILTER_MAP_CHAIN_OUTPUT);

        let advances = advance_functions(&dump);
        assert!(
            advances.is_empty(),
            "no advance of the chain should stand as a function of its own, but these do:\n{}",
            advances.join("\n")
        );
    }

    /// The same for `flatten`, whose walk passes the inner iterators that yield nothing.
    #[test]
    pub fn test_the_flatten_chain_leaves_no_advance_of_its_own() {
        let (_temp_dir, project_dir) = setup_test_env("flatten_chain");
        let dump = build_run_and_read_rc_ir(&project_dir, "max", FLATTEN_CHAIN_OUTPUT);

        let advances = advance_functions(&dump);
        assert!(
            advances.is_empty(),
            "no advance of the chain should stand as a function of its own, but these do:\n{}",
            advances.join("\n")
        );
    }

    #[test]
    pub fn test_a_chain_of_two_iterators_of_one_type_answers_the_same() {
        let (_temp_dir, project_dir) = setup_test_env("nested_iterators");
        for opt_level in OPT_LEVELS {
            build_run_and_read_rc_ir(&project_dir, opt_level, NESTED_ITERATORS_OUTPUT);
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
            build_run_and_read_rc_ir(&project_dir, opt_level, READ_CONSTRUCTIONS_OUTPUT);
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
            build_run_and_read_rc_ir(&project_dir, opt_level, SPLIT_FIELD_ORDER_OUTPUT);
        }
    }

    /// A field handed over as an argument of its own carries its name over the whole body, so a
    /// value bound to that name before the pattern is read as the field unless every local has a
    /// name of its own.
    #[test]
    pub fn test_a_value_named_like_a_field_before_the_pattern_stays_itself() {
        let (_temp_dir, project_dir) = setup_test_env("shadowed_field_name");
        for opt_level in OPT_LEVELS {
            build_run_and_read_rc_ir(&project_dir, opt_level, SHADOWED_FIELD_NAME_OUTPUT);
        }
    }

    /// An argument written beside the struct can name a field of the struct the call was given,
    /// while the struct argument builds the value the call passes on. The first reaches the call
    /// under a name of its own, so it keeps reading the value it was written against.
    #[test]
    pub fn test_an_argument_naming_a_field_reads_the_value_it_was_written_against() {
        let (_temp_dir, project_dir) = setup_test_env("field_in_another_argument");
        for opt_level in OPT_LEVELS {
            build_run_and_read_rc_ir(&project_dir, opt_level, FIELD_IN_ANOTHER_ARGUMENT_OUTPUT);
        }
    }
}
