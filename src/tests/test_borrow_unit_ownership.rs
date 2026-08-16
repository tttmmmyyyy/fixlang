//! Ownership at the granularity borrow-ification decides it: one parameter whose reference-counting
//! units are owned and borrowed separately.

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
}
