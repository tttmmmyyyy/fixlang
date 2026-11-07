//! Ownership at the granularity borrow-ification decides it, the reference-counting unit: one
//! parameter whose units are owned and borrowed separately, and one unit holding two parameters.

#[cfg(test)]
mod borrow_unit_ownership_tests {
    use crate::{
        configuration::Configuration,
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    /// Each reference-counting unit of one parameter is disposed of exactly once when the units are
    /// owned and borrowed separately.
    ///
    /// Borrow-ification answers ownership one reference-counting unit at a time, so a parameter
    /// holding several units can be owned at one of them and borrowed at the others. `take_middle`
    /// is that shape: of the three units its parameter holds, it reads one, hands one to its caller,
    /// and drops the third. An answer given for the whole parameter instead either releases a unit
    /// the function borrows, freeing it while the caller still holds it, or keeps one the function
    /// owns. A second name for the value stays live across both calls, so the caller does still hold
    /// it.
    ///
    /// A released borrowed unit leaves the computed values intact here, so this runs under Valgrind
    /// MemCheck, which is what reports it.
    #[test]
    pub fn test_units_of_one_parameter_get_separate_ownership() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let source = r#"
            module Main;

            type Cell = box struct { v : Array I64, w : I64 };
            type Trio = struct { x : Array I64, y : Array I64, z : Cell };

            // Reads `x`, hands `y` out, and drops `z`: the three units of one parameter get three
            // different fates.
            take_middle : Trio -> Array I64;
            take_middle = |t| (
                let Trio { x : ax, y : ay, z : _az } = t;
                let head = Iterator::range(0, ax.@size).fold(0, |i, s| s + ax.@(i) * (i + 1));
                ay.set(0, head)
            );

            // Reads every unit and hands nothing out.
            survey : Trio -> I64;
            survey = |t| (
                let a = Iterator::range(0, t.@x.@size).fold(0, |i, s| s + t.@x.@(i));
                let b = Iterator::range(0, t.@y.@size).fold(0, |i, s| s + t.@y.@(i) * 2);
                a + b + t.@z.@w + t.@z.@v.@(0)
            );

            make_trio : I64 -> Trio;
            make_trio = |s| Trio {
                x : Array::from_map(4, |i| i + s),
                y : Array::from_map(3, |i| i * 2 + s),
                z : Cell { v : Array::from_map(2, |i| i + s), w : s }
            };

            main : IO ();
            main = (
                let t = make_trio(5);
                let u = t;
                let s1 = survey(u);
                let m = take_middle(u);
                let s2 = survey(t);
                assert_eq(|_|"reading is repeatable", s1, s2);;
                assert_eq(|_|"the middle unit was taken", m.@(0), 70);;
                assert_eq(|_|"the source keeps its middle unit", t.@y.@(0), 5);;
                assert_eq(|_|"the whole answer", s1 + m.@(0) + t.@y.@(0), 153);;
                pure()
            );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A version owns both parameters whose objects one reference-counting unit of its body holds,
    /// when the call reaching it lends only one of them.
    ///
    /// `tally` only reads its two arrays, so both are borrowable, and it puts them into one unboxed
    /// union, whose single unit holds a reference from each. The call in `main` lends `a0`, which it
    /// reads afterwards, and passes `b0` at its last use. A version borrowing `a` and owning `b`
    /// would have to answer once for the union's unit, which is wrong for one of the two arrays:
    /// leaving out the release frees nothing of `b`, and keeping it releases `a`, which the caller
    /// still holds. So the version reached owns both.
    #[test]
    pub fn test_a_unit_holding_a_lent_and_an_owned_argument_is_owned() {
        let source = r#"
            module Main;

            // One reference-counting unit holding two arrays.
            type Pair = unbox union { pair : (Array I64, Array I64), none : () };

            // Reads both arrays of the pair. Recursive, so the inliner leaves it a call.
            peek : Pair -> I64 -> I64;
            peek = |s, n| (
                if n == 0 { s.as_pair.@0.@(0) + s.as_pair.@1.@(0) };
                peek(s, n - 1)
            );

            // Reads `a` and `b` through a pair built of both.
            tally : Array I64 -> Array I64 -> I64 -> I64;
            tally = |a, b, i| (
                if i == 0 { 0 };
                let s = Pair::pair((a, b));
                peek(s, 2) + tally(a, b, i - 1)
            );

            main : IO ();
            main = (
                let a0 = Array::fill(3, 1);
                let b0 = Array::fill(3, 2);
                let t = tally(a0, b0, 2);
                assert_eq(|_|"the sums, then the lent array", t + a0.@(0), 7);;
                pure()
            );
        "#;
        test_source(source, Configuration::develop_mode());
    }
    /// Every call of a function asked for more versions than the clone budget allows disposes of
    /// each argument exactly once, whichever version it reaches.
    ///
    /// `peek` only reads its five arrays, so each of them is borrowable, and a call lends the ones
    /// the caller reads afterwards. The 32 calls in `main` lend every subset of the five, so they ask
    /// for 31 versions besides the all-owning original, past the budget on one function's clones.
    /// The calls past it reach the original, which owns every argument, and the caller retains `h`
    /// for each position it passes `h` at.
    #[test]
    pub fn test_calls_past_the_clone_budget_count_their_arguments() {
        let source = r#"
            module Main;

            // Reads its five arrays. Recursive, so the inliner leaves it a call.
            peek : Array I64 -> Array I64 -> Array I64 -> Array I64 -> Array I64 -> I64 -> I64;
            peek = |a, b, c, d, e, n| (
                if n == 0 { a.@(0) + b.@(1) + c.@(2) + d.@(0) + e.@(1) };
                peek(a, b, c, d, e, n - 1)
            );

            main : IO ();
            main = (
                let s = Array::fill(1, 2).@(0);
                let h = Array::fill(3, s);
                let acc = 0;
                let acc = acc + peek(Array::fill(3, s + 1), Array::fill(3, s + 1), Array::fill(3, s + 1), Array::fill(3, s + 1), Array::fill(3, s + 1), 2);
                let acc = acc + peek(h, Array::fill(3, s + 1), Array::fill(3, s + 1), Array::fill(3, s + 1), Array::fill(3, s + 1), 2);
                let acc = acc + peek(Array::fill(3, s + 1), h, Array::fill(3, s + 1), Array::fill(3, s + 1), Array::fill(3, s + 1), 2);
                let acc = acc + peek(h, h, Array::fill(3, s + 1), Array::fill(3, s + 1), Array::fill(3, s + 1), 2);
                let acc = acc + peek(Array::fill(3, s + 1), Array::fill(3, s + 1), h, Array::fill(3, s + 1), Array::fill(3, s + 1), 2);
                let acc = acc + peek(h, Array::fill(3, s + 1), h, Array::fill(3, s + 1), Array::fill(3, s + 1), 2);
                let acc = acc + peek(Array::fill(3, s + 1), h, h, Array::fill(3, s + 1), Array::fill(3, s + 1), 2);
                let acc = acc + peek(h, h, h, Array::fill(3, s + 1), Array::fill(3, s + 1), 2);
                let acc = acc + peek(Array::fill(3, s + 1), Array::fill(3, s + 1), Array::fill(3, s + 1), h, Array::fill(3, s + 1), 2);
                let acc = acc + peek(h, Array::fill(3, s + 1), Array::fill(3, s + 1), h, Array::fill(3, s + 1), 2);
                let acc = acc + peek(Array::fill(3, s + 1), h, Array::fill(3, s + 1), h, Array::fill(3, s + 1), 2);
                let acc = acc + peek(h, h, Array::fill(3, s + 1), h, Array::fill(3, s + 1), 2);
                let acc = acc + peek(Array::fill(3, s + 1), Array::fill(3, s + 1), h, h, Array::fill(3, s + 1), 2);
                let acc = acc + peek(h, Array::fill(3, s + 1), h, h, Array::fill(3, s + 1), 2);
                let acc = acc + peek(Array::fill(3, s + 1), h, h, h, Array::fill(3, s + 1), 2);
                let acc = acc + peek(h, h, h, h, Array::fill(3, s + 1), 2);
                let acc = acc + peek(Array::fill(3, s + 1), Array::fill(3, s + 1), Array::fill(3, s + 1), Array::fill(3, s + 1), h, 2);
                let acc = acc + peek(h, Array::fill(3, s + 1), Array::fill(3, s + 1), Array::fill(3, s + 1), h, 2);
                let acc = acc + peek(Array::fill(3, s + 1), h, Array::fill(3, s + 1), Array::fill(3, s + 1), h, 2);
                let acc = acc + peek(h, h, Array::fill(3, s + 1), Array::fill(3, s + 1), h, 2);
                let acc = acc + peek(Array::fill(3, s + 1), Array::fill(3, s + 1), h, Array::fill(3, s + 1), h, 2);
                let acc = acc + peek(h, Array::fill(3, s + 1), h, Array::fill(3, s + 1), h, 2);
                let acc = acc + peek(Array::fill(3, s + 1), h, h, Array::fill(3, s + 1), h, 2);
                let acc = acc + peek(h, h, h, Array::fill(3, s + 1), h, 2);
                let acc = acc + peek(Array::fill(3, s + 1), Array::fill(3, s + 1), Array::fill(3, s + 1), h, h, 2);
                let acc = acc + peek(h, Array::fill(3, s + 1), Array::fill(3, s + 1), h, h, 2);
                let acc = acc + peek(Array::fill(3, s + 1), h, Array::fill(3, s + 1), h, h, 2);
                let acc = acc + peek(h, h, Array::fill(3, s + 1), h, h, 2);
                let acc = acc + peek(Array::fill(3, s + 1), Array::fill(3, s + 1), h, h, h, 2);
                let acc = acc + peek(h, Array::fill(3, s + 1), h, h, h, 2);
                let acc = acc + peek(Array::fill(3, s + 1), h, h, h, h, 2);
                let acc = acc + peek(h, h, h, h, h, 2);
                assert_eq(|_|"the sums, then the held array", acc + h.@(0), 402);;
                pure()
            );
        "#;
        test_source(source, Configuration::develop_mode());
    }
}
