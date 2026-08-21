// A value updated under a condition is, after the update, either the value that was there or one
// built out of it, so `origin` in `crate::rc_ir::ownership` decides which object each of its
// references belongs to by looking into both. Chaining such updates chains that question: the
// answer for one stage is asked of the stage before it, twice over. `origin` answers each value
// once, so a chain of `STAGES` updates compiles in a time that grows with its length, and the
// deadline on each build is what measures that: asking down both branches instead takes a time
// that doubles per stage.
//
// The answer decides one more thing: whether a write goes into the value or into a copy of it. A
// stage's value is one of two objects, and an answer that names a single one of them lets a write
// land in a value another name still holds — which a program that reads the value its chain started
// from, after the chain, reads back as changed.

/// The time a chain of conditional updates takes to compile.
#[cfg(test)]
mod build_time_tests {
    use crate::tests::test_util::build_within_and_run;
    use std::time::Duration;

    /// Long enough that a question asked down both branches of every stage is out of reach — that
    /// walk doubles per stage, and it already takes seconds at two thirds of this length — and
    /// short enough that asking it once per value compiles the chain in under a second. Odd, so
    /// that the swap chain's result tells a run that took every condition from one that took none.
    const STAGES: usize = 25;

    /// The swap chain's result tells a run that took every condition from one that took none only
    /// where the stages are odd in number, since an even number of swaps puts the fields back.
    const _: () = assert!(STAGES % 2 == 1);

    /// Generous next to the fraction of a second the build takes, and short enough to report a
    /// regression as a failure instead of occupying the machine.
    const TIMEOUT: Duration = Duration::from_secs(60);

    /// The fields of the pair the chains start from: the first element of `a`, then of `b`.
    const FIRST_A: i64 = 1;
    const FIRST_B: i64 = 3;

    /// The header both chains are written under: a pair of two arrays, unboxed, so that a field of
    /// it is a reference the pair itself holds.
    const HEADER: &str = "module Main;\n\
                          \n\
                          type Pair = unbox struct { a : Array I64, b : Array I64 };\n\
                          \n";

    /// The main the chains are called from. The conditions are read out of the program's arguments,
    /// which the compiler cannot fold away, and are all true for every run: a program is given its
    /// own name as its first argument.
    fn main_source(chain_name: &str) -> String {
        format!(
            "main : IO ();\n\
             main = (\n\
             \x20   let args = *IO::get_args;\n\
             \x20   let cs = Array::fill({STAGES}, args.@size > 0);\n\
             \x20   println $ {chain_name}(cs, Pair {{ a : [{FIRST_A}, 2], b : [{FIRST_B}, 4] }}).to_string\n\
             );\n"
        )
    }

    /// The source of a chain of `STAGES` stages over the pair the function `chain_name` is given
    /// as `{var_prefix}0`: stage `i` binds `{var_prefix}{i+1}` to what `stage_source` writes out of
    /// the stage before it, and the function returns the first elements of the last stage's two
    /// fields as one number.
    fn chain_source(
        chain_name: &str,
        var_prefix: &str,
        stage_source: impl Fn(usize, &str) -> String,
    ) -> String {
        let mut body = String::new();
        for i in 0..STAGES {
            body += &format!(
                "    let {var_prefix}{} = {};\n",
                i + 1,
                stage_source(i, &format!("{var_prefix}{i}"))
            );
        }
        body += &format!("    {var_prefix}{STAGES}.@a.@(0) * 10 + {var_prefix}{STAGES}.@b.@(0)\n");
        format!(
            "{HEADER}{chain_name} : Array Bool -> Pair -> I64;\n\
             {chain_name} = |cs, {var_prefix}0| (\n{body});\n\
             \n{}",
            main_source(chain_name)
        )
    }

    /// A chain of `STAGES` conditional swaps: each stage is either the pair before it, or a pair
    /// built out of that pair's two fields exchanged. Both arms reach the previous stage — the
    /// second through the fields it is built from — so every stage doubles the question.
    fn swap_chain_source() -> String {
        chain_source("swap_chain", "p", |i, prev| {
            format!("if cs.@({i}) {{ Pair {{ a : {prev}.@b, b : {prev}.@a }} }} else {{ {prev} }}")
        })
    }

    /// What `swap_chain_source` prints with every condition true: the two fields have changed places
    /// once per stage.
    fn swapped_value() -> i64 {
        let (a, b) = if STAGES % 2 == 0 {
            (FIRST_A, FIRST_B)
        } else {
            (FIRST_B, FIRST_A)
        };
        a * 10 + b
    }

    /// A chain of `STAGES` conditional field updates, written the way a program threads a value
    /// through conditions: each stage is either the pair before it, or that pair with one element
    /// of one field written.
    fn update_chain_source() -> String {
        chain_source("update_chain", "q", |i, prev| {
            format!(
                "if cs.@({i}) {{ {prev}.mod_a(|xs| xs.set(0, {})) }} else {{ {prev} }}",
                i + 1
            )
        })
    }

    /// What `update_chain_source` prints with every condition true: the last stage's write is the
    /// one that stands, and `b` is untouched.
    fn updated_value() -> i64 {
        STAGES as i64 * 10 + FIRST_B
    }

    /// Builds `source` within `TIMEOUT` and checks that the program prints `expected`. The build
    /// is at `max`, the lowest optimization level `origin` runs at.
    ///
    /// # Arguments
    /// * `description` — what is being compiled, as a phrase that reads after "compiling": it is
    ///   what a failure names, e.g. "a chain of 25 conditional swaps".
    fn assert_chain_compiles_and_prints(source: &str, expected: i64, description: &str) {
        let printed = build_within_and_run(source, "max", TIMEOUT, description);
        assert_eq!(
            printed,
            expected.to_string(),
            "{} returned a wrong value",
            description
        );
    }

    /// A chain of conditional swaps compiles in reasonable time, and computes what the source says.
    #[test]
    fn test_chain_of_conditional_swaps_compiles_in_reasonable_time() {
        assert_chain_compiles_and_prints(
            &swap_chain_source(),
            swapped_value(),
            &format!("a chain of {STAGES} conditional swaps"),
        );
    }

    /// A chain of conditional field updates compiles in reasonable time, and computes what the
    /// source says.
    #[test]
    fn test_chain_of_conditional_field_updates_compiles_in_reasonable_time() {
        assert_chain_compiles_and_prints(
            &update_chain_source(),
            updated_value(),
            &format!("a chain of {STAGES} conditional field updates"),
        );
    }
}

/// The answers the chain rests on, read through the value it started from.
#[cfg(test)]
mod aliasing_tests {
    use crate::configuration::Configuration;
    use crate::tests::test_util::test_source;

    /// A chain of conditional updates over a pair the caller keeps: each stage is either the pair
    /// before it, or that pair with one element of one field written, so every write reaches a value
    /// the caller still holds. A write that took the value for its own would go in place, and the
    /// pair the chain started from would read as that write left it.
    ///
    /// The conditions are read out of the program's arguments, which the compiler cannot fold away,
    /// and they alternate, so both the arm that writes and the arm that passes the value through are
    /// taken.
    #[test]
    pub fn test_a_chain_of_conditional_updates_leaves_the_value_it_started_from() {
        let source = r#"
            module Main;

            type Pair = unbox struct { a : Array I64, b : Array I64 };

            update_chain : Array Bool -> Pair -> Pair;
            update_chain = |cs, q0| (
                let q1 = if cs.@(0) { q0.mod_a(|xs| xs.set(0, 1)) } else { q0 };
                let q2 = if cs.@(1) { q1.mod_a(|xs| xs.set(0, 2)) } else { q1 };
                let q3 = if cs.@(2) { q2.mod_b(|xs| xs.set(0, 3)) } else { q2 };
                let q4 = if cs.@(3) { q3.mod_a(|xs| xs.set(0, 4)) } else { q3 };
                q4
            );

            main : IO ();
            main = (
                let args = *IO::get_args;
                let taken = args.@size > 0;
                let start = Pair { a : [10, 20], b : [30, 40] };
                let end = update_chain([taken, !taken, taken, !taken], start);
                assert_eq(|_|"the field the first stage wrote", end.@a.@(0), 1);;
                assert_eq(|_|"the field the third stage wrote", end.@b.@(0), 3);;
                assert_eq(|_|"the first field of the value the chain started from", start.@a.@(0), 10);;
                assert_eq(|_|"the second field of the value the chain started from", start.@b.@(0), 30);;
                pure()
            );
        "#;
        test_source(source, Configuration::develop_mode());
    }
}
