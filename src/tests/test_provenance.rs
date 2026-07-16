// Integration tests for the RC IR provenance analysis, checked through the `--emit-rc-ir` dump.
// The dump annotates each variable binding with the provenance the analysis computed, so a small
// program with named `let`s lets us assert the analysis end to end: allocators produce `fresh`
// values, reading a boxed element out of a boxed container is `dyn`, and constructing an unboxed
// tuple carries each component's provenance through.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, fix_command};
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_provenance/cases");
        path
    }

    // Copy the test cases into a fresh temporary directory so parallel test runs do not conflict,
    // and return the directory of the named case project.
    fn setup_test_env(case: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dst = temp_dir.path().to_path_buf();
        copy_dir_recursive(&get_test_cases_dir(), &dst).expect("Failed to copy test cases");
        let project_dir = dst.join(case);
        (temp_dir, project_dir)
    }

    /// Build the case project with `--emit-rc-ir Main` and return the dumped RC IR of the `Main`
    /// module. The build is pinned to the `max` optimization level: the borrow versions and the
    /// routing these tests assert on exist only for uncurried funptr functions, which the higher
    /// optimization levels produce — at `none` the same code stays as closures with no funptr
    /// version to borrow. Pinning makes the dumped structure the same regardless of the ambient
    /// `FIX_MAX_OPT_LEVEL` the test suite runs under.
    fn emit_main_rc_ir(project_dir: &Path) -> String {
        let output = fix_command()
            .arg("build")
            .arg("--emit-rc-ir")
            .arg("Main")
            .env("FIX_MAX_OPT_LEVEL", "max")
            .current_dir(project_dir)
            .output()
            .expect("Failed to execute fix build --emit-rc-ir");

        if !output.status.success() {
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix build --emit-rc-ir failed");
        }

        let dump_path = project_dir.join(".fixlang/rc_ir.Main.txt");
        std::fs::read_to_string(&dump_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", dump_path.display(), e))
    }

    /// Assert that the binding named `source_name` (its `(as ...)` annotation) is annotated with the
    /// given provenance in the dump.
    fn assert_binding_prov(dump: &str, source_name: &str, expected_prov: &str) {
        let marker = format!("(as {})", source_name);
        let line = dump
            .lines()
            .find(|l| l.contains(&marker))
            .unwrap_or_else(|| {
                panic!(
                    "no binding `(as {})` in the RC IR dump:\n{}",
                    source_name, dump
                )
            });
        assert!(
            line.contains(expected_prov),
            "binding `(as {})` should have provenance `{}`, but its line is:\n{}",
            source_name,
            expected_prov,
            line
        );
    }

    #[test]
    fn test_provenance_dump_basic() {
        let (_temp_dir, project_dir) = setup_test_env("basic");
        let dump = emit_main_rc_ir(&project_dir);

        // `Array::fill` and an array literal produce a fresh array.
        assert_binding_prov(&dump, "arr", "[fresh]");
        assert_binding_prov(&dump, "strs", "[fresh]");
        // Reading a boxed element out of a boxed container yields an unknown value.
        assert_binding_prov(&dump, "s0", "[dyn]");
        // Constructing an unboxed tuple carries each component's provenance through: `arr` is fresh,
        // `s0` is dyn.
        assert_binding_prov(&dump, "pair", "[(fresh, dyn)]");
    }

    #[test]
    fn test_provenance_interprocedural_composition() {
        let (_temp_dir, project_dir) = setup_test_env("interproc");
        let dump = emit_main_rc_ir(&project_dir);

        // `echo_arr` returns its array argument unchanged, so its effect — computed to a fixed point
        // over its recursion — is that argument. Calling it on a fresh array therefore composes to a
        // fresh result: the read-only recursion carries uniqueness through.
        assert_binding_prov(&dump, "r", "[fresh]");
    }

    /// The first signature line of a function whose name starts with `fn <name_prefix>` and whose
    /// name segment (up to the first space) satisfies `name_pred`.
    fn sig_line<'a>(dump: &'a str, name_prefix: &str, name_pred: impl Fn(&str) -> bool) -> &'a str {
        dump.lines()
            .find(|l| {
                l.starts_with(name_prefix) && name_pred(l.split(['(', ' ']).nth(1).unwrap_or(""))
            })
            .unwrap_or_else(|| {
                panic!(
                    "no matching `{}` function in the RC IR dump:\n{}",
                    name_prefix, dump
                )
            })
    }

    /// Whether the dump has a function whose name starts with `fn <name_prefix>` and satisfies
    /// `name_pred` on its name segment.
    fn has_sig(dump: &str, name_prefix: &str, name_pred: impl Fn(&str) -> bool) -> bool {
        dump.lines().any(|l| {
            l.starts_with(name_prefix) && name_pred(l.split(['(', ' ']).nth(1).unwrap_or(""))
        })
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

    #[test]
    fn test_borrow_rewrite() {
        let (_temp_dir, project_dir) = setup_test_env("ownership");
        let dump = emit_main_rc_ir(&project_dir);

        // `tally` only reads its array, so it is materialized in two versions: the all-`Own` baseline
        // (its name unsuffixed) and a borrowing clone (`#borrow`) whose array parameter is `borrow`.
        let tally_own = sig_line(&dump, "fn Main::tally", |n| !n.ends_with("#borrow"));
        assert!(
            tally_own.contains("Std::Array Std::I64 [arg0] {own}"),
            "the tally own version should have an owned array parameter:\n{}",
            tally_own
        );
        let tally_borrow_sig = sig_line(&dump, "fn Main::tally", |n| n.ends_with("#borrow"));
        assert!(
            tally_borrow_sig.contains("Std::Array Std::I64 [arg0] {borrow}"),
            "the tally borrow version should have a borrowed array parameter:\n{}",
            tally_borrow_sig
        );

        // `echo_arr` returns its array argument, consuming it, so it stays a single all-`Own` version
        // with no borrow clone.
        assert!(
            sig_line(&dump, "fn Main::echo_arr", |_| true)
                .contains("Std::Array Std::I64 [arg0] {own}"),
            "echo_arr should have an owned array parameter",
        );
        assert!(
            !has_sig(&dump, "fn Main::echo_arr", |n| n.ends_with("#borrow")),
            "echo_arr should not have a borrow version",
        );

        // `main` routes its non-tail, owned `tally(arr, ..)` call to the borrow version.
        // The main entry is `Main::main#<hash>#funptr1` (three `#`-segments); the lifted decap lambdas
        // have an extra segment.
        let main = func_block(&dump, "fn Main::main", |n| {
            n.split('#').count() == 3 && n.ends_with("#funptr1")
        });
        assert!(
            main.iter()
                .any(|l| l.contains("= Main::tally") && l.contains("#borrow(")),
            "main should call the tally borrow version:\n{}",
            main.join("\n")
        );

        // The borrow clone drops the reference counting on its borrowed parameter: its body performs
        // no retain or release.
        let tally_borrow = func_block(&dump, "fn Main::tally", |n| n.ends_with("#borrow"));
        assert!(
            tally_borrow
                .iter()
                .all(|l| !l.trim_start().starts_with("release ")
                    && !l.trim_start().starts_with("retain ")),
            "the tally borrow version should perform no reference counting:\n{}",
            tally_borrow.join("\n")
        );
    }

    #[test]
    fn test_benefit_routing_by_last_use() {
        let (_temp_dir, project_dir) = setup_test_env("benefit");
        let dump = emit_main_rc_ir(&project_dir);

        let main = func_block(&dump, "fn Main::main", |n| {
            n.split('#').count() == 3 && n.ends_with("#funptr1")
        });
        let tally_calls: Vec<&&str> = main
            .iter()
            .filter(|l| l.contains("= Main::tally"))
            .collect();
        let borrow_calls = tally_calls
            .iter()
            .filter(|l| l.contains("#borrow("))
            .count();
        let own_calls = tally_calls.len() - borrow_calls;

        // The array read again after its call is owned but not at its last use, so routing to the
        // borrow version removes a retain — that call goes to the borrow version. The array not used
        // after its call is at its last use, so borrowing it would remove no retain and only delay
        // its release — that call stays on the own version. Safe-only routing would send both to the
        // borrow version.
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

    #[test]
    fn test_split_rc_into_units() {
        let (_temp_dir, project_dir) = setup_test_env("multiunit");
        let dump = emit_main_rc_ir(&project_dir);

        let main = func_block(&dump, "fn Main::main", |n| {
            n.split('#').count() == 3 && n.ends_with("#funptr1")
        });
        // The whole-value retain of the pair `t` is normalized to one retain per field: `.0` and `.1`.
        // The tuple binding has no source name, so match the retains by their field paths.
        let retains: Vec<&&str> = main
            .iter()
            .filter(|l| l.trim_start().starts_with("retain "))
            .collect();
        let field0 = retains.iter().find(|l| l.trim_end().ends_with(".0"));
        let field1 = retains.iter().find(|l| l.trim_end().ends_with(".1"));
        assert!(
            field0.is_some() && field1.is_some(),
            "the pair retain should be split into `.0` and `.1` retains of the same variable:\n{}",
            main.join("\n")
        );
        // Both name the same tuple variable (the text before the field path).
        let var_of = |l: &str| {
            l.trim()
                .trim_start_matches("retain ")
                .trim_end_matches(".0")
                .trim_end_matches(".1")
                .to_string()
        };
        assert_eq!(
            var_of(field0.unwrap()),
            var_of(field1.unwrap()),
            "the split retains should name the same tuple variable"
        );
    }

    /// The first argument variable of a `...#borrow(a, b, ...)` call on a dump line.
    fn borrow_call_first_arg(line: &str) -> &str {
        line.split("#borrow(")
            .nth(1)
            .and_then(|after| after.split([',', ')']).next())
            .unwrap_or("")
            .trim()
    }

    #[test]
    fn test_cancel_removes_net_zero_bracket() {
        let (_temp_dir, project_dir) = setup_test_env("ownership");
        let dump = emit_main_rc_ir(&project_dir);

        let main = func_block(&dump, "fn Main::main", |n| {
            n.split('#').count() == 3 && n.ends_with("#funptr1")
        });
        let call = main
            .iter()
            .find(|l| l.contains("= Main::tally") && l.contains("#borrow("))
            .expect("main should call the tally borrow version");
        let arr = borrow_call_first_arg(call);

        // Borrow-ification brackets the borrow call with a retain before it and a release after it;
        // because nothing between them consumes the array, cancellation removes both, leaving no
        // reference counting on the array in `main`.
        for l in &main {
            let t = l.trim_start();
            for op in ["retain", "release"] {
                assert!(
                    t != format!("{} {}", op, arr) && !t.starts_with(&format!("{} {} ", op, arr)),
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

    #[test]
    fn test_borrow_union_no_double_release() {
        let (_temp_dir, project_dir) = setup_test_env("union");
        let dump = emit_main_rc_ir(&project_dir);

        // `via_union` reads its array `p` (directly and through `some(p)`), so it has a borrow version
        // whose array parameter is `borrow`.
        assert!(
            sig_line(&dump, "fn Main::via_union", |n| n.ends_with("#borrow"))
                .contains("Std::Array Std::I64 [arg0] {borrow}"),
            "via_union should have a borrow version with a borrowed array parameter",
        );

        // The borrow version builds `some(p)` around the borrowed `p` and passes it to a borrowing
        // position. Because the union only lays the borrowed payload in place (it does not own it),
        // the version must perform no reference counting — in particular no release of the union,
        // which would free the caller's still-owned array.
        let via_borrow = func_block(&dump, "fn Main::via_union", |n| n.ends_with("#borrow"));
        assert!(
            via_borrow.iter().any(|l| l.contains("union_1(")),
            "the via_union borrow version should build the union:\n{}",
            via_borrow.join("\n")
        );
        assert!(
            via_borrow
                .iter()
                .all(|l| !l.trim_start().starts_with("release ")
                    && !l.trim_start().starts_with("retain ")),
            "the via_union borrow version must not reference-count the borrowed value or its union:\n{}",
            via_borrow.join("\n")
        );
    }

    #[test]
    fn test_unique_check_elim_local_fresh() {
        let (_temp_dir, project_dir) = setup_test_env("unique_elim");
        let dump = emit_main_rc_ir(&project_dir);

        // Each operation on a locally fresh (proven unique) value renders its dropped check as a
        // `[unique]` marker. Asserting the markers guards against silent *under*-elimination: a
        // regression that stops dropping a check would still pass the memcheck correctness tests
        // (which run the same whether or not elimination fires), but fails here.
        for elided in [
            // An array `set`.
            "Array::set [unique]",
            // An array `swap`.
            "Array::swap [unique]",
            // A generic `act`, whose `unsafe_is_unique` folds to the constant `true`.
            "is_unique[unique]",
            // A boxed-struct field `set` (field 0).
            "set_0 [unique]",
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
            dump.contains("Array::set("),
            "the set on an array of unknown sharing should keep its force-unique check:\n{}",
            dump
        );
    }
}
