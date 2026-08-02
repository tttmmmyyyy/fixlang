// Tests for the RC IR locality inference, in two shapes.
//
// The integration tests read the `--emit-rc-ir` dump, which tags each kind of annotated site: a
// `Retain` / `Release` proved local with `@local` and one whose whole reachable graph is proved
// local with `@deeplocal`, an operation whose internal reference counting is proved local with a
// trailing `@local` on its right-hand side, and likewise a `destructure` and a boxed-union `case`
// arm. A small program with named `let`s therefore asserts the analysis end to end. The negative
// cases are the point: an operation that reaches a global object may not be tagged, and each case
// asserts on a site the dump really contains, so one that stopped producing that site would fail
// rather than pass vacuously.
//
// The in-process tests run the same programs under `Configuration::develop_mode()`, where every
// annotated site carries the assertion that its object really is in the local state. The dump tests
// pin what the analysis concluded; these pin that the conclusion holds at run time, which is what a
// wrong hand-written `locality_flow` declaration would break.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, fix_command};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_locality/cases");
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
    /// module. The build is pinned to the `max` optimization level, the only one the locality pass
    /// runs at, so the dump is the same whatever ambient `FIX_MAX_OPT_LEVEL` the suite runs under.
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

        let dump_path = project_dir.join(".fixlang/rc_ir.Main.post.txt");
        fs::read_to_string(&dump_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", dump_path.display(), e))
    }

    /// Every variable the dump binds under the source name `source_name` — one per clone of the
    /// function it lives in, since specialization renames the locals of each clone. The variable is
    /// the token before the type in `<var> : <type> (as <source_name>)`, which is how both a `let`
    /// and a `destructure` field render.
    fn binding_vars(dump: &str, source_name: &str) -> Vec<String> {
        let marker = format!("(as {})", source_name);
        let mut vars = vec![];
        for line in dump.lines() {
            for (at, _) in line.match_indices(&marker) {
                let var = line[..at]
                    .rsplit_once(" : ")
                    .and_then(|(before, _)| before.split_whitespace().last())
                    .unwrap_or_else(|| panic!("no variable before `{}` in:\n{}", marker, line));
                vars.push(var.to_string());
            }
        }
        assert!(
            !vars.is_empty(),
            "no binding `{}` in the RC IR dump:\n{}",
            marker,
            dump
        );
        vars
    }

    /// The `retain` / `release` lines operating on any of `vars`. A line names its variable right
    /// after the keyword, followed by the end of the line, the field path, or the state tag.
    fn rc_lines<'a>(dump: &'a str, vars: &[String]) -> Vec<&'a str> {
        dump.lines()
            .filter(|line| {
                let t = line.trim_start();
                ["retain ", "release "].iter().any(|op| {
                    t.strip_prefix(op).is_some_and(|rest| {
                        vars.iter().any(|v| {
                            rest == v
                                || rest
                                    .strip_prefix(v.as_str())
                                    .is_some_and(|tail| tail.starts_with([' ', '.']))
                        })
                    })
                })
            })
            .collect()
    }

    /// The `let` lines whose right-hand side is the operation `op` and which mention `value` — a
    /// variable of the dump, or the prefix a global's mangled name starts with. The value may be an
    /// operand of the operation or the binding it produces, since either identifies the site the
    /// source wrote.
    fn op_lines<'a>(dump: &'a str, op: &str, value: &str) -> Vec<&'a str> {
        let call = format!("= {}(", op);
        dump.lines()
            .filter(|line| line.contains(&call) && line.contains(value))
            .collect()
    }

    /// Assert that every application of `op` involving `value` carries the state tag `expected` (the
    /// empty string for an operation that must keep the runtime dispatch), and that there is at least
    /// one such application to judge.
    fn assert_op_state(dump: &str, op: &str, value: &str, expected: &str) {
        let lines = op_lines(dump, op, value);
        assert!(
            !lines.is_empty(),
            "no `{}` involving `{}` in the RC IR dump, so this case asserts nothing:\n{}",
            op,
            value,
            dump
        );
        for line in &lines {
            let tag = line.split_once('@').map_or("", |(_, t)| t.trim_end());
            assert_eq!(
                tag, expected,
                "`{}` involving `{}` should be tagged {:?}, but a line reads:\n{}",
                op, value, expected, line
            );
        }
    }

    /// Assert that every reference-counting operation on the value bound as `source_name` carries
    /// the state tag `expected` (the empty string for an operation that must keep the runtime
    /// dispatch), and that there is at least one such operation to judge.
    fn assert_rc_state(dump: &str, source_name: &str, expected: &str) {
        let vars = binding_vars(dump, source_name);
        let lines = rc_lines(dump, &vars);
        assert!(
            !lines.is_empty(),
            "`{}` is bound as {:?} but nothing counts a reference to it, so this case asserts \
             nothing:\n{}",
            source_name,
            vars,
            dump
        );
        for line in &lines {
            let tag = line.split_once('@').map_or("", |(_, t)| t.trim_end());
            assert_eq!(
                tag, expected,
                "the reference counting on `{}` should be tagged {:?}, but a line reads:\n{}",
                source_name, expected, line
            );
        }
    }

    /// Verifies the three-point chain on the one program that separates it from a two-point one, and
    /// the two doors a program can walk through without naming a global.
    #[test]
    fn test_locality_doors() {
        let (_temp_dir, project_dir) = setup_test_env("doors");
        let dump = emit_main_rc_ir(&project_dir);

        // An array of numbers built here reaches nothing but its own storage.
        assert_rc_state(&dump, "fresh", "deeplocal");

        // An array built here whose elements are the global: its storage is freshly allocated, so
        // its own reference count is counted directly, while nothing is claimed about what it
        // holds. Collapsing the two facts would lose one of these two assertions — a lattice
        // tracking only reachability would leave the container itself dispatching, and one tracking
        // only the root would hand out the element below as local.
        assert_rc_state(&dump, "mixed", "local");
        assert_rc_state(&dump, "elem", "");

        // A field taken out of a global unboxed struct is part of the global's graph.
        assert_rc_state(&dump, "p", "");
        assert_rc_state(&dump, "q", "");

        // A value rebuilt from a retained pointer may name anything, including a global's graph or
        // an object another thread holds.
        assert_rc_state(&dump, "restored", "");

        // The reference counting an operation performs inside itself, on the same operation applied
        // to a local container and to the global. Appending to a fresh array retains the value it
        // takes in, so appending the global keeps the dispatch while appending a number drops it.
        assert_op_state(&dump, "array_append_value[unique]", "Main::g#", "");
        let fresh = binding_vars(&dump, "fresh");
        assert_op_state(&dump, "array_append_value[unique]", &fresh[0], "local");

        // Reading a boxed element out of the array that holds the global retains that element.
        let mixed = binding_vars(&dump, "mixed");
        assert_op_state(&dump, "array_get", &mixed[0], "");
    }

    /// Verifies the annotation of an operation's internal reference counting on the two shapes that
    /// take a boxed value out of a boxed container — a struct field and a union payload. The source
    /// writes each shape once and reaches it twice, with a container built here and with a global,
    /// so the pair turns on the container's locality alone.
    #[test]
    fn test_locality_of_a_read_out_of_a_boxed_container() {
        let (_temp_dir, project_dir) = setup_test_env("containers");
        let dump = emit_main_rc_ir(&project_dir);

        let local_pair = binding_vars(&dump, "local_pair");
        let local_holder = binding_vars(&dump, "local_holder");

        assert_op_state(&dump, "struct_get_0", &local_pair[0], "local");
        assert_op_state(&dump, "struct_get_1", &local_pair[0], "local");
        assert_op_state(&dump, "union_as_0", &local_holder[0], "local");

        assert_op_state(&dump, "struct_get_0", "Main::gpair#", "");
        assert_op_state(&dump, "struct_get_1", "Main::gpair#", "");
        assert_op_state(&dump, "union_as_0", "Main::gholder#", "");
    }

    /// Verifies that the release borrow-ification leaves on a global keeps its runtime dispatch.
    /// Reference counting is inserted for locals, but the borrow rewrite adds a release naming
    /// whatever was passed at a borrowed position — a global included — so resolving an operand by
    /// whether it is a local binding is what stands between the pass and a freed global.
    #[test]
    fn test_release_of_a_global_keeps_its_dispatch() {
        let (_temp_dir, project_dir) = setup_test_env("global_release");
        let dump = emit_main_rc_ir(&project_dir);

        let release_of_global: Vec<&str> = dump
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                t.starts_with("release Main::g#") || t.starts_with("retain Main::g#")
            })
            .collect();
        assert!(
            !release_of_global.is_empty(),
            "borrow-ification should leave a release naming the global, but the dump has none:\n{}",
            dump
        );
        for line in &release_of_global {
            assert!(
                !line.contains('@'),
                "the reference counting on a global must keep its runtime dispatch:\n{}",
                line
            );
        }
    }
}

/// The same case programs, compiled and run in process. Development mode puts a state check at every
/// site the analysis annotated, so running these turns each conclusion the dump tests read into a
/// claim the program itself has to satisfy.
#[cfg(test)]
mod runtime_tests {
    use crate::configuration::Configuration;
    use crate::tests::test_util::test_source;

    /// Verifies that the annotations on a program walking all three doors out of the local state
    /// hold at run time — a global read directly, out of a container and out of a global struct, and
    /// a value rebuilt from a retained pointer.
    #[test]
    fn test_annotations_of_the_doors_hold_at_run_time() {
        let source = include_str!("test_locality/cases/doors/main.fix");
        test_source(source, Configuration::develop_mode());
    }

    /// Verifies that the annotations on reads out of a boxed struct and a boxed union hold at run
    /// time, over a container built by the program and the same shape over a global.
    #[test]
    fn test_annotations_of_reads_out_of_boxed_containers_hold_at_run_time() {
        let source = include_str!("test_locality/cases/containers/main.fix");
        test_source(source, Configuration::develop_mode());
    }

    /// Verifies that the annotations hold at run time where borrow-ification leaves a release naming
    /// the global itself.
    #[test]
    fn test_annotations_of_a_borrowed_global_hold_at_run_time() {
        let source = include_str!("test_locality/cases/global_release/main.fix");
        test_source(source, Configuration::develop_mode());
    }
}
