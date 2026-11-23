//! Values read out of a boxed container without a retain (`lend::lend_reads`), which hold no
//! reference of their own while the container's object holds them.

#[cfg(test)]
mod lent_reads_tests {
    use crate::{configuration::Configuration, tests::test_util::test_source};

    /// A value read out of a boxed struct keeps its own reference when the struct is then updated
    /// in place, though a variable used afterwards may hold the struct's object.
    ///
    /// On the branch `main` takes, `step` reads `order` out of the unique `w`, then replaces that
    /// field: the update is made in place and releases the array `order` was read out of, which
    /// `order` is read again after. The match binding `r`, used afterwards, holds the walk's
    /// object on the other branch alone, so it does not hold the array on this one. A read lending
    /// `order` would read it after it was freed. The program runs under Valgrind MemCheck, which
    /// reports that read.
    #[test]
    pub fn test_a_value_read_out_of_a_struct_updated_in_place_afterwards_is_retained() {
        let source = r#"
            module Main;

            type Walk = box struct { order : Array I64, size : I64 };

            // Builds a fresh array. Recursive, so the inliner leaves it a call.
            fresh : I64 -> Array I64;
            fresh = |n| if n <= 0 { [] } else { fresh(n - 1).push_back(n) };

            // On one branch returns the walk as it is; on the other reads its array, replaces the
            // array in the walk, and reads the array read out before the update.
            step : Walk -> Bool -> (Array I64, Walk);
            step = |w, keep| (
                let r = if keep {
                    (fresh(1), w)
                } else {
                    let order = w.@order;
                    let w = w.set_order([7, 8]);
                    let s = order.@(1);
                    (fresh(s), w)
                };
                r
            );

            main : IO ();
            main = (
                let n = (*get_args).@size;
                let w = Walk { order : Array::from_map(n + 2, |i| i + 3), size : n };
                let (a, w) = step(w, n == 0);
                assert_eq(|_|"the array read out before the update", a.@size * 100 + w.@order.@(0) * 10 + w.@size, 471);;
                pure()
            );
        "#;
        test_source(source, Configuration::develop_mode());
    }
}
