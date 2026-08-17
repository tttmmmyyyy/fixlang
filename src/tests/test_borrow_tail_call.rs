//! Tail recursion across the versions borrow-ification routes between: a tail call must stay a
//! tail call whichever version it is routed to, so these loops must run in constant stack.

#[cfg(test)]
mod borrow_tail_call_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        tests::test_util::test_source,
    };

    /// Deep tail recursion with a read-only parameter, and with a read-only parameter beside an
    /// owned one. The first loop is routed to its borrow version, which must add no release after
    /// the call; the second passes an owned argument in tail position, which must keep the call on
    /// the owning version rather than un-tail it with an after-call release. Either mistake turns
    /// the loop into real recursion, and the depth here overflows the stack.
    ///
    /// The depth makes this a stack test, not a memcheck one, so Valgrind is switched off to keep
    /// it fast.
    #[test]
    pub fn test_tail_recursion_survives_borrow_routing() {
        let source = r#"
            module Main;

            // Only reads `a`: its tail call is routed to the borrow version.
            loop_sum : I64 -> I64 -> Array I64 -> I64;
            loop_sum = |n, acc, a| (
                if n == 0 { acc };
                loop_sum(n - 1, acc + a.@(n % 3), a)
            );

            // Reads `ro` and updates `rw`: the tail call passes an owned argument, so it stays on
            // the owning version.
            loop_mix : I64 -> Array I64 -> Array I64 -> I64;
            loop_mix = |n, ro, rw| (
                if n == 0 { ro.@(0) + rw.@(0) };
                let rw = rw.set(n % 3, ro.@(n % 3));
                loop_mix(n - 1, ro, rw)
            );

            // Borrows both parameters, and its tail call passes a borrowed value at one and a
            // freshly built (owned) array at the other. Routing that call to the borrow version
            // would put the fresh array's release after the call; it must stay on the owning
            // version, which consumes the array in the call itself.
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
}
