// Application inlining moves an application into the subexpressions of the function it applies.
// An argument that is already a variable is moved in as it is, which is what keeps the cost linear
// in the number of arguments pushed through a chain of `let`s — the shape uncurrying's eta
// expansion builds for a function of many parameters. A variable that the target binds is bound to
// a fresh name first, so moving it in leaves it referring to the same value.

#[cfg(test)]
mod tests {
    use crate::configuration::Configuration;
    use crate::tests::test_util::{
        fix_build_source_command, test_source, test_source_fail, wait_within,
    };
    use std::fs::{self, File};
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use tempfile::TempDir;

    /// The argument `x` of `(let x = ..; ..)(x)` denotes the outer `x` after the application is
    /// moved inside the `let`, where the name `x` denotes the bound lambda.
    #[test]
    fn test_variable_argument_pushed_into_a_let_keeps_its_meaning() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let x = 10;
            let y = (let x = |v : I64| v + 1; x)(x);
            assert_eq(|_|"the argument denotes the lambda bound by the inner `let`", y, 11);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The argument `a` of `(match o { some(a) => a, .. })(a)` denotes the outer `a` after the
    /// application is moved into the arms, where the name `a` denotes what the pattern binds.
    #[test]
    fn test_variable_argument_pushed_into_a_match_arm_keeps_its_meaning() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let a = 20;
            let o : Option (I64 -> I64) = Option::some(|w| w + 1);
            let z = (match o { some(a) => a, none(_) => |w| w + 100 })(a);
            assert_eq(|_|"the argument denotes the function bound by the arm", z, 21);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    // Compiling this arity takes a few seconds while the cost is linear in it, and is out of reach
    // while the cost doubles per parameter.
    const ARITY: usize = 25;

    // Generous next to the few seconds the build takes, and short enough to report a regression as
    // a failure instead of occupying the machine.
    const TIMEOUT: Duration = Duration::from_secs(180);

    /// The text `item` gives each of the `ARITY` parameter positions, joined by `separator`.
    fn joined_over_parameters(separator: &str, item: impl Fn(usize) -> String) -> String {
        (0..ARITY).map(item).collect::<Vec<_>>().join(separator)
    }

    /// Builds and runs a global function of `ARITY` parameters, failing if the build does not
    /// finish within `TIMEOUT`. The body weights each parameter by its position, so the result also
    /// pins the order the arguments arrive in.
    ///
    /// The time bound catches the doubling at `Basic` and above, where uncurrying eta-expands a
    /// global into a function pointer per arity. At `None` uncurrying is off and the bound is slack,
    /// while building and running a function of this many parameters is a check of its own.
    #[test]
    fn test_many_parameter_function_compiles_in_reasonable_time() {
        let params = joined_over_parameters(", ", |i| format!("x{}", i));
        let signature = vec!["I64"; ARITY + 1].join(" -> ");
        let body = joined_over_parameters(" + ", |i| format!("x{} * {}", i, i));
        let args = joined_over_parameters(", ", |i| i.to_string());
        let expected: usize = (0..ARITY).map(|i| i * i).sum();
        let source = format!(
            "module Main;\n\
             \n\
             g : {signature};\n\
             g = |{params}| {body};\n\
             \n\
             main : IO ();\n\
             main = println(g({args}).to_string);\n"
        );

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let program_path = temp_dir.path().join("many_params");

        // The compiler writes its diagnostics to a file rather than a pipe, which nothing reads
        // until the child exits — long enough to fill a pipe's buffer and block the very build
        // being timed.
        let log_path = temp_dir.path().join("build.log");
        let log = File::create(&log_path).expect("Failed to create the build log");
        let log_for_stderr = log
            .try_clone()
            .expect("Failed to clone the build log handle");

        let mut command = fix_build_source_command(temp_dir.path(), &source, "basic");
        command
            .arg("-o")
            .arg(&program_path)
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(log_for_stderr));
        let mut child = command.spawn().expect("Failed to execute fix build");
        let status = wait_within(
            &mut child,
            TIMEOUT,
            &format!("compiling a {}-parameter function", ARITY),
        );
        assert!(
            status.success(),
            "compiling a {}-parameter function failed: {}\n{}",
            ARITY,
            status,
            fs::read_to_string(&log_path).expect("Failed to read the build log")
        );

        let output = Command::new(&program_path)
            .output()
            .expect("Failed to run the compiled program");
        assert!(
            output.status.success(),
            "the compiled program exited with {}",
            output.status
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            expected.to_string(),
            "the {}-parameter function returned a wrong value",
            ARITY
        );
    }

    /// A tuple pattern binds the argument's name next to another name, so the renaming has to reach
    /// the sub-pattern that binds it.
    #[test]
    fn test_variable_argument_pushed_into_a_tuple_pattern_let_keeps_its_meaning() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let a = 5;
            let r = (let (a, b) = (100, |v : I64| v * 2); b)(a);
            assert_eq(|_|"the argument denotes the outer `a`", r, 10);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A struct pattern binds the argument's name, so the renaming has to reach the field pattern
    /// that binds it.
    #[test]
    fn test_variable_argument_pushed_into_a_struct_pattern_let_keeps_its_meaning() {
        let source = r#"
        module Main;

        type S = struct { x : I64, f : I64 -> I64 };

        main : IO ();
        main = (
            let x = 5;
            let s = S { x : 100, f : |v : I64| v * 2 };
            let r = (let S { x : x, f : f } = s; f)(x);
            assert_eq(|_|"the argument denotes the outer `x`", r, 10);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// An arm's pattern binds the argument's name inside a payload sub-pattern.
    #[test]
    fn test_variable_argument_pushed_into_a_nested_pattern_keeps_its_meaning() {
        let source = r#"
        module Main;

        type U = union { p : (I64, I64 -> I64), q : () };

        main : IO ();
        main = (
            let a = 5;
            let u : U = U::p((100, |v : I64| v * 2));
            let r = (match u { p((a, f)) => f, q(_) => |v : I64| v + 1000 })(a);
            assert_eq(|_|"the argument denotes the outer `a`", r, 10);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// An `if` binds nothing itself, and each branch binds the argument's name; the argument keeps
    /// its meaning in whichever branch runs.
    #[test]
    fn test_variable_argument_pushed_into_if_branches_that_shadow_it_keeps_its_meaning() {
        let source = r#"
        module Main;

        run : Bool -> IO ();
        run = |c| (
            let x = 5;
            let y = (if c { let x = |v : I64| v + 1; x } else { let x = |v : I64| v - 1; x })(x);
            assert_eq(|_|"the argument denotes the outer `x` in the branch that runs", y, if c { 6 } else { 4 });;
            pure()
        );

        main : IO ();
        main = (
            run(true);;
            run(false);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// An `eval` hands the argument on to its main expression, which binds the argument's name.
    #[test]
    fn test_variable_argument_pushed_into_an_eval_keeps_its_meaning() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let x = 5;
            let r = (eval x + 1; let x = 100; |v : I64| v * 2)(x);
            assert_eq(|_|"the argument denotes the outer `x`", r, 10);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The renamed binder is read by an array primitive, so the renaming has to reach the operand
    /// names an LLVM operation carries.
    #[test]
    fn test_renaming_a_binder_reaches_llvm_operands() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let arr = Array::from_map(3, |k| k * 10);
            let i = 1;
            let r = (let i = 0; |v : I64| arr.@(i) + v)(i);
            assert_eq(|_|"the index reads the inner `i` and the argument the outer one", r, 1);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// Different arms bind different names and only one collides with the argument; both arms are
    /// taken.
    #[test]
    fn test_variable_argument_pushed_into_arms_binding_different_names() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let a = 5;
            let some_f : Option (I64 -> I64) = Option::some(|v : I64| v * 2);
            let none_f : Option (I64 -> I64) = Option::none();
            let taken = (match some_f { some(a) => a, none(b) => |v : I64| v + 1000 })(a);
            assert_eq(|_|"the taken arm applies its bound function to the outer `a`", taken, 10);;
            let fallback = (match none_f { some(a) => a, none(b) => |v : I64| v + 1000 })(a);
            assert_eq(|_|"the other arm applies its own function to the outer `a`", fallback, 1005);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A chain of `let`s each rebinding the argument's name, so the name is renamed away at every
    /// level the argument is pushed through.
    #[test]
    fn test_variable_argument_pushed_through_a_chain_of_lets_keeps_its_meaning() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let a = 5;
            let r = (
                let a = 100;
                let a = a + 1;
                let a = a + 1;
                |v : I64| v * 2 + a
            )(a);
            assert_eq(|_|"the argument denotes the outer `a`", r, 112);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A `let` over a `match` over a `let`, each binding the argument's name, so the argument is
    /// renamed away from three binders of different kinds on its way in.
    #[test]
    fn test_variable_argument_pushed_through_nested_shadowing_binders_keeps_its_meaning() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let x = 9;
            let y = (
                let x = Option::some(7);
                match x {
                    some(x) => (let x = |v : I64| v * 2; x),
                    none(_) => |v : I64| v
                }
            )(x);
            assert_eq(|_|"the argument denotes the outer `x` through three shadowing binders", y, 18);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The argument names the `match` scrutinee and the arm's binder at once, and a `let`'s bound
    /// expression names the very variable the `let` binds.
    #[test]
    fn test_variable_argument_that_names_the_binder_it_lands_under_keeps_its_meaning() {
        let source = r#"
        module Main;

        scrutinee_and_arm : Option I64 -> I64;
        scrutinee_and_arm = |x| (
            (match x {
                some(x) => |v : Option I64| v.as_some + x,
                none(_) => |_ : Option I64| 0
            })(x)
        );

        bound_names_the_binder : I64 -> I64;
        bound_names_the_binder = |x| (
            (let x = |v : I64| v + x; x)(x)
        );

        main : IO ();
        main = (
            assert_eq(|_|"the argument denotes the scrutinee, the arm's `x` the payload", scrutinee_and_arm(Option::some(7)), 14);;
            assert_eq(|_|"both the bound expression and the argument denote the outer `x`", bound_names_the_binder(10), 20);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A `fix`-defined global is lifted into a directly recursive global whose locals are renamed to
    /// `#vN`, the names uncurrying's eta expansion also gives the parameters it adds, so those
    /// arguments are pushed into a body that already binds their names.
    #[test]
    fn test_eta_expanded_arguments_meet_a_body_that_binds_their_names() {
        let source = r#"
        module Main;

        f : I64 -> I64 -> I64;
        f = fix(|self, a| (
            let s = 1;
            let s = s + 1;
            let s = s + 1;
            let s = s + 1;
            if a <= 0 {
                |b| b * 1000 + s
            } else {
                |b| self(a - 1)(b) + s
            }
        ));

        main : IO ();
        main = (
            assert_eq(|_|"each argument reaches the parameter it was applied to", f(2, 7), 7012);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A global whose body interleaves `if`, `let`, `match`, `eval` and a lambda, so eta expansion
    /// pushes its arguments through every shape the pass rewrites.
    #[test]
    fn test_eta_expanded_arguments_travel_through_every_shape() {
        let source = r#"
        module Main;

        g : I64 -> I64 -> I64 -> I64;
        g = (
            let k = 7;
            if k > 0 {
                let m = 3;
                |a| (
                    let n = 5;
                    eval n + 1;
                    match Option::some(2) {
                        some(t) => |b| (
                            let u = 11;
                            |c| a * 10000 + b * 100 + c + k + m + n + t + u
                        ),
                        none(_) => |b| |c| 0
                    }
                )
            } else {
                |a| |b| |c| -1
            }
        );

        main : IO ();
        main = (
            assert_eq(|_|"each argument reaches the parameter it was applied to", g(1, 2, 3), 10231);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The bound expression of the `let` writes to the array that is also pushed in as the
    /// argument, so the write clones and the argument reads the contents from before it.
    #[test]
    fn test_array_written_by_the_let_the_argument_is_pushed_into_is_cloned() {
        let source = r#"
        module Main;

        f : I64 -> I64;
        f = |d| (
            let arr = Array::fill(3, d);
            (
                let written = arr.set(0, 77);
                |a : Array I64| a.@(0) * 1000 + written.@(0)
            )(arr)
        );

        main : IO ();
        main = (
            assert_eq(|_|"the argument reads the array as it was before the write", f(4), 4077);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A chain of `let`s that only read the array is pushed through, so the array reaching the
    /// lambda is still the only reference to its storage and the write lands in place.
    #[test]
    fn test_argument_pushed_through_reading_lets_stays_unique() {
        let source = r#"
        module Main;

        f : I64 -> I64;
        f = |d| (
            let arr = Array::fill(4, d);
            (
                let first = arr.@(0);
                let second = arr.@(1) + first;
                let sized = arr.@size + second;
                |a : Array I64| a.assert_unique_array(|_|"the pushed array is shared").set(0, sized)
            )(arr).to_iter.sum
        );

        main : IO ();
        main = (
            assert_eq(|_|"the reads leave the array unique", f(2), 14);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A boxed value read by the `let`'s bound expression and pushed in as the argument is held by
    /// both, so the uniqueness check in the bound expression reports it shared.
    #[test]
    fn test_value_read_by_the_let_and_pushed_in_is_reported_shared() {
        let source = r#"
        module Main;

        type Res = box struct { id : I64 };

        f : I64 -> I64;
        f = |d| (
            let res = Res { id : d };
            (
                let (unique, checked) = res.unsafe_is_unique;
                |x : Res| x.@id * 10 + (if unique { 1 } else { 0 }) + checked.@id * 100
            )(res)
        );

        main : IO ();
        main = (
            assert_eq(|_|"the value is shared with the pushed argument", f(3), 330);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The bound expression asserts that the array's storage is uniquely referenced while the same
    /// array is pushed in as the argument, whose later use keeps a reference alive across the
    /// assertion.
    #[test]
    fn test_array_asserted_unique_in_the_let_it_is_pushed_into_is_shared() {
        let source = r#"
        module Main;

        f : I64 -> I64;
        f = |d| (
            let arr = Array::fill(3, d);
            (
                let checked = arr.assert_unique_array(|_|"shared with the argument");
                |a : Array I64| a.@(0) + checked.@(0)
            )(arr)
        );

        main : IO ();
        main = println(f(4).to_string);
        "#;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "Array storage is not unique",
        );
    }
}
