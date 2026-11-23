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

    /// A lent value the body returns is retained before the return, which hands the caller a
    /// reference.
    ///
    /// `pick` borrows the table and returns one of its fields, a field on either arm, so both reads
    /// lend. `main` then pushes onto the array it was handed and reads the table's own field again.
    /// Without the retain the caller's array is the table's field itself with one reference: the
    /// push updates it in place and its release frees the field the table still holds, which
    /// Valgrind MemCheck reports.
    #[test]
    pub fn test_a_lent_value_the_body_returns_is_retained_for_the_caller() {
        let source = r#"
            module Main;

            type Table = box struct { cells : Array I64, tag : Array I64 };

            // Builds a fresh array. Recursive, so the inliner leaves it a call.
            fresh : I64 -> Array I64;
            fresh = |n| if n <= 0 { [] } else { fresh(n - 1).push_back(n) };

            // Returns a field of the table, which the caller goes on holding. The recursion keeps `pick`
            // a call, and its result is a field on either arm.
            pick : I64 -> Table -> Array I64;
            pick = |n, t| if n > 0 && pick(n - 1, t).@size < 0 { t.@tag } else { t.@cells };

            main : IO ();
            main = (
                let n = (*get_args).@size;
                let t = Table { cells : fresh(n + 3), tag : fresh(n + 1) };
                let a = pick(1, t);
                let b = a.push_back(9);
                assert_eq(|_|"the field returned and the table it was read out of", b.@size * 10 + t.@cells.@size, 54);;
                pure()
            );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A lent value that is an unboxed union of two boxed variants is retained once before a call
    /// that owns it, however many variants it has.
    ///
    /// `measured` borrows `rs` and hands an element of it to `measure`, which owns its argument. The
    /// element is one reference-counting unit of two leaves, one per variant. A retain per leaf
    /// retains the variant the element holds twice, and the call consumes it once, so the array it
    /// holds leaks, which Valgrind MemCheck reports.
    #[test]
    pub fn test_a_lent_union_of_two_boxed_variants_is_retained_once_per_owning_position() {
        let source = r#"
            module Main;

            // Builds a fresh array. Recursive, so the inliner leaves it a call.
            fresh : I64 -> Array I64;
            fresh = |n| if n <= 0 { [] } else { fresh(n - 1).push_back(n) };

            // Owns the value it is given. Recursive, so the inliner leaves it a call.
            measure : I64 -> Result (Array I64) (Array I64) -> I64;
            measure = |n, r| (
                if n > 0 { measure(n - 1, r) } else {
                    match r { ok(a) => a.push_back(1).@size, err(e) => e.push_back(2).@size * 10 }
                }
            );

            // Hands `measure` an element of `rs`, which the caller goes on holding. The element is an unboxed
            // union either variant of which holds an array.
            measured : I64 -> Array (Result (Array I64) (Array I64)) -> I64;
            measured = |n, rs| (
                let k = if n > 0 { measured(n - 1, rs) } else { 0 };
                let r = rs.@(n);
                measure(1, r) + k
            );

            main : IO ();
            main = (
                let n = (*get_args).@size;
                let rs = [Result::ok(fresh(n + 2)), Result::err(fresh(n + 3))];
                assert_eq(|_|"both elements measured, and the array they were read out of", measured(1, rs) * 10 + rs.@size, 542);;
                pure()
            );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A value read out of a lent value is held only while the outermost container is, so one used
    /// after that container's last use keeps a reference of its own.
    ///
    /// `inner` reads `row` out of `rows` and `x` out of `row`, reads `rows` once more, and reads `x`
    /// after that. `row` is lent, since every use of it comes before `rows` is released. Releasing
    /// `rows` frees `row` and with it the array `x` names, so a read lending `x` would read it
    /// after it was freed, which Valgrind MemCheck reports.
    #[test]
    pub fn test_a_value_read_out_of_a_lent_value_keeps_its_own_reference_past_the_outer_container()
    {
        let source = r#"
            module Main;

            // Builds a fresh array. Recursive, so the inliner leaves it a call.
            fresh : I64 -> Array I64;
            fresh = |n| if n <= 0 { [] } else { fresh(n - 1).push_back(n) };

            // Builds a fresh array of rows of arrays. Recursive, so the inliner leaves it a call.
            fresh_rows : I64 -> Array (Array (Array I64));
            fresh_rows = |n| if n <= 0 { [] } else { fresh_rows(n - 1).push_back([fresh(n), fresh(n + 1)]) };

            // Reads `x` out of a row read out of `rows`, reads `rows` once more, and reads `x` after that last
            // use of `rows`. The recursion keeps `inner` a call.
            inner : I64 -> I64;
            inner = |n| (
                let k = if n > 0 { inner(n - 1) } else { 0 };
                let rows = fresh_rows(n + 2);
                let row = rows.@(0);
                let x = row.@(1);
                let s = rows.@(x.@size - 1).@size;
                let y = fresh(40 + s);
                x.@(0) + y.@size + k
            );

            main : IO ();
            main = (
                let n = (*get_args).@size;
                assert_eq(|_|"the inner element read after the outer array is gone", inner(n), 86);;
                pure()
            );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A value read out of a container before a branch is lent past the branch only where the
    /// container stays referenced on every arm.
    ///
    /// `step` reads `order` out of the unique `w`, then on one arm returns `w` as it is and on the
    /// other replaces that field, and reads `order` after the arms join. On the arm `main` takes
    /// the update is made in place and releases the array `order` names, so a read lending `order`
    /// would read it after it was freed, which Valgrind MemCheck reports.
    #[test]
    pub fn test_a_value_read_before_a_branch_updating_its_container_on_one_arm_keeps_its_own_reference(
    ) {
        let source = r#"
            module Main;

            type Walk = box struct { order : Array I64, size : I64 };

            // Builds a fresh array. Recursive, so the inliner leaves it a call.
            fresh : I64 -> Array I64;
            fresh = |n| if n <= 0 { [] } else { fresh(n - 1).push_back(n) };

            // Reads `order` out of the walk, then on one arm returns the walk as it is and on the other
            // replaces the array in it, and reads `order` after the arms join. The recursion keeps `step` a
            // call.
            step : I64 -> Walk -> Bool -> (I64, Walk);
            step = |n, w, keep| (
                let k = if n > 0 { step(n - 1, Walk { order : fresh(3), size : 0 }, keep).@0 } else { 0 };
                let order = w.@order;
                let w2 = if keep { w } else { w.set_order(fresh(4 + k)) };
                (order.@(1) + k, w2)
            );

            main : IO ();
            main = (
                let n = (*get_args).@size;
                let w = Walk { order : Array::from_map(n + 2, |i| i + 3), size : n };
                let (s, w) = step(1, w, n == 0);
                assert_eq(|_|"the array read before the update, and the walk after it", s * 100 + w.@order.@size * 10 + w.@size, 661);;
                pure()
            );
        "#;
        test_source(source, Configuration::develop_mode());
    }
}
