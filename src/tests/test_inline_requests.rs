//! What the optimizer asks of the back end about inlining, read off the emitted LLVM IR.
//!
//! A global whose body is small enough to stand where it is called is one the back end is asked to
//! inline at every call, which reaches the back end as the `alwaysinline` attribute on each
//! function generated from that global. These tests assert on that attribute: the program answers
//! the same whether or not the request is made, so a suite that only runs the program stays green
//! with every request withdrawn.

#[cfg(test)]
mod integration_tests {
    use crate::constants::{CLOSURE_LAM_SUFFIX, CLOSURE_SPEC_SUFFIX};
    use crate::tests::test_util::{
        emitted_llvm_ir, fix_build_source_command, llvm_function_attribute_flags, EmittedIr,
    };
    use std::process::Command;
    use tempfile::TempDir;

    /// The LLVM function attribute that asks for a body at every place that calls it.
    const ALWAYS_INLINE: &str = "alwaysinline";

    /// How a lambda that stays where it was written is named: the enclosing function's name
    /// followed by `::closure#<n>`. Such a body is reached through the closure holding it, so the
    /// places that call it are not known where it is generated.
    const IN_PLACE_CLOSURE: &str = "::closure#";

    /// The optimization level these read, which is the level the request is made at.
    const OPT_LEVEL: &str = "max";

    /// A lambda passed to a global function, which is the shape both techniques of the pass act on:
    /// the lambda is lifted to a global function, and `fold` is copied into a version that calls
    /// that function by name.
    const SMALL_LAMBDA_SOURCE: &str = r#"
module Main;

main : IO () = (
    let n = 3;
    println(Iterator::range(0, 10).fold(0, |i, acc| acc + i * n).to_string)
);
"#;

    /// What `SMALL_LAMBDA_SOURCE` prints: the sum of `3 * i` over `0..9`.
    const SMALL_LAMBDA_OUTPUT: &str = "135";

    /// The same shape carrying a lambda whose body is about three times the size the pass lets a
    /// body reach and still be asked for at every place that calls it.
    const OVERSIZED_LAMBDA_SOURCE: &str = r#"
module Main;

weighted_sum : I64 -> I64;
weighted_sum = |n| Iterator::range(0, n).fold(0, |i, acc| (
    acc + i * 1 + i * 2 + i * 3 + i * 4 + i * 5 + i * 6 + i * 7 + i * 8 + i * 9 + i * 10
        + i * 11 + i * 12 + i * 13 + i * 14 + i * 15 + i * 16 + i * 17 + i * 18 + i * 19 + i * 20
        + i * 21 + i * 22 + i * 23 + i * 24 + i * 25 + i * 26 + i * 27 + i * 28 + i * 29 + i * 30
        + i * 31 + i * 32 + i * 33 + i * 34 + i * 35 + i * 36 + i * 37 + i * 38 + i * 39 + i * 40
        + i * 41 + i * 42 + i * 43 + i * 44 + i * 45 + i * 46 + i * 47 + i * 48 + i * 49 + i * 50
        + i * 51 + i * 52 + i * 53 + i * 54 + i * 55 + i * 56 + i * 57 + i * 58 + i * 59 + i * 60
));

main : IO () = println(weighted_sum(10).to_string);
"#;

    /// What `OVERSIZED_LAMBDA_SOURCE` prints: the weights 1 through 60 sum to 1830, and the `i` they
    /// multiply sums to 45 over `0..9`.
    const OVERSIZED_LAMBDA_OUTPUT: &str = "82350";

    /// The prefix the lifted body of `weighted_sum`'s lambda is named with.
    const OVERSIZED_LAMBDA_OWNER: &str = "Main::weighted_sum#";

    /// A program whose specialized copies the RC IR copies again: `sum_by` reads an array it is
    /// given, which is what borrow-ification copies a function for, and it is called at two
    /// lambdas, which is what the uniqueness and locality of its inputs are decided per.
    const COPIED_BODIES_SOURCE: &str = r#"
module Main;

sum_by : (I64 -> I64) -> Array I64 -> I64;
sum_by = |f, xs| (
    let n = xs.get_size;
    Iterator::range(0, n).fold(0, |i, acc| acc + f(xs.@(i)))
);

main : IO () = (
    let k = 3;
    let xs = Array::from_map(8, |i| i * i);
    let a = sum_by(|x| x * k + 1, xs);
    let b = sum_by(|x| x - k, xs);
    println((a + b).to_string)
);
"#;

    /// What `COPIED_BODIES_SOURCE` prints: the squares below 64 sum to 140, so the two readings of
    /// them come to `3 * 140 + 8` and `140 - 8 * 3`.
    const COPIED_BODIES_OUTPUT: &str = "544";

    /// Builds `source` at the `max` optimization level, runs it, and returns the LLVM IR of the
    /// modules the build emitted, as code generation wrote them.
    ///
    /// Running the program is what keeps the IR honest: the attributes asserted on come off a build
    /// that is known to answer correctly. The IR read is the generated one, because the pass
    /// pipeline consumes the request and leaves the attribute nowhere behind.
    ///
    /// Each call builds in a directory of its own, which is what makes the returned IR the work of
    /// this build alone. `FIX_MAX_OPT_LEVEL` is pinned to the level asked for, so that the level is
    /// the one this test wants whatever the level the suite is being run at.
    fn build_run_and_read_ir(source: &str, expected_output: &str) -> String {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dir = temp_dir.path();
        let build_output = fix_build_source_command(dir, source, OPT_LEVEL)
            .arg("--emit-llvm")
            .env("FIX_MAX_OPT_LEVEL", OPT_LEVEL)
            .output()
            .expect("Failed to execute fix build");
        assert!(
            build_output.status.success(),
            "the build should succeed.\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build_output.stdout),
            String::from_utf8_lossy(&build_output.stderr),
        );

        let run_output = Command::new(dir.join("a.out"))
            .current_dir(dir)
            .output()
            .expect("Failed to execute the built program");
        assert!(
            run_output.status.success(),
            "the built program should run cleanly, but exited with {}.\nstderr: {}",
            run_output.status,
            String::from_utf8_lossy(&run_output.stderr),
        );
        assert_eq!(
            String::from_utf8_lossy(&run_output.stdout).trim(),
            expected_output,
            "the program should answer the same with the bodies the pass minted inlined",
        );

        emitted_llvm_ir(dir, EmittedIr::BeforeOptimization)
    }

    /// The small bodies the optimizer leaves behind are asked for at every call — the lambda lifted
    /// out of `main`, and the copy of `fold` specialized on it. A lambda that stays where it was
    /// written is reached through the closure holding it, so no call site of it is known and it is
    /// asked for nowhere.
    #[test]
    fn test_a_small_body_is_asked_for_at_every_call_and_a_closure_is_not() {
        let ir = build_run_and_read_ir(SMALL_LAMBDA_SOURCE, SMALL_LAMBDA_OUTPUT);
        let functions = llvm_function_attribute_flags(&ir, ALWAYS_INLINE);

        for minted_with in [CLOSURE_LAM_SUFFIX, CLOSURE_SPEC_SUFFIX] {
            let minted = functions
                .iter()
                .filter(|(name, _)| name.contains(minted_with) && !name.contains(IN_PLACE_CLOSURE))
                .collect::<Vec<_>>();
            assert!(
                !minted.is_empty(),
                "the pass should mint a body named with `{}`, but the build generated none",
                minted_with
            );
            let unasked = minted
                .iter()
                .filter(|(_, asked)| !asked)
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>();
            assert!(
                unasked.is_empty(),
                "a body named with `{}` is small enough to stand at its call sites, so it should \
                 carry `{}`, but these do not: {:?}",
                minted_with,
                ALWAYS_INLINE,
                unasked
            );
        }

        let asked_closures = functions
            .iter()
            .filter(|(name, asked)| *asked && name.contains(IN_PLACE_CLOSURE))
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>();
        assert!(
            asked_closures.is_empty(),
            "a lambda reached through the closure holding it should carry no `{}`, but these do: \
             {:?}",
            ALWAYS_INLINE,
            asked_closures
        );
    }

    /// A body too large for every place that calls it to hold a copy is left to the back end's own
    /// accounting, while the small bodies of the same build are still asked for.
    #[test]
    fn test_a_body_too_large_to_stand_at_its_call_sites_is_not_asked_for() {
        let ir = build_run_and_read_ir(OVERSIZED_LAMBDA_SOURCE, OVERSIZED_LAMBDA_OUTPUT);
        let functions = llvm_function_attribute_flags(&ir, ALWAYS_INLINE);

        let oversized = functions
            .iter()
            .filter(|(name, _)| {
                name.starts_with(OVERSIZED_LAMBDA_OWNER)
                    && name.contains(CLOSURE_LAM_SUFFIX)
                    && !name.contains(IN_PLACE_CLOSURE)
            })
            .collect::<Vec<_>>();
        assert!(
            !oversized.is_empty(),
            "the pass should lift the lambda `weighted_sum` passes on, but the build generated no \
             body named with `{}` under `{}`",
            CLOSURE_LAM_SUFFIX,
            OVERSIZED_LAMBDA_OWNER
        );
        let asked = oversized
            .iter()
            .filter(|(_, asked)| *asked)
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>();
        assert!(
            asked.is_empty(),
            "the lifted body of `weighted_sum` is far larger than a call site can hold, so it \
             should carry no `{}`, but these do: {:?}",
            ALWAYS_INLINE,
            asked
        );

        // What tells a withdrawal apart from a build that asks for nothing: the copy of `fold`
        // specialized on that lambda is small, and is asked for.
        let specialized = functions
            .iter()
            .filter(|(name, _)| name.contains(CLOSURE_SPEC_SUFFIX))
            .collect::<Vec<_>>();
        assert!(
            !specialized.is_empty() && specialized.iter().all(|(_, asked)| *asked),
            "the copy specialized on the lambda is small enough to stand at its call sites, so it \
             should carry `{}`: {:?}",
            ALWAYS_INLINE,
            specialized
        );
    }

    /// A copy the RC IR makes of a function — for the ownership of its inputs, for their
    /// uniqueness, for their locality — is called where the function it copies was called, so it
    /// carries the request that function carried. The request is decided once per global, which is
    /// what makes the copies agree.
    #[test]
    fn test_a_copy_carries_the_request_of_the_function_it_copies() {
        let ir = build_run_and_read_ir(COPIED_BODIES_SOURCE, COPIED_BODIES_OUTPUT);
        let functions = llvm_function_attribute_flags(&ir, ALWAYS_INLINE);

        // A copy is named after the function it copies, with a segment of its own appended.
        let copies = functions
            .iter()
            .filter_map(|(name, asked)| {
                let copied = functions.iter().find(|(other, _)| {
                    name.starts_with(other.as_str()) && name[other.len()..].starts_with('#')
                })?;
                Some((name, *asked, copied.1))
            })
            .collect::<Vec<_>>();

        let copies_of_asked = copies
            .iter()
            .filter(|(_, _, copied_asked)| *copied_asked)
            .collect::<Vec<_>>();
        assert!(
            !copies_of_asked.is_empty(),
            "the RC IR should copy a body the pass minted, but the build generated no copy of one"
        );
        let dropped = copies_of_asked
            .iter()
            .filter(|(_, asked, _)| !asked)
            .map(|(name, _, _)| name.as_str())
            .collect::<Vec<_>>();
        assert!(
            dropped.is_empty(),
            "a copy stands where the body it copies stood, so it should carry `{}`, but these do \
             not: {:?}",
            ALWAYS_INLINE,
            dropped
        );
    }
}
