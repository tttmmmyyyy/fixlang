//! Integration tests for the RC IR provenance analysis, checked through the `--emit-rc-ir` dump.
//!
//! The dump annotates each variable binding with the provenance the analysis computed, so a small
//! program with named `let`s pins the analysis end to end: allocators produce `fresh` values,
//! reading a boxed element out of a boxed container is `unknown`, and constructing an unboxed tuple
//! carries each component's provenance through.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{
        build_project_and_read_rc_ir, build_project_run_and_read_rc_ir, copy_dir_recursive,
        rc_ir_callee_of,
    };
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    /// The directory in the source tree holding the case projects these tests build.
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_provenance/cases");
        path
    }

    /// Copy the test cases into a temporary directory of this test's own, so that parallel test
    /// runs stay out of each other's way, and return that directory together with the directory of
    /// the case project named by `case`.
    fn setup_test_env(case: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dst = temp_dir.path().to_path_buf();
        copy_dir_recursive(&get_test_cases_dir(), &dst).expect("Failed to copy test cases");
        let project_dir = dst.join(case);
        (temp_dir, project_dir)
    }

    /// Build the case project with `--emit-rc-ir Main` and return the dumped RC IR of the `Main`
    /// module. The build works at the `max` optimization level: the borrow versions and the routing
    /// these tests assert on exist only for uncurried functions, which the higher optimization
    /// levels produce — at `none` the same code stays as closures with no uncurried function to
    /// borrow.
    fn emit_main_rc_ir(project_dir: &Path) -> String {
        build_project_and_read_rc_ir(project_dir, "Main", "rc_ir.Main.post.txt")
    }

    /// Build the case project at the `max` optimization level and return the dumped RC IR of every
    /// module.
    ///
    /// The body of a loop written in `Main` is put into the copy of `Std::loop` made for it, so a
    /// case asserting on what such a body does reads the whole program rather than one module.
    fn emit_all_rc_ir(project_dir: &Path) -> String {
        build_project_and_read_rc_ir(project_dir, "all", "rc_ir.post.txt")
    }

    /// The lines of the program's root, without its signature line. An operation performed in
    /// several functions is named once inside any one of them, and the dump opens by naming its
    /// roots, of which a case compiled from one `main` has exactly one.
    fn root_body(dump: &str) -> &str {
        let mut lines = dump.lines();
        assert_eq!(
            lines.next(),
            Some("roots (1)"),
            "a case compiled from one `main` has one root:\n{}",
            dump
        );
        let root = lines
            .next()
            .map(str::trim)
            .filter(|root| !root.is_empty())
            .unwrap_or_else(|| panic!("no root named in the RC IR dump:\n{}", dump));
        let before_sig = dump
            .find(&format!("\nfn {}(", root))
            .unwrap_or_else(|| panic!("no function `{}` in the RC IR dump:\n{}", root, dump));
        let after_sig = dump[before_sig + 1..]
            .split_once('\n')
            .expect("a signature line")
            .1;
        after_sig
            .split_once("\n\n")
            .unwrap_or_else(|| {
                panic!(
                    "no blank line closes the block of `{}` in the RC IR dump:\n{}",
                    root, dump
                )
            })
            .0
    }

    /// The line binding the result of the operation whose text begins with `rhs_prefix`.
    ///
    /// A value carries the name of the source `let` that bound it, and an elimination that removes
    /// that `let` leaves the value on whatever binding it lands in, under that binding's name. The
    /// operation producing the value does not move, so a test pointing at one value points at its
    /// operation. `rhs_prefix` has to name an operation the dump performs once.
    fn binding_by_rhs<'a>(dump: &'a str, rhs_prefix: &str) -> &'a str {
        let mut bindings = dump.lines().filter(|l| {
            l.split_once(" = ")
                .is_some_and(|(_, rhs)| rhs.starts_with(rhs_prefix))
        });
        let line = bindings.next().unwrap_or_else(|| {
            panic!(
                "no binding of `{}` in the RC IR dump:\n{}",
                rhs_prefix, dump
            )
        });
        assert!(
            bindings.next().is_none(),
            "`{}` is performed more than once, so it names no one value:\n{}",
            rhs_prefix,
            dump
        );
        line
    }

    /// Assert that the binding line `line`, which holds the value `subject` names, carries the given
    /// provenance.
    fn assert_line_prov(line: &str, subject: &str, expected_prov: &str) {
        assert!(
            line.contains(expected_prov),
            "the value {} should have provenance `{}`, but its line is:\n{}",
            subject,
            expected_prov,
            line
        );
    }

    /// Assert that the value the operation `rhs_prefix` produces carries the given provenance.
    fn assert_prov_of(dump: &str, rhs_prefix: &str, expected_prov: &str) {
        let line = binding_by_rhs(dump, rhs_prefix);
        assert_line_prov(line, &format!("`{}` produces", rhs_prefix), expected_prov);
    }

    /// The line of the binding named `source_name` (its `(as ...)` annotation).
    fn binding_by_source_name<'a>(dump: &'a str, source_name: &str) -> &'a str {
        let marker = format!("(as {})", source_name);
        dump.lines()
            .find(|l| l.contains(&marker))
            .unwrap_or_else(|| {
                panic!(
                    "no binding `(as {})` in the RC IR dump:\n{}",
                    source_name, dump
                )
            })
    }

    /// The variable a binding line of the dump binds: the name between `let` and the type.
    fn var_bound_on(line: &str) -> String {
        line.trim_start()
            .strip_prefix("let ")
            .and_then(|rest| rest.split_once(" : "))
            .map(|(var, _)| var.to_string())
            .unwrap_or_else(|| panic!("no variable bound on:\n{}", line))
    }

    /// The variable bound to the result of the operation `rhs_prefix` names.
    fn var_produced_by(dump: &str, rhs_prefix: &str) -> String {
        var_bound_on(binding_by_rhs(dump, rhs_prefix))
    }

    /// Whether an RC IR line retains the variable `var` or one of its fields.
    ///
    /// # Examples
    /// `is_retain_of("  retain v1.0 @local", "v1")` is `true`, and
    /// `is_retain_of("  retain v12 @local", "v1")` is `false`.
    fn is_retain_of(line: &str, var: &str) -> bool {
        line.trim_start()
            .strip_prefix("retain ")
            .and_then(|rest| rest.split_whitespace().next())
            .is_some_and(|target| target.split('.').next() == Some(var))
    }

    /// The variable the binding named `source_name` (its `(as ...)` annotation) binds.
    fn var_bound_as(dump: &str, source_name: &str) -> String {
        var_bound_on(binding_by_source_name(dump, source_name))
    }

    /// Assert that the binding named `source_name` (its `(as ...)` annotation) is annotated with the
    /// given provenance in the dump.
    fn assert_binding_prov(dump: &str, source_name: &str, expected_prov: &str) {
        let line = binding_by_source_name(dump, source_name);
        assert_line_prov(
            line,
            &format!("`(as {})` binds", source_name),
            expected_prov,
        );
    }

    /// A field read out of a boxed container is `unknown`. The borrow such a read takes rests on
    /// this: an operation that borrows an operand declares no leaf of its result as a pass-through
    /// of that operand, and for a boxed container that holds because every leaf is `unknown`. A
    /// field carrying the container's leaf through would make the read's borrow unsound.
    #[test]
    fn test_provenance_dump_field_of_boxed_struct() {
        let (_temp_dir, project_dir) = setup_test_env("boxed_struct_field");
        let dump = emit_main_rc_ir(&project_dir);

        assert_binding_prov(&dump, "field", "[unknown]");
    }

    /// A field read out of a boxed container leaves the container to whoever owns it. Nothing
    /// retains the container to pay for the read, so a container still read afterwards is retained
    /// nowhere and released once, at its last use.
    #[test]
    fn test_provenance_dump_field_read_borrows_the_container() {
        let (_temp_dir, project_dir) = setup_test_env("boxed_struct_field");
        let dump = emit_main_rc_ir(&project_dir);

        let container = var_bound_as(&dump, "h");
        assert!(
            !dump.lines().any(|l| is_retain_of(l, &container)),
            "a field read out of a boxed container should not retain the container, but `retain {}` \
             stands in:\n{}",
            container,
            dump
        );
    }

    /// Verifies the three provenance judgements a single function produces: an allocation is
    /// `fresh`, a boxed element read out of a boxed container is `unknown`, and an unboxed tuple
    /// carries each component's own judgement.
    #[test]
    fn test_provenance_dump_basic() {
        let (_temp_dir, project_dir) = setup_test_env("basic");
        let dump = emit_main_rc_ir(&project_dir);

        // `Array::fill` and an array literal produce a fresh array.
        assert_binding_prov(&dump, "arr", "[fresh]");
        assert_prov_of(&dump, "array_lit(", "[fresh]");
        // Reading a boxed element out of a boxed container yields an unknown value.
        assert_binding_prov(&dump, "s0", "[unknown]");
        // Constructing an unboxed tuple carries each component's provenance through: `arr` is fresh,
        // `s0` is unknown. The pair is the one construction of its type.
        let pairs: Vec<&str> = dump
            .lines()
            .filter(|l| {
                l.contains(" : (Std::Array Std::I64, Std::Array Std::U8) ")
                    && l.contains("= struct_make(")
            })
            .collect();
        assert_eq!(
            pairs.len(),
            1,
            "the pair should be constructed exactly once in the RC IR dump:\n{}",
            dump
        );
        assert_line_prov(pairs[0], "`pair`", "[{.0=fresh, .1=unknown}]");
    }

    /// Verifies that a call composes its callee's effect: a recursive function returning its own
    /// argument, whose effect takes a fixed point to compute, keeps a fresh array fresh across the
    /// call.
    #[test]
    fn test_provenance_interprocedural_composition() {
        let (_temp_dir, project_dir) = setup_test_env("interproc");
        let dump = emit_main_rc_ir(&project_dir);

        // `echo_arr` returns its array argument unchanged, so its effect — computed to a fixed point
        // over its recursion — is that argument. Calling it on a fresh array therefore composes to a
        // fresh result: the read-only recursion carries uniqueness through.
        assert_prov_of(root_body(&dump), "Main::echo_arr", "[fresh]");
    }

    /// Whether `line` is a function signature starting with `name_prefix` (which carries the `fn`
    /// keyword) and whose name segment, up to the first space or parenthesis, satisfies `name_pred`.
    fn is_sig(line: &str, name_prefix: &str, name_pred: &impl Fn(&str) -> bool) -> bool {
        if !line.starts_with(name_prefix) {
            return false;
        }
        let name = line
            .split(['(', ' '])
            .nth(1)
            .unwrap_or_else(|| panic!("a signature line carries no name segment:\n{}", line));
        name_pred(name)
    }

    /// The first function signature in `dump` starting with `name_prefix` and whose name segment
    /// satisfies `name_pred`.
    fn sig_line<'a>(dump: &'a str, name_prefix: &str, name_pred: impl Fn(&str) -> bool) -> &'a str {
        dump.lines()
            .find(|l| is_sig(l, name_prefix, &name_pred))
            .unwrap_or_else(|| {
                panic!(
                    "no matching `{}` function in the RC IR dump:\n{}",
                    name_prefix, dump
                )
            })
    }

    /// Whether `dump` has a function signature starting with `name_prefix` and whose name segment
    /// satisfies `name_pred`.
    fn has_sig(dump: &str, name_prefix: &str, name_pred: impl Fn(&str) -> bool) -> bool {
        dump.lines().any(|l| is_sig(l, name_prefix, &name_pred))
    }

    /// The body block of the first function whose signature satisfies the predicates: the lines from
    /// that signature up to the blank line ending the function.
    fn func_block<'a>(
        dump: &'a str,
        name_prefix: &str,
        name_pred: impl Fn(&str) -> bool,
    ) -> Vec<&'a str> {
        let sig = sig_line(dump, name_prefix, name_pred);
        let mut block = vec![sig];
        for l in dump.lines().skip_while(|l| *l != sig).skip(1) {
            if l.is_empty() {
                break;
            }
            block.push(l);
        }
        block
    }

    /// The body block of the function holding the code `main` is written with, found by the call of
    /// `callee` that code makes. That function is `main` itself, or one lifted out of it where `main`
    /// opens with an `IO` bind such as `pure();;`.
    fn main_block<'a>(dump: &'a str, callee: &str) -> Vec<&'a str> {
        let mut blocks: Vec<Vec<&str>> = vec![];
        let mut in_main = false;
        for l in dump.lines() {
            if l.starts_with("fn ") {
                in_main = l.starts_with("fn Main::main#");
                if in_main {
                    blocks.push(vec![]);
                }
            } else if l.is_empty() {
                in_main = false;
            }
            if in_main {
                blocks.last_mut().unwrap().push(l);
            }
        }
        blocks
            .into_iter()
            .find(|block| block.iter().any(|l| l.contains(&format!("= {}", callee))))
            .unwrap_or_else(|| {
                panic!(
                    "neither `main` nor a function lifted out of it calls `{}`:\n{}",
                    callee, dump
                )
            })
    }

    /// Assert that `block` performs no reference counting: no line of it is a `Retain` or a
    /// `Release`. `subject` names the block in the message.
    fn assert_counts_nothing(block: &[&str], subject: &str) {
        assert!(
            block.iter().all(|l| !l.trim_start().starts_with("retain ")
                && !l.trim_start().starts_with("release ")),
            "{} should count no reference:\n{}",
            subject,
            block.join("\n")
        );
    }

    /// Whether a dump line calls a borrowing version of `Main::tally`, whose name carries the
    /// `#borrow` tag.
    fn calls_tally_borrow_version(line: &str) -> bool {
        rc_ir_callee_of(line).is_some_and(|c| c.starts_with("Main::tally") && c.contains("#borrow"))
    }

    /// How many of the `tally` calls in `main` route to a borrowing version, and how many stay on
    /// the owning one.
    fn tally_call_routing(main: &[&str]) -> (usize, usize) {
        let calls = main
            .iter()
            .filter(|l| l.contains("= Main::tally"))
            .collect::<Vec<_>>();
        let borrow_calls = calls
            .iter()
            .filter(|l| calls_tally_borrow_version(l))
            .count();
        (borrow_calls, calls.len() - borrow_calls)
    }

    /// Assert how the `tally` calls in `main` of the test project `case` route: `expected` is the
    /// number routed to the borrow version and the number left on the owning one, and `what` states
    /// the expectation in the message.
    fn assert_tally_routing(case: &str, expected: (usize, usize), what: &str) {
        let (_temp_dir, project_dir) = setup_test_env(case);
        let dump = emit_main_rc_ir(&project_dir);
        let main = main_block(&dump, "Main::tally");
        assert_eq!(
            tally_call_routing(&main),
            expected,
            "{}:\n{}",
            what,
            main.join("\n")
        );
    }

    /// Verifies which functions get a borrow version and what it buys: a function that only reads
    /// its array gets a borrowing version its call site routes to, one that consumes its array stays
    /// single, and the borrowing version performs no reference counting on the borrowed parameter.
    /// The owning baseline the borrowing version was cloned from is left with no caller, so it is
    /// absent from the dump, which is taken after dead-code elimination.
    #[test]
    fn test_borrow_rewrite() {
        let (_temp_dir, project_dir) = setup_test_env("ownership");
        let dump = emit_main_rc_ir(&project_dir);

        // `tally` only reads its array, so it gets a borrowing clone (`#borrow`) whose array
        // parameter is `borrow`.
        let tally_borrow_sig = sig_line(&dump, "fn Main::tally", |n| n.contains("#borrow"));
        assert!(
            tally_borrow_sig.contains("Std::Array Std::I64 [arg0] {borrow}"),
            "the tally borrow version should have a borrowed array parameter:\n{}",
            tally_borrow_sig
        );
        // The one call of `tally` routes to that clone, which leaves the owning baseline it was made
        // from with no caller at all.
        assert!(
            !has_sig(&dump, "fn Main::tally", |n| !n.contains("#borrow")),
            "the tally owning version has no caller once the call routes to the borrow version, so \
             it should be gone:\n{}",
            dump
        );

        // `echo_arr` returns its array argument, consuming it, so it stays a single all-`Own` version
        // with no borrow clone.
        assert!(
            sig_line(&dump, "fn Main::echo_arr", |_| true)
                .contains("Std::Array Std::I64 [arg0] {own}"),
            "echo_arr should have an owned array parameter",
        );
        assert!(
            !has_sig(&dump, "fn Main::echo_arr", |n| n.contains("#borrow")),
            "echo_arr should not have a borrow version",
        );

        // `main` routes its non-tail, owned `tally(arr, ..)` call to the borrow version.
        let main = main_block(&dump, "Main::tally");
        assert!(
            main.iter().any(|l| calls_tally_borrow_version(l)),
            "main should call the tally borrow version:\n{}",
            main.join("\n")
        );

        // The borrow clone drops the reference counting on its borrowed parameter: its body performs
        // no retain or release.
        let tally_borrow = func_block(&dump, "fn Main::tally", |n| n.contains("#borrow"));
        assert_counts_nothing(&tally_borrow, "the tally borrow version");
    }

    /// Verifies that a call lends an owned argument only where the caller reads it afterwards: the
    /// call whose array is read again afterwards is routed to the borrow version, and the call at
    /// the array's last use keeps the owning version, since borrowing there would only delay a
    /// release.
    #[test]
    fn test_routing_by_last_use() {
        let (_temp_dir, project_dir) = setup_test_env("routing_last_use");
        let dump = emit_main_rc_ir(&project_dir);

        let main = main_block(&dump, "Main::tally");
        let (borrow_calls, own_calls) = tally_call_routing(&main);

        // The array read again after its call is owned but not at its last use, so routing to the
        // borrow version removes a retain — that call goes to the borrow version. The array not used
        // after its call is at its last use, so borrowing it would remove no retain and only delay
        // its release — that call stays on the own version.
        assert_eq!(
            borrow_calls,
            1,
            "the non-last-use call should route to the borrow version:\n{}",
            main.join("\n")
        );
        assert_eq!(
            own_calls,
            1,
            "the last-use call should stay on the own version:\n{}",
            main.join("\n")
        );
    }

    /// Verifies that the routing reads the code after the match whose branch a call ends: an
    /// array read again after an `if` is not at its last use at a call that ends a branch of that
    /// `if`, so the call is routed to the borrow version.
    #[test]
    fn test_routing_reads_past_the_match() {
        assert_tally_routing(
            "routing_after_match",
            (1, 0),
            "the call whose array is read after the `if` should route to the borrow version",
        );
    }

    /// Verifies that the routing reads past every match whose branch a call ends, out to the
    /// outermost: an array read again only after the outer of two nested `if`s is not at its last
    /// use at a call that ends a branch of the inner one, so the call is routed to the borrow
    /// version.
    #[test]
    fn test_routing_reads_past_nested_matches() {
        assert_tally_routing(
            "routing_after_nested_match",
            (1, 0),
            "the call whose array is read after the outer `if` should route to the borrow version",
        );
    }

    /// Verifies that the routing follows the lifetime of an argument's object past the match
    /// whose branch the call ends: a leaf read out of an aggregate the caller uses again after the
    /// `if` is a reference this function made for the call, so the call is routed to the borrow
    /// version.
    #[test]
    fn test_routing_by_the_objects_lifetime_past_the_match() {
        assert_tally_routing(
            "routing_aggregate_after_match",
            (1, 0),
            "the call handed a leaf of an aggregate read after the `if` should route to the borrow \
             version",
        );
    }

    /// Verifies that the routing finds an argument's object held after the call through the variable
    /// a pattern took it out of: `env` comes out of a pair nothing names afterwards, and `env` is read
    /// after the call, so the array read out of it outlives the call and the call is routed to the
    /// borrow version.
    #[test]
    fn test_routing_reads_the_variable_a_pattern_binds() {
        assert_tally_routing(
            "routing_field_of_a_pattern",
            (1, 0),
            "the call handed a field of a value read after it should route to the borrow version",
        );
    }

    /// Verifies that the routing finds an argument's object held after the call through the match
    /// binding it flows out of: `env` is whichever value the `if` takes, and `env` is read after the
    /// call, so the array read out of it outlives the call on either branch and the call is routed
    /// to the borrow version.
    #[test]
    fn test_routing_reads_the_match_binding() {
        assert_tally_routing(
            "routing_field_of_a_join",
            (1, 0),
            "the call handed a field of an `if` result read after it should route to the borrow \
             version",
        );
    }

    /// Verifies that a call owns an argument at its last use and borrows the one beside it the
    /// caller reads afterwards: the version it reaches takes `rows` owned, so the callee frees `rows`
    /// once it has read a value out of it and that value stays unique for an update, and takes `key`
    /// borrowed, so the caller makes no retain for it.
    #[test]
    fn test_routing_owns_a_last_use_beside_a_borrow() {
        let (_temp_dir, project_dir) = setup_test_env("routing_last_use_beside_a_borrow");
        let dump = emit_main_rc_ir(&project_dir);
        let main = main_block(&dump, "Main::tally");
        assert_eq!(
            tally_call_routing(&main),
            (1, 0),
            "the call should route to a borrowing version:\n{}",
            main.join("\n")
        );
        let tally_sig = sig_line(&dump, "fn Main::tally", |n| n.contains("#borrow"));
        assert!(
            tally_sig.contains("Std::Array (Std::Array Std::I64) [arg1] {own}")
                && tally_sig.contains("Std::Array Std::I64 [arg2] {borrow}"),
            "the version the call reaches should own `rows` and borrow `key`:\n{}",
            tally_sig
        );
    }

    /// Verifies that two calls of one function lending different arguments reach two versions:
    /// the call at `rows`'s last use reaches a version owning `rows` and borrowing `key`, and the
    /// call at `key`'s last use reaches one borrowing `rows` and owning `key`.
    #[test]
    fn test_routing_gives_each_lent_set_its_own_version() {
        let (_temp_dir, project_dir) = setup_test_env("routing_two_calls_two_versions");
        let dump = emit_main_rc_ir(&project_dir);
        let main = main_block(&dump, "Main::tally");
        let callees: Vec<&str> = main
            .iter()
            .filter(|l| calls_tally_borrow_version(l))
            .filter_map(|l| rc_ir_callee_of(l))
            .collect();
        assert!(
            callees.len() == 2 && callees[0] != callees[1],
            "the two calls should reach two different borrowing versions:\n{}",
            main.join("\n")
        );
        for (rows, key) in [("own", "borrow"), ("borrow", "own")] {
            let rows_param = format!("Std::Array (Std::Array Std::I64) [arg1] {{{}}}", rows);
            let key_param = format!("Std::Array Std::I64 [arg2] {{{}}}", key);
            assert!(
                dump.lines().any(|l| {
                    is_sig(l, "fn Main::tally", &|n: &str| n.contains("#borrow"))
                        && l.contains(&rows_param)
                        && l.contains(&key_param)
                }),
                "a borrowing version should take `rows` {} and `key` {}:\n{}",
                rows,
                key,
                dump
            );
        }
    }

    /// Verifies that the routing lends a field of a match binding only when the object is held
    /// after the call whichever arm the match takes: the call whose `env` may be `e1` or `e2`, both
    /// read afterwards, routes to the borrow version, and the call whose `env2` may be `f2`, which
    /// nothing reads afterwards, keeps the owning version.
    #[test]
    fn test_routing_holds_every_arm_of_a_match_binding() {
        assert_tally_routing(
            "routing_field_of_a_join_held_on_one_arm",
            (1, 1),
            "the call whose array is held after it on both arms should route to the borrow version, \
             and the call whose array ends there on one arm should stay on the own version",
        );
    }

    /// Verifies that the routing finds an argument's object held after the call through an unboxed
    /// union built around it: the union `Slot::full(arr)` is named nowhere after the call, but `arr`
    /// is, so the call is routed to the borrow version.
    #[test]
    fn test_routing_reads_through_a_union_built_for_the_call() {
        assert_tally_routing(
            "routing_union_built_for_the_call",
            (1, 0),
            "the call handed a union holding an array read after it should route to the borrow \
             version",
        );
    }

    /// Verifies that a call ending a branch of a match keeps the owning version when nothing after
    /// the match names its argument: the call is the array's last use, and borrowing there would
    /// only delay the release.
    #[test]
    fn test_routing_last_use_in_a_branch() {
        assert_tally_routing(
            "routing_last_use_in_branch",
            (0, 1),
            "the call at the array's last use should stay on the own version",
        );
    }

    /// Verifies that routing to a borrow version reads the lifetime of the object an argument refers
    /// to rather than of the name it arrives under: a leaf read out of an aggregate the caller goes
    /// on to use is a reference this function made for the call, so routing removes it together with
    /// the retain that made it. The call whose aggregate is named nowhere afterwards keeps the
    /// owning version, since the object that leaf refers to ends there either way.
    #[test]
    fn test_routing_by_the_objects_lifetime() {
        let (_temp_dir, project_dir) = setup_test_env("routing_aggregate");
        let dump = emit_main_rc_ir(&project_dir);

        let main = main_block(&dump, "Main::tally");
        let (borrow_calls, own_calls) = tally_call_routing(&main);
        assert_eq!(
            borrow_calls,
            1,
            "the call handed a leaf of an aggregate that outlives it should route to the borrow \
             version:\n{}",
            main.join("\n")
        );
        assert_eq!(
            own_calls,
            1,
            "the call handed a leaf whose object ends there should stay on the own version:\n{}",
            main.join("\n")
        );
        // Handing an owning callee a leaf read out of an aggregate the caller goes on to use costs a
        // retain. Routing is what removes it, so `main` counts no reference at all.
        assert!(
            main.iter().all(|l| !l.trim_start().starts_with("retain ")),
            "the routed call should leave `main` with no retain to make:\n{}",
            main.join("\n")
        );
    }

    /// Verifies that a whole-value retain of an unboxed pair is normalized into one retain per
    /// reference-counting unit, both naming the same variable at its own field path.
    #[test]
    fn test_split_rc_into_units() {
        let (_temp_dir, project_dir) = setup_test_env("multiunit");
        let dump = emit_main_rc_ir(&project_dir);

        let main = main_block(&dump, "Main::hand_back");
        // The whole-value retain of the pair `t` is normalized to one retain per field: `.0` and `.1`.
        // The tuple binding has no source name, so match the retains by their field paths. A retain
        // may carry a trailing reference-counting state tag, which is not part of the target.
        let target = |l: &str| {
            l.trim()
                .trim_start_matches("retain ")
                .split(" @")
                .next()
                .unwrap()
                .to_string()
        };
        let retains: Vec<String> = main
            .iter()
            .filter(|l| l.trim_start().starts_with("retain "))
            .map(|l| target(l))
            .collect();
        let field0 = retains.iter().find(|t| t.ends_with(".0"));
        let field1 = retains.iter().find(|t| t.ends_with(".1"));
        assert!(
            field0.is_some() && field1.is_some(),
            "the pair retain should be split into `.0` and `.1` retains of the same variable:\n{}",
            main.join("\n")
        );
        // Both name the same tuple variable (the text before the field path).
        let var_of = |t: &String| t.trim_end_matches(".0").trim_end_matches(".1").to_string();
        assert_eq!(
            var_of(field0.unwrap()),
            var_of(field1.unwrap()),
            "the split retains should name the same tuple variable"
        );
    }

    /// The first argument variable of a call on a dump line.
    fn call_first_arg(line: &str) -> &str {
        let (_, args) = line
            .split_once('(')
            .unwrap_or_else(|| panic!("the line carries no call:\n{}", line));
        args.split([',', ')']).next().unwrap().trim()
    }

    /// Verifies that the retain/release bracket borrow-ification puts around a borrow call is
    /// cancelled when nothing between the two consumes the value, and that the value therefore
    /// stays `fresh` for the operation following the call.
    #[test]
    fn test_cancel_removes_net_zero_bracket() {
        let (_temp_dir, project_dir) = setup_test_env("ownership");
        let dump = emit_main_rc_ir(&project_dir);

        let main = main_block(&dump, "Main::tally");
        let call = main
            .iter()
            .find(|l| calls_tally_borrow_version(l))
            .expect("main should call the tally borrow version");
        let arr = call_first_arg(call);

        // Borrow-ification brackets the borrow call with a retain before it and a release after it;
        // because nothing between them consumes the array, cancellation removes both, leaving no
        // reference counting on the array in `main`.
        for l in &main {
            let trimmed = l.trim_start();
            for op in ["retain", "release"] {
                assert!(
                    trimmed != format!("{} {}", op, arr)
                        && !trimmed.starts_with(&format!("{} {} ", op, arr)),
                    "the array {} bracketing the borrow call should have been cancelled:\n{}",
                    op,
                    l
                );
            }
        }

        // With no retain left to demote it, the array stays `fresh` from its `fill`, so the following
        // `set` receives a unique array — the elision the borrow-plus-cancel pipeline exists to enable.
        assert_binding_prov(&dump, "arr", "[fresh]");
    }

    /// Verifies that the borrow version of a function that wraps its borrowed argument in a union
    /// reference-counts nothing: the union lays the borrowed payload in place without owning it, so
    /// releasing the union would free an array its caller still holds.
    #[test]
    fn test_borrow_union_no_double_release() {
        let (_temp_dir, project_dir) = setup_test_env("union");
        let dump = emit_main_rc_ir(&project_dir);

        // `via_union` reads its array `p` (directly and through `some(p)`), so it has a borrow version
        // whose array parameter is `borrow`.
        assert!(
            sig_line(&dump, "fn Main::via_union", |n| n.contains("#borrow"))
                .contains("Std::Array Std::I64 [arg0] {borrow}"),
            "via_union should have a borrow version with a borrowed array parameter",
        );

        // The borrow version builds `some(p)` around the borrowed `p` and passes it to a borrowing
        // position. Because the union only lays the borrowed payload in place (it does not own it),
        // the version must perform no reference counting — in particular no release of the union,
        // which would free the caller's still-owned array.
        let via_borrow = func_block(&dump, "fn Main::via_union", |n| n.contains("#borrow"));
        assert!(
            via_borrow.iter().any(|l| l.contains("union_make_1(")),
            "the via_union borrow version should build the union:\n{}",
            via_borrow.join("\n")
        );
        assert_counts_nothing(
            &via_borrow,
            "the via_union borrow version, over the borrowed value and its union,",
        );
    }

    /// Verifies that a locally fresh value carries its provenance through a guard and through a
    /// fill loop, so that every in-place operation reached with it drops its uniqueness check,
    /// while an operation on a value read out of a boxed container keeps one.
    #[test]
    fn test_unique_check_elim_local_fresh() {
        let (_temp_dir, project_dir) = setup_test_env("unique_elim");
        let dump = emit_main_rc_ir(&project_dir);

        // A guard that aborts rather than producing a value says nothing about the value it guards,
        // so a fresh array passed through `assert_unique` comes out fresh. Asking whether a value is
        // being copied would otherwise remove the elimination the question is about.
        assert_binding_prov(&dump, "guarded", "[fresh]");

        // A fill loop writes into the array through primitives that leave the uniqueness check to
        // their caller, which std satisfies by building the array a line earlier. What they return is
        // therefore uniquely owned, so the operation after such a loop drops its check.
        assert_binding_prov(&dump, "built", "[fresh]");

        // Each operation on a locally fresh (proven unique) value renders its dropped check as a
        // `[unique]` marker. Asserting the markers guards against silent *under*-elimination: a
        // regression that stops dropping a check would still pass the memcheck correctness tests
        // (which run the same whether or not elimination fires), but fails here.
        for elided in [
            // An array `set`.
            "array_set[unique]",
            // An array `swap`.
            "array_swap[unique]",
            // A generic `act`, whose `_unsafe_is_storage_unique` folds to the constant `true`.
            "is_storage_unique[unique]",
            // The punch and the plug that `act` carries the update out with. Reaching the arm the
            // storage-uniqueness check guards means the array's reference count was one, so both drop
            // their checks.
            "array_punch[unique]",
            "array_plug[unique]",
            // A boxed-struct field `set` (field 0).
            "struct_set_0[unique]",
            // A `truncate`, whose `_unsafe_truncate_bounds_unchecked` core folds its check.
            "array_truncate[unique]",
            // A `fill` / `from_map`, whose `_unsafe_append_value_capacity_unchecked` core folds its
            // check when it appends into the fresh array the builder just allocated.
            "array_append_value[unique]",
            // A `reserve`, whose `_unsafe_set_capacity_bounds_unchecked` core folds its check and
            // reallocs the fresh array in place.
            "array_set_capacity[unique]",
            // An `append`, whose `_unsafe_append_capacity_unchecked` core folds the check on the
            // fresh destination it appends into.
            "array_append[unique]",
            // A `get_sub`, whose `_unsafe_copy_capacity_bounds_unchecked` core folds the check on
            // the fresh array it copies the range into.
            "array_copy_range[unique]",
            // A `_unsafe_grow_size` on the fresh array folds its check.
            "array_grow_size[unique]",
            // An `unsafe_set_bounds_unchecked` on the fresh array folds its check.
            "array_set_unchecked[unique]",
            // The punch and the plug that a boxed-struct field `act` carries the update out with.
            "struct_punch_0[unique]",
            "struct_plug_in_0[unique]",
            // An `unsafe_is_unique`, whose flag folds to the constant `true`.
            "is_unique[unique]",
            // The `_mutate_boxed_ios_internal` core of `mutate_boxed` and `mutate_boxed_io`, on a
            // freshly allocated value.
            "mutate_ptr[unique]",
        ] {
            assert!(
                dump.contains(elided),
                "the operation on a locally fresh value should render `{}`:\n{}",
                elided,
                dump
            );
        }
        // `set` on an array read out of a boxed container is of unknown sharing, so its check stays
        // (a plain `Array::set(` with no `[unique]` marker).
        assert!(
            dump.contains("array_set("),
            "the set on an array of unknown sharing should keep its force-unique check:\n{}",
            dump
        );
    }

    /// Verifies that copying a range out of an array leaves the array provably unique. The copy
    /// borrows the array it reads, so the caller keeps the reference it already held rather than
    /// duplicating it; a duplicated reference would be a `Retain`, which is the analysis's only
    /// demotion and lands on the absorbing `unknown`, and every write after the copy would then
    /// re-check a value nothing had shared.
    #[test]
    fn test_unique_check_elim_after_copy() {
        let (_temp_dir, project_dir) = setup_test_env("unique_elim_after_copy");
        let dump = emit_main_rc_ir(&project_dir);

        // The copy writes into the array it just allocated for the result, so its own check goes.
        assert!(
            dump.contains("array_copy_range[unique]"),
            "the copy into the freshly allocated result should drop its check:\n{}",
            dump
        );
        // The array copied out of is the one the writes go into, and nothing shared it. Assert the
        // writes are there as well as unchecked, so that the case cannot pass by having no write
        // left in the dump at all.
        assert!(
            dump.contains("array_set[unique]"),
            "the writes after the copy should be in the dump, with their checks dropped:\n{}",
            dump
        );
        assert!(
            !dump.contains("array_set("),
            "every write after the copy should drop its check:\n{}",
            dump
        );
    }

    /// Verifies that publishing a value to other threads yields a handle of unknown sharing: the
    /// write before publication drops its check and the write on the published handle keeps it. The
    /// elimination is sound for values other threads can reach only as long as this holds.
    #[test]
    fn test_published_value_keeps_its_check() {
        let (_temp_dir, project_dir) = setup_test_env("mark_threaded");
        let dump = emit_main_rc_ir(&project_dir);

        assert_prov_of(&dump, "mark_threaded(", "[unknown]");
        assert!(
            dump.contains("array_set[unique]"),
            "the set on the array before it is published should drop its check:\n{}",
            dump
        );
        let published = var_produced_by(&dump, "mark_threaded(");
        assert!(
            dump.lines()
                .any(|l| l.contains("array_set(") && l.contains(&published)),
            "the set on the published handle {} should keep its check:\n{}",
            published,
            dump
        );
    }

    /// Assert both halves of what the two writes through a lent pointer in the case `case` declare:
    /// that each drops its own check on a value proven unique, and that the value each returns,
    /// bound as `mutated` and `mutated_io`, is `fresh`.
    fn assert_writes_through_lent_pointer_are_unique(case: &str) {
        let (_temp_dir, project_dir) = setup_test_env(case);
        let dump = emit_main_rc_ir(&project_dir);

        // The value each write hands back is uniquely owned, which is what lets the write that
        // follows drop its check. A wrong result position would leave these of unknown sharing.
        assert_binding_prov(&dump, "mutated", "[fresh]");
        assert_binding_prov(&dump, "mutated_io", "[fresh]");

        // Both writes go to a value nothing else holds, so both drop their check. The case's own
        // writes are the only ones in the dump, so the checked form appearing at all is a failure.
        assert_eq!(
            dump.matches("mutate_ptr[unique]").count(),
            2,
            "both writes to a value proven unique should render `mutate_ptr[unique]`:\n{}",
            dump
        );
        assert!(
            !dump.contains("mutate_ptr("),
            "no write should keep its check:\n{}",
            dump
        );
    }

    /// Verifies both halves of what a write through an array's element pointer declares: that its
    /// own check is dropped on an array proven unique, and that the array it returns is `fresh`.
    #[test]
    fn test_unique_check_elim_mutate_elements() {
        assert_writes_through_lent_pointer_are_unique("unique_elim_mutate_elements");
    }

    /// Verifies both halves of what a write through the pointer to a boxed value's payload
    /// declares: that its own check is dropped on a value proven unique, and that the value it
    /// returns is `fresh`.
    #[test]
    fn test_unique_check_elim_mutate_boxed() {
        assert_writes_through_lent_pointer_are_unique("unique_elim_mutate_boxed");
    }

    /// Verifies that `borrow_boxed` borrows the value it lends a pointer into: nothing retains the
    /// value to pay for the borrow, so a value still read afterwards is retained nowhere.
    #[test]
    fn test_borrow_boxed_retains_nothing() {
        let (_temp_dir, project_dir) = setup_test_env("borrow_boxed");
        let dump = emit_main_rc_ir(&project_dir);

        let rec = var_bound_as(&dump, "rec");
        assert!(
            dump.lines()
                .any(|l| l.contains("borrow_ptr(") && l.contains(&rec)),
            "the case should lend a pointer into `{}`:\n{}",
            rec,
            dump
        );
        assert!(
            !dump.lines().any(|l| is_retain_of(l, &rec)),
            "`borrow_boxed` should not retain the value it borrows, but `retain {}` stands in:\n{}",
            rec,
            dump
        );
    }

    /// Verifies that a write into an array read out of a global keeps its uniqueness check.
    ///
    /// A global object is shared whatever its reference count says, and the count is not raised to
    /// record that, so proving one unique would licence a write everything else in the program can
    /// see. The elimination is sound for globals only as long as this holds.
    #[test]
    fn test_global_value_keeps_its_check() {
        let (_temp_dir, project_dir) = setup_test_env("unique_elim_global");
        let dump = emit_main_rc_ir(&project_dir);

        assert!(
            dump.contains("array_set("),
            "the set on an array read out of a global should keep its check:\n{}",
            dump
        );
        assert!(
            !dump.contains("array_set[unique]"),
            "the set on an array read out of a global should not be proven unique:\n{}",
            dump
        );
    }

    /// Verifies that a value updated through a field of an unboxed struct keeps the provenance that
    /// lets its check be dropped, so a loop over such a struct re-checks nothing.
    #[test]
    fn test_unique_check_elim_through_struct_field() {
        let (_temp_dir, project_dir) = setup_test_env("unique_elim_struct_field");
        let dump = emit_all_rc_ir(&project_dir);

        // The loop updates the array through a field of an unboxed struct. Taking that struct apart
        // and putting it back together moves the fields in registers without touching a reference
        // count, so the array reaches the `set` as the same value the loop was entered with: the
        // clone reached with the fresh array drops the check, exactly as for a bare array. Without
        // the field update carrying provenance through, every iteration would re-check an array
        // already proven unique.
        assert!(
            dump.contains("array_set[unique]"),
            "the set on the array carried through the struct field should drop its check:\n{}",
            dump
        );
    }

    /// Verifies that reading a field out of an unboxed struct that is updated afterwards retains
    /// nothing, so every field reaches its update with its uniqueness intact, whether the read goes
    /// through the field getter or through a struct pattern naming the one field. The read takes the
    /// struct apart and releases the field it leaves behind by a node of its own, which cancels
    /// against the retain that keeps the struct for the update. A release inside the read would
    /// leave that retain standing, and the retain would make the field left behind of unknown
    /// sharing, so its update would keep its check.
    #[test]
    fn test_unique_check_elim_after_field_read() {
        let (_temp_dir, project_dir) = setup_test_env("unique_elim_read_field");
        let dump = emit_main_rc_ir(&project_dir);

        for function in ["add_edges_by_getter", "add_edges_by_pattern"] {
            let block = func_block(&dump, &format!("fn Main::{}", function), |_| true);
            assert!(
                block.iter().all(|l| !l.trim_start().starts_with("retain ")),
                "`{}` reads a field of the struct before updating it, and should retain nothing:\n{}",
                function,
                block.join("\n")
            );
            let appends: Vec<&&str> = block
                .iter()
                .filter(|l| l.contains("= array_append_value"))
                .collect();
            assert_eq!(
                appends.len(),
                2,
                "`{}` should append to each of its two arrays once:\n{}",
                function,
                block.join("\n")
            );
            assert!(
                appends
                    .iter()
                    .all(|l| l.contains("array_append_value[unique]")),
                "both appends of `{}` should drop their checks:\n{}",
                function,
                block.join("\n")
            );
        }
    }

    /// Verifies that a loop entered with a shared array pays its uniqueness check on the first
    /// iteration alone.
    ///
    /// Specialization clones the loop body per input uniqueness, so the source-level `set` appears
    /// in two functions: a checked one, whose check clones the shared array, and one reached with
    /// that fresh clone, which writes in place.
    #[test]
    fn test_unique_check_elim_shared_loop_entry() {
        let (_temp_dir, project_dir) = setup_test_env("unique_elim_shared_loop");
        let dump = emit_all_rc_ir(&project_dir);

        // Specialization clones the loop body per input uniqueness, so a loop entered with a shared
        // array pays the uniqueness check on its first iteration only: that check clones the array,
        // and the clone the loop then runs on is fresh, which the version reached with it writes in
        // place. The case has one source-level `set`, so seeing it in both forms — and in two
        // different functions — is that split. Assert both: without the checked version the first
        // write would corrupt the array's other holder, and without the `[unique]` version every
        // iteration would re-check a value already proven unique.
        let mut fns_with_checked_set = vec![];
        let mut fns_with_elided_set = vec![];
        let mut current_fn = "";
        for line in dump.lines() {
            if line.starts_with("fn ") {
                current_fn = line;
            } else if line.contains("array_set[unique]") {
                fns_with_elided_set.push(current_fn);
            } else if line.contains("array_set(") {
                fns_with_checked_set.push(current_fn);
            }
        }
        assert!(
            !fns_with_checked_set.is_empty(),
            "the loop version entered with the shared array should keep its check:\n{}",
            dump
        );
        assert!(
            !fns_with_elided_set.is_empty(),
            "the loop version reached with the freshly cloned array should drop its check:\n{}",
            dump
        );
        assert!(
            fns_with_checked_set
                .iter()
                .all(|f| !fns_with_elided_set.contains(f)),
            "the checked and the elided set should live in different clones of the loop body, but \
             a function holds both:\n{}",
            dump
        );
    }

    /// Verifies that a retain only one path of a loop body needs is moved onto that path. The body
    /// takes its captures apart at the top, so the tree it captured is retained there for the
    /// capture handed on to the next iteration; the path that starts again from the root hands that
    /// reference on, and the paths that stop or go on down release it. On those paths the retain and
    /// the release do nothing between them, and they go together.
    #[test]
    fn test_retain_sinks_to_the_path_consuming_it() {
        let (_temp_dir, project_dir) = setup_test_env("sink_retain");
        let dump = build_project_run_and_read_rc_ir(&project_dir, "max", "7");

        let sig = dump
            .lines()
            .find(|l| l.starts_with("fn Std::loop#") && l.contains("LsCap@walk"))
            .unwrap_or_else(|| panic!("no copy of `loop` runs the body of `walk`:\n{}", dump));
        let body: Vec<&str> = dump
            .lines()
            .skip_while(|l| *l != sig)
            .skip(1)
            .take_while(|l| !l.is_empty())
            .collect();
        let retains: Vec<usize> = (0..body.len())
            .filter(|&i| body[i].trim_start().starts_with("retain "))
            .collect();
        assert_eq!(
            retains.len(),
            1,
            "the loop body should retain the tree once, on the path starting again from the root:\n{}",
            body.join("\n")
        );
        assert!(
            body[retains[0] + 1].contains("= Std::loop#"),
            "the retain should stand right before the call starting the next iteration:\n{}",
            body.join("\n")
        );
    }
}
