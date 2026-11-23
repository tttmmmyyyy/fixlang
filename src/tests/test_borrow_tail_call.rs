//! Tail recursion across the versions borrow-ification routes between: a tail call must stay a
//! tail call whichever version it is routed to, so these loops must run in constant stack.

#[cfg(test)]
mod borrow_tail_call_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        tests::test_util::test_source,
    };

    /// Deep tail recursion with a read-only parameter, and with a read-only parameter beside an
    /// owned one. Each tail call is routed to a version borrowing the read-only parameter, and must
    /// leave no release after the call. The owned argument a tail call passes is at its last use,
    /// so the version reached owns it and frees it. A release left after the call
    /// turns the loop into real recursion, and the depth here overflows the stack.
    ///
    /// The depth makes this a stack test, not a memcheck one, so Valgrind is switched off to keep
    /// it fast.
    #[test]
    pub fn test_tail_recursion_survives_borrow_routing() {
        let source = r#"
            module Main;

            // Only reads `a`: its tail call is routed to a version borrowing `a`.
            loop_sum : I64 -> I64 -> Array I64 -> I64;
            loop_sum = |n, acc, a| (
                if n == 0 { acc };
                loop_sum(n - 1, acc + a.@(n % 3), a)
            );

            // Reads `ro` and updates `rw`: the tail call lends `ro` and passes `rw` at its last
            // use, so it reaches a version borrowing `ro` and owning `rw`.
            loop_mix : I64 -> Array I64 -> Array I64 -> I64;
            loop_mix = |n, ro, rw| (
                if n == 0 { ro.@(0) + rw.@(0) };
                let rw = rw.set(n % 3, ro.@(n % 3));
                loop_mix(n - 1, ro, rw)
            );

            // Borrows both parameters, and its tail call passes a borrowed value at one and a
            // freshly built (owned) array at the other. Borrowing the fresh array would put its
            // release after the call, so the call reaches a version borrowing `ro` and owning `b`,
            // which consumes the array in the call itself.
            churn : I64 -> Array I64 -> Array I64 -> I64;
            churn = |n, ro, b| (
                if n == 0 { ro.@(0) + b.@(0) };
                let v = ro.@(n % 3) + b.@(0);
                churn(n - 1, ro, [v % 1000, v % 999, v % 998])
            );

            main : IO ();
            main = (
                let a = Array::from_map(3, |i| i + 1);
                let b = Array::from_map(3, |i| 10 * (i + 1));
                let n = 30000000;
                assert_eq(|_|"read-only loop", loop_sum(n, 0, a), 60000000);;
                assert_eq(|_|"mixed loop", loop_mix(n, a, b), 2);;
                assert_eq(|_|"fresh-array loop", churn(n, a, b), 11);;
                assert_eq(|_|"the borrowed arrays survive", a.@(0) + b.@(0), 11);;
                pure()
            );
        "#;
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::None);
        test_source(source, config);
    }

    /// Deep tail recursion whose tail call passes a value read out of a parameter the version
    /// borrows. The borrowed `ring` holds the array past the call, so outside tail position the
    /// call would lend it; a call in tail position lends no owned value, since lending would leave
    /// the release of the reference the read took after the call. So the call reaches a version
    /// owning `a`, and the loop runs in constant stack.
    ///
    /// The depth makes this a stack test, not a memcheck one, so Valgrind is switched off to keep
    /// it fast.
    #[test]
    pub fn test_tail_call_passing_a_value_read_out_of_a_borrowed_parameter() {
        let source = r#"
            module Main;

            type Ring = box struct { even : Array I64, odd : Array I64 };

            // Only reads `ring` and `a`, and its tail call passes at `a` an array read out of
            // `ring`.
            spin : I64 -> I64 -> Ring -> Array I64 -> I64;
            spin = |n, acc, ring, a| (
                if n == 0 { acc + a.@(0) };
                spin(n - 1, acc + a.@(0), ring, if n % 2 == 0 { ring.@even } else { ring.@odd })
            );

            main : IO ();
            main = (
                let a = Array::from_map(3, |i| i + 1);
                let b = Array::from_map(3, |i| 10 * (i + 1));
                let ring = Ring { even : a, odd : b };
                let n = 30000000;
                assert_eq(|_|"loop over a borrowed struct", spin(n, 0, ring, a), 165000001);;
                assert_eq(|_|"the borrowed struct survives", ring.@even.@(0) + ring.@odd.@(0), 11);;
                pure()
            );
        "#;
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::None);
        test_source(source, config);
    }
}
