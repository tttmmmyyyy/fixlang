// A value updated under a condition is, after the update, either the value that was there or one
// built out of it, so which object each of its references belongs to is decided by looking into
// both. Chaining such updates chains that question: the answer for one stage is asked of the stage
// before it, twice over. The compiler answers each value once, so a chain of `STAGES` updates
// compiles in a time that grows with its length, and the deadline on each build is what measures
// that: asking down both branches instead takes a time that doubles per stage.

#[cfg(test)]
mod build_time_tests {
    use crate::tests::test_util::build_within_and_run;
    use std::time::Duration;

    /// Long enough that a question asked down both branches of every stage is out of reach — that
    /// walk doubles per stage, and it already takes seconds at two thirds of this length — and
    /// short enough that asking it once per value compiles the chain in under a second. Odd, so
    /// that the swap chain's result tells a run that took every condition from one that took none.
    const STAGES: usize = 25;

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
    fn main_calling(chain: &str) -> String {
        format!(
            "main : IO ();\n\
             main = (\n\
             \x20   let args = *IO::get_args;\n\
             \x20   let cs = Array::fill({STAGES}, args.@size > 0);\n\
             \x20   println $ {chain}(cs, Pair {{ a : [{FIRST_A}, 2], b : [{FIRST_B}, 4] }}).to_string\n\
             );\n"
        )
    }

    /// A chain of `STAGES` conditional swaps: each stage is either the pair before it, or a pair
    /// built out of that pair's two fields exchanged. Both arms reach the previous stage — the
    /// second through the fields it is built from — so every stage doubles the question.
    fn swap_chain_source() -> String {
        let mut body = String::new();
        for i in 0..STAGES {
            body += &format!(
                "    let p{} = if cs.@({i}) {{ Pair {{ a : p{i}.@b, b : p{i}.@a }} }} else {{ p{i} }};\n",
                i + 1
            );
        }
        body += &format!("    p{STAGES}.@a.@(0) * 10 + p{STAGES}.@b.@(0)\n");
        format!(
            "{HEADER}swaps : Array Bool -> Pair -> I64;\n\
             swaps = |cs, p0| (\n{body});\n\
             \n{}",
            main_calling("swaps")
        )
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
        let mut body = String::new();
        for i in 0..STAGES {
            body += &format!(
                "    let q{} = if cs.@({i}) {{ q{i}.mod_a(|xs| xs.set(0, {})) }} else {{ q{i} }};\n",
                i + 1,
                i + 1
            );
        }
        body += &format!("    q{STAGES}.@a.@(0) * 10 + q{STAGES}.@b.@(0)\n");
        format!(
            "{HEADER}updates : Array Bool -> Pair -> I64;\n\
             updates = |cs, q0| (\n{body});\n\
             \n{}",
            main_calling("updates")
        )
    }

    /// What `update_chain_source` prints with every condition true: the last stage's write is the
    /// one that stands, and `b` is untouched.
    fn updated_value() -> i64 {
        STAGES as i64 * 10 + FIRST_B
    }

    /// A chain of conditional swaps compiles in reasonable time, and computes what the source says.
    /// `max` is where the reference counting the chain drives is inferred.
    #[test]
    fn test_chain_of_conditional_swaps_compiles_in_reasonable_time() {
        let printed = build_within_and_run(
            &swap_chain_source(),
            "max",
            TIMEOUT,
            &format!("a chain of {} conditional swaps", STAGES),
        );
        assert_eq!(
            printed,
            swapped_value().to_string(),
            "a chain of {} conditional swaps returned a wrong value",
            STAGES
        );
    }

    /// A chain of conditional field updates compiles in reasonable time, and computes what the
    /// source says.
    #[test]
    fn test_chain_of_conditional_field_updates_compiles_in_reasonable_time() {
        let printed = build_within_and_run(
            &update_chain_source(),
            "max",
            TIMEOUT,
            &format!("a chain of {} conditional field updates", STAGES),
        );
        assert_eq!(
            printed,
            updated_value().to_string(),
            "a chain of {} conditional field updates returned a wrong value",
            STAGES
        );
    }
}
